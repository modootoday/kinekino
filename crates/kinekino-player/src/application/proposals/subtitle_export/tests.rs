use super::*;
use crate::application::api::ProposalClient;
use crate::{
    application::{
        api::{PlayerSdk, RequestContext},
        proposals::StoredProposal,
    },
    domain::proposals::{
        AcceptanceReceipt, Correction, CueSourceSnapshot, DraftState, GeneratedResult,
        ProposalDocument, ProposalPayload, SourceCue, SourceSnapshot, proposal_document_version,
    },
};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
struct Snapshot(Arc<StoredProposal>);
impl ProposalStore for Snapshot {
    fn load(&self, id: &str) -> Result<StoredProposal, ApiError> {
        if id != self.0.document.proposal_id {
            return Err(ApiError::new(
                ErrorCode::NotFound,
                "fixture proposal absent",
            ));
        }
        Ok(self.0.as_ref().clone())
    }
    fn publish(&self, _: Option<u64>, _: &StoredProposal) -> Result<(), ApiError> {
        panic!("export must never publish or Accept a proposal")
    }
}
#[derive(Clone, Default)]
struct Files {
    outputs: Arc<Mutex<BTreeMap<String, String>>>,
    failure: Arc<Mutex<Option<ApiError>>>,
}
impl SubtitleStorage for Files {
    fn read_utf8(&self, _: &str) -> Result<String, ApiError> {
        panic!("export must consume accepted cue data, not another subtitle file")
    }
    fn publish_new(&self, path: &str, contents: &str) -> Result<(), ApiError> {
        if let Some(error) = self.failure.lock().unwrap().take() {
            return Err(error);
        }
        let mut outputs = self.outputs.lock().unwrap();
        if outputs.contains_key(path) {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "fixture output already exists",
            ));
        }
        outputs.insert(path.into(), contents.into());
        Ok(())
    }
}
fn state(times: &[(u64, u64)]) -> StoredProposal {
    let source = SourceSnapshot::Cue(CueSourceSnapshot {
        source_id: "accepted-source".into(),
        source_revision: DecimalU64(1),
        source_sha256: "a".repeat(64),
        cues: times
            .iter()
            .enumerate()
            .map(|(index, &(start, end))| SourceCue {
                cue_id: format!("cue-{index}"),
                text: format!("원문-{index}"),
                start_us: Some(DecimalU64(start)),
                end_us: Some(DecimalU64(end)),
            })
            .collect(),
    });
    let output = ProposalPayload::Correction {
        edits: vec![Correction {
            cue_id: "cue-0".into(),
            text: "교정 & <보존>".into(),
        }],
    };
    let receipt = AcceptanceReceipt {
        proposal_id: "accepted-export".into(),
        draft_revision: DecimalU64(1),
        output_revision: DecimalU64(2),
        source_id: source.source_id().into(),
        source_revision: source.source_revision(),
        source_sha256: source.source_sha256().into(),
        output: output.clone(),
        automatic_application_enabled: false,
    };
    let state = StoredProposal {
        document: ProposalDocument {
            schema_version: proposal_document_version(&source, None).into(),
            proposal_id: receipt.proposal_id.clone(),
            source,
            generated: GeneratedResult {
                task: output.task(),
                raw_text: "original fixture model text; no inference".into(),
            },
            draft_revision: DecimalU64(2),
            state: DraftState::Editable,
            draft: Some(output),
            import_error: None,
        },
        acceptance: Some(receipt),
        historical: None,
    };
    state.validate().unwrap();
    state
}
fn input(state: &StoredProposal, path: &str) -> ExportAcceptedSubtitles {
    ExportAcceptedSubtitles {
        target: OutputReviewTarget::from_stored(state).unwrap(),
        output_path: path.into(),
        output_format: SubtitleFormat::Srt,
        allow_loss: false,
    }
}

#[test]
fn whole_domain_export_preserves_nonzero_unsorted_times_untouched_text_and_snapshot() {
    let snapshot = state(&[(9_000_000, 10_000_000), (1_000_000, 2_000_000)]);
    let before = serde_json::to_vec(&snapshot).unwrap();
    let request = input(&snapshot, "accepted.srt");
    let target = request.target.clone();
    let store = Snapshot(Arc::new(snapshot));
    let files = Files::default();
    let result = execute(&store, &files, request).unwrap();
    let expected = "1\n00:00:09,000 --> 00:00:10,000\n교정 &amp; &lt;보존&gt;\n\n2\n00:00:01,000 --> 00:00:02,000\n원문-1\n\n";
    assert_eq!(files.outputs.lock().unwrap()["accepted.srt"], expected);
    assert_eq!(result.target, target);
    assert_eq!(
        result.output_sha256,
        format!("{:x}", Sha256::digest(expected.as_bytes()))
    );
    assert_eq!(result.output_bytes.0, expected.len() as u64);
    assert_eq!(result.cue_count, 2);
    assert!(result.losses.is_empty());
    assert!(!result.overwrite);
    assert_eq!(serde_json::to_vec(store.0.as_ref()).unwrap(), before);
}

#[test]
fn whole_domain_upper_max_precision_loss_requires_acknowledgement_before_publish() {
    let snapshot = state(&[(u64::MAX - 5_000, u64::MAX)]);
    let mut request = input(&snapshot, "accepted.vtt");
    request.output_format = SubtitleFormat::Vtt;
    let store = Snapshot(Arc::new(snapshot));
    let files = Files::default();
    let error = execute(&store, &files, request.clone()).unwrap_err();
    assert_eq!(error.code, ErrorCode::NeedsInput);
    assert_eq!(
        error.details["losses"][0]["code"],
        "timestamp_precision_reduced"
    );
    assert!(files.outputs.lock().unwrap().is_empty());
    request.allow_loss = true;
    let result = execute(&store, &files, request).unwrap();
    let output = files.outputs.lock().unwrap()["accepted.vtt"].clone();
    assert!(output.starts_with("WEBVTT\n\n"));
    assert!(output.contains("5124095576:01:49.546 --> 5124095576:01:49.551"));
    assert_eq!(result.losses[0].code, "timestamp_precision_reduced");
    assert_eq!(result.target.source_revision, DecimalU64(1));
}

#[test]
fn collapse_empty_and_unknown_timing_never_publish_or_infer_replacement_time() {
    let snapshot = state(&[(1001, 1999)]);
    let mut request = input(&snapshot, "collapsed.srt");
    request.allow_loss = true;
    let files = Files::default();
    assert_eq!(
        execute(&Snapshot(Arc::new(snapshot)), &files, request)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    let mut unknown = state(&[(1000, 2000)]);
    if let SourceSnapshot::Cue(source) = &mut unknown.document.source {
        source.cues[0].start_us = None;
        source.cues[0].end_us = None;
    }
    let request = input(&unknown, "unknown.vtt");
    assert_eq!(
        execute(
            &Snapshot(Arc::new(unknown.clone())),
            &files,
            request.clone()
        )
        .unwrap_err()
        .code,
        ErrorCode::NeedsInput
    );
    if let SourceSnapshot::Cue(source) = &mut unknown.document.source {
        source.cues.clear();
    }
    assert_eq!(
        execute(&Snapshot(Arc::new(unknown)), &files, request)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert!(files.outputs.lock().unwrap().is_empty());
}

#[test]
fn stale_target_and_original_publication_failure_do_not_mutate_or_auto_accept() {
    let snapshot = state(&[(1000, 3000)]);
    let before = serde_json::to_vec(&snapshot).unwrap();
    let request = input(&snapshot, "output.srt");
    let store = Snapshot(Arc::new(snapshot));
    let files = Files::default();
    let mut stale = request.clone();
    stale.target.output_sha256 = "b".repeat(64);
    assert_eq!(
        execute(&store, &files, stale).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    let mut original = ApiError::new(ErrorCode::PermissionDenied, "original publisher failure");
    original.details = json!({"original":true});
    let message_buffer = original.message.as_ptr();
    *files.failure.lock().unwrap() = Some(original);
    let error = execute(&store, &files, request).unwrap_err();
    assert_eq!(error.code, ErrorCode::PermissionDenied);
    assert_eq!(error.message.as_ptr(), message_buffer);
    assert_eq!(error.details, json!({"original":true}));
    assert!(files.outputs.lock().unwrap().is_empty());
    assert_eq!(serde_json::to_vec(store.0.as_ref()).unwrap(), before);
}

#[test]
fn typed_sdk_wire_schema_and_adapter_capabilities_share_the_export_use_case() {
    let snapshot = state(&[(1000, 3000)]);
    let request = input(&snapshot, "typed.srt");
    let files = Files::default();
    let sdk = PlayerSdk::new(files.clone()).with_proposal_store(Snapshot(Arc::new(snapshot)));
    let result = sdk
        .export_accepted_subtitles(RequestContext::default(), request.clone())
        .unwrap();
    assert_eq!(result.target, request.target);
    let mut wire = Request::new(COMMAND);
    let mut vtt = request.clone();
    vtt.output_path = "wire.vtt".into();
    vtt.output_format = SubtitleFormat::Vtt;
    wire.payload = serde_json::to_value(vtt).unwrap();
    let response = sdk.execute(wire);
    assert!(response.ok);
    let data = response.data.unwrap();
    assert_eq!(data["output_format"], "vtt");
    assert!(data["output_bytes"].is_string());
    assert_eq!(
        data["target"],
        serde_json::to_value(&request.target).unwrap()
    );
    let mut describe = Request::new("schema");
    describe.payload = json!({"command":COMMAND});
    assert_eq!(
        sdk.execute(describe.clone()).data.unwrap()["status"],
        "supported"
    );
    let available = sdk.execute(Request::new("capabilities")).data.unwrap();
    assert_eq!(
        available["commands"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["command"] == COMMAND)
            .unwrap()["status"],
        "supported"
    );
    let unavailable = PlayerSdk::default();
    assert_eq!(
        unavailable.execute(describe).data.unwrap()["status"],
        "unsupported"
    );
    assert_eq!(
        unavailable
            .export_accepted_subtitles(RequestContext::default(), request.clone())
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
    let context = RequestContext {
        expected_revision: Some("2".into()),
        ..RequestContext::default()
    };
    assert_eq!(
        sdk.export_accepted_subtitles(context, request)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
}

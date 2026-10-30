use super::*;
use crate::application::api::CardClient;
use crate::{
    application::api::{PlayerSdk, RequestContext},
    domain::proposals::*,
};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};
#[derive(Clone)]
struct Store(Arc<StoredProposal>);
impl ProposalStore for Store {
    fn load(&self, _: &str) -> Result<StoredProposal, ApiError> {
        Ok(self.0.as_ref().clone())
    }
    fn publish(&self, _: Option<u64>, _: &StoredProposal) -> Result<(), ApiError> {
        panic!("Review export cannot publish or accept a proposal")
    }
}
#[derive(Clone, Default)]
struct Port {
    calls: Arc<AtomicUsize>,
    published: Arc<AtomicUsize>,
    failure: Arc<Mutex<Option<ApiError>>>,
    publish_failure: Arc<Mutex<Option<ApiError>>>,
}
impl CardImagePort for Port {
    fn render(
        &self,
        card: SelectedAcceptedCard<'_>,
        _: &ExportCardReviewPng,
    ) -> Result<PreparedReviewPng, ApiError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(
            (card.front, card.back),
            ("front <原文>", "back & unchanged")
        );
        assert_eq!(card.position, 0);
        assert_eq!(card.source_refs, &["cue-1".to_owned()]);
        if let Some(error) = self.failure.lock().unwrap().take() {
            return Err(error);
        }
        Ok(PreparedReviewPng {
            bytes: b"synthetic-port-not-a-real-PNG".to_vec(),
            provenance: json!({"fixture":"application-port-only"}),
        })
    }
    fn publish_new(&self, _: &str, _: &[u8]) -> Result<(), ApiError> {
        self.published.fetch_add(1, Ordering::SeqCst);
        if let Some(error) = self.publish_failure.lock().unwrap().take() {
            return Err(error);
        }
        Ok(())
    }
}
fn stored() -> StoredProposal {
    let output = ProposalPayload::Cards {
        cards: vec![Card {
            front: "front <原文>".into(),
            back: "back & unchanged".into(),
            source_refs: vec!["cue-1".into()],
        }],
    };
    let state = StoredProposal {
        document: ProposalDocument {
            schema_version: "1".into(),
            proposal_id: "proposal-1".into(),
            source: SourceSnapshot::Cue(CueSourceSnapshot {
                source_id: "source-1".into(),
                source_revision: DecimalU64(3),
                source_sha256: "a".repeat(64),
                cues: vec![SourceCue {
                    cue_id: "cue-1".into(),
                    text: "Synthetic source".into(),
                    start_us: None,
                    end_us: None,
                }],
            }),
            generated: GeneratedResult {
                task: ProposalTask::Cards,
                raw_text: serde_json::to_string(&output).unwrap(),
            },
            draft_revision: DecimalU64(2),
            state: DraftState::Editable,
            draft: Some(output.clone()),
            import_error: None,
        },
        acceptance: Some(AcceptanceReceipt {
            proposal_id: "proposal-1".into(),
            draft_revision: DecimalU64(1),
            output_revision: DecimalU64(2),
            source_id: "source-1".into(),
            source_revision: DecimalU64(3),
            source_sha256: "a".repeat(64),
            output,
            automatic_application_enabled: false,
        }),
        historical: None,
    };
    state.validate().unwrap();
    state
}
fn input(state: &StoredProposal) -> ExportCardReviewPng {
    ExportCardReviewPng {
        target: OutputReviewTarget::from_stored(state).unwrap(),
        position: DecimalU64(0),
        frame_path: absolute_png("frame.png"),
        frame_sha256: "b".repeat(64),
        template: TemplateSelection {
            id: "explicit-proposal".into(),
            version: "proposed".into(),
            declaration_sha256: "c".repeat(64),
        },
        output_path: absolute_png("review.png"),
        review_artifact_only: true,
    }
}
#[test]
fn accepted_card_review_artifact_preserves_identity_text_and_snapshot() {
    let state = Arc::new(stored());
    let before = serde_json::to_vec(state.as_ref()).unwrap();
    let port = Port::default();
    let request = input(&state);
    let expected = request.target.clone();
    let result = execute(&Store(state.clone()), Some(&port), request).unwrap();
    assert_eq!(result.target, expected);
    assert_eq!(result.state, "review_artifact");
    assert!(!result.quality_approved && !result.template_approved && !result.overwrite);
    assert_eq!(result.limitations.len(), 7);
    assert!(!result.cancellation_supported && !result.timeout_supported);
    assert_eq!(result.source_refs, ["cue-1"]);
    assert_eq!(port.published.load(Ordering::SeqCst), 1);
    assert_eq!(serde_json::to_vec(state.as_ref()).unwrap(), before);
}
#[test]
fn stale_target_position_and_unacknowledged_review_never_reach_renderer() {
    let state = Arc::new(stored());
    let store = Store(state.clone());
    let port = Port::default();
    let mut request = input(&state);
    request.target.output_sha256 = "d".repeat(64);
    assert_eq!(
        execute(&store, Some(&port), request).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    let mut request = input(&state);
    request.position = DecimalU64(u64::MAX);
    assert_eq!(
        execute(&store, Some(&port), request).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    let mut request = input(&state);
    request.review_artifact_only = false;
    assert_eq!(
        execute(&store, Some(&port), request).unwrap_err().code,
        ErrorCode::NeedsInput
    );
    assert_eq!(port.calls.load(Ordering::SeqCst), 0);
    assert_eq!(port.published.load(Ordering::SeqCst), 0);
}
#[test]
fn non_card_or_unaccepted_output_is_not_accepted_or_rendered() {
    let mut state = stored();
    let original_target = input(&state);
    state.acceptance = None;
    let port = Port::default();
    assert_eq!(
        execute(&Store(Arc::new(state)), Some(&port), original_target)
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    let mut state = stored();
    let output = ProposalPayload::Correction {
        edits: vec![Correction {
            cue_id: "cue-1".into(),
            text: "corrected".into(),
        }],
    };
    state.document.generated.task = ProposalTask::Correction;
    state.document.generated.raw_text = serde_json::to_string(&output).unwrap();
    state.document.draft = Some(output.clone());
    state.acceptance.as_mut().unwrap().output = output;
    state.validate().unwrap();
    let request = input(&state);
    assert_eq!(
        execute(&Store(Arc::new(state)), Some(&port), request)
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
    assert_eq!(port.calls.load(Ordering::SeqCst), 0);
    assert_eq!(port.published.load(Ordering::SeqCst), 0);
}
#[test]
fn original_render_and_publication_causes_are_not_reconstructed() {
    let state = Arc::new(stored());
    let store = Store(state.clone());
    let port = Port::default();
    for render_failure in [true, false] {
        let mut cause = ApiError::new(ErrorCode::PermissionDenied, "original owned cause");
        cause.details = json!({"identity":"original"});
        cause.request_id = "original-id".into();
        let pointer = cause.message.as_ptr();
        if render_failure {
            *port.failure.lock().unwrap() = Some(cause);
        } else {
            *port.publish_failure.lock().unwrap() = Some(cause);
        }
        let error = execute(&store, Some(&port), input(&state)).unwrap_err();
        assert_eq!(error.message.as_ptr(), pointer);
        assert_eq!(error.code, ErrorCode::PermissionDenied);
        assert_eq!(error.details, json!({"identity":"original"}));
        assert_eq!(error.request_id, "original-id");
        assert_eq!(
            port.published.load(Ordering::SeqCst),
            usize::from(!render_failure)
        );
    }
}
#[test]
fn typed_sdk_wire_schema_and_unverified_capability_share_review_only_contract() {
    let state = Arc::new(stored());
    let port = Port::default();
    let sdk = PlayerSdk::default()
        .with_proposal_store(Store(state.clone()))
        .with_card_image_export(port.clone());
    let input = input(&state);
    assert_eq!(
        sdk.export_card_review_png(RequestContext::default(), input.clone())
            .unwrap()
            .state,
        "review_artifact"
    );
    let mut request = Request::new(COMMAND);
    request.payload = serde_json::to_value(&input).unwrap();
    let response = sdk.execute(request);
    assert!(response.ok);
    assert_eq!(response.data.unwrap()["quality_approved"], false);
    let capabilities = sdk.execute(Request::new("capabilities")).data.unwrap();
    assert_eq!(
        capabilities["card_review_png"]["verification"],
        "not_actual_CLI_verified"
    );
    assert_eq!(
        capabilities["card_review_png"]["production_quality_export"],
        false
    );
    let mut schema = Request::new("schema");
    schema.payload = json!({"command":COMMAND});
    assert_eq!(
        sdk.execute(schema).data.unwrap()["status"],
        "available_unverified"
    );
    let context = RequestContext {
        timeout_ms: Some(1),
        ..RequestContext::default()
    };
    assert_eq!(
        sdk.export_card_review_png(context, input.clone())
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
    assert_eq!(
        PlayerSdk::default()
            .export_card_review_png(RequestContext::default(), input)
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
}

/// Absolute local PNG path in the host OS's syntax (no file is touched).
fn absolute_png(name: &str) -> String {
    std::env::temp_dir().join(name).display().to_string()
}

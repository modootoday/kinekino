use super::*;
use kinekino_media_contracts::errors::ErrorCode;
use kinekino_player::{
    application::api::{API_VERSION, Request as ApiRequest, Response},
    domain::proposals::*,
};
use serde_json::{Value, json, to_value};
use std::{collections::VecDeque, sync::Mutex};

pub(super) struct RecordingClient {
    requests: Mutex<Vec<ApiRequest>>,
    responses: Mutex<VecDeque<Result<CaptureMutationResult, ApiError>>>,
}
impl RecordingClient {
    pub(super) fn new(responses: Vec<Result<CaptureMutationResult, ApiError>>) -> Self {
        Self {
            requests: Mutex::new(Vec::new()),
            responses: Mutex::new(responses.into()),
        }
    }
    pub(super) fn payloads(&self) -> Vec<Value> {
        self.requests
            .lock()
            .unwrap()
            .iter()
            .map(|r| r.payload.clone())
            .collect()
    }
}
impl ApplicationClient for RecordingClient {
    fn execute(&self, request: ApiRequest) -> Response {
        assert!(matches!(
            request.command.as_str(),
            "proposals.edit_capture" | "proposals.accept_capture"
        ));
        let request_id = request.context.request_id.clone();
        self.requests.lock().unwrap().push(request);
        match self
            .responses
            .lock()
            .unwrap()
            .pop_front()
            .expect("Unexpected SDK call")
        {
            Ok(result) => Response {
                api_version: API_VERSION,
                request_id,
                ok: true,
                data: Some(to_value(result).unwrap()),
                error: None,
            },
            Err(error) => Response::failure(request_id, error),
        }
    }
}
pub(super) fn anchor(id: &str) -> CaptureAnchor {
    CaptureAnchor {
        project_id: id.into(),
        project_revision: DecimalU64(9_007_199_254_740_993),
        project_revision_sha256: "b".repeat(64),
        proposal_id: "draft".into(),
        capture_sha256: "c".repeat(64),
    }
}
pub(super) fn payload(text: &str) -> ProposalPayload {
    ProposalPayload::Cards {
        cards: vec![Card {
            front: text.into(),
            back: "back".into(),
            source_refs: vec!["cue".into()],
        }],
    }
}
pub(super) fn state(revision: u64) -> StoredProposal {
    StoredProposal {
        document: ProposalDocument {
            schema_version: "1".into(),
            proposal_id: "draft".into(),
            source: SourceSnapshot::Cue(CueSourceSnapshot {
                source_id: "source".into(),
                source_revision: DecimalU64(3),
                source_sha256: "a".repeat(64),
                cues: vec![SourceCue {
                    cue_id: "cue".into(),
                    text: "원문".into(),
                    start_us: None,
                    end_us: None,
                }],
            }),
            generated: GeneratedResult {
                task: ProposalTask::Cards,
                raw_text: serde_json::to_string(&payload("generated")).unwrap(),
            },
            draft_revision: DecimalU64(revision),
            state: DraftState::Editable,
            draft: Some(payload("saved")),
            import_error: None,
        },
        acceptance: None,
        historical: None,
    }
}
pub(super) fn result(revision: u64) -> CaptureMutationResult {
    CaptureMutationResult {
        state: state(revision),
        continuation: CaptureExpectation {
            draft_revision: DecimalU64(revision),
            state_sha256: "d".repeat(64),
        },
        acceptance: None,
        reused_acceptance: false,
    }
}
fn complete(adapter: &mut CaptureMutationAdapter, intent: CaptureIntent, client: &RecordingClient) {
    assert!(
        adapter
            .complete(intent.execute(client, RequestContext::default()))
            .is_ok()
    );
}

#[test]
fn original_anchor_and_only_successful_continuation_feed_typed_requests() {
    let client = RecordingClient::new(vec![Ok(result(2)), Ok(result(3))]);
    let selected = anchor("selected");
    let mut adapter = CaptureMutationAdapter::new(selected.clone(), state(1));
    let edit = adapter.edit(payload("first edit")).unwrap();
    assert_eq!(adapter.display().state.document.draft_revision.0, 1);
    complete(&mut adapter, edit, &client);
    let edit = adapter.edit(payload("second edit")).unwrap();
    complete(&mut adapter, edit, &client);
    let requests = client.payloads();
    assert_eq!(requests[0]["anchor"], to_value(&selected).unwrap());
    assert!(requests[0]["expected"].is_null());
    assert_eq!(
        requests[1]["expected"],
        to_value(result(2).continuation).unwrap()
    );
    assert_eq!(adapter.display().state.document.draft_revision.0, 3);
    assert_eq!(adapter.display().continuation.unwrap().draft_revision.0, 3);
    for request in client.requests.lock().unwrap().iter() {
        assert!(request.context.expected_revision.is_none());
        assert!(request.context.idempotency_key.is_none());
        assert!(request.context.timeout_ms.is_none());
    }
}

#[test]
fn pending_selection_and_second_action_cannot_replace_original_request() {
    let client = RecordingClient::new(vec![Ok(result(2))]);
    let mut adapter = CaptureMutationAdapter::new(anchor("original"), state(1));
    let edit = adapter.edit(payload("edit")).unwrap();
    assert_eq!(
        adapter.select(anchor("other"), state(9)),
        Err(CaptureViewError::Busy)
    );
    assert!(matches!(
        adapter.accept(DecimalU64(3)),
        Err(CaptureViewError::Busy)
    ));
    assert_eq!(adapter.display().anchor.project_id, "original");
    complete(&mut adapter, edit, &client);
    assert_eq!(client.payloads()[0]["anchor"]["project_id"], "original");
    adapter.select(anchor("other"), state(9)).unwrap();
    assert!(adapter.display().continuation.is_none());
    assert_eq!(adapter.display().anchor.project_id, "other");
}

#[test]
fn sdk_errors_remain_visible_without_inspect_or_expectation_advancement() {
    for code in [
        ErrorCode::RevisionConflict,
        ErrorCode::NeedsInput,
        ErrorCode::Unsupported,
        ErrorCode::InvalidArgument,
    ] {
        let mut error = ApiError::new(code, "Original SDK diagnostic");
        error.details = json!({"field":"capture"});
        error.remediation = "create_explicit_live_capture".into();
        let client = RecordingClient::new(vec![Ok(result(2)), Err(error), Ok(result(3))]);
        let mut adapter = CaptureMutationAdapter::new(anchor("original"), state(1));
        let first = adapter.edit(payload("one")).unwrap();
        complete(&mut adapter, first, &client);
        let failed = adapter.edit(payload("two")).unwrap();
        complete(&mut adapter, failed, &client);
        let display = adapter.display();
        assert_eq!(display.state.document.draft_revision.0, 2);
        assert_eq!(display.continuation.unwrap().draft_revision.0, 2);
        assert_eq!(display.sdk_error.unwrap().code, code);
        assert_eq!(
            display.sdk_error.unwrap().details,
            json!({"field":"capture"})
        );
        assert_eq!(
            display.sdk_error.unwrap().remediation,
            "create_explicit_live_capture"
        );
        assert_eq!(
            display.sdk_error.unwrap().message,
            "Original SDK diagnostic"
        );
        let retry = adapter.edit(payload("explicit retry")).unwrap();
        complete(&mut adapter, retry, &client);
        let requests = client.payloads();
        assert_eq!(requests.len(), 3);
        assert_eq!(requests[1]["expected"], requests[2]["expected"]);
        assert!(adapter.display().sdk_error.is_none());
    }
}

#[test]
fn accept_retry_repeats_preaccept_payload_even_after_successful_continuation() {
    let mut accepted = result(3);
    accepted.reused_acceptance = true;
    let client = RecordingClient::new(vec![
        Ok(result(2)),
        Ok(result(3)),
        Ok(accepted),
        Ok(result(4)),
    ]);
    let mut adapter = CaptureMutationAdapter::new(anchor("original"), state(1));
    let edit = adapter.edit(payload("edit")).unwrap();
    complete(&mut adapter, edit, &client);
    let accept = adapter.accept(DecimalU64(3)).unwrap();
    complete(&mut adapter, accept, &client);
    assert_eq!(adapter.display().continuation.unwrap().draft_revision.0, 3);
    let retry = adapter.retry_accept().unwrap();
    complete(&mut adapter, retry, &client);
    let requests = client.payloads();
    assert_eq!(requests[1], requests[2]);
    assert_eq!(requests[2]["expected"]["draft_revision"], "2");
    assert!(adapter.display().reused_acceptance);
    let new_edit = adapter.edit(payload("intentional later edit")).unwrap();
    // The owner still holds this pending operation; a retry cannot supersede it.
    assert!(matches!(
        adapter.retry_accept(),
        Err(CaptureViewError::Busy)
    ));
    complete(&mut adapter, new_edit, &client);
    assert!(matches!(
        adapter.retry_accept(),
        Err(CaptureViewError::NoAcceptRetry)
    ));
}

#[test]
fn foreign_completion_is_returned_intact_for_original_pending_owner() {
    let client = RecordingClient::new(vec![Ok(result(2))]);
    let mut original = CaptureMutationAdapter::new(anchor("original"), state(1));
    let mut other = CaptureMutationAdapter::new(anchor("other"), state(7));
    let intent = original.edit(payload("edit")).unwrap();
    let completion = intent.execute(&client, RequestContext::default());
    let rejected = match other.complete(completion) {
        Err(rejected) => rejected,
        Ok(()) => panic!("Foreign completion admitted"),
    };
    assert_eq!(rejected.reason, CaptureViewError::StaleCompletion);
    assert_eq!(other.display().state.document.draft_revision.0, 7);
    assert!(other.display().continuation.is_none());
    assert_eq!(original.display().pending, Some(CaptureAction::Edit));
    assert!(original.complete(rejected.completion).is_ok());
    assert_eq!(original.display().state.document.draft_revision.0, 2);
}

#[test]
fn selection_replacement_removes_old_retry_and_initial_failure_keeps_none() {
    let client = RecordingClient::new(vec![Err(ApiError::new(ErrorCode::NeedsInput, "legacy"))]);
    let mut adapter = CaptureMutationAdapter::new(anchor("legacy"), state(1));
    let accept = adapter.accept(DecimalU64(3)).unwrap();
    complete(&mut adapter, accept, &client);
    assert!(adapter.display().continuation.is_none());
    assert_eq!(adapter.display().state.document.draft_revision.0, 1);
    adapter.select(anchor("different"), state(5)).unwrap();
    assert!(matches!(
        adapter.retry_accept(),
        Err(CaptureViewError::NoAcceptRetry)
    ));
    assert!(adapter.display().sdk_error.is_none());
    assert_eq!(client.payloads().len(), 1);
}

#[test]
fn explicit_retry_after_failed_accept_retains_original_payload_and_error_until_success() {
    let client = RecordingClient::new(vec![
        Err(ApiError::new(
            ErrorCode::RevisionConflict,
            "Original accept conflict",
        )),
        Ok(result(2)),
    ]);
    let mut adapter = CaptureMutationAdapter::new(anchor("original"), state(1));
    let accept = adapter.accept(DecimalU64(3)).unwrap();
    complete(&mut adapter, accept, &client);
    let retry = adapter.retry_accept().unwrap();
    assert_eq!(
        adapter.display().sdk_error.unwrap().code,
        ErrorCode::RevisionConflict
    );
    assert!(adapter.display().continuation.is_none());
    complete(&mut adapter, retry, &client);
    assert_eq!(client.payloads()[0], client.payloads()[1]);
    assert!(adapter.display().sdk_error.is_none());
    assert_eq!(adapter.display().continuation.unwrap().draft_revision.0, 2);
}

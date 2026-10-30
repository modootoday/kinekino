use super::*;
use crate::application::api::ProposalClient;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        projects::ProjectStore,
        proposals::guarded::{CaptureAnchor, EditCapturedProposal},
    },
    domain::projects::StoredProjectRevision,
    infrastructure::projects::ProjectFiles,
    presentation::web_ui_adapter::proposals::{
        capture_mutation::{
            CaptureDispatchProgress, CaptureExecutionAcknowledgement, CaptureExecutionDisposition,
            CaptureExecutionProgress, CaptureIntent, CaptureMutationAdapter,
        },
        card_selection_queue::{CAPTURED_EDIT_BINDING, CARD_SELECTION_BINDING},
    },
};
use meari::application::host_callbacks::PendingCallback;
use meari_native::infrastructure::native_host::{IntentAdmission, NativeWindowHost};
#[path = "guarded_ipc/fixture.rs"]
mod fixture;
use fixture::*;
#[path = "guarded_ipc/product_surface.rs"]
mod product_surface;

enum GuardedFailure {
    Start(Box<StartFailure>),
    Case(Box<RetainedCaseFailure>),
}
impl std::fmt::Debug for GuardedFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Start(error) => std::fmt::Debug::fmt(error, f),
            Self::Case(error) => std::fmt::Debug::fmt(error, f),
        }
    }
}
fn with_rig(body: impl FnOnce(&mut ViewerHost, &Rig)) -> Result<(), Box<GuardedFailure>> {
    let mut rig = Rig::start().map_err(|e| Box::new(GuardedFailure::Start(e)))?;
    let host = rig.host.take().expect("one original host transfer");
    run_retaining_case(host, |host| body(host, &rig)).map_err(|e| Box::new(GuardedFailure::Case(e)))
}
fn anchor(project: &StoredProjectRevision) -> CaptureAnchor {
    CaptureAnchor {
        project_id: project.project.project_id.clone(),
        project_revision: project.project.revision,
        project_revision_sha256: project.revision_sha256.clone(),
        proposal_id: project.project.proposals[0]
            .captured
            .document
            .proposal_id
            .clone(),
        capture_sha256: project.project.proposals[0].capture_sha256.clone(),
    }
}
fn assert_immutable(rig: &Rig, project: &StoredProjectRevision) {
    let loaded = ProjectFiles::open(&rig.root.join("projects"))
        .unwrap()
        .load(
            &project.project.project_id,
            Some(project.project.revision.0),
        )
        .unwrap();
    assert_eq!(
        serde_json::to_value(loaded).unwrap(),
        serde_json::to_value(project).unwrap()
    );
    let calls = requests(&rig.requests);
    assert!(!calls.iter().any(|r| matches!(
        r.command.as_str(),
        "proposals.inspect" | "projects.save_revision"
    )));
    assert!(
        rig.transport
            .snapshot()
            .iter()
            .any(|e| e.operation == "client_verified")
    );
    for request in calls.iter().filter(|r| r.command.ends_with("_capture")) {
        assert!(request.context.expected_revision.is_none());
        assert!(request.context.idempotency_key.is_none());
        assert!(request.context.timeout_ms.is_none());
    }
}

#[test]
#[ignore = "Explicit positive guarded named-pipe/owner slot; no window or renderer"]
fn explicit_new_origin_callback_edits_twice_accepts_and_retries_original_payload_over_pipe()
-> Result<(), Box<GuardedFailure>> {
    with_rig(|host, rig| {
        let project = seed(&rig.client);
        let immutable = serde_json::to_value(&project).unwrap();
        enter(host, project.clone());
        for text in ["First edit", "Second edit"] {
            let edit = host
                .capture_adapter_mut()
                .unwrap()
                .edit(draft(text))
                .unwrap();
            dispatch(host, edit);
            terminal(host);
            let evidence = consume_ack(host);
            assert!(evidence.terminal.as_ref().unwrap().response.ok);
        }
        let source_revision = project.project.proposals[0]
            .captured
            .document
            .source
            .source_revision();
        let accept = host
            .capture_adapter_mut()
            .unwrap()
            .accept(source_revision)
            .unwrap();
        dispatch(host, accept);
        terminal(host);
        let accepted = consume_ack(host);
        assert!(accepted.terminal.as_ref().unwrap().response.ok);
        let retry = host.capture_adapter_mut().unwrap().retry_accept().unwrap();
        dispatch(host, retry);
        terminal(host);
        let retried = consume_ack(host);
        assert_eq!(accepted.request.payload, retried.request.payload);
        let capture = host.capture_adapter_mut().unwrap();
        assert_eq!(capture.display().state.document.draft_revision.0, 4);
        assert!(capture.display().reused_acceptance);
        let old = &accepted.terminal.as_ref().unwrap().response.data;
        let new = &retried.terminal.as_ref().unwrap().response.data;
        assert_eq!(
            old.as_ref().unwrap()["state"],
            new.as_ref().unwrap()["state"]
        );
        drop(capture);
        assert_eq!(serde_json::to_value(&project).unwrap(), immutable);
        assert_immutable(rig, &project);
        let mutations: Vec<_> = requests(&rig.requests)
            .into_iter()
            .filter(|r| r.command.ends_with("_capture"))
            .collect();
        assert_eq!(mutations.len(), 4);
        assert_eq!(
            mutations[0].payload["anchor"],
            serde_json::to_value(anchor(&project)).unwrap()
        );
        assert!(mutations[0].payload["expected"].is_null());
        assert_eq!(mutations[2].payload, mutations[3].payload);
    })
}

#[test]
#[ignore = "Explicit positive guarded named-pipe/owner slot; stale capture, no window"]
fn stored_capture_stale_after_external_edit_is_not_refreshed_by_ui()
-> Result<(), Box<GuardedFailure>> {
    with_rig(|host, rig| {
        let project = seed(&rig.client);
        rig.client
            .edit_captured_proposal(
                RequestContext::default(),
                EditCapturedProposal {
                    anchor: anchor(&project),
                    expected: None,
                    draft: draft("External edit"),
                },
            )
            .unwrap();
        enter(host, project.clone());
        let edit = host
            .capture_adapter_mut()
            .unwrap()
            .edit(draft("Stale UI edit"))
            .unwrap();
        dispatch(host, edit);
        terminal(host);
        let evidence = consume_ack(host);
        assert_eq!(
            evidence
                .terminal
                .unwrap()
                .response
                .error
                .as_ref()
                .unwrap()
                .code,
            ErrorCode::RevisionConflict
        );
        let adapter = host.capture_adapter_mut().unwrap();
        assert!(adapter.display().continuation.is_none());
        assert_eq!(
            adapter.display().state.document.draft_revision.0,
            project.project.proposals[0]
                .captured
                .document
                .draft_revision
                .0
        );
        assert_eq!(
            adapter.display().sdk_error.unwrap().code,
            ErrorCode::RevisionConflict
        );
        drop(adapter);
        assert_immutable(rig, &project);
    })
}

#[test]
#[ignore = "Explicit positive guarded named-pipe/owner slot; legacy remains readonly"]
fn separately_published_legacy_capture_is_denied_by_callback_and_backend()
-> Result<(), Box<GuardedFailure>> {
    with_rig(|host, rig| {
        let project = seed(&rig.client);
        let mut legacy = project.project.clone();
        legacy.project_id = "legacy-work".into();
        legacy.schema_version = "1".into();
        legacy.proposals[0].proposal_origin = None;
        let legacy = ProjectFiles::open(&rig.root.join("projects"))
            .unwrap()
            .publish(None, &legacy)
            .unwrap();
        let event = button(host, legacy.clone(), CAPTURED_EDIT_BINDING);
        let stamp = host.document.as_ref().unwrap().stamp().unwrap();
        let before = requests(&rig.requests).len();
        assert!(matches!(
            host.admit_callback(&stamp, &event),
            IntentAdmission::Rejected(_)
        ));
        assert_eq!(requests(&rig.requests).len(), before);
        assert!(host.card_selection_queue().unwrap().borrow().is_empty());
        assert!(
            host.callbacks()
                .borrow_mut()
                .prepare_route_release(&stamp.owner)
                .unwrap()
                .is_none()
        );
        // Trusted test directly calls the same guarded API to prove UI refusal is not authority.
        let adapter = CaptureMutationAdapter::new(
            anchor(&legacy),
            legacy.project.proposals[0].captured.clone(),
        );
        if let Err(failed) = host.attach_capture_adapter(adapter) {
            panic!("Original attachment: {:?}", failed.error);
        }
        let edit = host
            .capture_adapter_mut()
            .unwrap()
            .edit(draft("Forbidden legacy edit"))
            .unwrap();
        dispatch(host, edit);
        terminal(host);
        let evidence = consume_ack(host);
        let raw = evidence.terminal.unwrap();
        let error = raw.response.error.as_ref().unwrap();
        assert_eq!(error.code, ErrorCode::NeedsInput);
        assert_eq!(error.remediation, "create_explicit_live_capture");
        assert!(
            host.capture_adapter_mut()
                .unwrap()
                .display()
                .continuation
                .is_none()
        );
        assert_immutable(rig, &legacy);
        assert_immutable(rig, &project);
    })
}

#[test]
#[ignore = "Explicit positive guarded named-pipe/owner slot; close before consumer acceptance"]
fn unobserved_guarded_success_blocks_original_close_until_explicit_consumption()
-> Result<(), Box<GuardedFailure>> {
    with_rig(|host, rig| {
        let project = seed(&rig.client);
        enter(host, project.clone());
        let edit = host
            .capture_adapter_mut()
            .unwrap()
            .edit(draft("Completed before UI observation"))
            .unwrap();
        dispatch(host, edit);
        terminal(host);
        let raw = host
            .capture_adapter_mut()
            .unwrap()
            .execution_terminal()
            .unwrap()
            .clone();
        assert!(raw.response.ok);
        let end = Instant::now() + Duration::from_secs(3);
        loop {
            let progress = host.poll_unrun_shutdown().unwrap();
            if matches!(progress, ViewerCloseProgress::CardScopePending) {
                break;
            }
            assert!(!matches!(progress, ViewerCloseProgress::Complete { .. }));
            assert!(Instant::now() < end);
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(
            host.capture_adapter_mut()
                .unwrap()
                .display()
                .continuation
                .is_none()
        );
        host.capture_adapter_mut()
            .unwrap()
            .consume_execution_terminal()
            .unwrap();
        loop {
            if matches!(
                host.poll_unrun_shutdown().unwrap(),
                ViewerCloseProgress::Complete { .. }
            ) {
                break;
            }
            assert!(Instant::now() < end);
            std::thread::sleep(Duration::from_millis(1));
        }
        let cards = host.cards.borrow();
        let evidence = cards.capture_close_receipt.as_ref().unwrap();
        assert!(Arc::ptr_eq(
            &evidence.terminal.as_ref().unwrap().response,
            &raw.response
        ));
        assert_eq!(evidence.disposition, CaptureExecutionDisposition::Consumed);
        drop(cards);
        assert_immutable(rig, &project);
    })
}

#[test]
#[ignore = "Explicit positive guarded named-pipe/owner slot; concrete worker catches client panic"]
fn worker_client_panic_after_backend_commit_preserves_uncertainty_without_refresh_or_resubmit()
-> Result<(), Box<GuardedFailure>> {
    with_rig(|host, rig| {
        let project = seed(&rig.client);
        enter(host, project.clone());
        rig.fault_after_guard.store(1, Ordering::SeqCst);
        let edit = host
            .capture_adapter_mut()
            .unwrap()
            .edit(draft("Committed before client panic"))
            .unwrap();
        dispatch(host, edit);
        terminal(host);
        let evidence = consume_ack(host);
        let raw = evidence.terminal.unwrap();
        let error = raw.response.error.as_ref().unwrap();
        assert_eq!(error.code, ErrorCode::EngineFailed);
        assert_eq!(error.details["execution_may_have_occurred"], true);
        assert!(
            host.capture_adapter_mut()
                .unwrap()
                .display()
                .continuation
                .is_none()
        );
        let retry = host
            .capture_adapter_mut()
            .unwrap()
            .edit(draft("Explicit stale retry"))
            .unwrap();
        dispatch(host, retry);
        terminal(host);
        let retried = consume_ack(host);
        assert_eq!(
            retried
                .terminal
                .unwrap()
                .response
                .error
                .as_ref()
                .unwrap()
                .code,
            ErrorCode::RevisionConflict
        );
        assert_immutable(rig, &project);
        let responses = rig.responses.lock().unwrap().clone();
        assert!(
            responses
                .iter()
                .any(|r| r["ok"] == true && r["data"]["continuation"]["draft_revision"] == "2")
        );
    })
}

#[test]
#[ignore = "Explicit positive guarded named-pipe/owner slot; consumer decode fault retains actual success"]
fn consumer_decode_failure_retains_original_operation_until_explicit_discard()
-> Result<(), Box<GuardedFailure>> {
    with_rig(|host, rig| {
        let project = seed(&rig.client);
        enter(host, project.clone());
        rig.fault_after_guard.store(2, Ordering::SeqCst);
        let edit = host
            .capture_adapter_mut()
            .unwrap()
            .edit(draft("Committed before damaged UI response"))
            .unwrap();
        dispatch(host, edit);
        terminal(host);
        let mut capture = host.capture_adapter_mut().unwrap();
        let raw = capture.execution_terminal().unwrap().clone();
        assert!(raw.response.ok);
        assert!(capture.consume_execution_terminal().is_err());
        assert!(capture.execution_fault().is_some());
        assert!(capture.display().continuation.is_none());
        assert!(capture.release_execution().is_err());
        capture
            .discard_execution_terminal("explicit damaged terminal discard".into())
            .unwrap();
        let evidence = capture.acknowledge_execution_terminal().unwrap();
        assert!(Arc::ptr_eq(evidence.terminal.as_ref().unwrap(), &raw));
        assert!(evidence.first_fault.is_some());
        drop(capture);
        assert_immutable(rig, &project);
        let responses = rig.responses.lock().unwrap().clone();
        assert!(
            responses
                .iter()
                .any(|r| r["ok"] == true && r["data"]["continuation"]["draft_revision"] == "2")
        );
    })
}

#[test]
#[ignore = "Explicit positive guarded named-pipe/owner slot; old selection binding remains readonly"]
fn new_origin_does_not_activate_legacy_selection_binding() -> Result<(), Box<GuardedFailure>> {
    with_rig(|host, rig| {
        let project = seed(&rig.client);
        let event = button(host, project.clone(), CARD_SELECTION_BINDING);
        let stamp = host.document.as_ref().unwrap().stamp().unwrap();
        let before = requests(&rig.requests).len();
        assert!(matches!(
            host.admit_callback(&stamp, &event),
            IntentAdmission::Rejected(_)
        ));
        assert_eq!(requests(&rig.requests).len(), before);
        assert!(host.card_selection_queue().unwrap().borrow().is_empty());
        assert_immutable(rig, &project);
    })
}

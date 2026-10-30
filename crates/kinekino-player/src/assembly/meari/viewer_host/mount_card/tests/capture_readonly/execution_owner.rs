use super::*;
use crate::{
    application::proposals::guarded::CaptureAnchor,
    domain::projects::canonical_sha256,
    presentation::web_ui_adapter::proposals::capture_mutation::{
        CaptureDispatchProgress, CaptureExecutionDisposition, CaptureExecutionProgress,
        CaptureMutationAdapter,
    },
};

fn selected_adapter(host: &ViewerHost, scope: &PluginScope) -> CaptureMutationAdapter {
    let queue = enqueue_legacy_capture(host, scope);
    let selected = queue.borrow().front().unwrap().clone();
    let project = selected.project.as_ref();
    let snapshot = &project.proposals[0];
    CaptureMutationAdapter::new(
        CaptureAnchor {
            project_id: project.project_id.clone(),
            project_revision: project.revision,
            project_revision_sha256: canonical_sha256(project).unwrap(),
            proposal_id: snapshot.captured.document.proposal_id.clone(),
            capture_sha256: snapshot.capture_sha256.clone(),
        },
        snapshot.captured.clone(),
    )
}

#[test]
#[ignore = "Requires an explicit LocalHost ownership slot; no window or renderer"]
fn capture_attach_failure_returns_original_adapter_without_releasing_read_owner()
-> Result<(), Box<RetainedCaseFailure>> {
    let ((input, owner, gate), callbacks) = fixture_with_capture_choices(true);
    let scope = input.document.stamp().unwrap().owner;
    let execution = owner.handle();
    let host = host(&input, owner, callbacks.clone());
    run_retaining_case(host, |host| {
        let bindings = callbacks
            .borrow_mut()
            .resume_released_route(&scope)
            .unwrap();
        input
            .document
            .try_reproject(&input.document.stamp().unwrap(), |document| {
                let props = input.frame.workspace_props(
                    document,
                    WorkspaceRoute::Viewing,
                    NonZeroU64::new(1).unwrap(),
                    read_props(&StartLabels {
                        connection: "Not observed".into(),
                        time: "Not observed".into(),
                    }),
                )?;
                document.update_module_props_with_callbacks(props, bindings)
            })
            .unwrap();
        assert!(!callbacks.borrow().is_released());
        assert!(execution.attach().is_err());
        let adapter = selected_adapter(host, &scope);
        let anchor = serde_json::to_value(adapter.display().anchor).unwrap();
        let before = gate.all_calls.lock().unwrap().clone();
        let rejected = match host.attach_capture_adapter(adapter) {
            Err(rejected) => rejected,
            Ok(()) => panic!("Existing read attachment was replaced"),
        };
        assert_eq!(
            serde_json::to_value(rejected.adapter.display().anchor).unwrap(),
            anchor
        );
        assert!(!rejected.adapter.execution_attached());
        assert!(!callbacks.borrow().is_released());
        assert!(execution.attach().is_err());
        let after = gate.all_calls.lock().unwrap().clone();
        assert_eq!(after, before);
    })
}

#[test]
#[ignore = "Requires an explicit LocalHost ownership slot; unsupported backend, no window"]
fn original_host_close_retains_capture_terminal_until_explicit_discard_and_ack()
-> Result<(), Box<RetainedCaseFailure>> {
    let ((input, owner, gate), callbacks) = fixture_with_capture_choices(true);
    let scope = input.document.stamp().unwrap().owner;
    let execution = owner.handle();
    let host = host(&input, owner, callbacks.clone());
    run_retaining_case(host, |host| {
        let adapter = selected_adapter(host, &scope);
        let initial = serde_json::to_value(adapter.display().state).unwrap();
        assert!(
            callbacks
                .borrow_mut()
                .prepare_route_release(&scope)
                .unwrap()
                .is_none()
        );
        assert!(callbacks.borrow().is_released());
        if let Err(rejected) = host.attach_capture_adapter(adapter) {
            panic!("Original attachment handoff failed: {:?}", rejected.error);
        }
        {
            let mut capture = host.capture_adapter_mut().unwrap();
            let draft = capture.display().state.document.draft.clone().unwrap();
            let edit = capture.edit(draft).unwrap();
            assert!(matches!(
                capture.dispatch_to_execution(edit, RequestContext::default()),
                Ok(CaptureDispatchProgress::Submitted(_))
            ));
        }
        let end = Instant::now() + Duration::from_secs(3);
        loop {
            if matches!(
                host.capture_adapter_mut()
                    .unwrap()
                    .poll_execution()
                    .unwrap(),
                CaptureExecutionProgress::TerminalReady { .. }
            ) {
                break;
            }
            assert!(
                Instant::now() < end,
                "Original capture response did not arrive"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        let raw = host
            .capture_adapter_mut()
            .unwrap()
            .execution_terminal()
            .unwrap()
            .clone();
        // No configured ProjectStore: this proves explicit rejection, not positive guard authority.
        assert_eq!(
            raw.response.error.as_ref().unwrap().code,
            crate::application::api::ErrorCode::Unsupported
        );
        loop {
            let progress = host.poll_unrun_shutdown().unwrap();
            if matches!(progress, ViewerCloseProgress::CardScopePending) {
                break;
            }
            assert!(!matches!(progress, ViewerCloseProgress::Complete { .. }));
            assert!(
                Instant::now() < end,
                "Original endpoint close did not reach capture barrier"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(execution.retained_operations().unwrap().len(), 1);
        assert!(host.capture_adapter_mut().unwrap().execution_attached());
        {
            let mut capture = host.capture_adapter_mut().unwrap();
            assert_eq!(
                serde_json::to_value(capture.display().state).unwrap(),
                initial
            );
            assert!(capture.display().continuation.is_none());
            capture
                .discard_execution_terminal("explicit unsupported capture discard".into())
                .unwrap();
        }
        loop {
            if matches!(
                host.poll_unrun_shutdown().unwrap(),
                ViewerCloseProgress::Complete { .. }
            ) {
                break;
            }
            assert!(
                Instant::now() < end,
                "Original capture owner did not close after explicit discard"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        let cards = host.cards.borrow();
        let receipt = cards.capture_close_receipt.as_ref().unwrap();
        assert!(Arc::ptr_eq(
            &receipt.terminal.as_ref().unwrap().response,
            &raw.response
        ));
        assert!(matches!(
            receipt.disposition,
            CaptureExecutionDisposition::Discarded { .. }
        ));
        assert!(!cards.capture.as_ref().unwrap().execution_attached());
        assert!(execution.retained_operations().unwrap().is_empty());
        let commands = gate.all_calls.lock().unwrap().clone();
        assert_eq!(
            commands
                .iter()
                .filter(|c| c.as_str() == "proposals.edit_capture")
                .count(),
            1
        );
        assert!(!commands.iter().any(|c| c == "proposals.edit"));
    })
}

use super::super::tests::{RecordingClient, anchor, payload, result, state};
use super::*;
mod fixture;
use fixture::*;
use serde_json::to_value;

#[test]
fn actual_worker_retains_original_request_until_consume_ack_release() -> Result<(), Box<Failure>> {
    with_original(
        Client::new(Mode::Normal, vec![Ok(result(2))]),
        budget(),
        |adapter, owner, client| {
            let edit = adapter.edit(payload("worker edit")).unwrap();
            dispatch(adapter, edit);
            terminal(adapter);
            let raw = adapter.execution_terminal().unwrap().clone();
            assert_eq!(raw.request.command, "proposals.edit_capture");
            assert_eq!(
                raw.request.payload["anchor"],
                to_value(anchor("original")).unwrap()
            );
            assert!(raw.request.payload["expected"].is_null());
            assert_eq!(adapter.display().state.document.draft_revision.0, 1);
            assert!(adapter.release_execution().is_err());
            assert_eq!(
                owner.poll_close().unwrap_err().code,
                ErrorCode::PermissionDenied
            );
            assert!(adapter.acknowledge_execution_terminal().is_err());
            assert!(matches!(
                adapter.edit(payload("cannot replace")),
                Err(CaptureViewError::Busy)
            ));
            assert_eq!(
                adapter.consume_execution_terminal().unwrap(),
                CaptureExecutionProgress::ConsumedAckPending
            );
            assert_eq!(adapter.display().continuation.unwrap().draft_revision.0, 2);
            assert_eq!(
                adapter.consume_execution_terminal().unwrap(),
                CaptureExecutionProgress::ConsumedAckPending
            );
            assert!(adapter.release_execution().is_err());
            let evidence = adapter.acknowledge_execution_terminal().unwrap();
            assert!(Arc::ptr_eq(evidence.terminal.as_ref().unwrap(), &raw));
            assert_eq!(evidence.disposition, CaptureExecutionDisposition::Consumed);
            assert!(owner.handle().retained_operations().unwrap().is_empty());
            assert_eq!(client.inner.payloads().len(), 1);
            adapter.release_execution().unwrap();
        },
    )
}

#[test]
fn admission_failure_keeps_exact_request_for_retry_or_unsubmitted_discard()
-> Result<(), Box<Failure>> {
    let mut small = budget();
    small.max_request_bytes = 4;
    with_original(
        Client::new(Mode::Normal, vec![]),
        small,
        |adapter, owner, client| {
            let edit = adapter.edit(payload("oversize")).unwrap();
            assert!(
                matches!(adapter.dispatch_to_execution(edit, RequestContext::default()),
            Ok(CaptureDispatchProgress::Retained(error)) if error.code == ErrorCode::ResourceExhausted)
            );
            assert!(adapter.execution_operation().is_none());
            assert_eq!(
                adapter.retry_execution_submission().unwrap_err().code,
                ErrorCode::ResourceExhausted
            );
            assert_eq!(client.inner.payloads().len(), 0);
            assert!(owner.handle().retained_operations().unwrap().is_empty());
            assert!(adapter.release_execution().is_err());
            adapter
                .discard_execution_terminal("explicit unsubmitted discard".into())
                .unwrap();
            let evidence = adapter.acknowledge_execution_terminal().unwrap();
            assert!(evidence.operation.is_none() && evidence.terminal.is_none());
            assert_eq!(
                evidence.request.payload["draft"],
                to_value(payload("oversize")).unwrap()
            );
            assert_eq!(
                evidence.first_fault.unwrap().code,
                ErrorCode::ResourceExhausted
            );
            assert!(adapter.display().continuation.is_none());
            adapter.release_execution().unwrap();
        },
    )
}

#[test]
fn running_cancel_keeps_actual_success_without_reapplication() -> Result<(), Box<Failure>> {
    with_original(
        Client::new(Mode::Blocked, vec![Ok(result(2))]),
        budget(),
        |adapter, _, client| {
            let edit = adapter.edit(payload("running mutation")).unwrap();
            dispatch(adapter, edit);
            client.entered();
            adapter.request_execution_close().unwrap();
            assert!(adapter.execution_closing());
            assert!(
                adapter
                    .discard_execution_terminal("too early".into())
                    .is_err()
            );
            client.unblock();
            terminal(adapter);
            let raw = adapter.execution_terminal().unwrap();
            assert!(raw.response.ok && raw.cancellation_requested && !raw.queued_cancelled);
            adapter.consume_execution_terminal().unwrap();
            assert_eq!(adapter.display().state.document.draft_revision.0, 2);
            adapter.consume_execution_terminal().unwrap();
            assert!(matches!(
                adapter.accept(DecimalU64(3)),
                Err(CaptureViewError::Busy)
            ));
            let evidence = adapter.acknowledge_execution_terminal().unwrap();
            assert!(evidence.terminal.unwrap().response.ok);
            assert_eq!(client.inner.payloads().len(), 1);
            adapter.release_execution().unwrap();
        },
    )
}

#[test]
fn invalid_terminal_preserves_raw_and_diagnostic_until_explicit_discard() -> Result<(), Box<Failure>>
{
    for mode in [Mode::MalformedData, Mode::WrongEnvelope] {
        with_original(
            Client::new(mode, vec![Ok(result(2))]),
            budget(),
            |adapter, _, _| {
                let edit = adapter.edit(payload("unreadable result")).unwrap();
                dispatch(adapter, edit);
                terminal(adapter);
                let raw = adapter.execution_terminal().unwrap().clone();
                assert!(adapter.consume_execution_terminal().is_err());
                assert!(adapter.execution_fault().is_some());
                assert!(adapter.display().continuation.is_none());
                assert!(adapter.release_execution().is_err());
                adapter
                    .discard_execution_terminal("explicit raw-result discard".into())
                    .unwrap();
                let evidence = adapter.acknowledge_execution_terminal().unwrap();
                assert!(Arc::ptr_eq(evidence.terminal.as_ref().unwrap(), &raw));
                assert!(evidence.first_fault.is_some());
                assert!(matches!(
                    evidence.disposition,
                    CaptureExecutionDisposition::Discarded { .. }
                ));
                adapter.release_execution().unwrap();
            },
        )?;
    }
    Ok(())
}

#[test]
fn terminal_budget_fault_keeps_actual_success_for_explicit_discard() -> Result<(), Box<Failure>> {
    let mut small = budget();
    small.max_terminal_bytes = 16;
    with_original(
        Client::new(Mode::Normal, vec![Ok(result(2))]),
        small,
        |adapter, _, _| {
            let intent = adapter.edit(payload("actual oversize response")).unwrap();
            dispatch(adapter, intent);
            terminal(adapter);
            let raw = adapter.execution_terminal().unwrap().clone();
            assert!(raw.measurement.is_faulted());
            assert!(raw.response.ok);
            assert!(adapter.consume_execution_terminal().is_err());
            assert!(adapter.display().continuation.is_none());
            adapter
                .discard_execution_terminal("explicit terminal budget discard".into())
                .unwrap();
            let evidence = adapter.acknowledge_execution_terminal().unwrap();
            assert!(Arc::ptr_eq(evidence.terminal.as_ref().unwrap(), &raw));
            assert!(evidence.first_fault.is_some());
            adapter.release_execution().unwrap();
        },
    )
}

#[test]
fn sdk_error_is_consumed_without_advancing_capture_expectation() -> Result<(), Box<Failure>> {
    let mut error = ApiError::new(ErrorCode::RevisionConflict, "original guard conflict");
    error.details = serde_json::json!({"original":"details"});
    error.remediation = "explicit selection".into();
    with_original(
        Client::new(Mode::Normal, vec![Err(error)]),
        budget(),
        |adapter, _, _| {
            let edit = adapter.edit(payload("stale edit")).unwrap();
            dispatch(adapter, edit);
            terminal(adapter);
            adapter.consume_execution_terminal().unwrap();
            let display = adapter.display();
            assert!(display.continuation.is_none());
            assert_eq!(display.state.document.draft_revision.0, 1);
            let error = display.sdk_error.unwrap();
            assert_eq!(error.code, ErrorCode::RevisionConflict);
            assert_eq!(error.details, serde_json::json!({"original":"details"}));
            assert_eq!(error.remediation, "explicit selection");
            let evidence = adapter.acknowledge_execution_terminal().unwrap();
            assert!(!evidence.terminal.unwrap().response.ok);
            adapter.release_execution().unwrap();
        },
    )
}

#[test]
fn close_preserves_prepared_intent_without_submitting_it() -> Result<(), Box<Failure>> {
    with_original(
        Client::new(Mode::Normal, vec![]),
        budget(),
        |adapter, _, client| {
            let intent = adapter.edit(payload("prepared before close")).unwrap();
            assert_eq!(
                adapter.request_execution_close().unwrap(),
                CaptureExecutionProgress::Idle
            );
            assert!(adapter.release_execution().is_err());
            assert!(matches!(
                adapter.dispatch_to_execution(intent, RequestContext::default()),
                Ok(CaptureDispatchProgress::Retained(_))
            ));
            assert!(adapter.execution_operation().is_none());
            adapter
                .discard_execution_terminal("explicit prepared-intent discard".into())
                .unwrap();
            let evidence = adapter.acknowledge_execution_terminal().unwrap();
            assert!(evidence.operation.is_none());
            assert_eq!(
                evidence.request.payload["draft"],
                to_value(payload("prepared before close")).unwrap()
            );
            assert_eq!(client.inner.payloads().len(), 0);
            adapter.release_execution().unwrap();
        },
    )
}

#[test]
fn foreign_intent_returns_intact_without_replacing_original_operation() -> Result<(), Box<Failure>>
{
    with_original(
        Client::new(Mode::Normal, vec![Ok(result(2))]),
        budget(),
        |adapter, _, _| {
            let local = adapter.edit(payload("local")).unwrap();
            dispatch(adapter, local);
            terminal(adapter);
            let original = adapter.execution_operation().unwrap().clone();
            let mut foreign = CaptureMutationAdapter::new(anchor("foreign"), state(1));
            let foreign_intent = foreign.edit(payload("foreign")).unwrap();
            let rejected =
                match adapter.dispatch_to_execution(foreign_intent, RequestContext::default()) {
                    Err(rejected) => rejected,
                    Ok(_) => panic!("Foreign intent admitted"),
                };
            assert!(Arc::ptr_eq(&rejected.intent.owner, &foreign.owner));
            assert_eq!(adapter.execution_operation(), Some(&original));
            adapter.consume_execution_terminal().unwrap();
            let _evidence = adapter.acknowledge_execution_terminal().unwrap();
            adapter.release_execution().unwrap();
        },
    )
}

#[test]
fn original_preaccept_retry_payload_survives_worker_consumption_and_ack() -> Result<(), Box<Failure>>
{
    with_original(
        Client::new(
            Mode::Normal,
            vec![Ok(result(2)), Ok(result(3)), Ok(result(3))],
        ),
        budget(),
        |adapter, _, client| {
            let edit = adapter.edit(payload("edited")).unwrap();
            dispatch(adapter, edit);
            terminal(adapter);
            adapter.consume_execution_terminal().unwrap();
            let _edit = adapter.acknowledge_execution_terminal().unwrap();
            let accept = adapter.accept(DecimalU64(3)).unwrap();
            dispatch(adapter, accept);
            terminal(adapter);
            adapter.consume_execution_terminal().unwrap();
            let _accept = adapter.acknowledge_execution_terminal().unwrap();
            let retry = adapter.retry_accept().unwrap();
            dispatch(adapter, retry);
            terminal(adapter);
            adapter.consume_execution_terminal().unwrap();
            let _retry = adapter.acknowledge_execution_terminal().unwrap();
            let requests = client.inner.payloads();
            assert_eq!(requests.len(), 3);
            assert_eq!(requests[1], requests[2]);
            assert_eq!(requests[2]["expected"]["draft_revision"], "2");
            assert_eq!(adapter.display().continuation.unwrap().draft_revision.0, 3);
            adapter.release_execution().unwrap();
        },
    )
}

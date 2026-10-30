use super::*;
mod close_progress;
#[cfg(windows)]
mod cooperative_frame;
mod cooperative_model;
#[cfg(windows)]
mod frame_publication;
#[cfg(windows)]
mod frame_recovery;
mod host;
mod initial;
mod recovery;
mod submission;
use crate::application::api::{API_VERSION, ApplicationClient};
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

struct Notify {
    signals: AtomicUsize,
    fail: bool,
}
impl CompletionNotifier for Notify {
    fn notify(&self) -> Result<(), ApiError> {
        self.signals.fetch_add(1, Ordering::SeqCst);
        if self.fail {
            Err(internal("test wake unavailable"))
        } else {
            Ok(())
        }
    }
}
struct Client {
    calls: AtomicUsize,
    entered: mpsc::Sender<std::thread::ThreadId>,
    gate: (Mutex<bool>, Condvar),
    output_bytes: usize,
}
impl ApplicationClient for Client {
    fn execute(&self, request: Request) -> Response {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.entered.send(std::thread::current().id()).unwrap();
        let mut ready = self.gate.0.lock().unwrap();
        while !*ready {
            ready = self.gate.1.wait(ready).unwrap();
        }
        Response {
            api_version: API_VERSION,
            request_id: request.context.request_id,
            ok: true,
            data: Some(
                serde_json::json!({"command":request.command,"payload":request.payload,"text":"x".repeat(self.output_bytes)}),
            ),
            error: None,
        }
    }
}
fn budget() -> ExecutionBudget {
    ExecutionBudget {
        max_operations: 2,
        max_request_bytes: 4096,
        max_total_request_bytes: 16384,
        max_terminal_bytes: 4096,
        max_total_terminal_bytes: 8192,
    }
}
fn setup(
    output_bytes: usize,
    fail_notify: bool,
) -> (
    ExecutionOwner,
    Arc<Client>,
    mpsc::Receiver<std::thread::ThreadId>,
) {
    let (tx, rx) = mpsc::channel();
    let client = Arc::new(Client {
        calls: AtomicUsize::new(0),
        entered: tx,
        gate: (Mutex::new(false), Condvar::new()),
        output_bytes,
    });
    let owner = ExecutionOwner::start(
        client.clone(),
        budget(),
        Arc::new(Notify {
            signals: AtomicUsize::new(0),
            fail: fail_notify,
        }),
    )
    .unwrap();
    (owner, client, rx)
}
fn release(client: &Client) {
    *client.gate.0.lock().unwrap() = true;
    client.gate.1.notify_all();
}
fn terminal(handle: &ExecutionHandle, id: &OperationId) -> Arc<ExecutionTerminal> {
    let until = Instant::now() + Duration::from_secs(3);
    loop {
        if let ExecutionSnapshot::Terminal(t) = handle.peek(id).unwrap() {
            return t;
        }
        assert!(Instant::now() < until);
        std::thread::sleep(Duration::from_millis(1));
    }
}
fn close(handle: &ExecutionHandle) {
    let until = Instant::now() + Duration::from_secs(3);
    loop {
        if let Ok(receipt) = handle.close() {
            assert!(receipt.worker_stopped);
            assert_eq!(receipt.unacknowledged, 0);
            return;
        }
        assert!(Instant::now() < until);
        std::thread::sleep(Duration::from_millis(1));
    }
}
#[test]
fn worker_preserves_context_terminal_and_capacity_until_ack() {
    let (owner, client, entered) = setup(0, false);
    let handle = owner.handle();
    let mut request = Request::new("proposals.inspect");
    request.context.request_id = "원본-문맥".into();
    request.context.expected_revision = Some("7".into());
    request.context.idempotency_key = Some("preserved".into());
    request.context.timeout_ms = Some(123);
    request.payload = serde_json::json!({"proposal_id":"explicit"});
    let id = handle.submit(request.clone()).unwrap();
    assert_ne!(
        entered.recv_timeout(Duration::from_secs(3)).unwrap(),
        std::thread::current().id()
    );
    release(&client);
    let first = terminal(&handle, &id);
    let second = terminal(&handle, &id);
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(
        first.request.context.expected_revision,
        request.context.expected_revision
    );
    assert_eq!(
        first.request.context.idempotency_key,
        request.context.idempotency_key
    );
    assert_eq!(first.request.context.timeout_ms, request.context.timeout_ms);
    assert_eq!(first.response.request_id, "원본-문맥");
    assert_eq!(client.calls.load(Ordering::SeqCst), 1);
    assert_eq!(handle.close().unwrap_err().code, ErrorCode::NeedsInput);
    assert!(!handle.acknowledge(&id).unwrap().already_acknowledged);
    assert!(handle.acknowledge(&id).unwrap().already_acknowledged);
    close(&handle);
}
#[test]
fn queued_cancel_has_zero_effect_and_running_cancel_retains_actual_success() {
    let (owner, client, entered) = setup(0, false);
    let handle = owner.handle();
    let first = handle.submit(Request::new("proposals.edit")).unwrap();
    entered.recv_timeout(Duration::from_secs(3)).unwrap();
    let queued = handle
        .submit(Request::new("proposals.import_historical"))
        .unwrap();
    assert_eq!(
        handle
            .submit(Request::new("proposals.inspect"))
            .unwrap_err()
            .code,
        ErrorCode::ResourceExhausted
    );
    assert_eq!(
        handle.cancel(&queued).unwrap(),
        CancelDisposition::QueuedCancelled
    );
    let cancelled = terminal(&handle, &queued);
    assert!(cancelled.queued_cancelled);
    assert_eq!(
        cancelled.response.error.as_ref().unwrap().code,
        ErrorCode::Cancelled
    );
    assert_eq!(
        cancelled.response.error.as_ref().unwrap().details["execution_started"],
        false
    );
    assert_eq!(handle.cancel(&first).unwrap(), CancelDisposition::Requested);
    release(&client);
    let actual = terminal(&handle, &first);
    assert!(actual.response.ok);
    assert!(actual.cancellation_requested);
    assert!(!actual.queued_cancelled);
    assert_eq!(client.calls.load(Ordering::SeqCst), 1);
    assert_eq!(handle.cancel(&first).unwrap(), CancelDisposition::Requested);
    handle.acknowledge(&first).unwrap();
    handle.acknowledge(&queued).unwrap();
    assert_eq!(
        handle.cancel(&first).unwrap(),
        CancelDisposition::AlreadyAcknowledged
    );
    close(&handle);
}
#[test]
fn invalid_or_oversize_request_is_rejected_before_sdk_and_foreign_owner_is_stale() {
    let (owner, client, _) = setup(0, false);
    let handle = owner.handle();
    let mut invalid = Request::new("proposals.inspect");
    invalid.context.request_id.clear();
    assert_eq!(
        handle.submit(invalid).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    let mut large = Request::new("proposals.create");
    large.payload = serde_json::json!({"raw":"x".repeat(4097)});
    assert_eq!(
        handle.submit(large).unwrap_err().code,
        ErrorCode::ResourceExhausted
    );
    assert_eq!(client.calls.load(Ordering::SeqCst), 0);
    let (other, other_client, other_entered) = setup(0, false);
    let other_handle = other.handle();
    let id = other_handle
        .submit(Request::new("proposals.inspect"))
        .unwrap();
    other_entered.recv_timeout(Duration::from_secs(3)).unwrap();
    assert_eq!(handle.peek(&id).unwrap_err().code, ErrorCode::SourceExpired);
    release(&other_client);
    terminal(&other_handle, &id);
    other_handle.acknowledge(&id).unwrap();
    close(&other_handle);
    close(&handle);
}
#[test]
fn wake_failure_and_owner_drop_preserve_unacknowledged_actual_result() {
    let (owner, client, entered) = setup(0, true);
    let handle = owner.handle();
    let id = handle.submit(Request::new("proposals.accept")).unwrap();
    entered.recv_timeout(Duration::from_secs(3)).unwrap();
    drop(owner);
    assert!(handle.close().is_err());
    release(&client);
    let result = terminal(&handle, &id);
    assert!(result.response.ok);
    assert_eq!(client.calls.load(Ordering::SeqCst), 1);
    let until = Instant::now() + Duration::from_secs(3);
    while handle.notification_error().unwrap().is_none() {
        assert!(Instant::now() < until);
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(handle.close().is_err());
    handle.acknowledge(&id).unwrap();
    close(&handle);
}
#[test]
fn unexpected_large_result_is_preserved_as_fault_and_stops_further_sdk_work() {
    let (owner, client, entered) = setup(8192, false);
    let handle = owner.handle();
    let first = handle.submit(Request::new("proposals.inspect")).unwrap();
    entered.recv_timeout(Duration::from_secs(3)).unwrap();
    let queued = handle.submit(Request::new("proposals.edit")).unwrap();
    release(&client);
    let result = terminal(&handle, &first);
    assert!(result.measurement.is_faulted());
    assert!(result.response.ok);
    assert_eq!(
        result.response.data.as_ref().unwrap()["text"]
            .as_str()
            .unwrap()
            .len(),
        8192
    );
    assert_eq!(result.measurement, EncodingMeasurement::LowerBound(4097));
    assert!(terminal(&handle, &queued).queued_cancelled);
    assert_eq!(client.calls.load(Ordering::SeqCst), 1);
    assert!(handle.submit(Request::new("proposals.inspect")).is_err());
    handle.acknowledge(&first).unwrap();
    handle.acknowledge(&queued).unwrap();
    close(&handle);
}

#[test]
fn encoding_measurement_distinguishes_exact_lower_bound_and_serialization_failure() {
    assert_eq!(
        admission::encoded_size(&"abc", 5),
        EncodingMeasurement::Exact(5)
    );
    assert_eq!(
        admission::encoded_size(&"abc", 4),
        EncodingMeasurement::LowerBound(5)
    );
    struct Broken;
    impl serde::Serialize for Broken {
        fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom(
                "explicit serialization fault fixture",
            ))
        }
    }
    assert_eq!(
        admission::encoded_size(&Broken, 100),
        EncodingMeasurement::SerializationFailed
    );
}

#[test]
fn released_attachment_cannot_recover_authority_or_revoke_its_successor() {
    let (owner, client, entered) = setup(0, false);
    let handle = owner.handle();
    let previous = handle.attach().unwrap();
    previous.release().unwrap();
    assert_eq!(
        previous
            .submit(Request::new("proposals.edit"))
            .unwrap_err()
            .code,
        ErrorCode::PermissionDenied
    );
    assert_eq!(client.calls.load(Ordering::SeqCst), 0);
    let current = handle.attach().unwrap();
    let id = current.submit(Request::new("proposals.inspect")).unwrap();
    entered.recv_timeout(Duration::from_secs(3)).unwrap();
    assert_eq!(
        previous.peek(&id).unwrap_err().code,
        ErrorCode::PermissionDenied
    );
    assert_eq!(
        previous.cancel(&id).unwrap_err().code,
        ErrorCode::PermissionDenied
    );
    assert_eq!(
        previous.acknowledge(&id).unwrap_err().code,
        ErrorCode::PermissionDenied
    );
    previous.release().unwrap();
    drop(previous);
    assert_eq!(current.release().unwrap_err().code, ErrorCode::NeedsInput);
    release(&client);
    terminal(&handle, &id);
    assert!(matches!(
        current.peek(&id).unwrap(),
        ExecutionSnapshot::Terminal(_)
    ));
    current.acknowledge(&id).unwrap();
    current.release().unwrap();
    assert_eq!(client.calls.load(Ordering::SeqCst), 1);
    close(&handle);
}

#[test]
fn release_and_submission_share_one_authority_gate() {
    let (owner, client, _entered) = setup(0, false);
    let handle = owner.handle();
    release(&client);
    for _ in 0..32 {
        let attachment = handle.attach().unwrap();
        let barrier = std::sync::Barrier::new(2);
        let (released, submitted) = std::thread::scope(|scope| {
            let releasing = scope.spawn(|| {
                barrier.wait();
                attachment.release()
            });
            barrier.wait();
            let submitted = attachment.submit(Request::new("proposals.inspect"));
            (releasing.join().unwrap(), submitted)
        });
        match (released, submitted) {
            (Ok(()), Err(error)) => assert_eq!(error.code, ErrorCode::PermissionDenied),
            (Err(error), Ok(id)) => {
                assert_eq!(error.code, ErrorCode::NeedsInput);
                terminal(&handle, &id);
                attachment.acknowledge(&id).unwrap();
                attachment.release().unwrap();
            }
            _ => panic!("release and admission must have one linearized winner"),
        }
    }
    close(&handle);
}

#[test]
fn concurrent_release_is_idempotent_and_keeps_successor_authority() {
    let (owner, _client, _entered) = setup(0, false);
    let handle = owner.handle();
    for _ in 0..32 {
        let attachment = handle.attach().unwrap();
        let barrier = std::sync::Barrier::new(2);
        std::thread::scope(|scope| {
            let releasing = scope.spawn(|| {
                barrier.wait();
                attachment.release().unwrap();
            });
            barrier.wait();
            attachment.release().unwrap();
            releasing.join().unwrap();
        });
        let successor = handle.attach().unwrap();
        drop(attachment);
        successor.release().unwrap();
    }
    close(&handle);
}

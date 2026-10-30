use super::*;
use crate::{
    application::{
        api::PlayerSdk,
        sdk_execution::{
            CancelDisposition, CompletionNotifier, ExecutionBudget, ExecutionHandle,
            ExecutionOwner, ExecutionSnapshot, ExecutionTerminal, OperationId,
        },
    },
    infrastructure::local_host::LocalHost,
};
use std::{
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

struct Gate {
    entered: mpsc::Sender<()>,
    release: (Mutex<bool>, Condvar),
}
impl Gate {
    fn pause(&self) {
        self.entered.send(()).unwrap();
        let (released, timeout) = self
            .release
            .1
            .wait_timeout_while(
                self.release.0.lock().unwrap(),
                Duration::from_secs(10),
                |r| !*r,
            )
            .unwrap();
        assert!(
            !timeout.timed_out() || *released,
            "test gate release expired"
        );
    }
}
struct Release(Arc<Gate>);
impl Drop for Release {
    fn drop(&mut self) {
        *self.0.release.0.lock().unwrap() = true;
        self.0.release.1.notify_all();
    }
}
fn gate() -> (Arc<Gate>, mpsc::Receiver<()>) {
    let (entered, receiver) = mpsc::channel();
    (
        Arc::new(Gate {
            entered,
            release: (Mutex::new(false), Condvar::new()),
        }),
        receiver,
    )
}
struct Server {
    calls: Arc<AtomicUsize>,
    gate: Option<Arc<Gate>>,
}
impl ApplicationClient for Server {
    fn execute(&self, request: Request) -> Response {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if let Some(gate) = &self.gate {
            gate.pause();
        }
        PlayerSdk::default().execute(request)
    }
}
struct Hooked {
    client: EndpointClient,
    gate: Arc<Gate>,
}
impl ApplicationClient for Hooked {
    fn execute(&self, request: Request) -> Response {
        self.client.execute(request)
    }
    fn execute_controlled(
        &self,
        request: Request,
        control: &(dyn MutationControl + Sync),
    ) -> Response {
        let gate = self.gate.clone();
        self.client
            .request_controlled_with_hook(&request, Some(control), move || gate.pause())
            .unwrap_or_else(|error| Response::failure(request.context.request_id, error))
    }
}
struct Notify;
impl CompletionNotifier for Notify {
    fn notify(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
fn owner(client: Arc<dyn ApplicationClient>) -> ExecutionOwner {
    ExecutionOwner::start(
        client,
        ExecutionBudget {
            max_operations: 2,
            max_request_bytes: 4096,
            max_total_request_bytes: 8192,
            max_terminal_bytes: 4096,
            max_total_terminal_bytes: 8192,
        },
        Arc::new(Notify),
    )
    .unwrap()
}
fn options() -> TransportOptions {
    TransportOptions {
        timeout: Duration::from_secs(10),
    }
}
fn terminal(handle: &ExecutionHandle, id: &OperationId) -> Arc<ExecutionTerminal> {
    let until = Instant::now() + Duration::from_secs(10);
    loop {
        if let ExecutionSnapshot::Terminal(terminal) = handle.peek(id).unwrap() {
            return terminal;
        }
        assert!(
            Instant::now() < until,
            "original SDK operation terminal expired"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}
fn finish_host(host: LocalHost) {
    let mut pending = host.begin_shutdown();
    let until = Instant::now() + Duration::from_secs(10);
    loop {
        if pending.poll().is_quiescent() {
            break;
        }
        assert!(Instant::now() < until, "original host worker joins expired");
        std::thread::sleep(Duration::from_millis(5));
    }
}
fn finish(owner: ExecutionOwner, handle: ExecutionHandle, id: OperationId, host: LocalHost) {
    handle.acknowledge(&id).unwrap();
    let until = Instant::now() + Duration::from_secs(10);
    loop {
        if owner.close().is_ok() {
            break;
        }
        assert!(Instant::now() < until, "original SDK worker join expired");
        std::thread::sleep(Duration::from_millis(5));
    }
    finish_host(host);
}

#[test]
fn cancellation_after_pipe_open_before_transmission_is_not_sent_and_host_can_resume() {
    let calls = Arc::new(AtomicUsize::new(0));
    let host = LocalHost::start(
        Arc::new(Server {
            calls: calls.clone(),
            gate: None,
        }),
        options(),
    )
    .unwrap();
    let (gate, entered) = gate();
    let release = Release(gate.clone());
    let owner = owner(Arc::new(Hooked {
        client: EndpointClient::new(host.endpoint(), options()).unwrap(),
        gate,
    }));
    let handle = owner.handle();
    let id = handle.submit(Request::new("version")).unwrap();
    entered.recv_timeout(Duration::from_secs(10)).unwrap();
    assert_eq!(handle.cancel(&id).unwrap(), CancelDisposition::Requested);
    drop(release);
    let result = terminal(&handle, &id);
    assert!(result.cancellation_requested);
    assert!(!result.queued_cancelled);
    let error = result.response.error.as_ref().unwrap();
    assert_eq!(error.code, ErrorCode::Cancelled);
    assert_eq!(error.details["delivery"], "not_sent");
    assert_eq!(error.details["automatic_retry"], false);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let response = EndpointClient::new(host.endpoint(), options())
        .unwrap()
        .execute(Request::new("version"));
    assert!(response.ok, "{response:?}");
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    eprintln!(
        "pre-transmission cancellation: server calls0, explicit next request1; original terminal={:?}",
        result.response
    );
    finish(owner, handle, id, host);
}

#[test]
fn cancellation_after_server_dispatch_preserves_actual_success_and_never_replays() {
    let calls = Arc::new(AtomicUsize::new(0));
    let (gate, entered) = gate();
    let release = Release(gate.clone());
    let host = LocalHost::start(
        Arc::new(Server {
            calls: calls.clone(),
            gate: Some(gate),
        }),
        options(),
    )
    .unwrap();
    let owner = owner(Arc::new(
        EndpointClient::new(host.endpoint(), options()).unwrap(),
    ));
    let handle = owner.handle();
    let id = handle.submit(Request::new("version")).unwrap();
    entered.recv_timeout(Duration::from_secs(10)).unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(handle.cancel(&id).unwrap(), CancelDisposition::Requested);
    drop(release);
    let result = terminal(&handle, &id);
    assert!(result.cancellation_requested);
    assert!(!result.queued_cancelled);
    assert!(result.response.ok, "{:?}", result.response);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    eprintln!(
        "post-dispatch cancellation: original success retained, server calls1; terminal={:?}",
        result.response
    );
    finish(owner, handle, id, host);
}

#[test]
fn pre_cancelled_endpoint_control_never_dispatches() {
    struct Cancelled;
    impl MutationControl for Cancelled {
        fn checkpoint(&self) -> Result<(), ApiError> {
            Err(ApiError::new(ErrorCode::Cancelled, "test cancel"))
        }
    }
    let calls = Arc::new(AtomicUsize::new(0));
    let host = LocalHost::start(
        Arc::new(Server {
            calls: calls.clone(),
            gate: None,
        }),
        options(),
    )
    .unwrap();
    let response = EndpointClient::new(host.endpoint(), options())
        .unwrap()
        .execute_controlled(Request::new("version"), &Cancelled);
    assert_eq!(response.error.as_ref().unwrap().code, ErrorCode::Cancelled);
    assert_eq!(
        response.error.as_ref().unwrap().details["delivery"],
        "not_sent"
    );
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    finish_host(host);
}

use super::*;
use kinekino_player::application::{
    api::{ApplicationClient, Response},
    sdk_execution::{CompletionNotifier, ExecutionBudget, ExecutionCloseProgress, ExecutionOwner},
};
use std::{
    any::Any,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
pub(super) enum Mode {
    Normal,
    Blocked,
    MalformedData,
    WrongEnvelope,
}
pub(super) struct Client {
    pub(super) inner: RecordingClient,
    mode: Mode,
    entered: AtomicBool,
    release: (Mutex<bool>, Condvar),
}
impl Client {
    pub(super) fn new(
        mode: Mode,
        responses: Vec<Result<CaptureMutationResult, ApiError>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            inner: RecordingClient::new(responses),
            mode,
            entered: AtomicBool::new(false),
            release: (Mutex::new(false), Condvar::new()),
        })
    }
    fn release(&self) {
        *self.release.0.lock().unwrap() = true;
        self.release.1.notify_all();
    }
    pub(super) fn entered(&self) {
        let end = Instant::now() + Duration::from_secs(3);
        while !self.entered.load(Ordering::Acquire) {
            assert!(Instant::now() < end, "Original worker did not enter");
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    pub(super) fn unblock(&self) {
        self.release();
    }
}
impl ApplicationClient for Client {
    fn execute(&self, request: ApiRequest) -> Response {
        self.entered.store(true, Ordering::Release);
        if matches!(self.mode, Mode::Blocked) {
            let mut released = self.release.0.lock().unwrap();
            while !*released {
                released = self.release.1.wait(released).unwrap();
            }
        }
        let mut response = self.inner.execute(request);
        match self.mode {
            Mode::MalformedData => {
                response.data = Some(serde_json::json!({"not":"capture result"}))
            }
            Mode::WrongEnvelope => response.request_id = "foreign-response".into(),
            _ => {}
        }
        response
    }
}
struct Notify;
impl CompletionNotifier for Notify {
    fn notify(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
pub(super) fn budget() -> ExecutionBudget {
    ExecutionBudget {
        max_operations: 2,
        max_request_bytes: 262144,
        max_total_request_bytes: 1048576,
        max_terminal_bytes: 262144,
        max_total_terminal_bytes: 524288,
    }
}
pub(super) fn dispatch(adapter: &mut CaptureMutationAdapter, intent: CaptureIntent) {
    match adapter.dispatch_to_execution(intent, RequestContext::default()) {
        Ok(CaptureDispatchProgress::Submitted(_)) => {}
        Ok(CaptureDispatchProgress::Retained(error)) => panic!("Admission retained: {error:?}"),
        Err(rejected) => panic!("Intent rejected: {:?}", rejected.error),
    }
}
pub(super) fn terminal(adapter: &mut CaptureMutationAdapter) {
    let end = Instant::now() + Duration::from_secs(3);
    loop {
        if matches!(
            adapter.poll_execution().unwrap(),
            CaptureExecutionProgress::TerminalReady { .. }
        ) {
            break;
        }
        assert!(
            Instant::now() < end,
            "Original capture terminal did not arrive"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}
pub(super) struct Failure {
    _owner: ExecutionOwner,
    _adapter: CaptureMutationAdapter,
    _evidence: Vec<CaptureExecutionAcknowledgement>,
    primary: Option<Box<dyn Any + Send>>,
    cleanup_error: Option<Arc<ApiError>>,
    closed: bool,
}
impl std::fmt::Debug for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OriginalCaptureCaseFailure")
            .field(
                "primary",
                &self.primary.as_ref().map(|p| {
                    p.downcast_ref::<&str>()
                        .copied()
                        .or_else(|| p.downcast_ref::<String>().map(String::as_str))
                }),
            )
            .field("cleanup_error", &self.cleanup_error)
            .field("closed", &self.closed)
            .finish()
    }
}
/// Private fixture retains the same owner/adapter/evidence through failure reporting.
pub(super) fn with_original(
    client: Arc<Client>,
    budget: ExecutionBudget,
    body: impl FnOnce(&mut CaptureMutationAdapter, &ExecutionOwner, &Arc<Client>),
) -> Result<(), Box<Failure>> {
    let owner = ExecutionOwner::start(client.clone(), budget, Arc::new(Notify)).unwrap();
    let mut adapter = CaptureMutationAdapter::new(anchor("original"), state(1));
    adapter.attach_execution(&owner.handle()).unwrap();
    let primary = catch_unwind(AssertUnwindSafe(|| body(&mut adapter, &owner, &client))).err();
    client.release();
    let mut evidence = Vec::new();
    let end = Instant::now() + Duration::from_secs(3);
    let cleanup: Result<bool, Arc<ApiError>> = (|| {
        while adapter.execution_attached() {
            match adapter.request_execution_close()? {
                CaptureExecutionProgress::TerminalReady { .. }
                | CaptureExecutionProgress::Unsubmitted => {
                    adapter
                        .discard_execution_terminal("explicit fixture failure cleanup".into())?;
                }
                CaptureExecutionProgress::ConsumedAckPending
                | CaptureExecutionProgress::DiscardedAckPending => {
                    evidence.push(adapter.acknowledge_execution_terminal()?);
                }
                CaptureExecutionProgress::Idle if adapter.display().pending.is_none() => {
                    adapter.release_execution()?
                }
                _ => {}
            }
            if Instant::now() >= end {
                return Ok(false);
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        loop {
            if let ExecutionCloseProgress::Closed(receipt) = owner.poll_close().map_err(Arc::new)? {
                return Ok(receipt.worker_stopped && receipt.unacknowledged == 0);
            }
            if Instant::now() >= end {
                return Ok(false);
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    })();
    let (closed, cleanup_error) = match cleanup {
        Ok(closed) => (closed, None),
        Err(error) => (false, Some(error)),
    };
    if primary.is_none() && closed && cleanup_error.is_none() {
        return Ok(());
    }
    Err(Box::new(Failure {
        _owner: owner,
        _adapter: adapter,
        _evidence: evidence,
        primary,
        cleanup_error,
        closed,
    }))
}

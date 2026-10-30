use super::*;
use crate::application::sdk_execution::host::HostExecution;
use serde_json::json;
struct RecoveringHostClient {
    entered: mpsc::Sender<u8>,
    advance: Mutex<mpsc::Receiver<()>>,
}

#[test]
fn host_rpc_preserves_original_failure_and_requires_strict_explicit_recovery() {
    let (entered, receive) = mpsc::channel();
    let (advance, gate) = mpsc::channel();
    let host = HostExecution::start(
        "c".repeat(32),
        Arc::new(RecoveringHostClient {
            entered,
            advance: Mutex::new(gate),
        }),
        budget(),
        Arc::new(Notify {
            signals: AtomicUsize::new(0),
            fail: false,
        }),
    )
    .unwrap();
    let rpc = |command: &str, payload: serde_json::Value| {
        let mut request = Request::new(command);
        request.payload = payload;
        host.execute_rpc(request)
    };
    assert!(
        !rpc(
            "execution.submit",
            json!({"request":Request::new("execution.list")})
        )
        .ok
    );
    assert!(!rpc("execution.list", json!({"pid":1})).ok);
    let mut unsupported_context = Request::new("execution.list");
    unsupported_context.context.idempotency_key = Some("not-an-operation-retry".into());
    assert!(!host.execute_rpc(unsupported_context).ok);
    let submitted = rpc(
        "execution.submit",
        json!({"request":Request::new("production.capture_frame")}),
    );
    assert!(submitted.ok);
    let operation = submitted.data.unwrap()["operation"].clone();
    assert_eq!(receive.recv_timeout(Duration::from_secs(5)).unwrap(), 1);
    let state = rpc("execution.peek", json!({"operation":operation}))
        .data
        .unwrap();
    assert_eq!(state["state"]["phase"], "running");
    assert_eq!(state["state"]["receipt_recovery"]["generation"], 1);
    assert_eq!(
        rpc("execution.list", json!({})).data.unwrap()["operations"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert!(
        !rpc(
            "execution.retry_receipt",
            json!({"operation":operation,"expected_generation":1,"pid":1})
        )
        .ok
    );
    for generation in 1..=2 {
        let retried = rpc(
            "execution.retry_receipt",
            json!({"operation":operation,"expected_generation":generation}),
        );
        assert!(retried.ok);
        assert_eq!(retried.data.unwrap()["completion"], "unobserved");
        advance.send(()).unwrap();
        if generation == 1 {
            assert_eq!(receive.recv_timeout(Duration::from_secs(5)).unwrap(), 2);
        }
    }
    let cutoff = Instant::now() + Duration::from_secs(5);
    loop {
        let response = rpc("execution.peek", json!({"operation":operation}));
        assert!(response.ok);
        let state = response.data.unwrap();
        if state["state"]["phase"] == "terminal" {
            assert_eq!(state["state"]["envelope_valid"], true);
            assert_eq!(state["state"]["response"]["ok"], false);
            assert_eq!(
                state["state"]["response"]["error"]["message"],
                "original failure retained"
            );
            break;
        }
        assert!(Instant::now() < cutoff);
        std::thread::yield_now();
    }
    assert!(rpc("execution.acknowledge", json!({"operation":operation})).ok);
    assert!(!rpc("execution.peek", json!({"operation":operation})).ok);
    assert!(
        rpc("execution.list", json!({})).data.unwrap()["operations"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    while !matches!(
        host.poll_close().unwrap(),
        ExecutionCloseProgress::Closed(_)
    ) {
        assert!(Instant::now() < cutoff);
        std::thread::yield_now();
    }
}
impl ApplicationClient for RecoveringHostClient {
    fn execute(&self, _: Request) -> Response {
        panic!("original recoverable worker required");
    }
    fn execute_recoverable(
        &self,
        request: Request,
        _: &(dyn crate::application::review_source::MutationControl + Sync),
        recovery: &dyn ReceiptRecoveryControl,
    ) -> Response {
        for generation in 1..=2 {
            assert_eq!(recovery.receipt_failed().unwrap(), generation);
            assert!(!recovery.take_retry(generation).unwrap());
            self.entered.send(generation).unwrap();
            self.advance
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_secs(5))
                .unwrap();
            assert!(recovery.take_retry(generation).unwrap());
            assert!(!recovery.take_retry(generation).unwrap());
        }
        Response::failure(
            request.context.request_id,
            ApiError::new(ErrorCode::EngineFailed, "original failure retained"),
        )
    }
}
#[test]
fn host_keeps_original_recovery_worker_and_rejects_foreign_stale_and_acknowledged_references() {
    let (entered, receive) = mpsc::channel();
    let (advance, gate) = mpsc::channel();
    let host = HostExecution::start(
        "a".repeat(32),
        Arc::new(RecoveringHostClient {
            entered,
            advance: Mutex::new(gate),
        }),
        budget(),
        Arc::new(Notify {
            signals: AtomicUsize::new(0),
            fail: false,
        }),
    )
    .unwrap();
    let id = host
        .submit(Request::new("production.capture_frame"))
        .unwrap();
    assert_eq!(receive.recv_timeout(Duration::from_secs(5)).unwrap(), 1);
    assert!(matches!(
        host.peek(&id).unwrap(),
        ExecutionSnapshot::Running { .. }
    ));
    let mut foreign = id.clone();
    foreign.host_scope = "b".repeat(32);
    assert_eq!(
        host.peek(&foreign).unwrap_err().code,
        ErrorCode::SourceExpired
    );
    let mut malformed = id.clone();
    malformed.ordinal = "01".into();
    assert_eq!(
        host.peek(&malformed).unwrap_err().code,
        ErrorCode::SourceExpired
    );
    let queued = host.submit(Request::new("version")).unwrap();
    assert_eq!(
        host.retry_receipt(&queued, 1).unwrap_err().code,
        ErrorCode::NeedsInput
    );
    assert_eq!(
        host.submit(Request::new("version")).unwrap_err().code,
        ErrorCode::ResourceExhausted
    );
    assert_eq!(host.receipt_recovery(&id).unwrap().unwrap().generation, 1);
    assert_eq!(
        host.retry_receipt(&id, 2).unwrap_err().code,
        ErrorCode::SourceExpired
    );
    assert_eq!(
        host.retry_receipt(&id, 1).unwrap(),
        CleanupRetryDisposition::Requested
    );
    assert_eq!(
        host.retry_receipt(&id, 1).unwrap(),
        CleanupRetryDisposition::AlreadyRequested
    );
    advance.send(()).unwrap();
    assert_eq!(receive.recv_timeout(Duration::from_secs(5)).unwrap(), 2);
    assert_eq!(
        host.retry_receipt(&id, 1).unwrap_err().code,
        ErrorCode::SourceExpired
    );
    assert_eq!(
        host.cancel(&queued).unwrap(),
        CancelDisposition::QueuedCancelled
    );
    host.acknowledge(&queued).unwrap();
    assert_eq!(
        host.peek(&queued).unwrap_err().code,
        ErrorCode::SourceExpired
    );
    assert_eq!(
        host.retry_receipt(&id, 2).unwrap(),
        CleanupRetryDisposition::Requested
    );
    advance.send(()).unwrap();
    let cutoff = Instant::now() + Duration::from_secs(5);
    loop {
        if let ExecutionSnapshot::Terminal(terminal) = host.peek(&id).unwrap() {
            assert_eq!(
                terminal.response.error.as_ref().unwrap().message,
                "original failure retained"
            );
            break;
        }
        assert!(Instant::now() < cutoff);
        std::thread::yield_now();
    }
    assert_eq!(
        host.retry_receipt(&id, 2).unwrap_err().code,
        ErrorCode::NeedsInput
    );
    host.acknowledge(&id).unwrap();
    assert_eq!(host.peek(&id).unwrap_err().code, ErrorCode::SourceExpired);
    loop {
        if matches!(
            host.poll_close().unwrap(),
            ExecutionCloseProgress::Closed(_)
        ) {
            break;
        }
        assert!(Instant::now() < cutoff);
        std::thread::yield_now();
    }
}

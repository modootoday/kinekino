use super::*;
use crate::{
    application::{api::ErrorCode, sdk_execution::ReceiptRecoveryControl},
    infrastructure::local_host::EndpointClient,
};
use serde_json::json;
use std::{
    sync::{Mutex, mpsc},
    time::{Duration, Instant},
};
struct CountGate {
    calls: std::sync::atomic::AtomicUsize,
    entered: mpsc::Sender<()>,
    gate: Mutex<mpsc::Receiver<()>>,
}
impl ApplicationClient for CountGate {
    fn execute(&self, request: Request) -> Response {
        self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.entered.send(()).unwrap();
        self.gate
            .lock()
            .unwrap()
            .recv_timeout(Duration::from_secs(10))
            .unwrap();
        crate::application::api::PlayerSdk::default().execute(request)
    }
}
#[test]
fn actual_pipe_unread_admission_response_resolves_same_submission_without_reexecution() {
    let (entered, receive) = mpsc::channel();
    let (advance, gate) = mpsc::channel();
    let counted = Arc::new(CountGate {
        calls: std::sync::atomic::AtomicUsize::new(0),
        entered,
        gate: Mutex::new(gate),
    });
    let host = ExecutionLocalHost::start(
        counted.clone(),
        ExecutionBudget {
            max_operations: 2,
            max_request_bytes: 4096,
            max_total_request_bytes: 8192,
            max_terminal_bytes: 4096,
            max_total_terminal_bytes: 8192,
        },
        TransportOptions::default(),
    )
    .unwrap();
    let submission =
        json!({"host_scope":host.host_scope(),"submission_id":"actual-unread-response"});
    let mut request = Request::new("execution.submit");
    request.payload = json!({"request":Request::new("version"),"submission":submission});
    let endpoint = crate::domain::local_endpoint::LocalEndpoint::parse(host.endpoint()).unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let mut pipe = crate::infrastructure::os_policy::ipc()
            .connect(&endpoint.id())
            .unwrap();
        super::super::security::verify(&*pipe, &endpoint).unwrap();
        super::super::framing::write(
            &mut pipe,
            &serde_json::to_vec(&request).unwrap(),
            crate::application::local_host::MAX_REQUEST_BYTES,
        )
        .await
        .unwrap();
        let cutoff = Instant::now() + Duration::from_secs(5);
        loop {
            if receive.try_recv().is_ok() {
                break;
            }
            assert!(Instant::now() < cutoff);
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
        // No response byte is read. Original SDK admission is already observed.
        drop(pipe);
    });
    let client = EndpointClient::new(host.endpoint(), TransportOptions::default()).unwrap();
    let mut inspect = Request::new("execution.submission.inspect");
    inspect.payload = json!({"submission":submission});
    let found = client.execute(inspect);
    assert!(found.ok, "{:?}", found.error);
    let operation = found.data.unwrap()["operation"].clone();
    let repeated = client.execute(request);
    assert!(repeated.ok);
    assert_eq!(repeated.data.as_ref().unwrap()["operation"], operation);
    assert_eq!(repeated.data.unwrap()["replayed"], true);
    assert_eq!(counted.calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    advance.send(()).unwrap();
    let cutoff = Instant::now() + Duration::from_secs(5);
    loop {
        let mut peek = Request::new("execution.peek");
        peek.payload = json!({"operation":operation});
        let state = client.execute(peek);
        assert!(state.ok);
        if state.data.unwrap()["state"]["phase"] == "terminal" {
            break;
        }
        assert!(Instant::now() < cutoff);
    }
    let mut ack = Request::new("execution.acknowledge");
    ack.payload = json!({"operation":operation});
    assert!(client.execute(ack).ok);
    let mut pending = host.begin_shutdown();
    while !pending.poll().unwrap().is_complete() {
        assert!(Instant::now() < cutoff);
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn pipe_shutdown_retains_unacknowledged_terminal_until_original_ack() {
    let host = ExecutionLocalHost::start(
        Arc::new(crate::application::api::PlayerSdk::default()),
        ExecutionBudget {
            max_operations: 1,
            max_request_bytes: 4096,
            max_total_request_bytes: 4096,
            max_terminal_bytes: 4096,
            max_total_terminal_bytes: 4096,
        },
        TransportOptions::default(),
    )
    .unwrap();
    let client = EndpointClient::new(host.endpoint(), TransportOptions::default()).unwrap();
    let mut submit = Request::new("execution.submit");
    submit.payload = json!({"request":Request::new("version")});
    let submitted = client.execute(submit);
    assert!(submitted.ok);
    let operation: crate::application::sdk_execution::host::HostOperationReference =
        serde_json::from_value(submitted.data.unwrap()["operation"].clone()).unwrap();
    let cutoff = Instant::now() + Duration::from_secs(5);
    let mut pending = host.begin_shutdown();
    loop {
        let status = pending.poll().unwrap();
        assert!(
            !status.is_complete(),
            "unacknowledged operation must remain retained"
        );
        if matches!(
            pending.execution().peek(&operation).unwrap(),
            crate::application::sdk_execution::ExecutionSnapshot::Terminal(_)
        ) {
            break;
        }
        assert!(Instant::now() < cutoff);
        std::thread::sleep(Duration::from_millis(1));
    }
    pending.execution().acknowledge(&operation).unwrap();
    loop {
        if pending.poll().unwrap().is_complete() {
            break;
        }
        assert!(Instant::now() < cutoff);
        std::thread::sleep(Duration::from_millis(1));
    }
}
struct Recovering {
    entered: mpsc::Sender<()>,
    advance: Mutex<mpsc::Receiver<()>>,
}
impl ApplicationClient for Recovering {
    fn execute(&self, _: Request) -> Response {
        panic!("recoverable worker required")
    }
    fn execute_recoverable(
        &self,
        request: Request,
        _: &(dyn crate::application::review_source::MutationControl + Sync),
        recovery: &dyn ReceiptRecoveryControl,
    ) -> Response {
        let generation = recovery.receipt_failed().unwrap();
        assert!(!recovery.take_retry(generation).unwrap());
        self.entered.send(()).unwrap();
        self.advance
            .lock()
            .unwrap()
            .recv_timeout(Duration::from_secs(10))
            .unwrap();
        assert!(recovery.take_retry(generation).unwrap());
        Response::failure(
            request.context.request_id,
            ApiError::new(ErrorCode::EngineFailed, "original pipe operation error"),
        )
    }
}
#[test]
fn actual_pipe_controls_pending_original_operation_and_joins_both_owners() {
    let (entered, receive) = mpsc::channel();
    let (advance, gate) = mpsc::channel();
    let host = ExecutionLocalHost::start(
        Arc::new(Recovering {
            entered,
            advance: Mutex::new(gate),
        }),
        ExecutionBudget {
            max_operations: 2,
            max_request_bytes: 4096,
            max_total_request_bytes: 8192,
            max_terminal_bytes: 4096,
            max_total_terminal_bytes: 8192,
        },
        TransportOptions::default(),
    )
    .unwrap();
    let client = EndpointClient::new(host.endpoint(), TransportOptions::default()).unwrap();
    let call = |command: &str, payload: serde_json::Value| {
        let mut r = Request::new(command);
        r.payload = payload;
        client.execute(r)
    };
    let submitted = call(
        "execution.submit",
        json!({"request":Request::new("production.capture_frame")}),
    );
    assert!(submitted.ok, "{:?}", submitted.error);
    let operation = submitted.data.unwrap()["operation"].clone();
    receive.recv_timeout(Duration::from_secs(5)).unwrap();
    let running = call("execution.peek", json!({"operation":operation}));
    assert!(running.ok);
    assert_eq!(running.data.unwrap()["state"]["phase"], "running");
    let retry = call(
        "execution.retry_receipt",
        json!({"operation":operation,"expected_generation":1}),
    );
    assert!(retry.ok);
    assert_eq!(retry.data.unwrap()["completion"], "unobserved");
    advance.send(()).unwrap();
    let cutoff = Instant::now() + Duration::from_secs(5);
    loop {
        let result = call("execution.peek", json!({"operation":operation}));
        assert!(result.ok);
        let state = result.data.unwrap();
        if state["state"]["phase"] == "terminal" {
            assert_eq!(
                state["state"]["response"]["error"]["message"],
                "original pipe operation error"
            );
            break;
        }
        assert!(Instant::now() < cutoff);
    }
    assert!(call("execution.acknowledge", json!({"operation":operation})).ok);
    let mut pending = host.begin_shutdown();
    loop {
        if pending.poll().unwrap().is_complete() {
            break;
        }
        assert!(Instant::now() < cutoff);
        std::thread::sleep(Duration::from_millis(1));
    }
}

use super::*;
use crate::application::api::{ApplicationClient, PlayerSdk};
use std::{
    sync::{
        Barrier,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}
fn joined(thread: std::thread::JoinHandle<()>) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !thread.is_finished() {
        assert!(Instant::now() < deadline, "SDK worker did not finish");
        std::thread::yield_now();
    }
    thread.join().unwrap();
}

#[test]
fn close_and_admission_have_one_order_without_replay() {
    struct Count(Arc<AtomicUsize>);
    impl ApplicationClient for Count {
        fn execute(&self, request: Request) -> Response {
            self.0.fetch_add(1, Ordering::SeqCst);
            PlayerSdk::default().execute(request)
        }
    }
    let calls = Arc::new(AtomicUsize::new(0));
    let (executor, thread) = Executor::start(Arc::new(Count(calls.clone()))).unwrap();
    let barrier = Arc::new(Barrier::new(2));
    let closer = executor.clone();
    let other = barrier.clone();
    let close = std::thread::spawn(move || {
        other.wait();
        closer.close();
    });
    barrier.wait();
    let admission = executor.admit(Request::new("version"));
    close.join().unwrap();
    match admission {
        Ok(response) => {
            assert!(runtime().block_on(response).unwrap().ok);
            assert_eq!(calls.load(Ordering::SeqCst), 1);
        }
        Err(error) => {
            assert_eq!(error.details["delivery"], "not_dispatched");
            assert_eq!(calls.load(Ordering::SeqCst), 0);
        }
    }
    let response = runtime().block_on(executor.execute(Request::new("version")));
    assert_eq!(
        response.error.unwrap().details["delivery"],
        "not_dispatched"
    );
    joined(thread);
}

#[test]
fn poisoned_admission_closes_without_dispatch_or_success_state() {
    let (executor, thread) = Executor::start(Arc::new(PlayerSdk::default())).unwrap();
    let state = executor.admission.clone();
    assert!(
        std::thread::spawn(move || {
            let _guard = state.lock().unwrap();
            panic!("poison admission fixture");
        })
        .join()
        .is_err()
    );
    let response = runtime().block_on(executor.execute(Request::new("version")));
    let error = response.error.unwrap();
    assert_eq!(error.code, ErrorCode::Internal);
    assert_eq!(error.details["delivery"], "not_dispatched");
    executor.close();
    assert_eq!(executor.state(), None);
    joined(thread);
}

#[test]
fn sdk_panic_is_response_failure_not_worker_join_panic() {
    struct Panics;
    impl ApplicationClient for Panics {
        fn execute(&self, _: Request) -> Response {
            panic!("SDK fixture");
        }
    }
    let (executor, thread) = Executor::start(Arc::new(Panics)).unwrap();
    let response = runtime().block_on(executor.execute(Request::new("version")));
    assert_eq!(response.error.unwrap().code, ErrorCode::Internal);
    assert_eq!(executor.state(), Some(DispatchState::Completed));
    executor.close();
    joined(thread);
}
struct TracedHold {
    calls: Arc<AtomicUsize>,
    entered: std::sync::mpsc::SyncSender<()>,
    release: Mutex<std::sync::mpsc::Receiver<()>>,
}
impl ApplicationClient for TracedHold {
    fn execute(&self, request: Request) -> Response {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.entered.send(()).unwrap();
        self.release.lock().unwrap().recv().unwrap();
        PlayerSdk::default().execute(request)
    }
}
struct TracedFlight {
    executor: Executor,
    worker: Option<std::thread::JoinHandle<()>>,
    caller: Option<std::thread::JoinHandle<Response>>,
    entered: std::sync::mpsc::Receiver<()>,
    release: std::sync::mpsc::Sender<()>,
    calls: Arc<AtomicUsize>,
}
impl Drop for TracedFlight {
    fn drop(&mut self) {
        self.executor.close();
        let _ = self.release.send(());
        if let Some(caller) = self.caller.take() {
            let _ = caller.join();
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
fn traced_flight(trace: TransportDiagnostics) -> TracedFlight {
    let (entered, receiver) = std::sync::mpsc::sync_channel(1);
    let (release, released) = std::sync::mpsc::channel();
    let calls = Arc::new(AtomicUsize::new(0));
    let (executor, worker) = Executor::start_observed(
        Arc::new(TracedHold {
            calls: calls.clone(),
            entered,
            release: Mutex::new(released),
        }),
        Some(trace),
    )
    .unwrap();
    TracedFlight {
        executor,
        worker: Some(worker),
        caller: None,
        entered: receiver,
        release,
        calls,
    }
}
#[test]
fn traced_sdk_delay_preserves_phase_order_and_never_logs_original_request_identifier() {
    let trace = TransportDiagnostics::default();
    let mut flight = traced_flight(trace.clone());
    let executor = flight.executor.clone();
    flight.caller = Some(std::thread::spawn(move || {
        let mut request = Request::new("version");
        request.context.request_id = "PRIVATE_REQUEST_IDENTIFIER".into();
        runtime().block_on(executor.execute(request))
    }));
    flight.entered.recv_timeout(Duration::from_secs(5)).unwrap();
    let operations: Vec<_> = trace
        .snapshot()
        .iter()
        .map(|event| event.operation)
        .collect();
    assert_eq!(
        operations,
        vec!["sdk_admitted", "sdk_running", "sdk_execute_begin"]
    );
    flight.executor.close();
    flight.release.send(()).unwrap();
    assert!(flight.caller.take().unwrap().join().unwrap().ok);
    joined(flight.worker.take().unwrap());
    assert_eq!(flight.calls.load(Ordering::SeqCst), 1);
    let events = trace.snapshot();
    assert_eq!(
        events
            .iter()
            .map(|event| event.operation)
            .collect::<Vec<_>>(),
        vec![
            "sdk_admitted",
            "sdk_running",
            "sdk_execute_begin",
            "sdk_execute_returned",
            "sdk_reply_sent"
        ]
    );
    assert!(
        !serde_json::to_string(&events)
            .unwrap()
            .contains("PRIVATE_REQUEST_IDENTIFIER")
    );
}
#[test]
fn traced_abandoned_reply_keeps_original_sdk_work_and_reports_receiver_loss_without_replay() {
    let trace = TransportDiagnostics::default();
    let mut flight = traced_flight(trace.clone());
    let reply = flight.executor.admit(Request::new("version")).unwrap();
    flight.entered.recv_timeout(Duration::from_secs(5)).unwrap();
    drop(reply);
    flight.executor.close();
    flight.release.send(()).unwrap();
    joined(flight.worker.take().unwrap());
    assert_eq!(flight.calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        trace.snapshot().last().unwrap().operation,
        "sdk_reply_receiver_gone"
    );
    assert_eq!(flight.executor.state(), Some(DispatchState::Completed));
}

use super::*;
use crate::{
    application::api::{ApplicationClient, PlayerSdk},
    infrastructure::local_host::{DispatchState, EndpointClient, PendingLocalHost, WorkerJoin},
};
use std::{
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

struct Release(Arc<(Mutex<bool>, Condvar)>);
impl Release {
    fn open(&self) {
        let (lock, changed) = &*self.0;
        *lock.lock().unwrap() = true;
        changed.notify_all();
    }
}
impl Drop for Release {
    fn drop(&mut self) {
        self.open();
    }
}
struct Blocked {
    gate: Arc<(Mutex<bool>, Condvar)>,
    started: mpsc::Sender<()>,
    calls: Arc<AtomicUsize>,
    dropped: Arc<AtomicBool>,
}
impl ApplicationClient for Blocked {
    fn execute(&self, request: Request) -> Response {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.started.send(()).unwrap();
        let (lock, changed) = &*self.gate;
        drop(
            changed
                .wait_while(lock.lock().unwrap(), |released| !*released)
                .unwrap(),
        );
        PlayerSdk::default().execute(request)
    }
}
impl Drop for Blocked {
    fn drop(&mut self) {
        self.dropped.store(true, Ordering::SeqCst);
    }
}
fn finish(pending: &mut PendingLocalHost) -> super::super::LocalHostShutdownStatus {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let status = pending.poll();
        if status.pipe != WorkerJoin::Pending && status.sdk != WorkerJoin::Pending {
            return status;
        }
        assert!(Instant::now() < deadline, "shutdown incomplete: {status:?}");
        std::thread::yield_now();
    }
}
struct Flight {
    host: LocalHost,
    release: Release,
    started: mpsc::Receiver<()>,
    calls: Arc<AtomicUsize>,
    dropped: Arc<AtomicBool>,
}
fn blocked_host() -> Flight {
    let gate = Arc::new((Mutex::new(false), Condvar::new()));
    let (started, receiver) = mpsc::channel();
    let calls = Arc::new(AtomicUsize::new(0));
    let dropped = Arc::new(AtomicBool::new(false));
    let host = LocalHost::start(
        Arc::new(Blocked {
            gate: gate.clone(),
            started,
            calls: calls.clone(),
            dropped: dropped.clone(),
        }),
        TransportOptions::default(),
    )
    .unwrap();
    Flight {
        host,
        release: Release(gate),
        started: receiver,
        calls,
        dropped,
    }
}

#[test]
fn blocked_sdk_retains_pending_owner_and_joins_both_after_release() {
    let Flight {
        host,
        release,
        started,
        calls,
        dropped,
    } = blocked_host();
    let client = EndpointClient::new(host.endpoint(), TransportOptions::default()).unwrap();
    let request = std::thread::spawn(move || client.execute(Request::new("version")));
    started.recv_timeout(Duration::from_secs(5)).unwrap();
    let executor = host.executor.clone();
    let mut pending = host.begin_shutdown();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let refused = runtime.block_on(executor.execute(Request::new("version")));
    assert_eq!(refused.error.unwrap().details["delivery"], "not_dispatched");
    let status = pending.poll();
    assert_eq!(status.sdk, WorkerJoin::Pending);
    assert_eq!(status.dispatch, Some(DispatchState::Running));
    assert!(!status.is_quiescent());
    assert!(!dropped.load(Ordering::SeqCst));
    let deadline = Instant::now() + Duration::from_secs(5);
    while pending.poll().pipe == WorkerJoin::Pending {
        assert!(
            Instant::now() < deadline,
            "pipe did not join while SDK remained blocked"
        );
        std::thread::yield_now();
    }
    assert_eq!(pending.poll().pipe, WorkerJoin::Joined);
    assert_eq!(pending.poll().sdk, WorkerJoin::Pending);
    assert!(!request.join().unwrap().ok);
    release.open();
    let status = finish(&mut pending);
    assert!(status.is_quiescent());
    assert_eq!(status.dispatch, Some(DispatchState::Completed));
    assert_eq!(pending.poll(), status);
    assert!(dropped.load(Ordering::SeqCst));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn abandoned_reply_does_not_cancel_accepted_sdk_work() {
    let Flight {
        host,
        release,
        started,
        calls,
        ..
    } = blocked_host();
    let executor = host.executor.clone();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let response = runtime.block_on(async {
        let result = tokio::time::timeout(
            Duration::from_millis(20),
            executor.execute(Request::new("version")),
        )
        .await;
        assert!(result.is_err());
        result
    });
    drop(response);
    started.recv_timeout(Duration::from_secs(5)).unwrap();
    let mut pending = host.begin_shutdown();
    assert_eq!(pending.poll().sdk, WorkerJoin::Pending);
    release.open();
    assert!(finish(&mut pending).is_quiescent());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn idle_host_shutdown_cancels_partial_pipe_read_and_joins_workers() {
    use tokio::io::AsyncWriteExt;
    let trace = TransportDiagnostics::default();
    let host = LocalHost::start_observed(
        Arc::new(PlayerSdk::default()),
        TransportOptions::default(),
        trace.clone(),
    )
    .unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let mut pipe = {
        let _entered = runtime.enter();
        let endpoint =
            crate::domain::local_endpoint::LocalEndpoint::parse(host.endpoint()).unwrap();
        crate::infrastructure::os_policy::ipc()
            .connect(&endpoint.id())
            .unwrap()
    };
    runtime.block_on(pipe.write_all(&[0, 0])).unwrap();
    assert!(trace.wait_for("client_verified", 0, Duration::from_secs(5)));
    let mut pending = host.begin_shutdown();
    let status = finish(&mut pending);
    assert!(status.is_quiescent());
    assert_eq!(status.dispatch, Some(DispatchState::Idle));
    assert_eq!(pending.poll(), status);
}

#[test]
fn pipe_panic_is_not_quiescent_success_and_other_worker_still_joins() {
    let (executor, sdk) = Executor::start(Arc::new(PlayerSdk::default())).unwrap();
    executor.close();
    let pipe = std::thread::spawn(|| panic!("pipe fixture"));
    let mut pending = PendingLocalHost::new(executor, Some(pipe), Some(sdk));
    let status = finish(&mut pending);
    assert_eq!(status.pipe, WorkerJoin::Panicked);
    assert_eq!(status.sdk, WorkerJoin::Joined);
    assert!(!status.is_quiescent());
    assert_eq!(pending.poll(), status);
}

#[test]
fn sdk_worker_drop_panic_is_separate_from_pipe_join() {
    struct DropPanics;
    impl ApplicationClient for DropPanics {
        fn execute(&self, request: Request) -> Response {
            PlayerSdk::default().execute(request)
        }
    }
    impl Drop for DropPanics {
        fn drop(&mut self) {
            panic!("SDK worker owner-drop fixture");
        }
    }
    let (executor, sdk) = Executor::start(Arc::new(DropPanics)).unwrap();
    executor.close();
    let pipe = std::thread::spawn(|| {});
    let mut pending = PendingLocalHost::new(executor, Some(pipe), Some(sdk));
    let status = finish(&mut pending);
    assert_eq!(status.pipe, WorkerJoin::Joined);
    assert_eq!(status.sdk, WorkerJoin::Panicked);
    assert!(!status.is_quiescent());
    assert_eq!(pending.poll(), status);
}

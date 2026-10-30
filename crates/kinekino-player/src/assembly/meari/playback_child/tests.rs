use super::PlaybackCleanupError;
use super::cleanup::CleanupMachine;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        playback_control::ControlReceipt,
        playback_session::*,
    },
    infrastructure::{
        mpv::{ParentEngineRouter, ParentOwnedEngine, ParentRouteHandle},
        thread_completion::{DispatcherHost, DispatcherShutdown},
    },
};
use meari_native::application::window_lifecycle::*;
use std::{
    cell::{Cell, RefCell},
    num::NonZeroUsize,
    rc::Rc,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    task::{Wake, Waker},
    time::{Duration, Instant},
};

const LIMIT: Duration = Duration::from_secs(5);
#[derive(Clone)]
struct Clock(Rc<Cell<Duration>>);
impl MonotonicClock for Clock {
    fn now(&self) -> Duration {
        self.0.get()
    }
}
struct ChannelWake(mpsc::Sender<()>);
impl Wake for ChannelWake {
    fn wake(self: Arc<Self>) {
        let _ = self.0.send(());
    }
}
type Captured = Rc<RefCell<Option<(CleanupRequest, CompletionSignal)>>>;
struct CapturingChild {
    machine: Rc<RefCell<CleanupMachine>>,
    captured: Captured,
    diagnostic: Option<PlaybackCleanupError>,
}
impl NativeChild for CapturingChild {
    type Error = PlaybackCleanupError;
    fn parent_lost(&mut self) {
        self.machine.borrow_mut().parent_lost();
    }
    fn begin_cleanup(
        &mut self,
        request: CleanupRequest,
        signal: CompletionSignal,
    ) -> Result<CleanupProgress, Self::Error> {
        *self.captured.borrow_mut() = Some((request, signal.clone()));
        let mut machine = self.machine.borrow_mut();
        let result = machine.begin_cleanup(request, signal);
        self.diagnostic = machine.observation_error().cloned();
        result
    }
    fn observe_cleanup(&mut self, request: CleanupRequest) -> Result<CleanupProgress, Self::Error> {
        let mut machine = self.machine.borrow_mut();
        let result = machine.observe_cleanup(request);
        self.diagnostic = machine.observation_error().cloned();
        result
    }
    fn observation_error(&self) -> Option<&Self::Error> {
        self.diagnostic.as_ref()
    }
}
struct Harness {
    lifecycle: WindowLifecycle<CapturingChild>,
    machine: Rc<RefCell<CleanupMachine>>,
    captured: Captured,
    lease: Option<ChildLease>,
    router: Arc<ParentEngineRouter>,
    route: ParentRouteHandle,
    clock: Clock,
    wake: mpsc::Receiver<()>,
}
fn host() -> DispatcherHost {
    DispatcherHost::start(NonZeroUsize::new(2).unwrap()).unwrap()
}
fn harness(host: &DispatcherHost, engine: Arc<FakeEngine>, prepared_partial: bool) -> Harness {
    let router = Arc::new(ParentEngineRouter::new(host.handle()));
    let route = router.prepare_test_engine(engine).unwrap();
    if !prepared_partial {
        router.activate(&route).unwrap();
    }
    let clock = Clock(Rc::new(Cell::new(Duration::ZERO)));
    let (sender, wake) = mpsc::channel();
    let lifecycle = WindowLifecycle::new(
        LifecycleLimits::new(1, Duration::from_secs(10)).unwrap(),
        clock.clone(),
        Waker::from(Arc::new(ChannelWake(sender))),
    )
    .unwrap();
    let window = lifecycle.ready(ReadyKind::NewWindow).unwrap();
    let captured = Rc::new(RefCell::new(None));
    let mut machine = None;
    let lease = lifecycle
        .register(window, |lease| {
            let owner = Rc::new(RefCell::new(
                CleanupMachine::new(lease.window(), lease.id(), router.clone(), route.clone())
                    .unwrap(),
            ));
            machine = Some(owner.clone());
            let child = CapturingChild {
                machine: owner,
                captured: captured.clone(),
                diagnostic: None,
            };
            if prepared_partial {
                Err(ChildCreationFailure {
                    error: PlaybackCleanupError::Identity,
                    partial: Some(child),
                })
            } else {
                Ok(child)
            }
        })
        .ok();
    Harness {
        lifecycle,
        machine: machine.unwrap(),
        captured,
        lease,
        router,
        route,
        clock,
        wake,
    }
}
fn poll(h: &Harness, wanted: CleanupStatus) {
    poll_lifecycle(&h.lifecycle, wanted);
}
fn poll_lifecycle(lifecycle: &WindowLifecycle<CapturingChild>, wanted: CleanupStatus) {
    let deadline = Instant::now() + LIMIT;
    loop {
        if lifecycle.poll().unwrap().status == wanted {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "cleanup did not reach {wanted:?}"
        );
        std::thread::yield_now();
    }
}
fn finish(host: &mut DispatcherHost) {
    host.request_shutdown();
    let deadline = Instant::now() + LIMIT;
    loop {
        match host.poll_shutdown() {
            DispatcherShutdown::Joined {
                worker_panicked: false,
                faults: 0,
            } => return,
            DispatcherShutdown::Pending => {
                assert!(Instant::now() < deadline);
                std::thread::yield_now();
            }
            other => panic!("dispatcher shutdown: {other:?}"),
        }
    }
}
fn drain(host: &DispatcherHost) {
    let deadline = Instant::now() + LIMIT;
    while host.handle().status().outstanding != 0 {
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
}
fn gate(engine: &FakeEngine) -> (mpsc::Receiver<()>, mpsc::Sender<()>) {
    let (started, entered) = mpsc::channel();
    let (resume, resumed) = mpsc::channel();
    *engine.gate.lock().unwrap() = Some((started, resumed));
    (entered, resume)
}

#[test]
fn gated_worker_uses_final_wake_and_keeps_original_attempt_on_foreign_input() {
    let mut dispatcher = host();
    let engine = Arc::new(FakeEngine::new());
    let (entered, resume) = gate(&engine);
    let h = harness(&dispatcher, engine.clone(), false);
    assert_eq!(
        h.lifecycle.begin_close().unwrap().status,
        CleanupStatus::Pending
    );
    entered.recv_timeout(LIMIT).unwrap();
    let (request, signal) = h.captured.borrow().clone().unwrap();
    let original = h.router.retirement_handle(&h.route).unwrap().unwrap();
    for _ in 0..3 {
        assert_eq!(
            h.machine
                .borrow_mut()
                .begin_cleanup(request, signal.clone())
                .unwrap(),
            CleanupProgress::Pending
        );
    }
    assert_eq!(
        h.machine
            .borrow_mut()
            .begin_cleanup(
                CleanupRequest {
                    reason: RevocationReason::ForcedLoss,
                    ..request
                },
                signal.clone(),
            )
            .unwrap(),
        CleanupProgress::Pending,
    );
    let mut foreign_dispatcher = host();
    let foreign = harness(&foreign_dispatcher, Arc::new(FakeEngine::new()), false);
    foreign.lifecycle.begin_close().unwrap();
    let (foreign_request, foreign_signal) = foreign.captured.borrow().clone().unwrap();
    assert_eq!(
        h.machine
            .borrow_mut()
            .begin_cleanup(request, foreign_signal)
            .unwrap(),
        CleanupProgress::Pending
    );
    assert_eq!(
        h.machine
            .borrow_mut()
            .observe_cleanup(foreign_request)
            .unwrap(),
        CleanupProgress::Pending
    );
    let duplicate = h
        .router
        .retire(
            &h.route,
            Arc::new(|| panic!("duplicate wake must not replace original")),
        )
        .unwrap();
    assert!(ParentEngineRouter::same_retirement_for_test(
        &original,
        &duplicate.handle
    ));
    assert_eq!(engine.calls.load(Ordering::SeqCst), 1);
    while h.wake.try_recv().is_ok() {}
    resume.send(()).unwrap();
    h.wake.recv_timeout(LIMIT).unwrap(); // No GUI timer/poll between release and final wake.
    poll(&h, CleanupStatus::RetryableFailure);
    drain(&dispatcher);
    h.lifecycle.retry_failed().unwrap();
    poll(&h, CleanupStatus::Complete);
    assert!(!signal.notify());
    assert!(
        h.machine
            .borrow_mut()
            .begin_cleanup(request, signal)
            .is_err()
    );
    let current = h.router.retirement_handle(&h.route).unwrap().unwrap();
    assert!(ParentEngineRouter::same_retirement_for_test(
        &original, &current
    ));
    assert_eq!(engine.calls.load(Ordering::SeqCst), 2);
    poll(&foreign, CleanupStatus::RetryableFailure);
    drain(&foreign_dispatcher);
    foreign.lifecycle.retry_failed().unwrap();
    poll(&foreign, CleanupStatus::Complete);
    finish(&mut foreign_dispatcher);
    finish(&mut dispatcher);
}

#[test]
fn closed_dispatcher_keeps_engine_owned_without_starting_cleanup() {
    let mut dispatcher = host();
    finish(&mut dispatcher);
    let engine = Arc::new(FakeEngine::new());
    let weak = Arc::downgrade(&engine);
    let h = harness(&dispatcher, engine.clone(), false);
    drop(engine);
    assert_eq!(
        h.lifecycle.begin_close().unwrap().status,
        CleanupStatus::RetryableFailure
    );
    assert_eq!(
        h.lifecycle.retry_failed().unwrap().status,
        CleanupStatus::RetryableFailure
    );
    assert_eq!(weak.upgrade().unwrap().calls.load(Ordering::SeqCst), 0);
    assert!(h.router.current_route().unwrap().is_some());
}

#[test]
fn incomplete_receipt_survives_explicit_retry_in_returned_error() {
    use crate::{application::catalog::RawArtifact, domain::api_values::DecimalU64};
    let engine = Arc::new(FakeEngine::new());
    *engine.incomplete.lock().unwrap() = Some(ClosedEpisodePlayback {
        engine_instance_id: DecimalU64(419),
        engine_exited: true,
        ipc_worker_exited: false,
        stderr_worker_exited: true,
        stdout_worker_exited: false,
        diagnostics: NativeEngineDiagnostics {
            artifact: RawArtifact {
                artifact_id: "a".repeat(64),
                body_sha256: "b".repeat(64),
                body_bytes: DecimalU64(17),
                body_path: "synthetic-body".into(),
                metadata_path: "synthetic-metadata".into(),
            },
            stderr_sha256: "c".repeat(64),
            stderr_bytes: DecimalU64(3),
            stdout_sha256: "d".repeat(64),
            stdout_bytes: DecimalU64(5),
            complete: false,
            truncated: true,
            read_failed: true,
        },
    });
    let mut dispatcher = host();
    let h = harness(&dispatcher, engine.clone(), false);
    h.lifecycle.begin_close().unwrap();
    poll(&h, CleanupStatus::RetryableFailure);
    let request = h.captured.borrow().as_ref().unwrap().0;
    let retained = match h.machine.borrow_mut().observe_cleanup(request).unwrap_err() {
        PlaybackCleanupError::Incomplete(receipt) => receipt,
        other => panic!("typed receipt missing: {other:?}"),
    };
    drain(&dispatcher);
    let (entered, resume) = gate(&engine);
    h.router.fail_observation_for_test(true);
    assert_eq!(
        h.lifecycle.retry_failed().unwrap().status,
        CleanupStatus::ObservationUnavailable
    );
    entered.recv_timeout(LIMIT).unwrap();
    assert!(h.lifecycle.with_error(h.lease.as_ref().unwrap(),|error|matches!(error,Some(PlaybackCleanupError::Api(error)) if error.code==ErrorCode::Internal)).unwrap());
    h.router.fail_observation_for_test(false);
    resume.send(()).unwrap();
    poll(&h, CleanupStatus::Complete);
    finish(&mut dispatcher);
    assert_eq!(retained.engine_instance_id.0, 419);
    assert!(retained.engine_exited && retained.stderr_worker_exited);
    assert!(!retained.ipc_worker_exited && !retained.stdout_worker_exited);
    assert_eq!(retained.diagnostics.artifact.body_bytes.0, 17);
    assert!(retained.diagnostics.truncated && retained.diagnostics.read_failed);
    assert_eq!(engine.calls.load(Ordering::SeqCst), 2);
}

#[test]
fn public_coordinator_retains_joined_failure_until_explicit_retry() {
    let mut dispatcher = host();
    let engine = Arc::new(FakeEngine::new());
    let h = harness(&dispatcher, engine.clone(), false);
    h.lifecycle.begin_close().unwrap();
    poll(&h, CleanupStatus::RetryableFailure);
    let retained = h
        .lifecycle
        .with_error(h.lease.as_ref().unwrap(), |error| error.cloned())
        .unwrap()
        .unwrap();
    for _ in 0..8 {
        assert_eq!(
            h.lifecycle.poll().unwrap().status,
            CleanupStatus::RetryableFailure
        );
    }
    assert_eq!(engine.calls.load(Ordering::SeqCst), 1);
    drain(&dispatcher);
    h.lifecycle.retry_failed().unwrap();
    poll(&h, CleanupStatus::Complete);
    finish(&mut dispatcher);
    assert!(
        matches!(retained,PlaybackCleanupError::Api(error) if error.code==ErrorCode::EngineFailed)
    );
    assert_eq!(engine.calls.load(Ordering::SeqCst), 2);
}

#[test]
fn foreign_router_constructor_is_rejected_before_revocation_or_dispatch() {
    let mut dispatcher = host();
    let engine = Arc::new(FakeEngine::new());
    let h = harness(&dispatcher, engine.clone(), true);
    let other = Arc::new(ParentEngineRouter::new(dispatcher.handle()));
    let error = match CleanupMachine::new(
        h.captured
            .borrow()
            .as_ref()
            .map(|c| c.0.stamp.window())
            .unwrap_or_else(|| h.lifecycle.stamp().unwrap()),
        1,
        other,
        h.route.clone(),
    ) {
        Err(error) => error,
        Ok(_) => panic!("foreign router accepted"),
    };
    assert_eq!(error.code, ErrorCode::RevisionConflict);
    h.router.activate(&h.route).unwrap(); // Constructor did not revoke the actual binding.
    assert_eq!(engine.calls.load(Ordering::SeqCst), 0);
    h.lifecycle.begin_close().unwrap();
    poll(&h, CleanupStatus::RetryableFailure);
    drain(&dispatcher);
    h.lifecycle.retry_failed().unwrap();
    poll(&h, CleanupStatus::Complete);
    finish(&mut dispatcher);
}

#[test]
fn unavailable_retains_original_attempt_deadline_and_late_signal_without_retry() {
    let mut dispatcher = host();
    let engine = Arc::new(FakeEngine::new());
    let (entered, resume) = gate(&engine);
    let h = harness(&dispatcher, engine.clone(), false);
    h.router.fail_observation_for_test(true);
    assert_eq!(
        h.lifecycle.begin_close().unwrap().status,
        CleanupStatus::ObservationUnavailable
    );
    entered.recv_timeout(LIMIT).unwrap();
    let (request, signal) = h.captured.borrow().clone().unwrap();
    let original_deadline = h.lifecycle.next_deadline().unwrap();
    let retained = h
        .lifecycle
        .with_error(h.lease.as_ref().unwrap(), |error| error.cloned())
        .unwrap()
        .unwrap();
    assert!(
        matches!(&retained,PlaybackCleanupError::Api(error) if error.code==ErrorCode::Internal)
    );
    assert_eq!(h.lifecycle.retry_failed().unwrap().unavailable, 1);
    assert_eq!(h.lifecycle.next_deadline().unwrap(), original_deadline);
    h.clock.0.set(Duration::from_secs(20));
    let report = h.lifecycle.poll().unwrap();
    assert_eq!(
        (report.unavailable, report.timed_out, report.remaining),
        (1, 1, 1)
    );
    assert_eq!(h.lifecycle.next_deadline().unwrap(), None);
    h.lifecycle
        .reobserve_unavailable(h.lifecycle.barrier_stamp().unwrap())
        .unwrap();
    assert_eq!(engine.calls.load(Ordering::SeqCst), 1);
    assert_eq!(h.captured.borrow().as_ref().unwrap().0.stamp, request.stamp);
    h.router.fail_observation_for_test(false);
    while h.wake.try_recv().is_ok() {}
    resume.send(()).unwrap();
    h.wake.recv_timeout(LIMIT).unwrap();
    poll(&h, CleanupStatus::RetryableFailure);
    assert_eq!(
        h.captured.borrow().as_ref().unwrap().1.stamp(),
        signal.stamp()
    );
    drain(&dispatcher);
    h.lifecycle.retry_failed().unwrap();
    poll(&h, CleanupStatus::Complete);
    finish(&mut dispatcher);
    assert_eq!(engine.calls.load(Ordering::SeqCst), 2);
}

#[test]
fn partial_prepared_child_retains_engine_and_uses_same_retirement_owner() {
    let mut dispatcher = host();
    let engine = Arc::new(FakeEngine::new());
    let weak = Arc::downgrade(&engine);
    let h = harness(&dispatcher, engine.clone(), true);
    assert!(h.lease.is_none());
    drop(engine);
    assert_eq!(h.lifecycle.report().unwrap().remaining, 1);
    let (entered, wait) = mpsc::channel();
    let (resume, resumed) = mpsc::channel();
    let locked = h.router.clone();
    let holding = std::thread::spawn(move || {
        locked.with_route_lock_for_test(|| {
            entered.send(()).unwrap();
            resumed.recv_timeout(LIMIT).unwrap();
        })
    });
    wait.recv_timeout(LIMIT).unwrap();
    h.machine.borrow_mut().parent_lost(); // Must return while the router state lock is held.
    resume.send(()).unwrap();
    holding.join().unwrap();
    assert_eq!(weak.upgrade().unwrap().calls.load(Ordering::SeqCst), 0);
    assert!(h.router.activate(&h.route).is_err());
    h.lifecycle.begin_close().unwrap();
    poll(&h, CleanupStatus::RetryableFailure);
    let owner = h.router.retirement_handle(&h.route).unwrap().unwrap();
    drain(&dispatcher);
    let retained_engine = weak.upgrade().unwrap();
    let router = h.router.clone();
    let route = h.route.clone();
    let lifecycle = h.lifecycle;
    drop(h.machine); // The coordinator remains the strong child owner.
    drop(h.router);
    drop(h.route);
    lifecycle.retry_failed().unwrap();
    poll_lifecycle(&lifecycle, CleanupStatus::Complete);
    let same = router.retirement_handle(&route).unwrap().unwrap();
    assert!(ParentEngineRouter::same_retirement_for_test(&owner, &same));
    finish(&mut dispatcher);
    assert_eq!(retained_engine.calls.load(Ordering::SeqCst), 2);
}

type WorkerGate = (mpsc::Sender<()>, mpsc::Receiver<()>);
struct FakeEngine {
    calls: AtomicUsize,
    gate: Mutex<Option<WorkerGate>>,
    incomplete: Mutex<Option<ClosedEpisodePlayback>>,
}
impl FakeEngine {
    fn new() -> Self {
        Self {
            calls: AtomicUsize::new(0),
            gate: Mutex::new(None),
            incomplete: Mutex::new(None),
        }
    }
}
fn unavailable<T>() -> Result<T, ApiError> {
    Err(ApiError::new(
        ErrorCode::Unsupported,
        "synthetic cleanup-only engine",
    ))
}
impl EpisodePlaybackEngine for FakeEngine {
    fn open(
        &self,
        _: EpisodePlaybackSource,
        _: bool,
        _: f64,
    ) -> Result<OpenedEpisodePlayback, ApiError> {
        unavailable()
    }
    fn close(&self, _: CloseEpisodePlayback) -> Result<ClosedEpisodePlayback, ApiError> {
        unavailable()
    }
    fn observe(&self) -> Result<EpisodePlaybackObservation, ApiError> {
        unavailable()
    }
    fn control(&self, _: ControlEpisodePlayback) -> Result<ControlReceipt, ApiError> {
        unavailable()
    }
}
impl ParentOwnedEngine for FakeEngine {
    fn shutdown_for_parent(&self) -> Result<Option<ClosedEpisodePlayback>, ApiError> {
        let call = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
        if let Some((started, resume)) = self.gate.lock().unwrap().take() {
            started.send(()).unwrap();
            resume.recv().unwrap();
        }
        if let Some(receipt) = self.incomplete.lock().unwrap().take() {
            return Ok(Some(receipt));
        }
        if call == 1 {
            Err(ApiError::new(
                ErrorCode::EngineFailed,
                "injected joined failure",
            ))
        } else {
            Ok(None)
        }
    }
}

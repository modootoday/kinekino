use super::*;
use meari_native::application::window_lifecycle::*;
use std::{
    cell::{Cell, RefCell},
    error::Error,
    fmt,
    rc::{Rc, Weak},
    sync::Arc,
    task::Waker,
    time::Duration,
};

#[derive(Clone, Copy)]
enum Step {
    Pending,
    Unknown,
    Failed,
    Complete,
}
#[derive(Debug)]
struct FaultData {
    stage: &'static str,
    detail: Vec<u8>,
}
#[derive(Clone, Debug)]
struct Fault(Arc<FaultData>);
impl fmt::Display for Fault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {:?}", self.0.stage, self.0.detail)
    }
}
impl Error for Fault {}
struct Backend {
    step: Step,
    begins: usize,
    observes: usize,
    losses: usize,
    drops: usize,
    request: Option<CleanupRequest>,
    signal: Option<CompletionSignal>,
    fault: Fault,
}
fn backend(stage: &'static str, step: Step) -> Rc<RefCell<Backend>> {
    Rc::new(RefCell::new(Backend {
        step,
        begins: 0,
        observes: 0,
        losses: 0,
        drops: 0,
        request: None,
        signal: None,
        fault: Fault(Arc::new(FaultData {
            stage,
            detail: vec![0, 1, 255],
        })),
    }))
}
struct Port {
    state: Rc<RefCell<Backend>>,
    diagnostic: Option<Fault>,
}
impl Port {
    fn outcome(&mut self) -> Result<CleanupProgress, Fault> {
        let state = self.state.borrow();
        self.diagnostic = None;
        match state.step {
            Step::Pending => Ok(CleanupProgress::Pending),
            Step::Unknown => {
                self.diagnostic = Some(state.fault.clone());
                Ok(CleanupProgress::ObservationUnavailable)
            }
            Step::Failed => Err(state.fault.clone()),
            Step::Complete => Ok(CleanupProgress::Complete),
        }
    }
}
impl NativeChild for Port {
    type Error = Fault;
    fn parent_lost(&mut self) {
        self.state.borrow_mut().losses += 1;
    }
    fn begin_cleanup(
        &mut self,
        request: CleanupRequest,
        signal: CompletionSignal,
    ) -> Result<CleanupProgress, Fault> {
        {
            let mut state = self.state.borrow_mut();
            state.begins += 1;
            state.request = Some(request);
            state.signal = Some(signal);
        }
        self.outcome()
    }
    fn observe_cleanup(&mut self, request: CleanupRequest) -> Result<CleanupProgress, Fault> {
        {
            let mut state = self.state.borrow_mut();
            let original = state.request.unwrap();
            assert_eq!(request.stamp, original.stamp);
            assert_eq!(request.reason, original.reason);
            state.observes += 1;
        }
        self.outcome()
    }
    fn observation_error(&self) -> Option<&Fault> {
        self.diagnostic.as_ref()
    }
}
impl Drop for Port {
    fn drop(&mut self) {
        self.state.borrow_mut().drops += 1;
    }
}
type Child = PlaybackSurfaceChild<Port, Port>;
type ChildError = PlaybackSurfaceError<Fault, Fault>;
struct SharedChild {
    child: Rc<RefCell<Child>>,
    diagnostic: Option<ChildError>,
}
impl NativeChild for SharedChild {
    type Error = ChildError;
    fn parent_lost(&mut self) {
        self.child.borrow_mut().parent_lost();
    }
    fn begin_cleanup(
        &mut self,
        request: CleanupRequest,
        signal: CompletionSignal,
    ) -> Result<CleanupProgress, ChildError> {
        let mut child = self.child.borrow_mut();
        let result = child.begin_cleanup(request, signal);
        self.diagnostic = child.observation_error().cloned();
        result
    }
    fn observe_cleanup(&mut self, request: CleanupRequest) -> Result<CleanupProgress, ChildError> {
        let mut child = self.child.borrow_mut();
        let result = child.observe_cleanup(request);
        self.diagnostic = child.observation_error().cloned();
        result
    }
    fn observation_error(&self) -> Option<&ChildError> {
        self.diagnostic.as_ref()
    }
}
#[derive(Clone)]
struct Clock(Rc<Cell<Duration>>);
impl MonotonicClock for Clock {
    fn now(&self) -> Duration {
        self.0.get()
    }
}
struct Harness {
    lifecycle: WindowLifecycle<SharedChild>,
    lease: ChildLease,
    child: Weak<RefCell<Child>>,
    engine: Rc<RefCell<Backend>>,
    surface: Rc<RefCell<Backend>>,
    clock: Clock,
}
fn harness(engine_step: Step, surface_step: Step) -> Harness {
    let engine = backend("engine", engine_step);
    let surface = backend("surface", surface_step);
    let clock = Clock(Rc::new(Cell::new(Duration::ZERO)));
    let lifecycle = WindowLifecycle::new(
        LifecycleLimits::new(1, Duration::from_secs(10)).unwrap(),
        clock.clone(),
        Waker::noop().clone(),
    )
    .unwrap();
    let window = lifecycle.ready(ReadyKind::NewWindow).unwrap();
    let mut child = Weak::new();
    let lease = lifecycle
        .register(window, |lease| {
            let owner = Rc::new(RefCell::new(PlaybackSurfaceChild::new(
                lease.window(),
                lease.id(),
                Port {
                    state: engine.clone(),
                    diagnostic: None,
                },
                Port {
                    state: surface.clone(),
                    diagnostic: None,
                },
            )));
            child = Rc::downgrade(&owner);
            Ok(SharedChild {
                child: owner,
                diagnostic: None,
            })
        })
        .unwrap();
    Harness {
        lifecycle,
        lease,
        child,
        engine,
        surface,
        clock,
    }
}
fn emit(state: &Rc<RefCell<Backend>>, step: Step) {
    let signal = {
        let mut state = state.borrow_mut();
        state.step = step;
        state.signal.as_ref().unwrap().clone()
    };
    assert!(signal.notify());
}

#[test]
fn actual_coordinator_orders_engine_then_surface_and_drops_both_only_after_completion() {
    let h = harness(Step::Pending, Step::Pending);
    assert_eq!(
        h.lifecycle.begin_close().unwrap().status,
        CleanupStatus::Pending
    );
    assert_eq!(
        (h.engine.borrow().begins, h.surface.borrow().begins),
        (1, 0)
    );
    let original = h.engine.borrow().signal.as_ref().unwrap().stamp();
    emit(&h.engine, Step::Complete);
    assert_eq!(h.lifecycle.poll().unwrap().status, CleanupStatus::Pending);
    assert_eq!(
        (h.engine.borrow().begins, h.surface.borrow().begins),
        (1, 1)
    );
    assert_eq!(
        h.surface.borrow().signal.as_ref().unwrap().stamp(),
        original
    );
    assert_eq!((h.engine.borrow().drops, h.surface.borrow().drops), (0, 0));
    emit(&h.surface, Step::Complete);
    assert_eq!(h.lifecycle.poll().unwrap().status, CleanupStatus::Complete);
    assert!(h.child.upgrade().is_none());
    assert_eq!((h.engine.borrow().drops, h.surface.borrow().drops), (1, 1));
}

#[test]
fn surface_only_retry_preserves_joined_engine_and_original_typed_surface_error() {
    let h = harness(Step::Complete, Step::Failed);
    assert_eq!(
        h.lifecycle.begin_close().unwrap().status,
        CleanupStatus::RetryableFailure
    );
    let retained = h
        .lifecycle
        .with_error(&h.lease, |error| error.cloned())
        .unwrap()
        .unwrap();
    let PlaybackSurfaceError::Surface(error) = retained else {
        panic!("surface fault lost")
    };
    assert!(Arc::ptr_eq(&error.0, &h.surface.borrow().fault.0));
    let old = h.surface.borrow().signal.as_ref().unwrap().clone();
    h.surface.borrow_mut().step = Step::Pending;
    assert_eq!(
        h.lifecycle.retry_failed().unwrap().status,
        CleanupStatus::Pending
    );
    assert_eq!(
        (
            h.engine.borrow().begins,
            h.engine.borrow().observes,
            h.surface.borrow().begins
        ),
        (1, 0, 2)
    );
    assert!(!old.notify());
    let current = h.surface.borrow().signal.as_ref().unwrap().stamp();
    assert!(current.attempt() > old.stamp().attempt());
    assert!(Arc::ptr_eq(
        &h.child
            .upgrade()
            .unwrap()
            .borrow()
            .surface_failure()
            .unwrap()
            .0,
        &error.0
    ));
    assert_eq!(h.engine.borrow().drops, 0);
    emit(&h.surface, Step::Complete);
    assert_eq!(h.lifecycle.poll().unwrap().status, CleanupStatus::Complete);
    assert_eq!(error.0.detail, vec![0, 1, 255]);
    assert_eq!(
        (h.engine.borrow().begins, h.surface.borrow().begins),
        (1, 2)
    );
}

#[test]
fn terminal_engine_failure_never_starts_surface_and_only_explicit_retry_restarts_engine() {
    let h = harness(Step::Failed, Step::Pending);
    assert_eq!(
        h.lifecycle.begin_close().unwrap().status,
        CleanupStatus::RetryableFailure
    );
    let retained = h
        .lifecycle
        .with_error(&h.lease, |error| error.cloned())
        .unwrap()
        .unwrap();
    for _ in 0..3 {
        assert_eq!(
            h.lifecycle.poll().unwrap().status,
            CleanupStatus::RetryableFailure
        );
    }
    assert_eq!(
        (h.engine.borrow().begins, h.surface.borrow().begins),
        (1, 0)
    );
    let old = h.engine.borrow().signal.as_ref().unwrap().clone();
    h.engine.borrow_mut().step = Step::Pending;
    h.lifecycle.retry_failed().unwrap();
    assert_eq!(
        (h.engine.borrow().begins, h.surface.borrow().begins),
        (2, 0)
    );
    assert!(!old.notify());
    assert!(matches!(retained, PlaybackSurfaceError::Engine(_)));
    emit(&h.engine, Step::Complete);
    h.lifecycle.poll().unwrap();
    emit(&h.surface, Step::Complete);
    assert_eq!(h.lifecycle.poll().unwrap().status, CleanupStatus::Complete);
}

#[test]
fn engine_unknown_retains_stamp_deadline_and_diagnostic_without_starting_surface() {
    let h = harness(Step::Unknown, Step::Pending);
    assert_eq!(
        h.lifecycle.begin_close().unwrap().status,
        CleanupStatus::ObservationUnavailable
    );
    let original = h.engine.borrow().signal.as_ref().unwrap().stamp();
    let deadline = h.lifecycle.next_deadline().unwrap();
    let retained = h
        .lifecycle
        .with_error(&h.lease, |error| error.cloned())
        .unwrap()
        .unwrap();
    let PlaybackSurfaceError::Engine(error) = retained else {
        panic!("unknown diagnostic missing")
    };
    assert!(Arc::ptr_eq(&error.0, &h.engine.borrow().fault.0));
    h.lifecycle.retry_failed().unwrap();
    assert_eq!(h.lifecycle.next_deadline().unwrap(), deadline);
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
    assert_eq!(
        (h.engine.borrow().begins, h.surface.borrow().begins),
        (1, 0)
    );
    assert_eq!(h.engine.borrow().signal.as_ref().unwrap().stamp(), original);
    emit(&h.engine, Step::Complete);
    h.lifecycle.poll().unwrap();
    assert!(matches!(
        h.child
            .upgrade()
            .unwrap()
            .borrow()
            .retained_observation_error(),
        Some(PlaybackSurfaceError::Engine(_))
    ));
    emit(&h.surface, Step::Complete);
    assert_eq!(h.lifecycle.poll().unwrap().status, CleanupStatus::Complete);
}

#[test]
fn surface_unknown_and_parent_loss_keep_original_attempt_and_never_rejoin_engine() {
    let h = harness(Step::Complete, Step::Unknown);
    assert_eq!(
        h.lifecycle.begin_close().unwrap().status,
        CleanupStatus::ObservationUnavailable
    );
    let original = h.surface.borrow().signal.as_ref().unwrap().stamp();
    let retained = h
        .lifecycle
        .with_error(&h.lease, |error| error.cloned())
        .unwrap()
        .unwrap();
    assert!(matches!(retained, PlaybackSurfaceError::Surface(_)));
    h.lifecycle.force_loss().unwrap();
    assert_eq!(
        (h.engine.borrow().losses, h.surface.borrow().losses),
        (1, 1)
    );
    assert_eq!(
        (h.engine.borrow().begins, h.surface.borrow().begins),
        (1, 1)
    );
    assert_eq!(
        h.surface.borrow().signal.as_ref().unwrap().stamp(),
        original
    );
    emit(&h.surface, Step::Complete);
    let report = h.lifecycle.poll().unwrap();
    assert_eq!(
        (report.status, report.remaining),
        (CleanupStatus::ForcedIncomplete, 0)
    );
    assert_eq!(h.engine.borrow().begins, 1);
}

#[test]
fn duplicate_foreign_and_reason_mismatches_never_invoke_either_component() {
    let h = harness(Step::Pending, Step::Pending);
    h.lifecycle.begin_close().unwrap();
    let (request, signal) = {
        let state = h.engine.borrow();
        (
            state.request.unwrap(),
            state.signal.as_ref().unwrap().clone(),
        )
    };
    let foreign = harness(Step::Pending, Step::Pending);
    foreign.lifecycle.begin_close().unwrap();
    let (other_request, other_signal) = {
        let state = foreign.engine.borrow();
        (
            state.request.unwrap(),
            state.signal.as_ref().unwrap().clone(),
        )
    };
    let owner = h.child.upgrade().unwrap();
    assert_eq!(
        owner
            .borrow_mut()
            .begin_cleanup(request, other_signal)
            .unwrap(),
        CleanupProgress::Pending
    );
    assert_eq!(
        owner.borrow_mut().observe_cleanup(other_request).unwrap(),
        CleanupProgress::Pending
    );
    assert_eq!(
        owner
            .borrow_mut()
            .begin_cleanup(
                CleanupRequest {
                    reason: RevocationReason::ForcedLoss,
                    ..request
                },
                signal.clone()
            )
            .unwrap(),
        CleanupProgress::Pending
    );
    assert_eq!(
        (
            h.engine.borrow().begins,
            h.engine.borrow().observes,
            h.surface.borrow().begins
        ),
        (1, 0, 0)
    );
    assert_eq!(
        owner.borrow_mut().begin_cleanup(request, signal).unwrap(),
        CleanupProgress::Pending
    );
    assert_eq!(
        (h.engine.borrow().begins, h.engine.borrow().observes),
        (1, 1)
    );
    drop(owner);
    emit(&h.engine, Step::Complete);
    h.lifecycle.poll().unwrap();
    emit(&h.surface, Step::Complete);
    h.lifecycle.poll().unwrap();
    emit(&foreign.engine, Step::Complete);
    foreign.lifecycle.poll().unwrap();
    emit(&foreign.surface, Step::Complete);
    foreign.lifecycle.poll().unwrap();
}

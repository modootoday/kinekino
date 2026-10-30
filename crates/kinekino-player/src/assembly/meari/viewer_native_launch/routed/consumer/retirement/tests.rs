use crate::assembly::meari::playback_surface::{PlaybackSurfaceChild, PlaybackSurfaceError};
use meari_native::application::window_lifecycle::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::Arc,
    task::Waker,
    time::Duration,
};

#[derive(Clone, Debug)]
struct Fault(Arc<()>);
impl std::fmt::Display for Fault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("original cleanup fault")
    }
}
impl std::error::Error for Fault {}
#[derive(Clone, Copy)]
enum Step {
    Pending,
    Complete,
    Failed,
}
struct Probe {
    name: &'static str,
    step: Rc<Cell<Step>>,
    events: Rc<RefCell<Vec<&'static str>>>,
    signal: Rc<RefCell<Option<CompletionSignal>>>,
    drops: Rc<Cell<usize>>,
    fault: Fault,
}
impl Probe {
    fn outcome(&self) -> Result<CleanupProgress, Fault> {
        match self.step.get() {
            Step::Pending => Ok(CleanupProgress::Pending),
            Step::Complete => Ok(CleanupProgress::Complete),
            Step::Failed => Err(self.fault.clone()),
        }
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
    }
}
impl NativeChild for Probe {
    type Error = Fault;
    fn parent_lost(&mut self) {}
    fn begin_cleanup(
        &mut self,
        request: CleanupRequest,
        signal: CompletionSignal,
    ) -> Result<CleanupProgress, Fault> {
        assert!(
            request.reason == RevocationReason::ExplicitRelease
                || (std::thread::panicking() && request.reason == RevocationReason::Close)
        );
        assert_eq!(request.stamp, signal.stamp());
        self.events.borrow_mut().push(self.name);
        *self.signal.borrow_mut() = Some(signal);
        self.outcome()
    }
    fn observe_cleanup(&mut self, _: CleanupRequest) -> Result<CleanupProgress, Fault> {
        self.outcome()
    }
}
struct Clock;
impl MonotonicClock for Clock {
    fn now(&self) -> Duration {
        Duration::ZERO
    }
}
struct Child(Box<PlaybackSurfaceChild<Probe, Probe>>);
impl NativeChild for Child {
    type Error = PlaybackSurfaceError<Fault, Fault>;
    fn parent_lost(&mut self) {
        self.0.parent_lost();
    }
    fn begin_cleanup(
        &mut self,
        request: CleanupRequest,
        signal: CompletionSignal,
    ) -> Result<CleanupProgress, Self::Error> {
        self.0.begin_cleanup(request, signal)
    }
    fn observe_cleanup(&mut self, request: CleanupRequest) -> Result<CleanupProgress, Self::Error> {
        self.0.observe_cleanup(request)
    }
    fn observation_error(&self) -> Option<&Self::Error> {
        self.0.observation_error()
    }
}
fn owned(window: WindowStamp, child: u64, engine: Probe, surface: Probe) -> Child {
    Child(Box::new(PlaybackSurfaceChild::new(
        window, child, engine, surface,
    )))
}
struct Fixture {
    children: WindowLifecycle<Child>,
    lease: ChildLease,
    engine: Rc<Cell<Step>>,
    surface: Rc<Cell<Step>>,
    engine_signal: Rc<RefCell<Option<CompletionSignal>>>,
    surface_signal: Rc<RefCell<Option<CompletionSignal>>>,
    events: Rc<RefCell<Vec<&'static str>>>,
    drops: Rc<Cell<usize>>,
    fault: Fault,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.engine.set(Step::Complete);
        self.surface.set(Step::Complete);
        if std::thread::panicking() {
            let _ = self.children.begin_close();
        } else {
            let _ = self.children.release_child(&self.lease);
        }
        // Model-only finalizer also drives the same retained owner when an assertion unwinds.
        for _ in 0..3 {
            for signal in [&self.engine_signal, &self.surface_signal] {
                if let Some(signal) = signal.borrow().as_ref() {
                    signal.notify();
                }
            }
            match self.children.report().map(|report| report.status) {
                Ok(CleanupStatus::Complete) => break,
                Ok(CleanupStatus::RetryableFailure) => {
                    let _ = self.children.retry_failed();
                }
                _ => {
                    let _ = self.children.poll();
                }
            }
        }
    }
}
fn fixture(engine_step: Step, surface_step: Step) -> Fixture {
    let children = WindowLifecycle::new(
        LifecycleLimits::new(2, Duration::from_secs(1)).unwrap(),
        Clock,
        Waker::noop().clone(),
    )
    .unwrap();
    let window = children.ready(ReadyKind::NewWindow).unwrap();
    let engine = Rc::new(Cell::new(engine_step));
    let surface = Rc::new(Cell::new(surface_step));
    let events = Rc::new(RefCell::new(Vec::new()));
    let drops = Rc::new(Cell::new(0));
    let fault = Fault(Arc::new(()));
    let engine_signal = Rc::new(RefCell::new(None));
    let surface_signal = Rc::new(RefCell::new(None));
    let lease = children
        .register(window, |lease| {
            Ok(owned(
                lease.window(),
                lease.id(),
                Probe {
                    name: "engine",
                    step: engine.clone(),
                    events: events.clone(),
                    signal: engine_signal.clone(),
                    drops: drops.clone(),
                    fault: fault.clone(),
                },
                Probe {
                    name: "surface",
                    step: surface.clone(),
                    events: events.clone(),
                    signal: surface_signal.clone(),
                    drops: drops.clone(),
                    fault: fault.clone(),
                },
            ))
        })
        .unwrap();
    Fixture {
        children,
        lease,
        engine,
        surface,
        engine_signal,
        surface_signal,
        events,
        drops,
        fault,
    }
}
#[test]
fn original_explicit_release_joins_engine_before_surface_and_duplicate_begin_never_restarts() {
    let f = fixture(Step::Pending, Step::Pending);
    assert_eq!(
        f.children.release_child(&f.lease).unwrap().status,
        CleanupStatus::Pending
    );
    assert_eq!(&*f.events.borrow(), &["engine"]);
    assert_eq!(
        f.children.release_child(&f.lease).unwrap().status,
        CleanupStatus::Pending
    );
    assert_eq!(&*f.events.borrow(), &["engine"]);
    assert_eq!(f.drops.get(), 0);
    f.engine.set(Step::Complete);
    assert!(f.engine_signal.borrow().as_ref().unwrap().notify());
    assert_eq!(f.children.poll().unwrap().status, CleanupStatus::Pending);
    assert_eq!(&*f.events.borrow(), &["engine", "surface"]);
    assert_eq!(f.drops.get(), 0);
    f.surface.set(Step::Complete);
    assert!(f.surface_signal.borrow().as_ref().unwrap().notify());
    assert_eq!(f.children.poll().unwrap().status, CleanupStatus::Complete);
    assert_eq!(f.drops.get(), 2);
    assert_eq!(f.children.phase().unwrap(), WindowPhase::Ready);
    let fresh = f
        .children
        .register(f.children.stamp().unwrap(), |lease| {
            Ok(owned(
                lease.window(),
                lease.id(),
                Probe {
                    name: "recovered-engine",
                    step: f.engine.clone(),
                    events: f.events.clone(),
                    signal: f.engine_signal.clone(),
                    drops: f.drops.clone(),
                    fault: f.fault.clone(),
                },
                Probe {
                    name: "recovered-surface",
                    step: f.surface.clone(),
                    events: f.events.clone(),
                    signal: f.surface_signal.clone(),
                    drops: f.drops.clone(),
                    fault: f.fault.clone(),
                },
            ))
        })
        .unwrap();
    assert_ne!(fresh.id(), f.lease.id());
    assert_eq!(
        f.children.release_child(&fresh).unwrap().status,
        CleanupStatus::Complete
    );
}
#[test]
fn cleanup_failure_retains_original_engine_surface_and_error_without_new_registration() {
    let f = fixture(Step::Failed, Step::Complete);
    assert_eq!(
        f.children.release_child(&f.lease).unwrap().status,
        CleanupStatus::RetryableFailure
    );
    assert_eq!(&*f.events.borrow(), &["engine"]);
    assert_eq!(f.drops.get(), 0);
    f.children
        .with_error(&f.lease, |error| match error.unwrap() {
            PlaybackSurfaceError::Engine(original) => assert!(Arc::ptr_eq(&original.0, &f.fault.0)),
            _ => panic!("Original engine error was replaced"),
        })
        .unwrap();
    assert_eq!(
        f.children.poll().unwrap().status,
        CleanupStatus::RetryableFailure
    );
    assert_eq!(f.drops.get(), 0);
    // Explicit original-owner recovery, not an automatic retry or a new surface.
    f.engine.set(Step::Complete);
    assert_eq!(
        f.children.retry_failed().unwrap().status,
        CleanupStatus::Complete
    );
    assert_eq!(&*f.events.borrow(), &["engine", "engine", "surface"]);
    assert_eq!(f.drops.get(), 2);
    assert_eq!(f.children.phase().unwrap(), WindowPhase::Ready);
    let fresh = f
        .children
        .register(f.children.stamp().unwrap(), |lease| {
            Ok(owned(
                lease.window(),
                lease.id(),
                Probe {
                    name: "recovered-engine",
                    step: f.engine.clone(),
                    events: f.events.clone(),
                    signal: f.engine_signal.clone(),
                    drops: f.drops.clone(),
                    fault: f.fault.clone(),
                },
                Probe {
                    name: "recovered-surface",
                    step: f.surface.clone(),
                    events: f.events.clone(),
                    signal: f.surface_signal.clone(),
                    drops: f.drops.clone(),
                    fault: f.fault.clone(),
                },
            ))
        })
        .unwrap();
    assert_ne!(fresh.id(), f.lease.id());
    assert_eq!(
        f.children.release_child(&fresh).unwrap().status,
        CleanupStatus::Complete
    );
    assert_eq!(f.drops.get(), 4);
}
#[test]
fn completed_child_release_keeps_window_and_fresh_registration_rejects_old_lease() {
    let f = fixture(Step::Complete, Step::Complete);
    let window = f.children.stamp().unwrap();
    assert_eq!(
        f.children.release_child(&f.lease).unwrap().status,
        CleanupStatus::Complete
    );
    assert_eq!(f.children.stamp().unwrap(), window);
    let fresh = f
        .children
        .register(window, |lease| {
            Ok(owned(
                lease.window(),
                lease.id(),
                Probe {
                    name: "new-engine",
                    step: f.engine.clone(),
                    events: f.events.clone(),
                    signal: f.engine_signal.clone(),
                    drops: f.drops.clone(),
                    fault: f.fault.clone(),
                },
                Probe {
                    name: "new-surface",
                    step: f.surface.clone(),
                    events: f.events.clone(),
                    signal: f.surface_signal.clone(),
                    drops: f.drops.clone(),
                    fault: f.fault.clone(),
                },
            ))
        })
        .unwrap();
    assert_ne!(fresh.id(), f.lease.id());
    assert_eq!(
        f.children.validate_access(&f.lease),
        Err(LifecycleError::Stale)
    );
    assert_eq!(
        f.children.release_child(&f.lease).unwrap_err(),
        LifecycleError::Stale
    );
    assert!(f.children.validate_access(&fresh).is_ok());
    assert_eq!(
        f.children.release_child(&fresh).unwrap().status,
        CleanupStatus::Complete
    );
}

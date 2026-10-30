//! Real task/dispatcher and pure lifecycle coordinator; synthetic stamps are not OS evidence.
use super::*;
use crate::{
    assembly::meari::playback_surface::{PlaybackSurfaceChild, PlaybackSurfaceError},
    infrastructure::thread_completion::{DispatcherHost, DispatcherShutdown, JoinRequest},
};
use meari_native::application::window_lifecycle::*;
use std::{
    cell::{Cell, RefCell},
    num::NonZeroUsize,
    rc::Rc,
    sync::mpsc,
    task::Waker,
    time::{Duration, Instant},
};
mod fixture;
const LIMIT: Duration = Duration::from_secs(5);

struct EnginePort {
    task: Rc<RefCell<UnboundCleanup>>,
    diagnostic: Option<RoutedChildError>,
}
impl NativeChild for EnginePort {
    type Error = RoutedChildError;
    fn parent_lost(&mut self) {}
    fn begin_cleanup(
        &mut self,
        _: CleanupRequest,
        signal: CompletionSignal,
    ) -> Result<CleanupProgress, Self::Error> {
        let result = self.task.borrow_mut().begin(signal);
        self.diagnostic = result.as_ref().err().cloned();
        result
    }
    fn observe_cleanup(&mut self, _: CleanupRequest) -> Result<CleanupProgress, Self::Error> {
        let result = self.task.borrow_mut().observe();
        self.diagnostic = result.as_ref().err().cloned();
        result
    }
    fn observation_error(&self) -> Option<&Self::Error> {
        self.diagnostic.as_ref()
    }
}
struct SurfaceProbe(Rc<Cell<usize>>);
impl NativeChild for SurfaceProbe {
    type Error = RoutedChildError;
    fn parent_lost(&mut self) {}
    fn begin_cleanup(
        &mut self,
        _: CleanupRequest,
        _: CompletionSignal,
    ) -> Result<CleanupProgress, Self::Error> {
        self.0.set(self.0.get() + 1);
        Ok(CleanupProgress::Complete)
    }
    fn observe_cleanup(&mut self, _: CleanupRequest) -> Result<CleanupProgress, Self::Error> {
        Ok(CleanupProgress::Complete)
    }
}
#[derive(Clone)]
struct Clock;
impl MonotonicClock for Clock {
    fn now(&self) -> Duration {
        Duration::ZERO
    }
}
fn finish(dispatcher: &mut DispatcherHost) {
    dispatcher.request_shutdown();
    let deadline = Instant::now() + LIMIT;
    loop {
        match dispatcher.poll_shutdown() {
            DispatcherShutdown::Joined {
                worker_panicked: false,
                faults: 0,
            } => break,
            DispatcherShutdown::Pending => {
                assert!(Instant::now() < deadline);
                std::thread::yield_now();
            }
            other => panic!("Unexpected original dispatcher shutdown: {other:?}"),
        }
    }
}

#[test]
#[expect(
    clippy::result_large_err,
    reason = "WindowLifecycle::register requires inline ChildCreationFailure retaining the original partial cleanup child and error"
)]
fn refusal_retains_original_task_and_engine_then_explicit_retry_joins_before_surface() {
    let mut dispatcher = DispatcherHost::start(NonZeroUsize::new(1).unwrap()).unwrap();
    let router = ParentEngineRouter::new(dispatcher.handle());
    let engine = fixture::engine();
    let weak_engine = Arc::downgrade(&engine);
    let task = Rc::new(RefCell::new(UnboundCleanup::new(&router, &engine).unwrap()));
    let original_task = std::ptr::from_ref(&task.borrow().task);
    drop(engine);
    assert!(weak_engine.upgrade().is_some());
    let handle = dispatcher.handle();
    let (entered, started) = mpsc::channel();
    let (release, blocked) = mpsc::channel();
    let mut blocker = handle
        .submit(JoinRequest::new(
            move || {
                entered.send(()).unwrap();
                blocked.recv().unwrap();
            },
            Arc::new(|| Ok(())),
        ))
        .unwrap();
    started.recv_timeout(LIMIT).unwrap();
    let (done, filler_done) = mpsc::channel();
    let mut filler = handle
        .submit(JoinRequest::new(
            || (),
            Arc::new(move || done.send(()).map_err(|error| error.to_string())),
        ))
        .unwrap();
    let surface_begins = Rc::new(Cell::new(0));
    let lifecycle = WindowLifecycle::new(
        LifecycleLimits::new(1, Duration::from_secs(10)).unwrap(),
        Clock,
        Waker::noop().clone(),
    )
    .unwrap();
    let window = lifecycle.ready(ReadyKind::NewWindow).unwrap();
    let lease = lifecycle
        .register(window, |lease| {
            Ok(PlaybackSurfaceChild::new(
                lease.window(),
                lease.id(),
                EnginePort {
                    task: task.clone(),
                    diagnostic: None,
                },
                SurfaceProbe(surface_begins.clone()),
            ))
        })
        .unwrap();
    assert_eq!(
        lifecycle.begin_close().unwrap().status,
        CleanupStatus::RetryableFailure
    );
    let retained = lifecycle
        .with_error(&lease, |error| error.cloned())
        .unwrap()
        .unwrap();
    let PlaybackSurfaceError::Engine(RoutedChildError::Api(ref refused)) = retained else {
        panic!("Original refusal missing");
    };
    assert_eq!(refused.code, ErrorCode::ResourceExhausted);
    assert_eq!(surface_begins.get(), 0);
    assert!(matches!(
        task.borrow_mut().task.observe(),
        ParentCleanupObservation::NotStarted
    ));
    assert!(task.borrow().task.notification().is_none());
    assert_eq!(task.borrow().wakes.lock().unwrap().len(), 0);
    for _ in 0..2 {
        assert_eq!(
            lifecycle.poll().unwrap().status,
            CleanupStatus::RetryableFailure
        );
    }
    assert_eq!(surface_begins.get(), 0);
    release.send(()).unwrap();
    filler_done.recv_timeout(LIMIT).unwrap();
    assert!(matches!(blocker.try_take(), Some(Ok(()))));
    assert!(matches!(filler.try_take(), Some(Ok(()))));
    let (entered2, started2) = mpsc::channel();
    let (release2, blocked2) = mpsc::channel();
    let mut blocker2 = handle
        .submit(JoinRequest::new(
            move || {
                entered2.send(()).unwrap();
                blocked2.recv().unwrap();
            },
            Arc::new(|| Ok(())),
        ))
        .unwrap();
    started2.recv_timeout(LIMIT).unwrap();
    lifecycle.retry_failed().unwrap();
    assert_eq!(std::ptr::from_ref(&task.borrow().task), original_task);
    assert_eq!(task.borrow().wakes.lock().unwrap().len(), 1);
    assert!(matches!(
        task.borrow_mut().task.observe(),
        ParentCleanupObservation::Pending
    ));
    assert_eq!(surface_begins.get(), 0);
    lifecycle.poll().unwrap();
    assert_eq!(surface_begins.get(), 0);
    assert!(weak_engine.upgrade().is_some());
    release2.send(()).unwrap();
    let deadline = Instant::now() + LIMIT;
    while handle.status().outstanding != 0 {
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    assert!(matches!(blocker2.try_take(), Some(Ok(()))));
    assert_eq!(lifecycle.poll().unwrap().status, CleanupStatus::Complete);
    assert_eq!(surface_begins.get(), 1);
    assert!(matches!(
        task.borrow_mut().task.observe(),
        ParentCleanupObservation::Finished(Ok(ParentCleanupStatus::Complete(None)))
    ));
    assert_eq!(refused.code, ErrorCode::ResourceExhausted);
    assert_eq!(task.borrow().wakes.lock().unwrap().len(), 0);
    finish(&mut dispatcher);
}

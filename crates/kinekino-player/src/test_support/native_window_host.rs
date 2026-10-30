//! Existing own-window startup/host adapter shared by concrete NoMedia scenarios.
use meari_native::{
    application::{
        WindowOptions,
        window_lifecycle::{LifecycleLimits, NativeChild, WindowLifecycle},
    },
    infrastructure::{native_host::*, window::*},
};
use std::{
    cell::RefCell,
    io,
    rc::Rc,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

pub(crate) trait Scenario: Sized + 'static {
    type Child: NativeChild;
    type Evidence: 'static;
    fn prepare(
        document: NativeDocumentHandle,
        video: String,
        children: Rc<WindowLifecycle<Self::Child>>,
        surfaces: NativeSurfaceRegistrar<Self::Child>,
        deadline: Instant,
        evidence: Rc<RefCell<Self::Evidence>>,
    ) -> Self;
    fn submitted(&mut self, notice: NativeWindowNotice) -> Result<(), String>;
    fn failed(&mut self, error: String);
    fn after_dispatch(&mut self, turn: HostTurn) -> Result<HostEffects, io::Error>;
    fn set_wake(&mut self, wake: NativeWindowWake);
    fn close(&self);
}
type Shared<S> = Rc<RefCell<Option<S>>>;
pub(crate) struct Host<S: Scenario>(Shared<S>);
impl<S: Scenario> NativeWindowHost for Host<S> {
    type Error = io::Error;
    type CloseCheckpoint = ();
    fn admit_callback(
        &mut self,
        _: &HostStamp,
        _: &meari::application::host_callbacks::PendingCallback,
    ) -> IntentAdmission<Self::Error> {
        IntentAdmission::Rejected(io::Error::other("own-window fixture has no user callbacks"))
    }
    fn after_dispatch(&mut self, turn: HostTurn) -> Result<HostEffects, Self::Error> {
        self.0
            .borrow_mut()
            .as_mut()
            .ok_or_else(|| io::Error::other("fixture not prepared"))?
            .after_dispatch(turn)
    }
    fn retry_after_dispatch(&self, error: &Self::Error) -> Option<HostRetry> {
        (error.kind() == io::ErrorKind::WouldBlock)
            .then_some(HostRetry::After(Duration::from_millis(20)))
    }
    fn close(&mut self, _: CloseRequest<()>) -> CloseDecision<(), Self::Error> {
        if let Some(fixture) = self.0.borrow().as_ref() {
            fixture.close();
        }
        CloseDecision::HostCleanupComplete { checkpoint: () }
    }
}
type Runtime<S> = NativeHostRuntime<ChildWindowHost<Host<S>, <S as Scenario>::Child>>;
pub(crate) struct Controller<S: Scenario> {
    pub(crate) pending: Option<(NativeDocumentHandle, String)>,
    pub(crate) shared: Shared<S>,
    pub(crate) evidence: Rc<RefCell<S::Evidence>>,
    pub(crate) deadline: Instant,
    pub(crate) cancel: Option<mpsc::Sender<()>>,
    pub(crate) timer: Option<thread::JoinHandle<()>>,
    pub(crate) attach_failure: Option<Box<NativeAttachFailure<Host<S>, S::Child>>>,
}
impl<S: Scenario> NativeStartupController for Controller<S> {
    type Host = ChildWindowHost<Host<S>, S::Child>;
    type Error = io::Error;
    fn observe(
        &mut self,
        context: &mut NativeStartupContext<'_>,
    ) -> Result<NativeStartupPoll<Self::Host>, Self::Error> {
        let (document, video) = self
            .pending
            .take()
            .ok_or_else(|| io::Error::other("startup reused"))?;
        let shared = self.shared.clone();
        let observer: NativeWindowObserver = Box::new(move |notice| {
            let mut shared = shared.try_borrow_mut().map_err(|e| format!("{e:?}"))?;
            if let Some(fixture) = shared.as_mut()
                && let Err(error) = fixture.submitted(notice)
            {
                fixture.failed(error.clone());
                return Err(error);
            }
            Ok(())
        });
        let preparation = context
            .prepare::<S::Child>(
                document.clone(),
                LifecycleLimits::new(2, Duration::from_secs(2)).map_err(io::Error::other)?,
                WindowOptions::default(),
                Some(observer),
            )
            .map_err(io::Error::other)?;
        let children = preparation.children();
        *self.shared.borrow_mut() = Some(S::prepare(
            document.clone(),
            video,
            children.clone(),
            preparation.surface_slots(),
            self.deadline,
            self.evidence.clone(),
        ));
        let runtime = Rc::new(RefCell::new(NativeHostRuntime::with_schedule_limits(
            document,
            ChildWindowHost::new(Host(self.shared.clone()), children),
            HostScheduleLimits::new(Duration::from_millis(10), Duration::from_secs(1), 1024)
                .map_err(io::Error::other)?,
        )));
        match context.attach_prepared(preparation, runtime) {
            Ok(attachment) => Ok(NativeStartupPoll::Attach(attachment)),
            Err(failure) => {
                let error = io::Error::other(format!("{:?}", failure.problem));
                self.attach_failure = Some(failure);
                Err(error)
            }
        }
    }
    fn attached(
        &mut self,
        _: Rc<RefCell<Runtime<S>>>,
        wake: NativeWindowWake,
    ) -> Result<(), Self::Error> {
        self.shared
            .borrow_mut()
            .as_mut()
            .ok_or_else(|| io::Error::other("missing prepared state"))?
            .set_wake(wake.clone());
        let (send, receive) = mpsc::channel();
        self.cancel = Some(send);
        let remaining = self.deadline.saturating_duration_since(Instant::now());
        let timer_wake = wake.clone();
        self.timer = Some(thread::spawn(move || {
            if matches!(
                receive.recv_timeout(remaining),
                Err(mpsc::RecvTimeoutError::Timeout)
            ) {
                let _ = timer_wake.wake();
            }
        }));
        wake.wake().map_err(io::Error::other)?;
        Ok(())
    }
}

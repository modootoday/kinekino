//! Same-loop attachment over the existing Native runtime and child-first close driver.
mod delivery;
pub(crate) mod registration_redraw;
#[cfg(test)]
use delivery::record_delivery;
use delivery::{ChildrenSlot, make_observer_with_frames};
pub use delivery::{ObservationFailure, ObservationFailureStage};
mod observation;
use super::player_shell_document::PreparedShellDocument;
use crate::{
    application::api::ApplicationClientHandle,
    presentation::web_ui_adapter::player_shell::{NavigationError, NavigationGuards},
};
use meari_native::{
    application::{
        WindowOptions,
        window_lifecycle::{LifecycleLimits, NativeChild},
    },
    infrastructure::{
        native_host::{
            ChildWindowHost, HostAccessError, HostScheduleLimits, NativeDocumentHandle,
            NativeHostRuntime, NativeWindowHost,
        },
        window::{
            NativeAttachFailure, NativeLoopOwner, NativeNoticeError, NativeRunOutcome,
            NativeStartup, NativeStartupContext, NativeStartupController, NativeStartupError,
            NativeStartupPoll, NativeSurfaceRegistrar, NativeWindowNotice, NativeWindowObservation,
            NativeWindowPreparation, NativeWindowWake,
        },
    },
};
pub use observation::StartRouteObservation;
use std::{
    cell::{Cell, RefCell},
    fmt,
    marker::PhantomData,
    rc::Rc,
};

#[derive(Debug)]
pub enum ObserverRetryError {
    NotAttached,
    Native(NativeNoticeError),
}

pub type ShellRuntime<H, C> = NativeHostRuntime<ChildWindowHost<H, C>>;
#[allow(
    dead_code,
    reason = "Opt-in frame installation and original-owner recovery are not enabled by the default host"
)]
pub(crate) struct FrameBinding<C: NativeChild> {
    pub document: NativeDocumentHandle,
    pub registrar: NativeSurfaceRegistrar<C>,
    pub children: Rc<meari_native::application::window_lifecycle::WindowLifecycle<C>>,
}
#[derive(Debug)]
pub(crate) struct FrameHookError(pub &'static str);
impl fmt::Display for FrameHookError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for FrameHookError {}
#[allow(
    dead_code,
    reason = "Opt-in frame installation and original-owner recovery are not enabled by the default host"
)]
pub(crate) struct FrameBindFailure<C: NativeChild> {
    pub error: FrameHookError,
    pub original: FrameBinding<C>,
}
/// Typed same-loop hook. Observations return only a wake intent after all checked borrows end.
pub(crate) trait ShellFrameConsumer<C: NativeChild>: 'static {
    fn bind(&mut self, binding: FrameBinding<C>) -> Result<(), Box<FrameBindFailure<C>>>;
    fn observe(&mut self, notice: &NativeWindowNotice) -> Result<bool, FrameHookError>;
    fn routed_observation(&self) -> Option<RoutedFrameObservation> {
        None
    }
}
#[derive(Clone, Default)]
pub(crate) struct RoutedFrameObservation {
    pub ready: Option<std::sync::Arc<crate::application::ui_observation::NativeRoutedFrameReadyV1>>,
    pub diagnostic_error: Option<std::sync::Arc<std::io::Error>>,
    pub trace: Option<std::sync::Arc<registration_redraw::RoutedFrameTrace>>,
}
pub(crate) type FrameConsumerSlot<C> = Rc<RefCell<Option<Box<dyn ShellFrameConsumer<C>>>>>;
#[allow(
    dead_code,
    reason = "Opt-in frame installation and original-owner recovery are not enabled by the default host"
)]
pub(crate) struct FrameSetupFailure<C: NativeChild> {
    pub failure: Box<FrameBindFailure<C>>,
    pub preparation: NativeWindowPreparation<C>,
}
pub struct ShellHostConfig {
    pub options: WindowOptions,
    pub children: LifecycleLimits,
    pub schedule: HostScheduleLimits,
}
#[derive(Debug)]
pub enum ShellHostError {
    AlreadyAttempted,
    Preparation,
    Attach,
    MissingOwner,
    FrameBinding,
}
impl fmt::Display for ShellHostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "shell host: {self:?}")
    }
}
impl std::error::Error for ShellHostError {}

#[must_use = "Retain the original startup, runtime, recovery and loop owners on return"]
pub struct ShellHostController<H: NativeWindowHost, C: NativeChild> {
    pub client: ApplicationClientHandle,
    pub route: Rc<RefCell<StartRouteObservation>>,
    pending: Option<(NativeDocumentHandle, H)>,
    config: ShellHostConfig,
    attempted: bool,
    pub preparation_error: Option<NativeStartupError>,
    pub attach_failure: Option<Box<NativeAttachFailure<H, C>>>,
    pub runtime: Option<Rc<RefCell<ShellRuntime<H, C>>>>,
    pub wake: Option<NativeWindowWake>,
    observation: Option<NativeWindowObservation>,
    observation_failure: Rc<Cell<Option<ObservationFailure>>>,
    child_type: PhantomData<C>,
    frame_consumer: FrameConsumerSlot<C>,
    frame_wake: Rc<RefCell<Option<NativeWindowWake>>>,
    window_actions: Rc<RefCell<Option<meari_native::infrastructure::window::NativeWindowActions>>>,
    pub(crate) frame_setup_failure: Option<Box<FrameSetupFailure<C>>>,
}
impl<H: NativeWindowHost, C: NativeChild> ShellHostController<H, C> {
    pub fn new(prepared: PreparedShellDocument<H>, config: ShellHostConfig) -> Self {
        Self::new_with_route_binding(prepared, config, |_, _| {})
    }
    /// Construction-time sharing of the original route observer, before any runtime/OS attach.
    pub(crate) fn new_with_route_binding(
        mut prepared: PreparedShellDocument<H>,
        config: ShellHostConfig,
        bind: impl FnOnce(&mut H, Rc<RefCell<StartRouteObservation>>),
    ) -> Self {
        let route = Rc::new(RefCell::new(StartRouteObservation::new(
            prepared.state,
            prepared.ticket,
        )));
        bind(&mut prepared.inputs.host_owner, route.clone());
        Self {
            client: prepared.inputs.client,
            route,
            pending: Some((prepared.document, prepared.inputs.host_owner)),
            config,
            attempted: false,
            preparation_error: None,
            attach_failure: None,
            runtime: None,
            wake: None,
            observation: None,
            observation_failure: Rc::new(Cell::new(None)),
            child_type: PhantomData,
            frame_consumer: Rc::new(RefCell::new(None)),
            frame_wake: Rc::new(RefCell::new(None)),
            window_actions: Rc::new(RefCell::new(None)),
            frame_setup_failure: None,
        }
    }
    pub fn pending_owner(&self) -> Option<&H> {
        self.pending.as_ref().map(|(_, host)| host)
    }
    pub(crate) fn frame_wake_source(&self) -> Rc<RefCell<Option<NativeWindowWake>>> {
        self.frame_wake.clone()
    }
    pub(crate) fn window_actions_source(
        &self,
    ) -> Rc<RefCell<Option<meari_native::infrastructure::window::NativeWindowActions>>> {
        self.window_actions.clone()
    }
    #[cfg(test)]
    pub(crate) fn pending_document_for_test(&self) -> Option<&NativeDocumentHandle> {
        self.pending.as_ref().map(|(document, _)| document)
    }
    #[allow(
        dead_code,
        reason = "Opt-in frame installation and original-owner recovery are not enabled by the default host"
    )]
    pub(crate) fn frame_setup_error(&self) -> Option<&FrameHookError> {
        self.frame_setup_failure
            .as_ref()
            .map(|failure| &failure.failure.error)
    }
    /// Returns the original unrun preparation/binding after a failed opt-in bind; no retry authority is minted.
    #[allow(
        dead_code,
        reason = "Opt-in frame installation and original-owner recovery are not enabled by the default host"
    )]
    pub(crate) fn take_frame_setup_failure(&mut self) -> Option<Box<FrameSetupFailure<C>>> {
        self.frame_setup_failure.take()
    }
    /// Installs once before startup. Rejection returns the same consumer with every original owner.
    #[allow(
        dead_code,
        reason = "Opt-in frame installation and original-owner recovery are not enabled by the default host"
    )]
    pub(crate) fn install_frame_consumer(
        &mut self,
        consumer: Box<dyn ShellFrameConsumer<C>>,
    ) -> Result<(), Box<dyn ShellFrameConsumer<C>>> {
        if self.attempted {
            return Err(consumer);
        }
        let Ok(mut slot) = self.frame_consumer.try_borrow_mut() else {
            return Err(consumer);
        };
        if slot.is_some() {
            return Err(consumer);
        }
        *slot = Some(consumer);
        Ok(())
    }
    /// An owned metadata/error snapshot; absence/borrow failure never means successful cleanup.
    pub(crate) fn routed_frame_observation(
        &self,
    ) -> Result<Option<RoutedFrameObservation>, FrameHookError> {
        let slot = self
            .frame_consumer
            .try_borrow()
            .map_err(|_| FrameHookError("Original frame consumer observation borrowed"))?;
        Ok(slot
            .as_ref()
            .and_then(|consumer| consumer.routed_observation()))
    }
    /// Cleanup access to the retained original host, never transfer/drop authority.
    pub fn pending_owner_mut(&mut self) -> Option<&mut H> {
        self.pending.as_mut().map(|(_, host)| host)
    }
    fn bind_runtime(
        &mut self,
        document: NativeDocumentHandle,
        host: H,
        children: Rc<meari_native::application::window_lifecycle::WindowLifecycle<C>>,
    ) -> Rc<RefCell<ShellRuntime<H, C>>> {
        let runtime = Rc::new(RefCell::new(NativeHostRuntime::with_schedule_limits(
            document,
            ChildWindowHost::new(host, children),
            self.config.schedule,
        )));
        self.runtime = Some(runtime.clone());
        runtime
    }
    #[cfg(test)]
    pub(crate) fn bind_unrun_runtime_for_test(
        &mut self,
        children: Rc<meari_native::application::window_lifecycle::WindowLifecycle<C>>,
    ) -> Result<Rc<RefCell<ShellRuntime<H, C>>>, ShellHostError> {
        let (document, host) = self.pending.take().ok_or(ShellHostError::MissingOwner)?;
        self.attempted = true;
        Ok(self.bind_runtime(document, host, children))
    }
    pub fn observation_failure(&self) -> Option<ObservationFailure> {
        self.observation_failure.get()
    }
    /// Explicit same-observer replay of the retained latest-per-kind queue.
    /// New notices of the same kind may supersede the failed revision while paused.
    /// No new loop, frame, attachment or SDK work is created, and failure remains visible.
    pub fn retry_observer(&self) -> Result<(), ObserverRetryError> {
        self.observation
            .as_ref()
            .ok_or(ObserverRetryError::NotAttached)?
            .retry_observer()
            .map_err(ObserverRetryError::Native)
    }
    /// Delivery is an intent only; the existing H and ChildWindowHost decide cleanup/KeepOpen.
    pub fn request_close(&mut self, guards: NavigationGuards) -> Result<bool, ShellCloseError> {
        let wake = self.wake.as_ref().ok_or(ShellCloseError::NotAttached)?;
        self.route
            .try_borrow_mut()
            .map_err(|_| ShellCloseError::Busy)?
            .state
            .request_close(guards)
            .map_err(ShellCloseError::Navigation)?;
        wake.request_close().map_err(ShellCloseError::Host)
    }
    /// Explicit UI recovery only; does not cancel host cleanup or reopen a native owner.
    pub fn dismiss_close_intent(&mut self) -> Result<(), ShellCloseError> {
        self.route
            .try_borrow_mut()
            .map_err(|_| ShellCloseError::Busy)?
            .state
            .dismiss_close_request();
        Ok(())
    }
}

#[cfg(test)]
#[path = "player_shell_host/route_binding_tests.rs"]
mod route_binding_tests;
#[derive(Debug)]
pub enum ShellCloseError {
    NotAttached,
    Busy,
    Navigation(NavigationError),
    Host(HostAccessError),
}

impl<H: NativeWindowHost, C: NativeChild> NativeStartupController for ShellHostController<H, C> {
    type Host = ChildWindowHost<H, C>;
    type Error = ShellHostError;
    fn observe(
        &mut self,
        context: &mut NativeStartupContext<'_>,
    ) -> Result<NativeStartupPoll<Self::Host>, Self::Error> {
        if std::mem::replace(&mut self.attempted, true) {
            return Err(ShellHostError::AlreadyAttempted);
        }
        let (document, host) = self.pending.take().ok_or(ShellHostError::MissingOwner)?;
        let children_slot: ChildrenSlot<C> = Rc::new(RefCell::new(None));
        let observer = make_observer_with_frames(
            document.clone(),
            children_slot.clone(),
            self.route.clone(),
            self.observation_failure.clone(),
            Rc::downgrade(&self.frame_consumer),
            self.frame_wake.clone(),
        );
        let preparation = match context.prepare::<C>(
            document.clone(),
            self.config.children,
            self.config.options.clone(),
            Some(observer),
        ) {
            Ok(preparation) => preparation,
            Err(error) => {
                self.pending = Some((document, host));
                self.preparation_error = Some(error);
                return Err(ShellHostError::Preparation);
            }
        };
        let children = preparation.children();
        *self.window_actions.borrow_mut() = Some(preparation.window_actions());
        *children_slot.borrow_mut() = Some(children.clone());
        let binding = FrameBinding {
            document: document.clone(),
            registrar: preparation.surface_slots(),
            children: children.clone(),
        };
        let bound = match self.frame_consumer.try_borrow_mut() {
            Ok(mut slot) => match slot.as_mut() {
                Some(consumer) => consumer.bind(binding),
                None => Ok(()),
            },
            Err(_) => Err(Box::new(FrameBindFailure {
                error: FrameHookError("frame consumer borrowed during bind"),
                original: binding,
            })),
        };
        #[cfg(test)]
        if let Some(probe) = super::viewer_native_probe::current() {
            let stamp = document.stamp();
            probe.record(
                super::viewer_native_probe::Kind::Bind,
                &(
                    bound.is_ok(),
                    stamp.as_ref().ok().map(super::viewer_native_probe::stamp),
                    bound.as_ref().err().map(|failure| failure.error.0),
                ),
            );
        }
        if let Err(failure) = bound {
            self.pending = Some((document, host));
            self.frame_setup_failure = Some(Box::new(FrameSetupFailure {
                failure,
                preparation,
            }));
            return Err(ShellHostError::FrameBinding);
        }
        let runtime = self.bind_runtime(document, host, children);
        match context.attach_prepared(preparation, runtime) {
            Ok(attachment) => {
                self.observation = attachment.observation();
                Ok(NativeStartupPoll::Attach(attachment))
            }
            Err(failure) => {
                self.attach_failure = Some(failure);
                Err(ShellHostError::Attach)
            }
        }
    }
    fn attached(
        &mut self,
        runtime: Rc<RefCell<ShellRuntime<H, C>>>,
        wake: NativeWindowWake,
    ) -> Result<(), Self::Error> {
        #[cfg(test)]
        {
            let route = Rc::downgrade(&self.route);
            let close_wake = wake.clone();
            super::routed_owner_fixture::attached(wake.clone(), move || {
                let route = route.upgrade().ok_or("original route expired")?;
                route
                    .try_borrow_mut()
                    .map_err(|_| "original route borrowed")?
                    .state
                    .request_close(NavigationGuards::default())
                    .map_err(|_| "original navigation guard rejected")?;
                close_wake
                    .request_close()
                    .map_err(|_| "original close wake rejected")
            });
        }
        #[cfg(test)]
        if let Some(probe) = super::viewer_native_probe::current() {
            let children = runtime
                .try_borrow()
                .ok()
                .map(|runtime| Rc::downgrade(runtime.host().children()));
            let route = Rc::downgrade(&self.route);
            let close_wake = wake.clone();
            probe.attach(
                move || {
                    let route = route.upgrade().ok_or("route expired")?;
                    route
                        .try_borrow_mut()
                        .map_err(|_| "route borrowed")?
                        .state
                        .request_close(NavigationGuards::default())
                        .map_err(|_| "navigation rejected")?;
                    close_wake.request_close().map_err(|_| "wake rejected")
                },
                move || {
                    let children = children.as_ref()?.upgrade()?;
                    Some((children.report().ok()?, children.stamp().ok()?))
                },
            );
        }
        self.runtime = Some(runtime);
        *self.frame_wake.borrow_mut() = Some(wake.clone());
        self.wake = Some(wake);
        // Notification only: actual View/frame has not been created yet.
        Ok(())
    }
}

pub fn run_shell_host<H: NativeWindowHost, C: NativeChild>(
    owner: NativeLoopOwner,
    startup: NativeStartup,
    controller: ShellHostController<H, C>,
) -> NativeRunOutcome<ShellHostController<H, C>> {
    owner.run_startup(startup, controller)
}

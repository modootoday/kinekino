use super::*;
use crate::application::route_media::{
    RetireTicket, RouteMediaError, RouteMediaPort, RouteMediaProgress, ViewingTicket,
};
use meari::application::plugins::PluginScope;
use std::cell::RefCell;
use std::rc::Weak;

pub(crate) struct RoutedFrameConsumer {
    inner: Rc<RefCell<RoutedFrameState>>,
}
struct OriginalPort(Weak<RefCell<RoutedFrameState>>);
impl RoutedFrameConsumer {
    pub(crate) fn new(
        video: String,
        origin: HostStamp,
        executable: TrustedMpvExecutable,
        artifacts: Arc<dyn CatalogArtifacts>,
        pending: PendingRoutedActivation,
        redraw: Rc<
            crate::assembly::meari::player_shell_host::registration_redraw::RegistrationRedraw,
        >,
    ) -> Self {
        Self {
            inner: Rc::new(RefCell::new(RoutedFrameState::new(
                video, origin, executable, artifacts, pending, redraw,
            ))),
        }
    }
    pub(crate) fn media_port(&self) -> Rc<dyn RouteMediaPort<PluginScope>> {
        Rc::new(OriginalPort(Rc::downgrade(&self.inner)))
    }
}
impl OriginalPort {
    fn with<T>(
        &self,
        action: impl FnOnce(&mut RoutedFrameState) -> Result<T, RouteMediaError>,
    ) -> Result<T, RouteMediaError> {
        let owner = self
            .0
            .upgrade()
            .ok_or_else(|| retirement::native_error("Original media consumer expired"))?;
        let mut owner = owner
            .try_borrow_mut()
            .map_err(|_| retirement::native_error("Original media consumer is borrowed"))?;
        action(&mut owner)
    }
}
impl RouteMediaPort<PluginScope> for OriginalPort {
    fn begin_retire(
        &self,
        owner: &PluginScope,
    ) -> Result<RetireTicket<PluginScope>, RouteMediaError> {
        self.with(|state| state.begin_retirement(owner))
    }
    fn poll_retire(
        &self,
        ticket: &RetireTicket<PluginScope>,
    ) -> Result<RouteMediaProgress, RouteMediaError> {
        self.with(|state| {
            let progress = state.media.policy.poll_retire(ticket)?;
            if progress == RouteMediaProgress::Pending {
                state.drive_retirement()?;
            }
            state.media.policy.poll_retire(ticket)
        })
    }
    fn cancel_retire(&self, ticket: &RetireTicket<PluginScope>) -> Result<(), RouteMediaError> {
        self.with(|state| state.media.policy.cancel_retire(ticket))
    }
    fn begin_viewing(
        &self,
        owner: &PluginScope,
        retired: &RetireTicket<PluginScope>,
    ) -> Result<ViewingTicket<PluginScope>, RouteMediaError> {
        self.with(|state| state.begin_viewing(owner, retired))
    }
    fn poll_viewing(
        &self,
        ticket: &ViewingTicket<PluginScope>,
    ) -> Result<RouteMediaProgress, RouteMediaError> {
        self.with(|state| state.media.policy.poll_viewing(ticket))
    }
}
impl ShellFrameConsumer<RegisteredRoutedChild> for RoutedFrameConsumer {
    fn bind(
        &mut self,
        binding: FrameBinding<RegisteredRoutedChild>,
    ) -> Result<(), Box<FrameBindFailure<RegisteredRoutedChild>>> {
        match self.inner.try_borrow_mut() {
            Ok(mut state) => state.bind(binding),
            Err(_) => Err(Box::new(FrameBindFailure {
                error: FrameHookError("Original media consumer borrowed during bind"),
                original: binding,
            })),
        }
    }
    fn observe(&mut self, notice: &NativeWindowNotice) -> Result<bool, FrameHookError> {
        self.inner
            .try_borrow_mut()
            .map_err(|_| FrameHookError("Original media consumer borrowed during observation"))?
            .observe(notice)
    }
    fn routed_observation(
        &self,
    ) -> Option<crate::assembly::meari::player_shell_host::RoutedFrameObservation> {
        self.inner
            .try_borrow()
            .ok()
            .and_then(|state| state.routed_observation())
    }
}

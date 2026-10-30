use super::*;
use crate::application::{
    api::{ApiError, ErrorCode},
    route_media::{
        RetireTicket, RouteMediaError, RouteMediaProgress, RouteMediaState, ViewingTicket,
    },
};
use meari::application::plugins::PluginScope;
use meari_native::application::window_lifecycle::{CleanupStatus, LifecycleError};

enum NativeFailure {
    Child(crate::assembly::meari::playback_child::routed::RoutedChildError),
    Lifecycle(LifecycleError),
    Api(RouteMediaError),
    Cleanup(
        crate::assembly::meari::playback_surface::PlaybackSurfaceError<
            crate::assembly::meari::playback_child::routed::RoutedChildError,
            crate::assembly::meari::playback_child::routed::RoutedChildError,
        >,
    ),
}
pub(super) struct MediaLifecycle {
    pub(super) policy: RouteMediaState<PluginScope>,
    pub(super) retiring: Option<RetireTicket<PluginScope>>,
    pub(super) viewing: Option<ViewingTicket<PluginScope>>,
    pub(super) retired: bool,
    pub(super) retained_client: Option<ActivatedRoutedClient<RoutedMediaReady>>,
    release_started: bool,
    first_native_failure: Option<NativeFailure>,
    transition_pause: Option<(
        Arc<crate::infrastructure::mpv::ParentEngineRouter>,
        crate::application::playback_session::transition::RoutePause,
    )>,
}
impl MediaLifecycle {
    pub(super) fn new(owner: PluginScope) -> Self {
        Self {
            policy: RouteMediaState::new(owner),
            retiring: None,
            viewing: None,
            retired: false,
            retained_client: None,
            release_started: false,
            first_native_failure: None,
            transition_pause: None,
        }
    }
    fn remember(&mut self, failure: NativeFailure) -> RouteMediaError {
        self.first_native_failure.get_or_insert(failure);
        match self
            .first_native_failure
            .as_ref()
            .expect("original native failure")
        {
            NativeFailure::Api(error) => error.clone(),
            NativeFailure::Child(error) => Arc::new(ApiError::new(
                ErrorCode::EngineFailed,
                format!("Original native child retirement failed: {error:?}"),
            )),
            NativeFailure::Lifecycle(error) => Arc::new(ApiError::new(
                ErrorCode::EngineFailed,
                format!("Original native child lifecycle failed: {error:?}"),
            )),
            NativeFailure::Cleanup(error) => Arc::new(ApiError::new(
                ErrorCode::EngineFailed,
                format!("Original engine/surface cleanup failed: {error:?}"),
            )),
        }
    }
}
pub(super) fn native_error(message: &'static str) -> RouteMediaError {
    Arc::new(ApiError::new(ErrorCode::RevisionConflict, message))
}
impl RoutedFrameState {
    fn current_media_owner(&self, owner: &PluginScope) -> Result<(), RouteMediaError> {
        let binding = self
            .binding
            .as_ref()
            .ok_or_else(|| native_error("Original media binding unavailable"))?;
        let current = binding
            .document
            .stamp()
            .map_err(|_| native_error("Original document unavailable"))?;
        if current.host_id != self.origin.host_id
            || current.owner != self.origin.owner
            || current.definition_revision != self.origin.definition_revision
            || owner != &self.origin.owner
            || binding
                .children
                .phase()
                .map_err(|_| native_error("Original window unavailable"))?
                != WindowPhase::Ready
        {
            return Err(native_error(
                "Original route media document/window owner differs",
            ));
        }
        Ok(())
    }
    pub(super) fn begin_retirement(
        &mut self,
        owner: &PluginScope,
    ) -> Result<RetireTicket<PluginScope>, RouteMediaError> {
        self.current_media_owner(owner)?;
        if self.first_error.is_some() {
            return Err(native_error(
                "Original frame failure must retain its cleanup owner",
            ));
        }
        if self.registration.is_none() || self.activated.is_none() {
            return Err(native_error(
                "Original media has not completed checked activation",
            ));
        }
        let ticket = self.media.policy.begin_retire(owner)?;
        if self.media.retiring.is_none() && !self.media.retired {
            self.media.retiring = Some(ticket.clone());
            self.redraw.cancel();
        }
        self.drive_retirement()?;
        Ok(ticket)
    }
    pub(super) fn drive_retirement(&mut self) -> Result<(), RouteMediaError> {
        let Some(ticket) = self.media.retiring.clone() else {
            return Ok(());
        };
        let result = self.drive_original_retirement();
        match result {
            Ok(RouteMediaProgress::Pending) => Ok(()),
            Ok(RouteMediaProgress::Complete) => {
                self.media.policy.complete_retire(&ticket)?;
                self.media.retired = true;
                self.media.retiring = None;
                self.capture_checkpoint();
                Ok(())
            }
            Err(failure) => {
                let error = self.media.remember(failure);
                self.media.policy.fail_retire(&ticket, error.clone())?;
                self.capture_checkpoint();
                Err(error)
            }
        }
    }
    fn drive_original_retirement(&mut self) -> Result<RouteMediaProgress, NativeFailure> {
        let registration = self.registration.as_ref().ok_or_else(|| {
            NativeFailure::Api(native_error(
                "Original registration missing during retirement",
            ))
        })?;
        let binding = self.binding.as_ref().ok_or_else(|| {
            NativeFailure::Api(native_error("Original binding missing during retirement"))
        })?;
        if registration.lease.window()
            != binding.children.stamp().map_err(NativeFailure::Lifecycle)?
        {
            return Err(NativeFailure::Api(native_error(
                "Original retirement lease window differs",
            )));
        }
        let report = if !self.media.release_started {
            let pause = registration
                .checkpoint
                .retire_original_route()
                .map_err(NativeFailure::Child)?;
            self.media.transition_pause = Some(pause);
            self.activated
                .as_ref()
                .ok_or_else(|| NativeFailure::Api(native_error("Original activated SDK missing")))?
                .suspend_original()
                .map_err(|error| NativeFailure::Api(Arc::new(error)))?;
            // The coordinator retains engine -> surface cleanup, its signal, lease and first failure.
            let report = binding
                .children
                .release_child(&registration.lease)
                .map_err(NativeFailure::Lifecycle)?;
            self.media.release_started = true;
            report
        } else {
            binding.children.poll().map_err(NativeFailure::Lifecycle)?
        };
        match report.status {
            CleanupStatus::Complete if report.remaining == 0 => {
                let (router, ticket) = self.media.transition_pause.as_ref().ok_or_else(|| {
                    NativeFailure::Api(native_error("Original transition pause missing"))
                })?;
                if router
                    .poll_route_transitions(ticket)
                    .map_err(|error| NativeFailure::Api(Arc::new(error)))?
                {
                    Ok(RouteMediaProgress::Complete)
                } else {
                    Ok(RouteMediaProgress::Pending)
                }
            }
            CleanupStatus::Pending => Ok(RouteMediaProgress::Pending),
            _ => {
                let original = binding
                    .children
                    .with_error(&registration.lease, |error| error.cloned())
                    .map_err(NativeFailure::Lifecycle)?;
                Err(match original {
                    Some(error) => NativeFailure::Cleanup(error),
                    None => NativeFailure::Api(Arc::new(ApiError::new(
                        ErrorCode::EngineFailed,
                        "Original cleanup did not complete; explicit recovery/close retains native owners",
                    ))),
                })
            }
        }
    }
    pub(super) fn resume_original_transitions(&mut self) -> Result<(), RouteMediaError> {
        let (original, ticket) = self
            .media
            .transition_pause
            .as_ref()
            .ok_or_else(|| native_error("Original transition pause missing"))?;
        let activated = self
            .activated
            .as_ref()
            .ok_or_else(|| native_error("Fresh Viewing activation missing"))?;
        activated
            .ready
            .with_ready_route(|router, fresh| {
                if !Arc::ptr_eq(original, router) {
                    return Err(native_error("Fresh Viewing router owner differs"));
                }
                router
                    .resume_route_transitions(ticket, fresh)
                    .map_err(Arc::new)
            })
            .map_err(|_| native_error("Fresh Viewing checked route unavailable"))??;
        self.media.transition_pause = None;
        Ok(())
    }
    pub(super) fn begin_viewing(
        &mut self,
        owner: &PluginScope,
        retired: &RetireTicket<PluginScope>,
    ) -> Result<ViewingTicket<PluginScope>, RouteMediaError> {
        self.current_media_owner(owner)?;
        if let Some(ticket) = &self.media.viewing {
            self.media.policy.poll_viewing(ticket)?;
            return self.media.policy.begin_viewing(owner, retired);
        }
        if self.media.policy.poll_retire(retired)? != RouteMediaProgress::Complete
            || !self.media.retired
        {
            return Err(native_error(
                "Original native child has not completed retirement",
            ));
        }
        // The retained registrar stays on this original window; no new SDK, router or parent.
        let current = self
            .binding
            .as_ref()
            .ok_or_else(|| native_error("Original binding missing"))?
            .document
            .stamp()
            .map_err(|_| native_error("Original Viewing commit unavailable"))?;
        let ticket = self.media.policy.begin_viewing(owner, retired)?;
        self.media.viewing = Some(ticket.clone());
        if let Err(error) = self.prepare_original_viewing(current) {
            let error = self.media.remember(NativeFailure::Api(error));
            self.media.policy.fail_viewing(&ticket, error.clone())?;
            return Err(error);
        }
        self.media.retired = false;
        Ok(ticket)
    }
    fn prepare_original_viewing(&mut self, current: HostStamp) -> Result<(), RouteMediaError> {
        let executable = TrustedMpvExecutable::open(&self.executable_path, &self.executable_sha256)
            .map_err(Arc::new)?;
        self.redraw
            .rearm_for_viewing(&current)
            .map_err(|_| native_error("Original registration redraw rearm rejected"))?;
        let mut activated = self
            .activated
            .take()
            .ok_or_else(|| native_error("Original activation owner missing"))?;
        let pending = match activated.prepare_reactivation() {
            Ok(pending) => pending,
            Err(error) => {
                self.activated = Some(activated);
                return Err(Arc::new(error));
            }
        };
        self.media.retained_client = Some(activated);
        let original = self
            .registration
            .take()
            .ok_or_else(|| native_error("Original retired registration missing"))?;
        self.registrar = Some(original._retained_registrar);
        self.pending = Some(pending);
        self.executable = Some(executable);
        // Geometry/renderer/video/lease are reacquired only from subsequent real Viewing frames.
        self.origin = current;
        self.media.release_started = false;
        Ok(())
    }
}

#[cfg(test)]
#[path = "retirement/tests.rs"]
mod tests;

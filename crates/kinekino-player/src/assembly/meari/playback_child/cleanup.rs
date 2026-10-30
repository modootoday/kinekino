use crate::{
    application::{api::ApiError, playback_session::ClosedEpisodePlayback},
    infrastructure::{
        mpv::{
            ParentEngineRouter, ParentRouteHandle, RetirementFailure, RetirementHandle,
            RetirementProgress,
        },
        thread_completion::CompletionWake,
    },
};
use meari_native::application::window_lifecycle::*;
use std::{fmt, sync::Arc};

#[derive(Clone, Debug)]
pub enum PlaybackCleanupError {
    Identity,
    Api(Arc<ApiError>),
    Incomplete(Arc<ClosedEpisodePlayback>),
    MissingAttempt,
}
impl fmt::Display for PlaybackCleanupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "native playback cleanup: {self:?}")
    }
}
impl std::error::Error for PlaybackCleanupError {}

struct Attempt {
    request: CleanupRequest,
    terminal: Option<Result<CleanupProgress, PlaybackCleanupError>>,
}
pub(super) struct CleanupMachine {
    window: WindowStamp,
    child: u64,
    router: Arc<ParentEngineRouter>,
    route: ParentRouteHandle,
    retirement: Option<RetirementHandle>,
    attempt: Option<Box<Attempt>>,
    retained_failure: Option<PlaybackCleanupError>,
    observation_failure: Option<PlaybackCleanupError>,
    unavailable: bool,
}
impl CleanupMachine {
    pub(super) fn new(
        window: WindowStamp,
        child: u64,
        router: Arc<ParentEngineRouter>,
        route: ParentRouteHandle,
    ) -> Result<Self, ApiError> {
        router.validate_route_identity(&route)?;
        Ok(Self {
            window,
            child,
            router,
            route,
            retirement: None,
            attempt: None,
            retained_failure: None,
            observation_failure: None,
            unavailable: false,
        })
    }
    fn valid(&self, request: CleanupRequest) -> bool {
        request.stamp.window() == self.window && request.stamp.child() == self.child
    }
    fn same(&self, request: CleanupRequest) -> bool {
        self.attempt
            .as_ref()
            .is_some_and(|a| a.request.stamp == request.stamp && a.request.reason == request.reason)
    }
    fn unknown(&mut self, error: ApiError) -> Result<CleanupProgress, PlaybackCleanupError> {
        self.observation_failure = Some(PlaybackCleanupError::Api(Arc::new(error)));
        self.unavailable = true;
        Ok(CleanupProgress::ObservationUnavailable)
    }
    fn observe_owner(&mut self) -> Result<CleanupProgress, PlaybackCleanupError> {
        if self.retirement.is_none() {
            match self.router.retirement_handle(&self.route) {
                Ok(Some(handle)) => self.retirement = Some(handle),
                Ok(None) => {
                    // The matching prepared/active route has no retirement task.
                    self.unavailable = false;
                    return Err(self
                        .observation_failure
                        .clone()
                        .unwrap_or(PlaybackCleanupError::MissingAttempt));
                }
                Err(error) => return self.unknown(error),
            }
        }
        let observation = match self
            .router
            .observe_retirement(self.retirement.as_ref().unwrap())
        {
            Ok(observation) => observation,
            Err(error) => return self.unknown(error),
        };
        self.unavailable = false;
        if let Some(failure) = observation.retained_failure {
            self.retained_failure = Some(match failure {
                RetirementFailure::Dispatch(error) | RetirementFailure::Engine(error) => {
                    PlaybackCleanupError::Api(error)
                }
                RetirementFailure::Incomplete(receipt) => {
                    PlaybackCleanupError::Incomplete(Arc::new(receipt))
                }
            });
        }
        match observation.progress {
            RetirementProgress::Pending => Ok(CleanupProgress::Pending),
            RetirementProgress::Complete(_) => Ok(CleanupProgress::Complete),
            RetirementProgress::NotStarted => Err(PlaybackCleanupError::MissingAttempt),
            RetirementProgress::RetryableFailure => Err(observation
                .dispatch_error
                .map(PlaybackCleanupError::Api)
                .or_else(|| self.retained_failure.clone())
                .unwrap_or(PlaybackCleanupError::MissingAttempt)),
        }
    }
    fn observe_current(&mut self) -> Result<CleanupProgress, PlaybackCleanupError> {
        if let Some(terminal) = self.attempt.as_ref().and_then(|a| a.terminal.clone()) {
            return terminal;
        }
        let result = self.observe_owner();
        if matches!(result, Ok(CleanupProgress::Complete) | Err(_))
            && let Some(attempt) = &mut self.attempt
        {
            attempt.terminal = Some(result.clone());
        }
        result
    }
    fn reject(&mut self) -> Result<CleanupProgress, PlaybackCleanupError> {
        match self.observe_current() {
            Ok(CleanupProgress::Pending) => Ok(CleanupProgress::Pending),
            Ok(CleanupProgress::ObservationUnavailable) => {
                Ok(CleanupProgress::ObservationUnavailable)
            }
            _ => Err(PlaybackCleanupError::Identity),
        }
    }
    fn wake(signal: CompletionSignal) -> CompletionWake {
        Arc::new(move || {
            signal.notify();
            Ok(())
        })
    }
}
impl NativeChild for CleanupMachine {
    type Error = PlaybackCleanupError;
    fn parent_lost(&mut self) {
        // Constructor fixed the owner. No state/task mutex, dispatch or SDK I/O here.
        if let Err(error) = self.router.revoke_route_admission(&self.route) {
            let _ = self.unknown(error);
        }
    }
    fn begin_cleanup(
        &mut self,
        request: CleanupRequest,
        signal: CompletionSignal,
    ) -> Result<CleanupProgress, Self::Error> {
        if !self.valid(request) || signal.stamp() != request.stamp {
            return self.reject();
        }
        if self.same(request) {
            return self.observe_current();
        }
        if let Some(a) = &self.attempt {
            if request.stamp.revocation() <= a.request.stamp.revocation()
                || request.stamp.attempt() <= a.request.stamp.attempt()
            {
                return self.reject();
            }
            // Only an observed terminal failure permits another attempt.
            if self.observe_current().is_ok() {
                return self.reject();
            }
        }
        self.attempt = Some(Box::new(Attempt {
            request,
            terminal: None,
        }));
        let wake = Self::wake(signal);
        let started = if let Some(handle) = &self.retirement {
            self.router.retry_retirement(handle, wake)
        } else {
            self.router.retire(&self.route, wake).map(|start| {
                self.retirement = Some(start.handle);
            })
        };
        if let Err(error) = started {
            // Preserve the failure, then distinguish a proven dispatch rejection from
            // an unreadable owner. Only the latter remains ObservationUnavailable.
            let _ = self.unknown(error);
        }
        self.observe_current()
    }
    fn observe_cleanup(&mut self, request: CleanupRequest) -> Result<CleanupProgress, Self::Error> {
        if !self.valid(request) || !self.same(request) {
            return self.reject();
        }
        self.observe_current()
    }
    fn observation_error(&self) -> Option<&Self::Error> {
        if self.unavailable {
            self.observation_failure.as_ref()
        } else {
            None
        }
    }
}

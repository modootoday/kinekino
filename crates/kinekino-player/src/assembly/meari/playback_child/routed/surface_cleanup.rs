use super::{
    errors::RoutedChildError,
    state::{State, SurfaceOwner},
};
use crate::infrastructure::mpv::{ProbeRelease, SurfaceInputError};
use meari_native::application::window_lifecycle::{
    CleanupProgress, CleanupRequest, CompletionSignal, NativeChild,
};
use std::{cell::RefCell, rc::Rc, sync::Arc};
pub(crate) struct SurfacePart {
    pub(super) state: Rc<RefCell<State>>,
    pub(super) request: Option<CleanupRequest>,
    pub(super) signal: Option<CompletionSignal>,
    pub(super) diagnostic: Option<RoutedChildError>,
}
impl SurfacePart {
    fn release(
        &mut self,
        request: CleanupRequest,
        begin: bool,
    ) -> Result<CleanupProgress, RoutedChildError> {
        let mut state = self
            .state
            .try_borrow_mut()
            .map_err(|_| RoutedChildError::Borrowed)?;
        if state.identity() != (request.stamp.window(), request.stamp.child()) {
            return Err(RoutedChildError::Identity);
        }
        let start = match self.request {
            Some(previous)
                if previous.stamp == request.stamp && previous.reason == request.reason =>
            {
                false
            }
            Some(previous)
                if begin
                    && request.stamp.revocation() > previous.stamp.revocation()
                    && request.stamp.attempt() > previous.stamp.attempt() =>
            {
                true
            }
            Some(_) => return Err(RoutedChildError::Identity),
            None if begin => true,
            None => return Err(RoutedChildError::NotReady),
        };
        let result = match &mut state.surface {
            SurfaceOwner::Ready(surface) => {
                let result = if start {
                    surface.begin_release(request)
                } else {
                    surface.observe_release(request)
                };
                self.diagnostic = surface.first_error().map(RoutedChildError::Surface);
                result.map_err(RoutedChildError::Surface)
            }
            SurfaceOwner::Rejected(failure) => failure
                .slot
                .with_cleanup_surface(request, |_| ProbeRelease::NotCreated)
                .map_err(|error| {
                    RoutedChildError::Input(Arc::new(SurfaceInputError::CleanupAccess(Arc::new(
                        error,
                    ))))
                }),
            SurfaceOwner::Prepare(input) => input
                .with_cleanup_parent(request, |_| ProbeRelease::NotCreated)
                .map_err(|error| RoutedChildError::Input(Arc::new(error))),
        };
        self.request = Some(request);
        match result {
            Ok(ProbeRelease::Complete { .. } | ProbeRelease::NotCreated) => {
                Ok(CleanupProgress::Complete)
            }
            Ok(ProbeRelease::Unobserved) => Ok(CleanupProgress::ObservationUnavailable),
            Err(error) => {
                let retry_ready = matches!(&state.surface, SurfaceOwner::Ready(surface) if surface.class_retry_ready());
                self.diagnostic = Some(state.remember(error.clone()));
                if retry_ready {
                    Err(error)
                } else {
                    Ok(CleanupProgress::ObservationUnavailable)
                }
            }
        }
    }
}
impl NativeChild for SurfacePart {
    type Error = RoutedChildError;
    fn parent_lost(&mut self) {
        let Ok(mut state) = self.state.try_borrow_mut() else {
            self.diagnostic.get_or_insert(RoutedChildError::Borrowed);
            return;
        };
        state.revoked = true;
        match &mut state.surface {
            SurfaceOwner::Ready(surface) => surface.revoke_admission(),
            SurfaceOwner::Rejected(failure) => failure.slot.revoke(),
            SurfaceOwner::Prepare(input) => input.revoke(),
        }
    }
    fn begin_cleanup(
        &mut self,
        request: CleanupRequest,
        signal: CompletionSignal,
    ) -> Result<CleanupProgress, Self::Error> {
        if signal.stamp() != request.stamp {
            return Err(RoutedChildError::Identity);
        }
        let result = self.release(request, true);
        if self
            .request
            .is_some_and(|saved| saved.stamp == request.stamp && saved.reason == request.reason)
        {
            self.signal = Some(signal);
        }
        result
    }
    fn observe_cleanup(&mut self, request: CleanupRequest) -> Result<CleanupProgress, Self::Error> {
        self.release(request, false)
    }
    fn observation_error(&self) -> Option<&Self::Error> {
        self.diagnostic.as_ref()
    }
}

use super::admission::OriginalAdmission;
use super::{errors::RoutedChildError, state::State};
use meari_native::application::window_lifecycle::{
    CleanupProgress, CleanupRequest, CompletionSignal, NativeChild,
};
use std::{cell::RefCell, rc::Rc, sync::Arc};
pub(crate) struct EnginePart {
    pub(super) admission: Rc<OriginalAdmission>,
    pub(super) state: Rc<RefCell<State>>,
    pub(super) diagnostic: Option<RoutedChildError>,
}
impl NativeChild for EnginePart {
    type Error = RoutedChildError;
    fn parent_lost(&mut self) {
        if let Err(error) = self.admission.revoke() {
            self.diagnostic = Some(error);
        }
        let Ok(mut state) = self.state.try_borrow_mut() else {
            self.diagnostic.get_or_insert(RoutedChildError::Borrowed);
            return;
        };
        state.revoked = true;
        if let Some(cleanup) = &mut state.cleanup {
            cleanup.parent_lost();
        } else if let Some(route) = &state.route
            && let Err(error) = state.router.revoke_route_admission(route)
        {
            self.diagnostic = Some(state.remember(RoutedChildError::Api(Arc::new(error))));
        }
    }
    fn begin_cleanup(
        &mut self,
        request: CleanupRequest,
        signal: CompletionSignal,
    ) -> Result<CleanupProgress, Self::Error> {
        // Also covers cleanup begun without an earlier parent_lost notification.
        // Original transition/route admission closes before the possibly borrowed State.
        if request.reason
            != meari_native::domain::window_lifecycle::RevocationReason::ExplicitRelease
        {
            self.admission.close_parent();
        }
        self.admission.revoke().inspect_err(|error| {
            self.diagnostic.get_or_insert_with(|| error.clone());
        })?;
        let mut state = self
            .state
            .try_borrow_mut()
            .map_err(|_| RoutedChildError::Borrowed)?;
        if state.cleanup.is_none()
            && let Some(route) = state.route.clone()
        {
            let (window, child) = state.identity();
            state.cleanup = Some(
                super::super::cleanup::CleanupMachine::new(
                    window,
                    child,
                    state.router.clone(),
                    route,
                )
                .map_err(|error| RoutedChildError::Api(Arc::new(error)))?,
            );
        }
        if let Some(cleanup) = &mut state.cleanup {
            let result = cleanup
                .begin_cleanup(request, signal)
                .map_err(RoutedChildError::Cleanup);
            self.diagnostic = cleanup
                .observation_error()
                .cloned()
                .map(RoutedChildError::Cleanup);
            result
        } else if state.original_engine.is_none() && state.route.is_none() {
            Ok(CleanupProgress::Complete)
        } else if state.route.is_none() {
            if state.unbound_cleanup.is_none() {
                let engine = state
                    .original_engine
                    .as_ref()
                    .ok_or(RoutedChildError::UnboundEngine)?;
                state.unbound_cleanup = Some(super::unbound_cleanup::UnboundCleanup::new(
                    &state.router,
                    engine,
                )?);
            }
            let result = state
                .unbound_cleanup
                .as_mut()
                .ok_or(RoutedChildError::UnboundEngine)?
                .begin(signal);
            self.diagnostic = result.as_ref().err().cloned();
            result
        } else {
            // Retain an unbound partial engine; never destroy its surface ahead of owner cleanup.
            self.diagnostic = Some(RoutedChildError::UnboundEngine);
            Ok(CleanupProgress::ObservationUnavailable)
        }
    }
    fn observe_cleanup(&mut self, request: CleanupRequest) -> Result<CleanupProgress, Self::Error> {
        let mut state = self
            .state
            .try_borrow_mut()
            .map_err(|_| RoutedChildError::Borrowed)?;
        if let Some(cleanup) = &mut state.cleanup {
            let result = cleanup
                .observe_cleanup(request)
                .map_err(RoutedChildError::Cleanup);
            self.diagnostic = cleanup
                .observation_error()
                .cloned()
                .map(RoutedChildError::Cleanup);
            result
        } else if state.original_engine.is_none() && state.route.is_none() {
            Ok(CleanupProgress::Complete)
        } else if let Some(cleanup) = &mut state.unbound_cleanup {
            let result = cleanup.observe();
            self.diagnostic = result.as_ref().err().cloned();
            result
        } else {
            self.diagnostic = Some(RoutedChildError::UnboundEngine);
            Ok(CleanupProgress::ObservationUnavailable)
        }
    }
    fn observation_error(&self) -> Option<&Self::Error> {
        self.diagnostic.as_ref()
    }
}

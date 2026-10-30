use super::super::cleanup::CleanupMachine;
use super::admission::OriginalAdmission;
use super::{
    errors::RoutedChildError,
    state::{State, SurfaceOwner},
};
use crate::{
    application::catalog::CatalogArtifacts,
    assembly::native_playback::routed::{ReadyRoutedNative, ready_seal},
    infrastructure::mpv::{ParentEngineRouter, ParentRouteHandle, TrustedMpvExecutable},
};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
    sync::Arc,
};
mod observation;
pub(crate) use observation::RoutedSurfaceObservation;
/// Weak frame checkpoint cannot create a second coordinator or retain a lost child.
pub(crate) struct RoutedMediaCheckpoint {
    pub(super) state: Weak<RefCell<State>>,
    pub(super) admission: Rc<OriginalAdmission>,
}
/// Strong handle to the same registered owner; contains no cloned parent lease or stored proof.
pub(crate) struct RoutedMediaReady {
    pub(super) state: Rc<RefCell<State>>,
    pub(super) admission: Rc<OriginalAdmission>,
}
impl ready_seal::Sealed for RoutedMediaReady {}
impl ReadyRoutedNative for RoutedMediaReady {
    type Error = RoutedChildError;
    fn with_ready_route<T>(
        &self,
        visit: impl FnOnce(&Arc<ParentEngineRouter>, &ParentRouteHandle) -> T,
    ) -> Result<T, Self::Error> {
        let result = (|| {
            self.admission.require_live()?;
            let state = self
                .state
                .try_borrow()
                .map_err(|_| RoutedChildError::Borrowed)?;
            if state.revoked || !Rc::ptr_eq(&state.admission, &self.admission) {
                return Err(RoutedChildError::NotReady);
            }
            let route = self.admission.original()?;
            let expected = state
                .original_engine
                .as_ref()
                .ok_or(RoutedChildError::NotReady)?;
            let SurfaceOwner::Ready(surface) = &state.surface else {
                return Err(RoutedChildError::NotReady);
            };
            surface
                .with_original_engine(|engine| {
                    if !Arc::ptr_eq(engine, expected) {
                        return Err(RoutedChildError::Identity);
                    }
                    Ok(visit(&self.admission.router, route))
                })
                .map_err(RoutedChildError::Surface)?
        })();
        result.map_err(|error| self.admission.fail_closed(error))
    }
}
impl RoutedMediaCheckpoint {
    /// UI calls for a legitimate subsequent original FrameSubmitted before forwarding SDK work.
    /// A changed document/slot/HMR must use invalidate_original instead of this operation.
    pub fn refresh_current_frame(&self) -> Result<(), RoutedChildError> {
        let result = (|| {
            self.admission.require_live()?;
            let owner = self.state.upgrade().ok_or(RoutedChildError::Expired)?;
            let mut state = owner
                .try_borrow_mut()
                .map_err(|_| RoutedChildError::Borrowed)?;
            self.admission.original()?;
            if state.revoked || !Rc::ptr_eq(&state.admission, &self.admission) {
                return Err(RoutedChildError::NotReady);
            }
            let SurfaceOwner::Ready(surface) = &mut state.surface else {
                return Err(RoutedChildError::NotReady);
            };
            surface
                .refresh_original_frame()
                .map_err(RoutedChildError::Surface)?;
            self.admission.require_live()
        })();
        result.map_err(|error| self.admission.fail_closed(error))
    }
    /// Call for every subsequent original frame before forwarding SDK work. Failure revokes;
    /// an unchanged original frame is reobserved, never replaced with a saved ready boolean.
    pub fn revalidate_active_frame(&self) -> Result<(), RoutedChildError> {
        let owner = self
            .state
            .upgrade()
            .ok_or_else(|| self.admission.fail_closed(RoutedChildError::Expired))?;
        RoutedMediaReady {
            state: owner,
            admission: self.admission.clone(),
        }
        .with_ready_route(|_, _| ())
    }
    /// UI invokes for original document/slot/HMR/resize invalidation, even during a state borrow.
    pub fn invalidate_original(&self) -> Result<(), RoutedChildError> {
        self.admission.revoke()
    }
    pub(crate) fn retire_original_route(
        &self,
    ) -> Result<
        (
            Arc<ParentEngineRouter>,
            crate::application::playback_session::transition::RoutePause,
        ),
        RoutedChildError,
    > {
        let ticket = self.admission.retire_route()?;
        Ok((self.admission.router.clone(), ticket))
    }
    /// UI calls once from its actual original frame observer after registration.
    pub fn activate(
        &self,
        executable: TrustedMpvExecutable,
        artifacts: Arc<dyn CatalogArtifacts>,
    ) -> Result<RoutedMediaReady, RoutedChildError> {
        self.admission.require_live()?;
        let owner = self.state.upgrade().ok_or(RoutedChildError::Expired)?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| RoutedChildError::Borrowed)?;
        if let Some(error) = &state.first_error {
            return Err(error.clone());
        }
        if state.revoked || state.original_engine.is_some() {
            return Err(RoutedChildError::NotReady);
        }
        let result = (|| {
            let SurfaceOwner::Ready(surface) = &mut state.surface else {
                return Err(RoutedChildError::NotReady);
            };
            surface
                .construct_visible()
                .map_err(RoutedChildError::Surface)?;
            self.admission.require_live()?;
            let engine = surface
                .bind_original_engine(executable, artifacts)
                .map_err(RoutedChildError::Surface)?;
            state.original_engine = Some(engine.clone());
            let route = state
                .router
                .prepare(engine)
                .map_err(|failure| RoutedChildError::Api(Arc::new(failure.error)))?;
            state.route = Some(route.clone());
            let (window, child) = state.identity();
            state.cleanup = Some(
                CleanupMachine::new(window, child, state.router.clone(), route.clone())
                    .map_err(|error| RoutedChildError::Api(Arc::new(error)))?,
            );
            self.admission.install(route.clone())?;
            self.admission.require_live()?;
            state
                .router
                .activate(&route)
                .map_err(|error| RoutedChildError::Api(Arc::new(error)))?;
            Ok(())
        })();
        if let Err(error) = result {
            return Err(state.remember(self.admission.fail_closed(error)));
        }
        drop(state);
        Ok(RoutedMediaReady {
            state: owner,
            admission: self.admission.clone(),
        })
    }
}

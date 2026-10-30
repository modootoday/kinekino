//! Immutable original-route identity with UI-thread invalidation independent of state borrowing.
use super::errors::RoutedChildError;
use crate::infrastructure::mpv::{ParentEngineRouter, ParentRouteHandle};
use std::{
    cell::{Cell, OnceCell, RefCell},
    sync::Arc,
};
#[cfg(test)]
mod tests;

pub(super) struct OriginalAdmission {
    pub(super) router: Arc<ParentEngineRouter>,
    route: OnceCell<ParentRouteHandle>,
    revoked: Cell<bool>,
    route_retiring: Cell<bool>,
    secondary: RefCell<Option<RoutedChildError>>,
}
pub(super) struct AdmissionObservation {
    pub(super) revoked: bool,
    pub(super) route_generation: Option<u64>,
    pub(super) secondary_error: Option<RoutedChildError>,
}
impl OriginalAdmission {
    /// Read-only diagnostics: no admission check, native access, route mutation or fail_closed.
    pub(super) fn diagnostic_snapshot(&self) -> Result<AdmissionObservation, RoutedChildError> {
        let secondary = self
            .secondary
            .try_borrow()
            .map_err(|_| RoutedChildError::Borrowed)?;
        Ok(AdmissionObservation {
            revoked: self.revoked.get(),
            route_generation: self.route.get().map(ParentRouteHandle::generation),
            secondary_error: secondary.clone(),
        })
    }
    pub(super) fn new(router: Arc<ParentEngineRouter>) -> Self {
        Self {
            router,
            route: OnceCell::new(),
            revoked: Cell::new(false),
            route_retiring: Cell::new(false),
            secondary: RefCell::new(None),
        }
    }
    pub(super) fn require_live(&self) -> Result<(), RoutedChildError> {
        if self.revoked.get() {
            Err(RoutedChildError::NotReady)
        } else {
            Ok(())
        }
    }
    pub(super) fn install(&self, route: ParentRouteHandle) -> Result<(), RoutedChildError> {
        self.route
            .set(route)
            .map_err(|_| RoutedChildError::Identity)?;
        if self.revoked.get() {
            self.revoke()?;
            return Err(RoutedChildError::NotReady);
        }
        Ok(())
    }
    pub(super) fn original(&self) -> Result<&ParentRouteHandle, RoutedChildError> {
        self.route.get().ok_or(RoutedChildError::NotReady)
    }
    pub(super) fn revoke(&self) -> Result<(), RoutedChildError> {
        if !self.route_retiring.get() {
            self.router.close_transition_admission();
        }
        self.revoked.set(true);
        if let Some(route) = self.route.get() {
            self.router
                .revoke_route_admission(route)
                .map_err(|error| RoutedChildError::Api(Arc::new(error)))?;
        }
        Ok(())
    }
    pub(super) fn retire_route(
        &self,
    ) -> Result<crate::application::playback_session::transition::RoutePause, RoutedChildError>
    {
        self.require_live()?;
        self.original()?;
        let ticket = self
            .router
            .pause_route_transitions()
            .map_err(|error| RoutedChildError::Api(Arc::new(error)))?;
        self.route_retiring.set(true);
        self.revoke()?;
        Ok(ticket)
    }
    pub(super) fn close_parent(&self) {
        self.router.close_transition_admission();
    }
    pub(super) fn fail_closed(&self, original: RoutedChildError) -> RoutedChildError {
        self.close_parent();
        if let Err(error) = self.revoke() {
            self.secondary.borrow_mut().get_or_insert(error);
        }
        original
    }
}

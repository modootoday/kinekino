//! Original owner metadata only; this value cannot register, wake, validate or authorize media.
use super::{RoutedChildError, RoutedMediaCheckpoint, SurfaceOwner};
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum RoutedSurfaceObservation {
    PreparedSurface,
    RetainedUnpreparedInput,
    RetainedRejectedInput,
}
#[derive(Debug)]
pub(crate) struct RoutedCheckpointObservation {
    pub surface: RoutedSurfaceObservation,
    pub original_engine_retained: bool,
    pub retained_route_generation: Option<u64>,
    pub admission_revoked: bool,
    pub state_revoked: bool,
    pub cleanup_retained: bool,
    pub first_error: Option<RoutedChildError>,
    pub revocation_error: Option<RoutedChildError>,
}
impl RoutedMediaCheckpoint {
    /// A failed read leaves original admission/slot/owners untouched. Not a Ready check.
    pub(crate) fn diagnostic_snapshot(
        &self,
    ) -> Result<RoutedCheckpointObservation, RoutedChildError> {
        let owner = self.state.upgrade().ok_or(RoutedChildError::Expired)?;
        let state = owner.try_borrow().map_err(|_| RoutedChildError::Borrowed)?;
        if !Rc::ptr_eq(&state.admission, &self.admission) {
            return Err(RoutedChildError::Identity);
        }
        let admission = self.admission.diagnostic_snapshot()?;
        Ok(RoutedCheckpointObservation {
            surface: match &state.surface {
                SurfaceOwner::Ready(_) => RoutedSurfaceObservation::PreparedSurface,
                SurfaceOwner::Prepare(_) => RoutedSurfaceObservation::RetainedUnpreparedInput,
                SurfaceOwner::Rejected(_) => RoutedSurfaceObservation::RetainedRejectedInput,
            },
            original_engine_retained: state.original_engine.is_some(),
            retained_route_generation: admission.route_generation,
            admission_revoked: admission.revoked,
            state_revoked: state.revoked,
            cleanup_retained: state.cleanup.is_some() || state.unbound_cleanup.is_some(),
            first_error: state.first_error.clone(),
            revocation_error: admission.secondary_error,
        })
    }
}

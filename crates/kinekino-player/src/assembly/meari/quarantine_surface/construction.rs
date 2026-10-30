use super::*;
use crate::infrastructure::mpv::{ChildInputError, DisabledChildProof, SurfaceInputError};
use meari_native::{
    domain::surface_slot::SurfaceError,
    infrastructure::{
        native_host::HostAccessError,
        window::{NativeParentClientError, NativeSurfaceAccessError},
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstructionStatus {
    Pending { request_redraw: bool },
    Constructed,
}
/// Weak checkpoint only: it cannot independently keep a child alive or issue cleanup.
pub struct QuarantineCheckpoint {
    pub(super) owner: Weak<RefCell<OwnerState>>,
}
fn pending_access(error: &NativeSurfaceAccessError) -> bool {
    matches!(
        error,
        NativeSurfaceAccessError::Slot(SurfaceError::NotReady | SurfaceError::Busy)
            | NativeSurfaceAccessError::Document(HostAccessError::Busy)
    )
}
pub(super) fn pending(error: &ChildInputError) -> bool {
    match error {
        ChildInputError::Input(SurfaceInputError::ParentClient(error)) => {
            matches!(error.as_ref(), NativeParentClientError::Access(error) if pending_access(error))
        }
        ChildInputError::Parent(NativeParentClientError::Access(error)) => pending_access(error),
        _ => false,
    }
}
pub(super) fn request_redraw(already_requested: &mut bool) -> bool {
    !std::mem::replace(already_requested, true)
}
impl QuarantineCheckpoint {
    /// Call after registration and after the host has left its dispatch/document borrow.
    /// Pending is not failure; apply returned redraw outside this method and wait for own frame notice.
    pub fn advance(&self) -> Result<ConstructionStatus, QuarantineError> {
        let owner = self.owner.upgrade().ok_or(QuarantineError::Unavailable)?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| QuarantineError::Unavailable)?;
        if state.revoked || state.request.is_some() {
            return Err(QuarantineError::Unavailable);
        }
        match state.construction {
            ConstructionState::Constructed => return Ok(ConstructionStatus::Constructed),
            ConstructionState::Failed | ConstructionState::Entered => {
                return Err(state
                    .first_error
                    .clone()
                    .unwrap_or(QuarantineError::Unavailable));
            }
            ConstructionState::Prepared => {}
        }
        state.construction = ConstructionState::Entered;
        let OwnerState { input, probe, .. } = &mut *state;
        let OriginalInput::Ready(input) = input else {
            return Err(QuarantineError::Unavailable);
        };
        let result = probe
            .as_mut()
            .ok_or(QuarantineError::Unavailable)?
            .construct_quarantine(input);
        match result {
            Ok(()) => {
                state.construction = ConstructionState::Constructed;
                Ok(ConstructionStatus::Constructed)
            }
            Err(error) if pending(&error) => {
                state.construction = ConstructionState::Prepared;
                Ok(ConstructionStatus::Pending {
                    request_redraw: request_redraw(&mut state.redraw_requested),
                })
            }
            Err(error) => {
                state.construction = ConstructionState::Failed;
                let error = QuarantineError::Child(Arc::new(error));
                state.first_error.get_or_insert_with(|| error.clone());
                Err(error)
            }
        }
    }
    /// No host reproject/after_dispatch is allowed inside read: the original document is borrowed.
    pub fn with_disabled_child<R>(
        &self,
        read: impl for<'a> FnOnce(&DisabledChildProof<'a>) -> R,
    ) -> Result<R, QuarantineError> {
        let owner = self.owner.upgrade().ok_or(QuarantineError::Unavailable)?;
        let state = owner
            .try_borrow()
            .map_err(|_| QuarantineError::Unavailable)?;
        if state.revoked
            || state.request.is_some()
            || state.construction != ConstructionState::Constructed
        {
            return Err(QuarantineError::Unavailable);
        }
        let OriginalInput::Ready(input) = &state.input else {
            return Err(QuarantineError::Unavailable);
        };
        state
            .probe
            .as_ref()
            .ok_or(QuarantineError::Unavailable)?
            .with_disabled_child(input, read)
            .map_err(|error| QuarantineError::Child(Arc::new(error)))
    }
    pub fn first_error(&self) -> Option<QuarantineError> {
        self.owner.upgrade()?.try_borrow().ok()?.first_error.clone()
    }
    /// Host notice seam only. Never performs teardown or marks the child complete.
    pub fn forward_cleanup_notice(&self) -> Result<bool, QuarantineError> {
        let owner = self.owner.upgrade().ok_or(QuarantineError::Unavailable)?;
        let signal = {
            let state = owner
                .try_borrow()
                .map_err(|_| QuarantineError::Unavailable)?;
            if !state
                .probe
                .as_ref()
                .is_some_and(HiddenSurfaceProbe::take_quarantine_notice)
            {
                return Ok(false);
            }
            state.signal.clone()
        };
        Ok(signal.is_some_and(|signal| signal.notify()))
    }
}

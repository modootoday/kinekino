use super::geometry;
use meari_native::{
    application::window_lifecycle::{CleanupRequest, LifecycleError, WindowStamp},
    domain::surface_slot::SurfaceError,
    infrastructure::window::{
        NativeCheckedParentClient, NativeParentClientError, NativeParentLease, NativeScopedParent,
        NativeSurfaceAccessError, NativeSurfaceSlot, NativeSurfaceSnapshot,
    },
};
use std::{error::Error, fmt, sync::Arc};

#[derive(Clone, Debug)]
pub enum SurfaceInputError {
    ParentClient(Arc<NativeParentClientError>),
    Identity,
    Access(Arc<NativeSurfaceAccessError>),
    CleanupAccess(Arc<LifecycleError>),
    Geometry(SurfaceError),
    NotVisible,
    VisibilityUnknown,
    ParentClientOriginUnverified,
}
impl fmt::Display for SurfaceInputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "native surface input: {self:?}")
    }
}
impl Error for SurfaceInputError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ParentClient(error) => Some(error.as_ref()),
            Self::Access(error) => Some(error.as_ref()),
            Self::CleanupAccess(error) => Some(error.as_ref()),
            Self::Geometry(error) => Some(error),
            _ => None,
        }
    }
}
pub struct SurfaceInputFailure {
    pub error: SurfaceInputError,
    pub parent: NativeParentLease,
    pub slot: NativeSurfaceSlot,
}
impl fmt::Debug for SurfaceInputFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SurfaceInputFailure")
            .field("error", &self.error)
            .finish_non_exhaustive()
    }
}

/// Original factory owners only. This is neither a ready token nor an OS surface owner.
/// No Clone/Send/serde/raw-handle constructor is introduced.
pub struct NativeSurfaceInput {
    parent: NativeParentLease,
    slot: NativeSurfaceSlot,
}
impl NativeSurfaceInput {
    /// Opens only this original slot. No caller can substitute an unrelated checked proof.
    pub(crate) fn with_checked_parent<R>(
        &self,
        visit: impl for<'a> FnOnce(&NativeCheckedParentClient<'a>) -> R,
    ) -> Result<R, SurfaceInputError> {
        let map = |error| SurfaceInputError::ParentClient(Arc::new(error));
        self.slot
            .with_parent_client(|checked| {
                checked
                    .with_parent(|_, snapshot| {
                        geometry::validate_pair(
                            self.parent.window(),
                            self.parent.child(),
                            self.slot.window(),
                            self.slot.child(),
                        )?;
                        geometry::validate_identity(self.parent.window(), snapshot.window)
                    })
                    .map_err(map)??;
                Ok(visit(checked))
            })
            .map_err(map)?
    }
    /// Compare original factory window/child metadata without querying Creating-phase normal access.
    /// This is not a ready proof; inspect revalidates current document/node/parent after registration.
    pub fn new(
        parent: NativeParentLease,
        slot: NativeSurfaceSlot,
    ) -> Result<Self, Box<SurfaceInputFailure>> {
        if let Err(error) =
            geometry::validate_pair(parent.window(), parent.child(), slot.window(), slot.child())
        {
            return Err(Box::new(SurfaceInputFailure {
                error,
                parent,
                slot,
            }));
        }
        Ok(Self { parent, slot })
    }
    pub fn window(&self) -> WindowStamp {
        self.parent.window()
    }
    pub fn child(&self) -> u64 {
        self.parent.child()
    }
    /// Reads the actual opaque slot each time. Returning metadata grants no placement authority.
    pub fn inspect(&self) -> Result<NativeSurfaceSnapshot, SurfaceInputError> {
        self.slot
            .with_surface(|_, snapshot| {
                geometry::validate_identity(self.parent.window(), snapshot.window)?;
                geometry::validate_snapshot(snapshot)?;
                Ok(snapshot.clone())
            })
            .map_err(|error| SurfaceInputError::Access(Arc::new(error)))?
    }
    /// Normal post-registration access only; this grants no display or media admission.
    pub(super) fn with_probe_parent<R>(
        &self,
        visit: impl for<'a> FnOnce(&NativeScopedParent<'a>) -> R,
    ) -> Result<R, SurfaceInputError> {
        self.slot
            .with_surface(|parent, snapshot| {
                geometry::validate_pair(
                    self.parent.window(),
                    self.parent.child(),
                    self.slot.window(),
                    self.slot.child(),
                )?;
                geometry::validate_identity(self.parent.window(), snapshot.window)?;
                Ok(visit(parent))
            })
            .map_err(|error| SurfaceInputError::Access(Arc::new(error)))?
    }
    /// Current producer data cannot authorize creation: origin/clip/plane/focus are unverified.
    /// No caller-supplied snapshot/boolean can turn this gate into OS startup authority.
    pub fn require_start(&self) -> Result<(), SurfaceInputError> {
        geometry::require_start(&self.inspect()?)
    }
    pub fn revoke(&self) {
        self.slot.revoke();
    }
    /// Cleanup access still validates the original full request, even after node removal/HMR.
    /// The callback is not a Destroy acknowledgement and must never start a new backend.
    pub fn with_cleanup_parent<R>(
        &self,
        request: CleanupRequest,
        read: impl for<'a> FnOnce(&NativeScopedParent<'a>) -> R,
    ) -> Result<R, SurfaceInputError> {
        if request.stamp.window() != self.parent.window()
            || request.stamp.child() != self.parent.child()
        {
            return Err(SurfaceInputError::Identity);
        }
        self.slot
            .with_cleanup_surface(request, read)
            .map_err(|error| SurfaceInputError::CleanupAccess(Arc::new(error)))
    }
    /// Returns exactly the original owners for engine-child/surface-adapter or failure assembly.
    /// Decomposition is not a ready proof; a future OS constructor must enforce require_start itself.
    pub fn into_owners(self) -> (NativeParentLease, NativeSurfaceSlot) {
        (self.parent, self.slot)
    }
}

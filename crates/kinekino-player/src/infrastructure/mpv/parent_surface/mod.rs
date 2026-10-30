//! Typed Meari input ownership and rejection gates; no OS surface is created here.
mod geometry;
mod input;
mod win32;
pub use input::{NativeSurfaceInput, SurfaceInputError, SurfaceInputFailure};
pub(crate) use win32::{
    CheckedMediaParent, ParentMediaFailure, ParentMediaSurface, ParentMediaSurfaceError,
};
pub use win32::{ChildInputError, DisabledChildProof};
pub use win32::{HiddenSurfaceProbe, ProbeError, ProbeFailure, ProbeObservation, ProbeRelease};

#[cfg(test)]
mod tests;

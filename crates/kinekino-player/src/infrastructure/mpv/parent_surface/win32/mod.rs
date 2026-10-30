//! Compatibility HWND probe. No route, placement, display, or engine admission is minted.
mod child_input;
mod construction_guard;
pub use child_input::{ChildInputError, DisabledChildProof};
#[cfg(test)]
mod construction_guard_tests;
mod owner;
mod preflight;
mod release;
#[cfg(test)]
mod tests;
#[allow(
    dead_code,
    reason = "Private visible-child adapter candidate; product installation is separately gated"
)]
mod visible_render_only;
pub(crate) use visible_render_only::routed_media::{
    CheckedMediaParent, ParentMediaFailure, ParentMediaSurface, ParentMediaSurfaceError,
};
mod window_events;
use super::SurfaceInputError;
pub use owner::{HiddenSurfaceProbe, ProbeFailure, ProbeObservation};
pub use release::ProbeRelease;

#[derive(Debug)]
pub enum ProbeError {
    Input(SurfaceInputError),
    HandleUnavailable,
    ForeignThread,
    CounterExhausted,
    Os(windows::core::Error),
    ObservationMismatch,
    DisplayAdmissionUnavailable,
    ClassFinalizationRequired,
    CallbackFault(u32),
}
impl std::fmt::Display for ProbeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "hidden surface probe: {self:?}")
    }
}
impl std::error::Error for ProbeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Input(e) => Some(e),
            Self::Os(e) => Some(e),
            _ => None,
        }
    }
}

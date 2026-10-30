//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

/// Infrastructure-owned engine and its existing explicit shutdown path.
pub trait ParentOwnedEngine: EpisodePlaybackEngine {
    /// Optional existing atomic replacement port; never emulate it with close followed by open.
    fn replacement_owner(
        &self,
    ) -> Option<&dyn crate::application::playback_session::transition::EpisodeReplacementOwner>
    {
        None
    }
    /// Error or an incomplete receipt must leave the same engine owner retryable.
    fn shutdown_for_parent(&self) -> Result<Option<ClosedEpisodePlayback>, ApiError>;
}

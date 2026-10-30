//! Trusted assembly construction payload, never a wire capability or a native guard.
use crate::application::{
    api::ApiError,
    playback_session::{EpisodePlaybackSessions, v1::NativeExportIssuer},
};
use std::sync::Arc;

#[must_use = "Consume once into the original routed registry or retain for setup cleanup"]
pub(crate) struct PreparedRoutedSkipOwner {
    sessions: Arc<EpisodePlaybackSessions>,
    issuer: NativeExportIssuer,
}
impl PreparedRoutedSkipOwner {
    /// Only trusted SDK assembly supplies the checked original route's captured issuer.
    pub(crate) fn new(sessions: Arc<EpisodePlaybackSessions>, issuer: NativeExportIssuer) -> Self {
        Self { sessions, issuer }
    }
    pub(crate) fn sessions(&self) -> Arc<EpisodePlaybackSessions> {
        self.sessions.clone()
    }
    pub(crate) fn issuer(&self) -> NativeExportIssuer {
        self.issuer.clone()
    }
    pub(crate) fn into_parts(self) -> (Arc<EpisodePlaybackSessions>, NativeExportIssuer) {
        (self.sessions, self.issuer)
    }
}

#[must_use = "Original error and prepared ownership remain available to the caller"]
pub(crate) struct RoutedSkipSetupFailure {
    pub(crate) error: ApiError,
    pub(crate) prepared: PreparedRoutedSkipOwner,
}

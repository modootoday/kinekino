//! Application replacement port; no OS calls or GUI event dependency.
use crate::{
    application::{
        api::ApiError,
        playback_session::{
            ClosedEpisodePlayback, EpisodePlaybackEngine, EpisodePlaybackSource,
            OpenedEpisodePlayback,
        },
    },
    domain::api_values::DecimalU64,
};

/// Engine owner must compare BOTH old identities and replace under the same lock.
/// A separate close() followed by open() does not implement this contract.
pub trait EpisodeReplacementOwner: EpisodePlaybackEngine {
    fn replace_verified(
        &self,
        expected_instance: DecimalU64,
        expected_artifact: &str,
        source: EpisodePlaybackSource,
        paused: bool,
        volume_percent: f64,
    ) -> Result<ReplacedEpisode, ApiError>;
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplacedEpisode {
    pub previous: ClosedEpisodePlayback,
    /// Process creation only. Caller independently observes loaded media and time progress.
    pub current: OpenedEpisodePlayback,
}

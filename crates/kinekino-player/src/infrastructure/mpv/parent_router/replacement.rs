//! One original admitted engine's atomic replacement; no close/open emulation or new route.
use super::ParentEngineRouter;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        playback_session::{
            EpisodePlaybackSource,
            transition::{EpisodeReplacementOwner, ReplacedEpisode},
        },
    },
    domain::api_values::DecimalU64,
};

impl EpisodeReplacementOwner for ParentEngineRouter {
    fn replace_verified(
        &self,
        expected_instance: DecimalU64,
        expected_artifact: &str,
        source: EpisodePlaybackSource,
        paused: bool,
        volume_percent: f64,
    ) -> Result<ReplacedEpisode, ApiError> {
        // Keep the original in-flight admission until the existing native operation returns.
        // admitted() releases the router state lock before engine I/O.
        let admission = self.admitted()?;
        let owner = admission.engine.replacement_owner().ok_or_else(|| {
            ApiError::new(
                ErrorCode::Unsupported,
                "Original parent engine cannot replace episodes",
            )
        })?;
        owner.replace_verified(
            expected_instance,
            expected_artifact,
            source,
            paused,
            volume_percent,
        )
    }
}

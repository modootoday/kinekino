use super::SkipBinding;
use crate::{
    application::{
        api::ApiError,
        playback_control::ControlReceipt,
        playback_session::{ControlEpisodePlayback, EpisodePlaybackSessions},
        playback_status::PlaybackStatus,
    },
    domain::skip_segments::Seekability,
};

pub struct SkipObservation {
    pub binding: SkipBinding,
    pub playback: PlaybackStatus,
    pub seekability: Seekability,
    /// A trusted host may authorize an observation; publication age alone cannot.
    pub observation_authorized: bool,
}
pub trait SkipPlaybackPort {
    fn observe(&self) -> Result<SkipObservation, ApiError>;
    /// Existing instance-checked control admission. Errors never authorize auto replay.
    fn submit(&self, input: ControlEpisodePlayback) -> Result<ControlReceipt, ApiError>;
}
impl SkipPlaybackPort for EpisodePlaybackSessions {
    fn observe(&self) -> Result<SkipObservation, ApiError> {
        let observation = EpisodePlaybackSessions::observe(self)?;
        Ok(SkipObservation {
            binding: SkipBinding {
                engine_instance_id: observation.engine_instance_id,
                resolution_artifact_id: observation.resolution_artifact_id,
            },
            playback: observation.playback,
            seekability: Seekability::Unknown,
            observation_authorized: false,
        })
    }
    fn submit(&self, input: ControlEpisodePlayback) -> Result<ControlReceipt, ApiError> {
        EpisodePlaybackSessions::control(self, input)
    }
}

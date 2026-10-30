//! Typed instance-bound option requests, receipts and observations.
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        playback_control::{ControlAcceptance, ControlCompletion, ControlReceipt},
    },
    domain::{
        api_values::DecimalU64,
        playback_options::{
            ObservedTrackChoice, ObservedTracks, PlaybackRate, TrackCatalogError, TrackKind,
            TrackSelection,
        },
    },
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlaybackOptionsObservation {
    /// Actual finite positive engine value; input policy bounds are a separate contract.
    pub rate: Option<f64>,
    pub tracks: Option<ObservedTracks>,
    pub tracks_unavailable_reason: Option<TrackCatalogError>,
    pub audio_selection: Option<ObservedTrackChoice>,
    pub video_selection: Option<ObservedTrackChoice>,
    pub subtitle_selection: Option<ObservedTrackChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetEpisodeRate {
    pub engine_instance_id: DecimalU64,
    pub rate: PlaybackRate,
}
impl SetEpisodeRate {
    pub fn validate(&self) -> Result<(), ApiError> {
        require_instance(self.engine_instance_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectEpisodeTrack {
    pub engine_instance_id: DecimalU64,
    pub expected_tracks_revision: DecimalU64,
    pub track_kind: TrackKind,
    pub selection: TrackSelection,
}
impl SelectEpisodeTrack {
    pub fn validate(&self) -> Result<(), ApiError> {
        require_instance(self.engine_instance_id)?;
        if self.expected_tracks_revision.0 == 0 {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Observed track revision must be nonzero",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrackControlReceipt {
    pub acceptance: ControlAcceptance,
    pub completion: ControlCompletion,
    pub engine_instance_id: DecimalU64,
    pub validated_tracks_revision: DecimalU64,
}
impl TrackControlReceipt {
    pub fn enqueued(receipt: ControlReceipt, revision: DecimalU64) -> Self {
        Self {
            acceptance: receipt.acceptance,
            completion: receipt.completion,
            engine_instance_id: receipt.engine_instance_id,
            validated_tracks_revision: revision,
        }
    }
}
fn require_instance(instance: DecimalU64) -> Result<(), ApiError> {
    if instance.0 == 0 {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Native engine instance ID must be nonzero",
        ));
    }
    Ok(())
}

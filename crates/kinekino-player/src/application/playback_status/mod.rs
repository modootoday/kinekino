#[path = "ports.rs"]
mod ports;
pub use ports::PlaybackStatusPort;
mod observed;
mod projection;
pub(crate) use projection::project_native_status;
#[cfg(test)]
mod projection_tests;
pub use observed::{NativeStatusPublisher, NativeStatusReader, native_status_channel};

use crate::{
    application::api::{ApiError, ErrorCode},
    domain::api_values::DecimalU64,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackStatusOrigin {
    NativeObservation,
}

/// Last published native observation; position is engine playback time, not source PTS.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlaybackStatus {
    pub origin: PlaybackStatusOrigin,
    /// Age of publication; native event freshness is not currently available.
    pub published_age_ms: DecimalU64,
    pub connected: bool,
    pub paused: Option<bool>,
    pub idle: Option<bool>,
    pub position_us: Option<DecimalU64>,
    pub duration_us: Option<DecimalU64>,
    pub volume_percent: Option<f64>,
    #[serde(default)]
    pub muted: Option<bool>,
    pub video_size: Option<VideoSize>,
    pub audio_channels: Option<u32>,
    pub has_error: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VideoSize {
    pub width: u32,
    pub height: u32,
}

pub struct UnavailablePlaybackStatus;
impl PlaybackStatusPort for UnavailablePlaybackStatus {
    fn available(&self) -> bool {
        false
    }
    fn status(&self) -> Result<PlaybackStatus, ApiError> {
        Err(ApiError::new(
            ErrorCode::Unsupported,
            "Playback observation provider unavailable",
        ))
    }
}

pub fn payload_schema() -> serde_json::Value {
    serde_json::json!({"type":"object","maxProperties":0,"additionalProperties":false})
}

pub fn response_schema() -> serde_json::Value {
    let micros = serde_json::json!({"type":["string","null"],"pattern":"^[0-9]+$"});
    serde_json::json!({"type":"object","additionalProperties":false,"required":["origin","published_age_ms","connected","paused","idle","position_us","duration_us","volume_percent","muted","video_size","audio_channels","has_error"],"properties":{
        "origin":{"const":"native_observation"},
        "published_age_ms":{"type":"string","pattern":"^[0-9]+$"},
        "connected":{"type":"boolean"},"paused":{"type":["boolean","null"]},"idle":{"type":["boolean","null"]},
        "position_us":micros,"duration_us":micros,
        "volume_percent":{"type":["number","null"],"minimum":0},"muted":{"type":["boolean","null"]},
        "video_size":{"type":["object","null"],"additionalProperties":false,"required":["width","height"],"properties":{"width":{"type":"integer","minimum":1,"maximum":u32::MAX},"height":{"type":"integer","minimum":1,"maximum":u32::MAX}}},
        "audio_channels":{"type":["integer","null"],"minimum":1,"maximum":u32::MAX},"has_error":{"type":"boolean"}
    }})
}

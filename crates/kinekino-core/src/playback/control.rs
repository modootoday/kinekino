//! Playback commands shared by every backend (Windows native, Unix mpv, fakes).
use kinekino_media_contracts::api_values::DecimalU64;
use serde::{Deserialize, Serialize};
pub const MAX_SEEK_US: u64 = 9_007_199_254_740_991;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum PlaybackControl {
    Play,
    Pause,
    Seek { position_us: DecimalU64 },
    SetVolume { volume_percent: f64 },
    SetMuted { muted: bool },
}
impl PlaybackControl {
    pub fn validate(&self) -> Result<(), &'static str> {
        match self {
            Self::Seek { position_us } if position_us.0 > MAX_SEEK_US => {
                Err("Seek position exceeds native numeric precision budget")
            }
            Self::SetVolume { volume_percent }
                if !volume_percent.is_finite() || !(0.0..=100.0).contains(volume_percent) =>
            {
                Err("Volume must be finite and between 0 and 100 percent")
            }
            _ => Ok(()),
        }
    }
    pub fn command(&self) -> &'static str {
        match self {
            Self::Play => "playback.play",
            Self::Pause => "playback.pause",
            Self::Seek { .. } => "playback.seek",
            Self::SetVolume { .. } => "playback.set_volume",
            Self::SetMuted { .. } => "playback.set_muted",
        }
    }
    pub fn payload(&self) -> serde_json::Value {
        match self {
            Self::Play | Self::Pause => serde_json::json!({}),
            Self::Seek { position_us } => serde_json::json!({"position_us":position_us}),
            Self::SetVolume { volume_percent } => {
                serde_json::json!({"volume_percent":volume_percent})
            }
            Self::SetMuted { muted } => serde_json::json!({"muted":muted}),
        }
    }
}

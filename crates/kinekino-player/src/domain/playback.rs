use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WebPlayback {
    pub host: String,
    pub time: f64,
    pub duration: f64,
    pub paused: bool,
    pub muted: bool,
    pub volume: f64,
    pub ready: u8,
    pub width: u32,
    pub height: u32,
    pub error: u8,
    pub hidden_ads: u32,
    pub visible_ads: u32,
    pub audio_bytes: u64,
    pub decoded_frames: u64,
}

impl WebPlayback {
    pub fn valid(&self) -> bool {
        matches!(
            self.host.as_str(),
            "media.example.test" | "frame.example.test" | "catalog.example.test"
        ) && self.time.is_finite()
            && self.time >= 0.0
            && self.duration.is_finite()
            && self.duration >= 0.0
            && self.time <= self.duration + 1.0
            && self.volume.is_finite()
            && (0.0..=1.0).contains(&self.volume)
            && self.ready <= 4
            && self.error <= 4
    }
}

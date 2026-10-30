//! Intended domain/native_audio.rs: explicit intent, never inferred from observed volume.
use super::playback_control::PlaybackControl;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub struct AudioIntent {
    volume_percent: f64,
    muted: bool,
}
impl AudioIntent {
    pub fn new(volume_percent: f64, muted: bool) -> Result<Self, &'static str> {
        PlaybackControl::SetVolume { volume_percent }.validate()?;
        Ok(Self {
            volume_percent,
            muted,
        })
    }
    pub fn volume_percent(self) -> f64 {
        self.volume_percent
    }
    pub fn muted(self) -> bool {
        self.muted
    }
}

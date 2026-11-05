//! Player viewing preferences (#9 / M1-7): volume, mute, speed, and auto-next.
//! These are the last user intent, not engine observations (sot:kinekino/viewing-preferences-and-resume).
use crate::port::error::{ErrorCode, PortError, PortResult};

/// Inclusive bounds the player UI and the engine already use for volume and speed.
pub const VOLUME_MIN: f64 = 0.0;
pub const VOLUME_MAX: f64 = 100.0;
pub const SPEED_MIN: f64 = 0.25;
pub const SPEED_MAX: f64 = 4.0;

/// One saved player prefs document. Defaults match the empty player view.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerPrefs {
    pub volume: f64,
    pub muted: bool,
    pub speed: f64,
    pub auto_next: bool,
}

impl Default for PlayerPrefs {
    fn default() -> Self {
        Self {
            volume: VOLUME_MAX,
            muted: false,
            speed: 1.0,
            auto_next: false,
        }
    }
}

impl PlayerPrefs {
    /// Clamps volume and speed into the ranges the player accepts; bools stay as given.
    pub fn clamped(self) -> Self {
        Self {
            volume: self.volume.clamp(VOLUME_MIN, VOLUME_MAX),
            muted: self.muted,
            speed: round2(self.speed.clamp(SPEED_MIN, SPEED_MAX)),
            auto_next: self.auto_next,
        }
    }

    /// Rejects non-finite numbers so a corrupt on-disk value cannot enter the view.
    pub fn checked(self) -> PortResult<Self> {
        if !self.volume.is_finite() || !self.speed.is_finite() {
            return Err(PortError::new(
                ErrorCode::InvalidInput,
                "player prefs: volume and speed must be finite",
            ));
        }
        Ok(self.clamped())
    }
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_empty_player_view() {
        let p = PlayerPrefs::default();
        assert_eq!(p.volume, 100.0);
        assert!(!p.muted);
        assert_eq!(p.speed, 1.0);
        assert!(!p.auto_next);
    }

    #[test]
    fn clamp_keeps_volume_and_speed_in_range() {
        let p = PlayerPrefs {
            volume: 250.0,
            muted: true,
            speed: 0.01,
            auto_next: true,
        }
        .clamped();
        assert_eq!(p.volume, 100.0);
        assert_eq!(p.speed, SPEED_MIN);
        assert!(p.muted && p.auto_next);
    }

    #[test]
    fn non_finite_values_are_rejected() {
        let p = PlayerPrefs {
            volume: f64::NAN,
            ..PlayerPrefs::default()
        };
        assert!(p.checked().is_err());
    }
}

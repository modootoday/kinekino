//! On-disk format v1:
//! `{"version":1,"volume":N,"muted":bool,"speed":N,"auto_next":bool}`.
//! Unknown fields are rejected; volume is stored as an integer percent.
use crate::playback::prefs::PlayerPrefs;
use serde::{Deserialize, Serialize};

pub const VERSION: u64 = 1;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    version: u64,
    volume: u64,
    muted: bool,
    speed: f64,
    auto_next: bool,
}

/// What a file body turned out to be.
#[derive(Debug, PartialEq)]
pub enum Decoded {
    V1(PlayerPrefs),
    /// Written by a newer build; must not be read or overwritten.
    Newer,
    /// Not JSON, wrong shape, unknown fields, or no `version`.
    Corrupt,
}

pub fn decode(body: &[u8]) -> Decoded {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(body) else {
        return Decoded::Corrupt;
    };
    match value.get("version").and_then(serde_json::Value::as_u64) {
        Some(v) if v > VERSION => Decoded::Newer,
        Some(VERSION) => match serde_json::from_value::<Document>(value) {
            Ok(doc) => {
                let prefs = PlayerPrefs {
                    volume: doc.volume as f64,
                    muted: doc.muted,
                    speed: doc.speed,
                    auto_next: doc.auto_next,
                };
                match prefs.checked() {
                    Ok(prefs) => Decoded::V1(prefs),
                    Err(_) => Decoded::Corrupt,
                }
            }
            Err(_) => Decoded::Corrupt,
        },
        _ => Decoded::Corrupt,
    }
}

pub fn encode(prefs: &PlayerPrefs) -> serde_json::Result<Vec<u8>> {
    let prefs = prefs.clamped();
    let doc = Document {
        version: VERSION,
        volume: prefs.volume.round().clamp(0.0, 100.0) as u64,
        muted: prefs.muted,
        speed: prefs.speed,
        auto_next: prefs.auto_next,
    };
    serde_json::to_vec(&doc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_keeps_values() {
        let prefs = PlayerPrefs {
            volume: 80.0,
            muted: true,
            speed: 1.25,
            auto_next: true,
        };
        let body = encode(&prefs).unwrap();
        assert_eq!(decode(&body), Decoded::V1(prefs));
    }

    #[test]
    fn unknown_field_is_corrupt() {
        let body =
            br#"{"version":1,"volume":50,"muted":false,"speed":1.0,"auto_next":false,"extra":1}"#;
        assert_eq!(decode(body), Decoded::Corrupt);
    }

    #[test]
    fn newer_version_is_not_read() {
        let body = br#"{"version":2,"volume":50,"muted":false,"speed":1.0,"auto_next":false}"#;
        assert_eq!(decode(body), Decoded::Newer);
    }

    #[test]
    fn missing_version_is_corrupt() {
        let body = br#"{"volume":50,"muted":false,"speed":1.0,"auto_next":false}"#;
        assert_eq!(decode(body), Decoded::Corrupt);
    }
}

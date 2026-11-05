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
        super::admitted_hosts::active_hosts().admits_player(&self.host)
            && self.time.is_finite()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::admitted_hosts::{AdmittedHosts, with_admitted_hosts};

    fn sample(host: &str) -> WebPlayback {
        WebPlayback {
            host: host.into(),
            time: 1.0,
            duration: 10.0,
            paused: false,
            muted: false,
            volume: 0.5,
            ready: 4,
            width: 1280,
            height: 720,
            error: 0,
            hidden_ads: 0,
            visible_ads: 0,
            audio_bytes: 1,
            decoded_frames: 1,
        }
    }

    #[test]
    fn fixture_player_hosts_remain_valid() {
        assert!(sample("media.example.test").valid());
        assert!(sample("frame.example.test").valid());
        // Catalog is navigable only — not a player host under the fixture.
        assert!(!sample("catalog.example.test").valid());
    }

    #[test]
    fn admits_player_override_accepts_installed_host_only() {
        let hosts = AdmittedHosts::new(["cdn.other.test"]);
        with_admitted_hosts(hosts, || {
            assert!(sample("cdn.other.test").valid());
            assert!(!sample("media.example.test").valid());
            assert!(!sample("frame.example.test").valid());
            assert!(!sample("catalog.example.test").valid());
        });
    }
}

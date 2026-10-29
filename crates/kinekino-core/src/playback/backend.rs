//! Port for a media engine that can open a source, accept [`PlaybackControl`]s and report
//! status. Adapters live in platform crates; [`FakeBackend`] is the in-memory test double.
use super::control::PlaybackControl;
use crate::port::error::{ErrorCode, PortError, PortResult};

/// Engine-reported state; positions in microseconds.
#[derive(Debug, Clone, PartialEq)]
pub struct BackendStatus {
    pub source: Option<String>,
    pub position_us: u64,
    pub duration_us: Option<u64>,
    pub paused: bool,
    pub volume_percent: f64,
    pub muted: bool,
    pub eof: bool,
}

pub trait PlaybackBackend {
    /// Load `source` (local path or URL) paused at `start_us`.
    fn open(&mut self, source: &str, start_us: u64) -> PortResult<()>;
    fn control(&mut self, control: &PlaybackControl) -> PortResult<()>;
    fn status(&mut self) -> PortResult<BackendStatus>;
    /// Stop and release the engine; later calls fail with `Unavailable`.
    fn close(&mut self) -> PortResult<()>;
}

fn validated(control: &PlaybackControl) -> PortResult<()> {
    control
        .validate()
        .map_err(|m| PortError::new(ErrorCode::InvalidInput, m))
}

/// Deterministic backend: time advances only through `advance`.
#[derive(Debug, Default)]
pub struct FakeBackend {
    status: Option<BackendStatus>,
    closed: bool,
    pub duration_us: u64,
}

impl FakeBackend {
    pub fn with_duration(duration_us: u64) -> Self {
        Self {
            duration_us,
            ..Self::default()
        }
    }
    /// Simulate `us` of playback when unpaused.
    pub fn advance(&mut self, us: u64) {
        if let Some(s) = self.status.as_mut().filter(|s| !s.paused) {
            s.position_us = (s.position_us + us).min(self.duration_us);
            s.eof = s.position_us == self.duration_us;
        }
    }
    fn live(&mut self) -> PortResult<&mut BackendStatus> {
        if self.closed {
            return Err(PortError::new(ErrorCode::Unavailable, "Backend is closed"));
        }
        self.status
            .as_mut()
            .ok_or(PortError::new(ErrorCode::Unavailable, "No media is open"))
    }
}

impl PlaybackBackend for FakeBackend {
    fn open(&mut self, source: &str, start_us: u64) -> PortResult<()> {
        if self.closed {
            return Err(PortError::new(ErrorCode::Unavailable, "Backend is closed"));
        }
        if source.is_empty() {
            return Err(PortError::new(ErrorCode::InvalidInput, "Source is empty"));
        }
        self.status = Some(BackendStatus {
            source: Some(source.into()),
            position_us: start_us.min(self.duration_us),
            duration_us: Some(self.duration_us),
            paused: true,
            volume_percent: 100.0,
            muted: false,
            eof: false,
        });
        Ok(())
    }
    fn control(&mut self, control: &PlaybackControl) -> PortResult<()> {
        validated(control)?;
        let duration = self.duration_us;
        let s = self.live()?;
        match control {
            PlaybackControl::Play => s.paused = false,
            PlaybackControl::Pause => s.paused = true,
            PlaybackControl::Seek { position_us } => {
                s.position_us = position_us.0.min(duration);
                s.eof = false;
            }
            PlaybackControl::SetVolume { volume_percent } => s.volume_percent = *volume_percent,
            PlaybackControl::SetMuted { muted } => s.muted = *muted,
        }
        Ok(())
    }
    fn status(&mut self) -> PortResult<BackendStatus> {
        self.live().map(|s| s.clone())
    }
    fn close(&mut self) -> PortResult<()> {
        self.closed = true;
        self.status = None;
        Ok(())
    }
}

/// Shared contract for every [`PlaybackBackend`] adapter. `source` must be a playable
/// medium at least 2 s long. Checks open-paused, seek, volume/mute, invalid input, close.
pub fn check_backend(backend: &mut dyn PlaybackBackend, source: &str) -> PortResult<()> {
    use kinekino_media_contracts::api_values::DecimalU64;
    let fail = |m: &str| PortError::new(ErrorCode::Internal, format!("contract: {m}"));
    backend.open(source, 500_000)?;
    let s = backend.status()?;
    if !s.paused || s.position_us.abs_diff(500_000) > 100_000 {
        return Err(fail("open must start paused near start_us"));
    }
    backend.control(&PlaybackControl::Seek {
        position_us: DecimalU64(1_000_000),
    })?;
    backend.control(&PlaybackControl::SetVolume {
        volume_percent: 40.0,
    })?;
    backend.control(&PlaybackControl::SetMuted { muted: true })?;
    let s = backend.status()?;
    if s.position_us.abs_diff(1_000_000) > 100_000
        || !s.muted
        || (s.volume_percent - 40.0).abs() > 0.5
    {
        return Err(fail("seek/volume/mute not reflected in status"));
    }
    let bad = PlaybackControl::SetVolume {
        volume_percent: 101.0,
    };
    if backend.control(&bad).map_err(|e| e.code) != Err(ErrorCode::InvalidInput) {
        return Err(fail("out-of-range volume must be InvalidInput"));
    }
    backend.control(&PlaybackControl::Play)?;
    backend.control(&PlaybackControl::Pause)?;
    if !backend.status()?.paused {
        return Err(fail("pause not reflected"));
    }
    backend.close()?;
    if backend.status().map_err(|e| e.code) != Err(ErrorCode::Unavailable) {
        return Err(fail("status after close must be Unavailable"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fake_meets_contract_and_advances_only_when_playing() {
        check_backend(&mut FakeBackend::with_duration(10_000_000), "fixture.mkv").unwrap();
        let mut b = FakeBackend::with_duration(3_000_000);
        assert!(b.status().is_err());
        b.open("fixture.mkv", 0).unwrap();
        b.advance(1_000_000);
        assert_eq!(b.status().unwrap().position_us, 0);
        b.control(&PlaybackControl::Play).unwrap();
        b.advance(5_000_000);
        let s = b.status().unwrap();
        assert!(s.eof && s.position_us == 3_000_000);
        assert!(b.open("", 0).is_err());
    }
}

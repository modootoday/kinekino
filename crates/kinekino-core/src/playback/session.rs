//! Ties a [`PlaybackBackend`] to a [`ResumeStore`] under a [`ResumePolicy`]:
//! open at the saved point, save (or clear) on checkpoint/stop.
use super::backend::{BackendStatus, PlaybackBackend};
use super::control::PlaybackControl;
use super::resume::ResumePolicy;
use super::store::ResumeStore;
use crate::port::error::{ErrorCode, PortError, PortResult};

pub struct ResumingPlayback<B, S> {
    pub backend: B,
    pub store: S,
    pub policy: ResumePolicy,
    key: Option<String>,
}

impl<B: PlaybackBackend, S: ResumeStore> ResumingPlayback<B, S> {
    pub fn new(backend: B, store: S, policy: ResumePolicy) -> Self {
        Self {
            backend,
            store,
            policy,
            key: None,
        }
    }

    /// Open `source` under `key`; returns the start offset used (0 = from the beginning).
    pub fn open(&mut self, key: &str, source: &str) -> PortResult<u64> {
        if key.is_empty() {
            return Err(PortError::new(
                ErrorCode::InvalidInput,
                "Resume key is empty",
            ));
        }
        let start = self.policy.start(self.store.get(key)?, None);
        self.backend.open(source, start)?;
        self.key = Some(key.into());
        Ok(start)
    }

    pub fn control(&mut self, control: &PlaybackControl) -> PortResult<()> {
        self.backend.control(control)
    }

    pub fn status(&mut self) -> PortResult<BackendStatus> {
        self.backend.status()
    }

    /// Persist the current position (call periodically and on pause).
    pub fn checkpoint(&mut self) -> PortResult<()> {
        let Some(key) = self.key.clone() else {
            return Err(PortError::new(ErrorCode::Unavailable, "No media is open"));
        };
        let s = self.backend.status()?;
        let finished = s.eof;
        match s
            .duration_us
            .filter(|_| !finished)
            .and_then(|d| self.policy.record(s.position_us, d))
        {
            Some(p) => self.store.put(&key, p),
            None => self.store.clear(&key).map(drop),
        }
    }

    /// Checkpoint, then close the backend.
    pub fn stop(&mut self) -> PortResult<()> {
        let saved = self.checkpoint();
        self.key = None;
        let closed = self.backend.close();
        saved.and(closed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::playback::{FakeBackend, MemoryResumeStore, ResumePoint};
    use kinekino_media_contracts::api_values::DecimalU64;

    fn player() -> ResumingPlayback<FakeBackend, MemoryResumeStore> {
        ResumingPlayback::new(
            FakeBackend::with_duration(1_200_000_000),
            MemoryResumeStore::default(),
            ResumePolicy::default(),
        )
    }

    #[test]
    fn saves_on_stop_and_resumes_with_rewind() {
        let mut p = player();
        assert_eq!(p.open("ep1", "ep1.mkv").unwrap(), 0);
        p.control(&PlaybackControl::Seek {
            position_us: DecimalU64(600_000_000),
        })
        .unwrap();
        p.stop().unwrap();
        let saved = p.store.get("ep1").unwrap().unwrap();
        assert_eq!(saved.position_us, 600_000_000);
        p.backend = FakeBackend::with_duration(1_200_000_000);
        assert_eq!(p.open("ep1", "ep1.mkv").unwrap(), 597_000_000);
        assert_eq!(p.status().unwrap().position_us, 597_000_000);
    }

    #[test]
    fn finishing_or_early_stop_clears_the_entry() {
        let mut p = player();
        p.store
            .put(
                "ep1",
                ResumePoint {
                    position_us: 600_000_000,
                    duration_us: 1_200_000_000,
                },
            )
            .unwrap();
        p.open("ep1", "ep1.mkv").unwrap();
        p.control(&PlaybackControl::Play).unwrap();
        p.backend.advance(u64::MAX / 2);
        p.checkpoint().unwrap();
        assert_eq!(p.store.get("ep1").unwrap(), None);
        p.control(&PlaybackControl::Seek {
            position_us: DecimalU64(2_000_000),
        })
        .unwrap();
        p.store
            .put(
                "ep1",
                ResumePoint {
                    position_us: 600_000_000,
                    duration_us: 1_200_000_000,
                },
            )
            .unwrap();
        p.checkpoint().unwrap();
        assert_eq!(p.store.get("ep1").unwrap(), None);
    }

    #[test]
    fn rejects_empty_key_and_checkpoint_before_open() {
        let mut p = player();
        assert_eq!(p.checkpoint().unwrap_err().code, ErrorCode::Unavailable);
        assert_eq!(
            p.open("", "x.mkv").unwrap_err().code,
            ErrorCode::InvalidInput
        );
    }
}

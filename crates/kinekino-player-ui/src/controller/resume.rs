//! 이어보기 (ADR 20261103115459): a local file restores its saved point once its open
//! finished, and its position is saved at the decided moments through the `ResumeStore`
//! port. Items without a key (plugin items, remote streams) never touch the store, and a
//! failing store never blocks playback: errors are swallowed and reported once by code.
use super::{Controller, Engine, Media};
use kinekino_core::playback::{ResumeKey, ResumePolicy, ResumeStore};
use kinekino_core::port::error::{PortError, PortResult};
use std::time::{Duration, Instant};

/// While playing, the position is saved after this much media time moved since the last save.
pub const SAVE_EVERY: f64 = 10.0;
/// How long the restored point is shown while the engine still reports its old time; a
/// restore seek normally lands well within this, one that never does stops freezing it.
pub const RESTORE_HOLD: Duration = Duration::from_secs(1);
/// The notice shown after a restore, followed by the restored time.
const RESUMED: &str = "이어서 재생";
const US: f64 = 1_000_000.0;

/// The store, the opened item's key and what was last written for it.
#[derive(Default)]
pub(super) struct Resume {
    store: Option<Box<dyn ResumeStore>>,
    policy: ResumePolicy,
    /// The item that finished opening, when it has a key; `None` while another opens.
    key: Option<ResumeKey>,
    /// Position (s) last written or restored for `key`; interval saves count from it.
    saved_at: f64,
    /// The item played to its end and its entry was cleared; nothing to save until a seek.
    ended: bool,
    /// A restore seek (s) the engine has not reported reaching yet; until it does, the
    /// engine's time is the old one and says nothing about where the user is.
    restoring: Option<f64>,
    /// When the restore seek was sent; bounds how long a stale time is hidden.
    restore_sent: Option<Instant>,
    /// A store error was already reported in this run.
    warned: bool,
}

impl Resume {
    /// Runs `call` against the store; an error is swallowed (warned about once).
    fn with<T>(&mut self, call: impl FnOnce(&mut dyn ResumeStore) -> PortResult<T>) -> Option<T> {
        let result = call(self.store.as_deref_mut()?);
        result.map_err(|e| self.warn(&e)).ok()
    }

    /// Only the error code: messages of file adapters name paths.
    fn warn(&mut self, e: &PortError) {
        if !std::mem::replace(&mut self.warned, true) {
            eprintln!(
                "{{\"warning\":\"resume position not saved or read\",\"code\":\"{}\"}}",
                e.code.name()
            );
        }
    }
}

impl<E: Engine> Controller<E> {
    /// Saves and restores positions of local files in `store` (policy defaults).
    pub fn with_resume(mut self, store: Box<dyn ResumeStore>) -> Self {
        self.resume.store = Some(store);
        self
    }

    /// Saves the current item's position now (e.g. when the window run ends).
    pub fn save_resume(&mut self) {
        if !self.view.has_media || self.opening.is_some() || self.resume.ended {
            return;
        }
        if self.resume_restoring() {
            return;
        }
        let Some(key) = self.resume.key.clone() else {
            return;
        };
        let (position, duration) = (self.view.position, self.view.duration);
        // An unknown length cannot be judged; the stored entry is kept.
        if duration <= 0.0 || position == self.resume.saved_at {
            return;
        }
        self.resume.saved_at = position;
        let point = self.resume.policy.record(to_us(position), to_us(duration));
        let key = key.as_str();
        match point {
            Some(point) => self.resume.with(|s| s.put(key, point)),
            None => self.resume.with(|s| s.clear(key).map(drop)),
        };
    }

    /// After a poll while playing: saves once enough media time moved.
    pub(super) fn resume_tick(&mut self) {
        if self.resume_restoring() {
            return;
        }
        if self.view.playing && (self.view.position - self.resume.saved_at).abs() >= SAVE_EVERY {
            self.save_resume();
        }
    }

    /// The current item played to its end: its entry goes.
    pub(super) fn resume_ended(&mut self) {
        if self.resume.ended || self.opening.is_some() {
            return;
        }
        if let Some(key) = self.resume.key.clone() {
            self.resume.with(|s| s.clear(key.as_str()).map(drop));
            self.resume.ended = true;
        }
    }

    /// A seek moved away from the end: positions count again. A user seek also replaces a
    /// restore still under way.
    pub(super) fn resume_rewound(&mut self) {
        self.resume.ended = false;
        self.resume.restoring = None;
    }

    /// True while a restore seek has not landed; the polled position is then stale.
    fn resume_restoring(&self) -> bool {
        self.resume.restoring.is_some()
    }

    /// The engine reports `time` (s): true while that is a stale time from before the
    /// restore seek and the hold has not run out, so it is not shown. The restore ends once
    /// the engine's time comes near the point; until then nothing is saved either way.
    pub(super) fn resume_holds(&mut self, time: f64) -> bool {
        let Some(target) = self.resume.restoring else {
            return false;
        };
        if (time - target).abs() < SAVE_EVERY {
            self.resume.restoring = None;
            return false;
        }
        self.resume
            .restore_sent
            .is_some_and(|t| t.elapsed() < RESTORE_HOLD)
    }

    /// Before another item opens: the current one is saved and forgotten.
    pub(super) fn resume_leave(&mut self) {
        self.save_resume();
        self.resume.key = None;
        self.resume.ended = false;
    }

    /// The open of queue item `index` finished: its key is kept, and a saved point that is
    /// still valid for the opened length is sought to, with a notice saying so.
    pub(super) fn resume_opened(&mut self, index: usize) {
        self.resume.key = None;
        self.resume.ended = false;
        self.resume.restoring = None;
        self.resume.saved_at = 0.0;
        if self.resume.store.is_none() {
            return;
        }
        let Some(Media::Local(path)) = self.files.get(index) else {
            return;
        };
        // A path given relative to the working directory still names one file.
        let path = std::path::absolute(path).unwrap_or_else(|_| path.clone());
        let Some(key) = ResumeKey::for_source(&path.to_string_lossy()) else {
            return;
        };
        let saved = self.resume.with(|s| s.get(key.as_str())).flatten();
        self.resume.key = Some(key);
        let duration = self.engine.property("duration").filter(|d| *d > 0.0);
        let start = self.resume.policy.start(saved, duration.map(to_us));
        if start == 0 {
            return;
        }
        let secs = start as f64 / US;
        let r = self
            .engine
            .command(&["seek", &format!("{secs:.3}"), "absolute"]);
        if r.is_err() {
            return self.report(r);
        }
        self.view.position = secs;
        self.view.duration = duration.unwrap_or(self.view.duration);
        self.resume.saved_at = secs;
        self.resume.restoring = Some(secs);
        self.resume.restore_sent = Some(Instant::now());
        self.view.notice = format!("{RESUMED} {}", crate::session::clock(secs));
        self.notice_since = Some(Instant::now());
    }
}

fn to_us(secs: f64) -> u64 {
    (secs.max(0.0) * US).round() as u64
}

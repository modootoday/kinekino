//! Player prefs (ADR 20261105092851): volume, mute, speed, and auto-next survive across runs
//! through the `PlayerPrefsStore` port. A failing store never blocks playback.
use super::{Controller, Engine};
use kinekino_core::playback::{PlayerPrefs, PlayerPrefsStore};
use kinekino_core::port::error::{PortError, PortResult};

#[derive(Default)]
pub(super) struct Prefs {
    store: Option<Box<dyn PlayerPrefsStore>>,
    /// A store error was already reported in this run.
    warned: bool,
}

impl Prefs {
    fn with<T>(
        &mut self,
        call: impl FnOnce(&mut dyn PlayerPrefsStore) -> PortResult<T>,
    ) -> Option<T> {
        let result = call(self.store.as_deref_mut()?);
        result.map_err(|e| self.warn(&e)).ok()
    }

    fn warn(&mut self, e: &PortError) {
        if !std::mem::replace(&mut self.warned, true) {
            eprintln!(
                "{{\"warning\":\"player prefs not saved or read\",\"code\":\"{}\"}}",
                e.code.name()
            );
        }
    }
}

impl<E: Engine> Controller<E> {
    /// Loads prefs from `store` into the view and engine, then keeps saving changes there.
    pub fn with_prefs(mut self, store: Box<dyn PlayerPrefsStore>) -> Self {
        match store.load() {
            Ok(prefs) => self.apply_prefs(prefs),
            Err(e) => {
                self.prefs.warned = true;
                eprintln!(
                    "{{\"warning\":\"player prefs not saved or read\",\"code\":\"{}\"}}",
                    e.code.name()
                );
            }
        }
        self.prefs.store = Some(store);
        self
    }

    /// Writes the current view prefs (e.g. when the window run ends).
    pub fn save_prefs(&mut self) {
        if self.prefs.store.is_none() {
            return;
        }
        let prefs = self.current_prefs();
        self.prefs.with(|s| s.save(&prefs));
    }

    fn current_prefs(&self) -> PlayerPrefs {
        PlayerPrefs {
            volume: self.view.volume,
            muted: self.view.muted,
            speed: self.view.speed,
            auto_next: self.view.auto_next,
        }
        .clamped()
    }

    fn apply_prefs(&mut self, prefs: PlayerPrefs) {
        let prefs = prefs.clamped();
        self.view.volume = prefs.volume;
        self.view.muted = prefs.muted;
        self.view.speed = prefs.speed;
        self.view.auto_next = prefs.auto_next;
        let _ = self
            .engine
            .command(&["set", "volume", &format!("{:.0}", prefs.volume)]);
        let mute = if prefs.muted { "yes" } else { "no" };
        let _ = self.engine.command(&["set", "mute", mute]);
        let _ = self
            .engine
            .command(&["set", "speed", &format!("{:.2}", prefs.speed)]);
    }

    pub(super) fn prefs_changed(&mut self) {
        self.save_prefs();
    }
}

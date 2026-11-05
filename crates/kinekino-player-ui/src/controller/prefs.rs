//! Player prefs (ADR 20261105092851): volume, mute, speed, and auto-next survive across runs
//! through the `PlayerPrefsStore` port. A failing store never blocks playback.
use super::{Controller, Engine, SPEED_MAX, SPEED_MIN};
use crate::session::clock;
use kinekino_core::playback::{PlayerPrefs, PlayerPrefsStore};
use kinekino_core::port::error::{PortError, PortResult};
use std::time::Instant;

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

    /// Sets the volume within 0–100; `unmute` lifts a mute when the result is audible.
    pub(super) fn set_volume(&mut self, v: f64, unmute: bool) {
        let v = v.clamp(0.0, 100.0);
        self.view.volume = v;
        let r = self.engine.command(&["set", "volume", &format!("{v:.0}")]);
        self.report(r);
        if self.view.muted && unmute && v > 0.0 {
            self.view.muted = false;
            let r = self.engine.command(&["set", "mute", "no"]);
            self.report(r);
        }
        self.prefs_changed();
        self.osd_volume();
    }

    /// Sets the playback speed within `SPEED_MIN`–`SPEED_MAX`; the engine keeps it for the
    /// next item. Over a video the new speed shows as a short notice, unless one is opening.
    pub(super) fn set_speed(&mut self, v: f64) {
        let v = ((v * 100.0).round() / 100.0).clamp(SPEED_MIN, SPEED_MAX);
        self.view.speed = v;
        let r = self.engine.command(&["set", "speed", &format!("{v:.2}")]);
        self.report(r);
        if !self.view.opening {
            let shown = format!("{v:.2}");
            let shown = shown.strip_suffix('0').unwrap_or(&shown);
            self.view.notice = format!("재생 속도 {shown}x");
            self.notice_since = Some(Instant::now());
        }
        self.prefs_changed();
    }

    /// Bottom-toast OSD for seek (ADR 20261105182156). Skipped while an item is opening.
    pub(super) fn osd_seek(&mut self) {
        self.show_osd(format!(
            "{} / {}",
            clock(self.view.position),
            clock(self.view.duration)
        ));
    }

    /// Bottom-toast OSD for the current volume level.
    pub(super) fn osd_volume(&mut self) {
        self.show_osd(format!("음량 {:.0}", self.view.volume));
    }

    /// Bottom-toast OSD after toggling mute.
    pub(super) fn osd_mute(&mut self) {
        if self.view.muted {
            self.show_osd("음소거".into());
        } else {
            self.osd_volume();
        }
    }

    fn show_osd(&mut self, message: String) {
        if self.view.opening {
            return;
        }
        self.view.notice = message;
        self.notice_since = Some(Instant::now());
    }
}

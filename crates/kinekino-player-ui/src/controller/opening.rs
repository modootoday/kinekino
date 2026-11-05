//! Opening a file or stream without holding the UI thread (KB-003). The engine is asked to
//! start the open and is then checked on every window tick; the view shows the opening
//! state meanwhile. A newer choice replaces the open and Esc cancels it, so a late result
//! of an abandoned open is never shown.
use super::{Controller, Engine, Located};
use std::time::{Duration, Instant};

/// Shown while an item opens: on the start screen, over a playing video, or in the browse
/// view (there as the usual loading line).
pub(super) const OPENING: &str = "여는 중…";
/// `finish` gives up on an open after this long (engines time out well before).
const FINISH_WITHIN: Duration = Duration::from_secs(30);

/// The open in flight: which queue item it is for and the title it will show.
pub(super) struct Opening {
    index: usize,
    title: String,
}

impl<E: Engine> Controller<E> {
    /// An engine open is in flight (the window checks it every tick).
    pub fn is_opening(&self) -> bool {
        self.opening.is_some()
    }

    /// A new mount drops old messages but keeps the line of an item still opening
    /// (the window polls every tick while one is, so it shows as soon as it is ready).
    pub fn remounted(&mut self) {
        if !self.view.opening {
            self.view.status.clear();
        }
    }

    /// Starts opening; instant engines finish here, slow ones on later `poll`s.
    pub(super) fn load(&mut self, index: usize, remote: bool, (source, title, headers): Located) {
        self.opening = None;
        // Engines open paused on the first frame; viewing starts playing.
        let begun = self
            .set_headers(remote, &headers)
            .and_then(|()| self.engine.begin_load(&source, 0.0));
        if let Err(e) = begun {
            return self.cannot_play(&e.message);
        }
        self.opening = Some(Opening { index, title });
        self.show_opening();
        self.settle_open();
    }

    /// Advances the open in flight; true when it ended (opened or failed).
    pub(super) fn settle_open(&mut self) -> bool {
        if self.opening.is_none() {
            return false;
        }
        let ready = match self.engine.load_ready() {
            Ok(true) => self.engine.command(&["set", "pause", "no"]).map(|()| true),
            other => other,
        };
        match ready {
            Ok(false) => false,
            Ok(true) => {
                let opening = self.opening.take();
                self.end_opening();
                if let Some(Opening { index, title }) = opening {
                    self.opened(index, title);
                }
                true
            }
            Err(e) => {
                self.opening = None;
                self.cannot_play(&e.message);
                true
            }
        }
    }

    fn opened(&mut self, index: usize, title: String) {
        self.view.current = Some(index);
        self.view.title = title;
        self.view.has_media = true;
        self.view.browsing = false;
        self.view.browse_status.clear();
        self.view.browse_mood = "";
        self.view.playing = true;
        self.view.position = 0.0;
        self.view.duration = 0.0;
        self.view.subtitle.clear();
        self.view.status.clear();
        self.resume_opened(index);
    }

    /// The opening line where the user is; also covers a plugin still resolving the item.
    pub(super) fn show_opening(&mut self) {
        self.view.opening = true;
        if self.view.browsing {
            self.view.browse_status = super::LOADING.into();
            self.view.browse_mood = "loading";
        } else if self.view.has_media {
            // Stays until the open ends; `idle` only expires timed notices.
            self.view.notice = OPENING.into();
            self.notice_since = None;
        } else {
            self.view.status = OPENING.into();
        }
    }

    /// Clears the opening line (not other messages shown since).
    pub(super) fn end_opening(&mut self) {
        if !std::mem::take(&mut self.view.opening) {
            return;
        }
        if self.view.notice == OPENING {
            self.view.notice.clear();
        }
        if self.view.status == OPENING {
            self.view.status.clear();
        }
        if self.view.browse_mood == "loading" {
            self.view.browse_status.clear();
            self.view.browse_mood = "";
        }
    }

    /// Esc while opening: the engine stops, the item's lookup is dropped, and a late answer
    /// of either is ignored. A video replaced by the open is gone, so the start screen or
    /// the browse view shows. True when there was something to cancel.
    pub(super) fn cancel_open(&mut self) -> bool {
        if !self.view.opening {
            return false;
        }
        self.pending = None;
        if self.opening.take().is_some() {
            let r = self.engine.command(&["stop"]);
            self.report(r);
            self.view.has_media = false;
            self.view.playing = false;
            self.view.current = None;
            self.view.title.clear();
            self.view.subtitle.clear();
            self.view.position = 0.0;
            self.view.duration = 0.0;
        }
        self.end_opening();
        true
    }

    /// Waits for the plugin call and the open in flight (headless use and tests).
    pub fn finish(&mut self) {
        let deadline = Instant::now() + FINISH_WITHIN;
        while (self.pending.is_some() || self.opening.is_some()) && Instant::now() < deadline {
            self.settle(Some(Duration::from_secs(10)));
            if !self.settle_open() && self.opening.is_some() {
                std::thread::sleep(Duration::from_millis(5));
            }
        }
    }
}

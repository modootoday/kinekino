//! Which maximize state the caption shows. Caption requests show at once; changes the
//! window manager makes on its own (shortcut, tiling, its menu) arrive as resize or move
//! events and are read back from the window.

use std::time::{Duration, Instant};

/// How long a caption request outranks a differing report. X11 window managers apply
/// `_NET_WM_STATE` changes asynchronously, so resize events in between still report the
/// old state; a request the window manager ignores stops holding after this.
pub const MAXIMIZE_SETTLE: Duration = Duration::from_secs(1);

#[derive(Debug, Default)]
pub struct MaximizeFollow {
    /// Requested state and when, until the window reports it or the settle time passes.
    pending: Option<(bool, Instant)>,
}

impl MaximizeFollow {
    /// The caption asked for `on` at `now` (and already shows it).
    pub fn requested(&mut self, on: bool, now: Instant) {
        self.pending = Some((on, now));
    }

    /// The window reports `maximized` after a resize or move at `now`: the state the
    /// caption should show, or `None` while a recent request is still being applied.
    pub fn observed(&mut self, maximized: bool, now: Instant) -> Option<bool> {
        if let Some((want, at)) = self.pending {
            if want != maximized && now.saturating_duration_since(at) <= MAXIMIZE_SETTLE {
                return None;
            }
            self.pending = None;
        }
        Some(maximized)
    }
}

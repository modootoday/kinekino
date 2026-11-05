//! Who draws the current subtitle: styled (ASS/SSA) and picture (PGS, DVD, DVB) tracks need
//! the engine's renderer for their positions, styles and bitmaps, so the engine draws them
//! into the frame; plain text tracks stay on the player's own overlay from `sub-text`.
use super::{Controller, Engine};

/// Subtitle codecs the overlay cannot show faithfully (mpv `current-tracks/sub/codec`).
const ENGINE_DRAWN: [&str; 6] = [
    "ass",
    "ssa",
    "hdmv_pgs_subtitle",
    "dvd_subtitle",
    "dvb_subtitle",
    "xsub",
];

/// Whether the engine draws a track with this codec; no track means the overlay (empty).
pub(super) fn engine_draws(codec: Option<&str>) -> bool {
    codec.is_some_and(|c| ENGINE_DRAWN.contains(&c.trim()))
}

impl<E: Engine> Controller<E> {
    /// Reads the current cue for the overlay, switching engine drawing when the selected
    /// track's kind changed. True when the overlay text changed.
    pub(super) fn poll_subtitle(&mut self) -> bool {
        let draws = engine_draws(self.engine.text("current-tracks/sub/codec").as_deref());
        if draws != self.engine_subtitles {
            let value = if draws { "yes" } else { "no" };
            // A refused switch is retried on the next poll.
            if self
                .engine
                .command(&["set", "sub-visibility", value])
                .is_ok()
            {
                self.engine_subtitles = draws;
            }
        }
        let subtitle = if self.engine_subtitles {
            String::new()
        } else {
            self.engine.text("sub-text").unwrap_or_default()
        };
        let changed = subtitle != self.view.subtitle;
        self.view.subtitle = subtitle;
        changed
    }
}

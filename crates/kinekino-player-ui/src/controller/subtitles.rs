//! Who draws the current subtitle: styled (ASS/SSA) and picture (PGS, DVD, DVB) tracks need
//! the engine's renderer for their positions, styles and bitmaps, so the engine draws them
//! into the frame; plain text tracks stay on the player's own overlay from `sub-text`.
//! While the controls show, the overlay lifts by the transport height in CSS; a cue the
//! engine draws is lifted the same distance through the engine's subtitle position.
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

/// Height in CSS pixels of the transport row (`--aw-transport-height` in tokens.css).
pub const TRANSPORT_HEIGHT_PX: u32 = 56;

/// The engine's default subtitle position: the bottom of the frame.
pub(super) const BOTTOM: u32 = 100;

/// Engine subtitle position (percent of the frame height, 100 = bottom) that keeps a cue
/// above the transport in a frame `height` pixels tall. The engine renders into the video
/// box at its CSS size, so its OSD height and the transport share one unit. Rounds down so
/// the cue never sits on the transport.
pub(super) fn lifted_position(height: f64) -> Option<u32> {
    let transport = f64::from(TRANSPORT_HEIGHT_PX);
    (height.is_finite() && height > transport)
        .then(|| (f64::from(BOTTOM) * (1.0 - transport / height)).floor() as u32)
}

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
        self.place_engine_subtitles();
        let subtitle = if self.engine_subtitles {
            String::new()
        } else {
            self.engine.text("sub-text").unwrap_or_default()
        };
        let changed = subtitle != self.view.subtitle;
        self.view.subtitle = subtitle;
        changed
    }

    /// Lifts engine-drawn cues above the transport while the controls show, and returns
    /// the engine to its bottom otherwise. Sent only on change; a refusal retries next poll.
    fn place_engine_subtitles(&mut self) {
        let wanted = if self.engine_subtitles && self.view.controls_visible {
            match self.engine.property("osd-height").and_then(lifted_position) {
                Some(position) => position,
                // No laid-out box yet: keep what was last placed.
                None => return,
            }
        } else {
            BOTTOM
        };
        if wanted != self.subtitle_position
            && self
                .engine
                .command(&["set", "sub-pos", &wanted.to_string()])
                .is_ok()
        {
            self.subtitle_position = wanted;
        }
    }
}

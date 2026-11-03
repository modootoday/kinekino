//! Audio and subtitle tracks: a key asks the engine for the next track of a kind and a
//! short notice names the one now selected.
use super::{Controller, Engine};
use std::time::Instant;

/// The kind of track a cycle key switches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Track {
    Audio,
    Subtitle,
}

impl Track {
    /// The engine's selection property (`aid`/`sid`).
    fn property(self) -> &'static str {
        match self {
            Track::Audio => "aid",
            Track::Subtitle => "sid",
        }
    }

    /// The engine's track type (`track-list/N/type`, `current-tracks/<type>`).
    fn kind(self) -> &'static str {
        match self {
            Track::Audio => "audio",
            Track::Subtitle => "sub",
        }
    }

    fn noun(self) -> &'static str {
        match self {
            Track::Audio => "오디오 트랙",
            Track::Subtitle => "자막",
        }
    }
}

/// The selected track number; the engine reports 'no' (or nothing) when none is selected.
fn track_number(selected: Option<&str>) -> Option<u64> {
    selected.and_then(|s| s.trim().parse().ok())
}

/// The notice after a cycle: none of that kind, off, or the selected number and its name.
fn label(track: Track, count: usize, selected: Option<&str>, name: Option<&str>) -> String {
    let noun = track.noun();
    if count == 0 {
        return format!("{noun} 없음");
    }
    match track_number(selected) {
        Some(n) => match name {
            Some(name) => format!("{noun} {n} · {name}"),
            None => format!("{noun} {n}"),
        },
        None => format!("{noun} 끔"),
    }
}

impl<E: Engine> Controller<E> {
    /// Asks the engine for the next `track`; over a video the result shows as a short
    /// notice, unless an item is opening.
    /// Audio never stops: the engine's cycle passes through 'no' after the last audio
    /// track, so one more step lands on the first (PotPlayer's "Cycle audio stream").
    /// Subtitles keep 'off' in their cycle.
    pub(super) fn cycle_track(&mut self, track: Track) {
        let r = self.engine.command(&["cycle", track.property()]);
        self.report(r);
        let count = self.track_count(track);
        let mut selected = self.engine.text(track.property());
        if track == Track::Audio && count > 0 && track_number(selected.as_deref()).is_none() {
            // Only once: an engine that stays at 'no' is not cycled forever.
            let r = self.engine.command(&["cycle", track.property()]);
            self.report(r);
            selected = self.engine.text(track.property());
        }
        if !self.view.opening {
            self.view.notice = self.track_label(track, count, selected.as_deref());
            self.notice_since = Some(Instant::now());
        }
    }

    /// How many tracks of `track`'s kind the engine lists.
    fn track_count(&mut self, track: Track) -> usize {
        // A missing, negative or NaN count is no tracks (`as` saturates at 0).
        let total = self.engine.property("track-list/count");
        let total = total.unwrap_or(0.0) as usize;
        (0..total)
            .filter(|i| {
                self.engine.text(&format!("track-list/{i}/type")).as_deref() == Some(track.kind())
            })
            .count()
    }

    /// Reads the selected track's name from the engine for the notice.
    fn track_label(&mut self, track: Track, count: usize, selected: Option<&str>) -> String {
        let mut name = None;
        for field in ["title", "lang"] {
            let at = format!("current-tracks/{}/{field}", track.kind());
            name = self.engine.text(&at).filter(|s| !s.trim().is_empty());
            if name.is_some() {
                break;
            }
        }
        label(track, count, selected, name.as_deref())
    }
}

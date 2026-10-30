//! Player behaviour without a window: commands → engine calls and view changes, engine
//! polling → view. The window host only forwards `Effect`s it cannot do itself.
pub use crate::port::Engine;
use crate::session::Command;
use crate::source::{Episode, View};
use kinekino_core::port::error::PortResult;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// VIEW-10: controls hide after this long without input while playing (fullscreen overlay).
pub const HIDE_AFTER: Duration = Duration::from_millis(2500);

/// "Previous" within this many seconds of the start goes to the previous episode.
pub const RESTART_WITHIN: f64 = 3.0;

/// Work only the window can do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    Minimize,
    Maximize(bool),
    Fullscreen(bool),
    PickFile,
    Close,
}

const VIDEO_EXT: [&str; 8] = ["mkv", "mp4", "webm", "avi", "mov", "m4v", "ts", "ogv"];

/// Video files next to `media`, sorted by name: the local folder is the episode list.
pub fn siblings(media: &Path) -> Vec<PathBuf> {
    let dir = media.parent().filter(|d| !d.as_os_str().is_empty());
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir.unwrap_or(Path::new(".")))
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| VIDEO_EXT.contains(&e.to_ascii_lowercase().as_str()))
        })
        .collect();
    files.sort();
    if !files.iter().any(|f| f.file_name() == media.file_name()) {
        files = vec![media.to_path_buf()];
    }
    files
}

fn stem(path: &Path) -> String {
    path.file_stem()
        .map_or_else(String::new, |s| s.to_string_lossy().into_owned())
}

pub struct Controller<E: Engine> {
    pub engine: E,
    pub view: View,
    files: Vec<PathBuf>,
    /// Bumped when the episode rows change (the window remounts).
    pub generation: u64,
    last_input: Option<Instant>,
}

impl<E: Engine> Controller<E> {
    pub fn new(engine: E) -> Self {
        Self {
            engine,
            view: View::default(),
            files: Vec::new(),
            generation: 0,
            last_input: None,
        }
    }

    /// Opens `media`; its folder becomes the episode list.
    pub fn open(&mut self, media: &Path) {
        let files = siblings(media);
        let index = files
            .iter()
            .position(|f| f.file_name() == media.file_name())
            .unwrap_or(0);
        self.view.episodes = files
            .iter()
            .enumerate()
            .map(|(i, f)| Episode {
                number: format!("{:02}", i + 1),
                title: stem(f),
            })
            .collect();
        self.view.work = media
            .parent()
            .and_then(|d| d.file_name())
            .map_or_else(String::new, |s| s.to_string_lossy().into_owned());
        self.files = files;
        self.generation += 1;
        self.play(index);
    }

    fn play(&mut self, index: usize) {
        let Some(path) = self.files.get(index).cloned() else {
            return;
        };
        // Engines open paused on the first frame; viewing starts playing.
        let loaded = self
            .engine
            .load(&path.to_string_lossy(), 0.0)
            .and_then(|()| self.engine.command(&["set", "pause", "no"]));
        match loaded {
            Ok(()) => {
                self.view.current = Some(index);
                self.view.title = stem(&path);
                self.view.has_media = true;
                self.view.playing = true;
                self.view.position = 0.0;
                self.view.duration = 0.0;
                self.view.subtitle.clear();
                self.view.status.clear();
            }
            Err(e) => self.view.status = format!("재생할 수 없습니다: {}", e.message),
        }
    }

    fn report(&mut self, result: PortResult<()>) {
        if let Err(e) = result {
            self.view.status = format!("재생 명령 실패: {}", e.message);
        }
    }

    /// Applies one user command; returns what the window must do, if anything.
    pub fn apply(&mut self, command: Command) -> Option<Effect> {
        let media = self.view.has_media;
        match command {
            Command::Open => return Some(Effect::PickFile),
            Command::Browse => {
                self.view.status =
                    "작품 탐색은 소스 플러그인을 연결하면 사용할 수 있습니다.".into();
            }
            Command::More => {
                self.view.status = "추가 메뉴는 아직 준비 중입니다.".into();
            }
            Command::PlayPause if media => {
                let r = self.engine.command(&["cycle", "pause"]);
                self.report(r);
                self.view.playing = !self.view.playing;
            }
            // Like common players: past the first seconds (or on the first episode)
            // "previous" restarts the current episode; otherwise it opens the one before.
            Command::Previous if media => match self.view.current {
                Some(i) if i > 0 && self.view.position < RESTART_WITHIN => self.play(i - 1),
                _ => {
                    let r = self.engine.command(&["seek", "0", "absolute"]);
                    self.report(r);
                    self.view.position = 0.0;
                }
            },
            Command::Next if media => {
                if let Some(i) = self.view.current {
                    self.play(i + 1);
                }
            }
            Command::Episode(i) => self.play(i),
            Command::Mute => {
                self.view.muted = !self.view.muted;
                let value = if self.view.muted { "yes" } else { "no" };
                let r = self.engine.command(&["set", "mute", value]);
                self.report(r);
            }
            Command::Seek(fraction) if media && self.view.duration > 0.0 => {
                let secs = fraction * self.view.duration;
                let r = self
                    .engine
                    .command(&["seek", &format!("{secs:.3}"), "absolute"]);
                self.report(r);
                self.view.position = secs;
            }
            Command::Volume(v) => {
                self.view.volume = v;
                let r = self.engine.command(&["set", "volume", &format!("{v:.0}")]);
                self.report(r);
                // Raising the volume while muted unmutes, as in common players.
                if self.view.muted && v > 0.0 {
                    self.view.muted = false;
                    let r = self.engine.command(&["set", "mute", "no"]);
                    self.report(r);
                }
            }
            Command::ToggleEpisodes => self.view.panel_open = !self.view.panel_open,
            Command::ToggleFullscreen if media => {
                return Some(Effect::Fullscreen(!self.view.fullscreen));
            }
            Command::ExitFullscreen if self.view.fullscreen => {
                return Some(Effect::Fullscreen(false));
            }
            Command::Minimize => return Some(Effect::Minimize),
            Command::Maximize => return Some(Effect::Maximize(!self.view.maximized)),
            Command::Close => return Some(Effect::Close),
            _ => {}
        }
        None
    }

    /// Reads time and duration from the engine; true when the view changed.
    pub fn poll(&mut self) -> bool {
        if !self.view.has_media {
            return false;
        }
        let before = (self.view.position, self.view.duration, self.view.playing);
        let subtitle = self.engine.text("sub-text").unwrap_or_default();
        let subtitle_changed = subtitle != self.view.subtitle;
        self.view.subtitle = subtitle;
        if let Some(p) = self.engine.property("time-pos") {
            self.view.position = p.max(0.0);
        }
        if let Some(d) = self.engine.property("duration") {
            self.view.duration = d.max(0.0);
        }
        if self.engine.property("eof-reached") == Some(1.0) {
            self.view.playing = false;
        }
        // Whole seconds are what the clock shows; finer changes need no re-projection.
        before.0.floor() != self.view.position.floor()
            || before.1 != self.view.duration
            || before.2 != self.view.playing
            || subtitle_changed
    }

    /// Pointer or key input at `now`: controls come back. True when the view changed.
    pub fn input(&mut self, now: Instant) -> bool {
        self.last_input = Some(now);
        let changed = !self.view.controls_visible;
        self.view.controls_visible = true;
        changed
    }

    /// VIEW-10 auto-hide: only the fullscreen overlay while playing hides; paused, an
    /// error/status, or an open panel keep it. True when the view changed.
    pub fn idle(&mut self, now: Instant) -> bool {
        let v = &self.view;
        let may_hide =
            v.fullscreen && v.has_media && v.playing && !v.panel_open && v.status.is_empty();
        let quiet = self
            .last_input
            .is_none_or(|t| now.saturating_duration_since(t) >= HIDE_AFTER);
        let visible = !(may_hide && quiet);
        if !may_hide {
            // Restart the clock so controls stay for a full period after e.g. resuming.
            self.last_input = Some(now);
        }
        let changed = visible != self.view.controls_visible;
        self.view.controls_visible = visible;
        changed
    }
}

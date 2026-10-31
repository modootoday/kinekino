//! Player behaviour without a window: commands → engine calls and view changes, engine
//! polling → view. The window host only forwards `Effect`s it cannot do itself.
use crate::catalog::{BROWSE_ROWS, Catalog, STREAM_HEADERS};
pub use crate::port::Engine;
use crate::session::Command;
use crate::source::{Episode, View};
use kinekino_core::port::error::PortResult;
use kinekino_core::source::{SourceError, SourceItem, SourceItemKind};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// VIEW-10: controls hide after this long without input while playing (fullscreen overlay).
pub const HIDE_AFTER: Duration = Duration::from_millis(2500);

/// "Previous" within this many seconds of the start goes to the previous episode.
pub const RESTART_WITHIN: f64 = 3.0;
/// A dialog answered faster than this was never on screen.
pub const NO_DIALOG_WITHIN: Duration = Duration::from_millis(400);
/// A notice over the playing video disappears after this long.
pub const NOTICE_FOR: Duration = Duration::from_secs(4);
const NO_SOURCES: &str = "설치된 소스 플러그인이 없습니다. 플러그인 폴더에 소스 플러그인을 설치하면 작품을 탐색할 수 있습니다.";
const BROWSE_TITLE: &str = "작품 탐색";
const NO_DIALOG: &str = "파일 선택 창을 열 수 없습니다. xdg-desktop-portal 또는 zenity를 설치하거나 `kinekino watch <파일>`로 여세요.";

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

/// One entry of the play queue: a local file or a source plugin's playable item (resolved to a
/// stream when it is played).
#[derive(Debug, Clone)]
enum Media {
    Local(PathBuf),
    Remote(SourceItem),
}

pub struct Controller<E: Engine> {
    pub engine: E,
    pub view: View,
    files: Vec<Media>,
    /// A stream set request headers that the next local file must clear.
    headers_set: bool,
    /// Bumped when the episode rows change (the window remounts).
    pub generation: u64,
    last_input: Option<Instant>,
    notice_since: Option<Instant>,
    catalog: Option<Box<dyn Catalog>>,
    /// Opened listings, outermost first, with the page shown.
    trail: Vec<(SourceItem, u32)>,
    /// Rows currently shown (sources at the top level).
    listing: Vec<SourceItem>,
    next_page: Option<u32>,
}

fn browse_error(e: &SourceError) -> String {
    match e {
        SourceError::Unsupported(_) => "이 소스는 이 목록을 지원하지 않습니다.".into(),
        SourceError::NotFound => "항목을 찾을 수 없습니다.".into(),
        SourceError::Unavailable(m) | SourceError::Invalid(m) => {
            format!("소스에서 목록을 받지 못했습니다: {m}")
        }
    }
}

/// Engine source, shown title and allowed request headers of a queue entry.
type Located = (String, String, Vec<(String, String)>);

impl<E: Engine> Controller<E> {
    pub fn new(engine: E) -> Self {
        Self {
            engine,
            view: View::default(),
            files: Vec::new(),
            headers_set: false,
            generation: 0,
            last_input: None,
            notice_since: None,
            catalog: None,
            trail: Vec::new(),
            listing: Vec::new(),
            next_page: None,
        }
    }

    /// Source plugins behind 작품 탐색.
    pub fn with_catalog(mut self, catalog: Box<dyn Catalog>) -> Self {
        self.catalog = Some(catalog);
        self
    }

    fn show(&mut self, heading: String, items: Vec<SourceItem>, next: Option<u32>) {
        self.view.browsing = true;
        self.view.browse_heading = heading;
        self.view.browse_rows = items
            .iter()
            .take(BROWSE_ROWS)
            .map(|i| i.title.clone())
            .collect();
        self.view.browse_more = next.is_some();
        self.view.browse_status.clear();
        self.listing = items;
        self.next_page = next;
    }

    fn show_sources(&mut self) {
        let sources = self
            .catalog
            .as_ref()
            .map(|c| c.sources())
            .unwrap_or_default();
        if sources.is_empty() {
            self.view.browsing = false;
            self.view.status = NO_SOURCES.into();
            return;
        }
        self.trail.clear();
        self.show(BROWSE_TITLE.into(), sources, None);
    }

    /// Lists `page` under `item`; false (with the reason shown) when the source refused.
    fn list(&mut self, item: &SourceItem, page: u32) -> bool {
        let Some(catalog) = &self.catalog else {
            return false;
        };
        match catalog.browse(&item.reference, page) {
            Ok(p) => {
                self.show(item.title.clone(), p.items, p.next_page);
                true
            }
            Err(e) => {
                self.view.browse_status = browse_error(&e);
                false
            }
        }
    }

    fn browse(&mut self, command: &Command) {
        match *command {
            Command::BrowseItem(i) => {
                let Some(item) = self.listing.get(i).cloned() else {
                    return;
                };
                if item.kind == SourceItemKind::Playable {
                    self.open_playlist(i);
                    return;
                }
                if self.list(&item, 1) {
                    self.trail.push((item, 1));
                }
            }
            Command::BrowseMore => {
                let (Some(next), Some((item, _))) = (self.next_page, self.trail.last().cloned())
                else {
                    return;
                };
                if self.list(&item, next)
                    && let Some(top) = self.trail.last_mut()
                {
                    top.1 = next;
                }
            }
            Command::BrowseBack => {
                if self.trail.pop().is_none() {
                    self.view.browsing = false;
                    return;
                }
                match self.trail.last().cloned() {
                    Some((item, page)) => {
                        self.list(&item, page);
                    }
                    None => self.show_sources(),
                }
            }
            _ => {}
        }
    }

    /// Opens `media`; its folder becomes the episode list.
    /// The file dialog's answer. `None` that came back faster than a person can cancel
    /// means no dialog was shown (no xdg-desktop-portal file chooser and no zenity on
    /// this desktop), so the start screen says how else to open a file.
    /// Returns whether the view changed.
    pub fn picked(&mut self, media: Option<&Path>, took: Duration) -> bool {
        match media {
            Some(path) => self.open(path),
            // The start screen has a status line; over a playing video it is a toast.
            None if took < NO_DIALOG_WITHIN && self.view.has_media => {
                self.view.notice = NO_DIALOG.into();
                self.notice_since = Some(Instant::now());
            }
            None if took < NO_DIALOG_WITHIN => {
                self.view.status = NO_DIALOG.into();
            }
            None => return false,
        }
        true
    }

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
        self.files = files.into_iter().map(Media::Local).collect();
        self.generation += 1;
        self.play(index);
    }

    /// Plays row `row` of the shown listing: the listing's playables become the queue,
    /// titled after the container the plugin listed them under.
    fn open_playlist(&mut self, row: usize) {
        let Some(chosen) = self.listing.get(row).cloned() else {
            return;
        };
        let queue: Vec<SourceItem> = self
            .listing
            .iter()
            .filter(|i| i.kind == SourceItemKind::Playable)
            .cloned()
            .collect();
        let index = queue.iter().position(|i| *i == chosen).unwrap_or(0);
        self.view.episodes = queue
            .iter()
            .enumerate()
            .map(|(i, e)| Episode {
                number: format!("{:02}", i + 1),
                title: e.title.clone(),
            })
            .collect();
        self.view.work = self
            .trail
            .last()
            .map_or_else(String::new, |(s, _)| s.title.clone());
        self.files = queue.into_iter().map(Media::Remote).collect();
        self.generation += 1;
        self.play(index);
    }

    /// Engine source and request headers for a queue entry.
    fn locate(&self, media: &Media) -> Result<Located, String> {
        match media {
            Media::Local(path) => Ok((path.to_string_lossy().into_owned(), stem(path), vec![])),
            Media::Remote(item) => {
                let catalog = self.catalog.as_ref().ok_or("소스 플러그인이 없습니다.")?;
                let streams = catalog
                    .resolve(&item.reference)
                    .map_err(|e| browse_error(&e))?;
                let stream = streams
                    .into_iter()
                    .next()
                    .ok_or("소스가 재생할 스트림을 주지 않았습니다.")?;
                let headers = stream
                    .headers
                    .into_iter()
                    .filter(|(k, v)| {
                        STREAM_HEADERS.iter().any(|h| h.eq_ignore_ascii_case(k))
                            && !v.contains(['\r', '\n', ','])
                    })
                    .collect();
                Ok((stream.url, item.title.clone(), headers))
            }
        }
    }

    /// mpv request headers for the next load; local files clear what a stream set.
    fn set_headers(&mut self, remote: bool, headers: &[(String, String)]) -> PortResult<()> {
        if !remote && !self.headers_set {
            return Ok(());
        }
        self.headers_set = remote;
        let field = |name: &str| {
            headers
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(name))
                .map(|(_, v)| v.as_str())
        };
        let referrer = field("Referer").unwrap_or("");
        let agent = field("User-Agent").unwrap_or("libmpv");
        let origin = field("Origin").map_or_else(String::new, |o| format!("Origin: {o}"));
        self.engine.command(&["set", "referrer", referrer])?;
        self.engine.command(&["set", "user-agent", agent])?;
        self.engine.command(&["set", "http-header-fields", &origin])
    }

    fn play(&mut self, index: usize) {
        let Some(media) = self.files.get(index).cloned() else {
            return;
        };
        let (source, title, headers) = match self.locate(&media) {
            Ok(located) => located,
            Err(why) => return self.cannot_play(&why),
        };
        // Engines open paused on the first frame; viewing starts playing.
        let loaded = self
            .set_headers(matches!(media, Media::Remote(_)), &headers)
            .and_then(|()| self.engine.load(&source, 0.0))
            .and_then(|()| self.engine.command(&["set", "pause", "no"]));
        match loaded {
            Ok(()) => {
                self.view.current = Some(index);
                self.view.title = title;
                self.view.has_media = true;
                self.view.browsing = false;
                self.view.playing = true;
                self.view.position = 0.0;
                self.view.duration = 0.0;
                self.view.subtitle.clear();
                self.view.status.clear();
            }
            Err(e) => self.cannot_play(&e.message),
        }
    }

    /// The reason shows where the user is: the browse view, over the video, or the start
    /// screen.
    fn cannot_play(&mut self, why: &str) {
        let message = format!("재생할 수 없습니다: {why}");
        if self.view.browsing {
            self.view.browse_status = message;
        } else if self.view.has_media {
            self.view.notice = message;
            self.notice_since = Some(Instant::now());
        } else {
            self.view.status = message;
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
        // Esc closes an open ⋮ menu first; only the next Esc leaves fullscreen.
        if matches!(command, Command::ExitFullscreen) && self.view.menu_open {
            self.view.menu_open = false;
            return None;
        }
        // Any other command (a menu item, Esc, a click elsewhere) closes the ⋮ menu.
        if !matches!(command, Command::More) {
            self.view.menu_open = false;
        }
        match command {
            Command::Open => return Some(Effect::PickFile),
            Command::Browse => self.show_sources(),
            Command::BrowseItem(_) | Command::BrowseMore | Command::BrowseBack => {
                self.browse(&command)
            }
            // Esc in the browse view goes one level back (windowed).
            Command::ExitFullscreen if self.view.browsing && !self.view.fullscreen => {
                self.browse(&Command::BrowseBack)
            }
            Command::More => {
                self.view.menu_open = !self.view.menu_open;
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
        let may_hide = v.fullscreen
            && v.has_media
            && v.playing
            && !v.panel_open
            && !v.menu_open
            && v.status.is_empty();
        let quiet = self
            .last_input
            .is_none_or(|t| now.saturating_duration_since(t) >= HIDE_AFTER);
        let visible = !(may_hide && quiet);
        if !may_hide {
            // Restart the clock so controls stay for a full period after e.g. resuming.
            self.last_input = Some(now);
        }
        let expired = self
            .notice_since
            .is_some_and(|t| now.saturating_duration_since(t) >= NOTICE_FOR);
        if expired {
            self.notice_since = None;
            self.view.notice.clear();
        }
        let changed = visible != self.view.controls_visible;
        self.view.controls_visible = visible;
        changed || expired
    }
}

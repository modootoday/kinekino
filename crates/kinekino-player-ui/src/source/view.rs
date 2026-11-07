//! View model and Meari props for the player window.
use crate::catalog::BROWSE_ROWS;
use crate::session::clock;
use std::collections::BTreeMap;

/// Product name shown in the title bar and on the start screen (mockup names are sample data).
pub const PRODUCT: &str = "kinekino";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaylistItem {
    pub number: String,
    pub title: String,
}

/// Everything the window shows. Times are seconds; volume is 0–100.
#[derive(Debug, Clone, PartialEq)]
pub struct View {
    pub title: String,
    pub work: String,
    pub playlist: Vec<PlaylistItem>,
    pub current: Option<usize>,
    pub has_media: bool,
    pub playing: bool,
    pub fullscreen: bool,
    pub maximized: bool,
    pub panel_open: bool,
    /// The ⋮ (더 보기) menu above the transport bar.
    pub menu_open: bool,
    /// The right-click menu's top-left corner (document CSS px) while it is open.
    pub context_menu: Option<(f64, f64)>,
    pub controls_visible: bool,
    pub position: f64,
    pub duration: f64,
    pub volume: f64,
    pub muted: bool,
    /// Playback speed, 1.0 is normal.
    pub speed: f64,
    pub status: String,
    /// An item is opening (or its plugin is looking it up); a status line says so.
    pub opening: bool,
    /// A short notice over the video (bottom toast) while playing; cleared after a while.
    pub notice: String,
    /// Current subtitle cue from the engine (`sub-text`), drawn over the video.
    pub subtitle: String,
    /// 작품 탐색 view over the start screen.
    pub browsing: bool,
    pub browse_heading: String,
    /// At most `BROWSE_ROWS` titles of the current listing.
    pub browse_rows: Vec<String>,
    /// Bumped when a new listing replaces the rows (not on 더 보기), so the window
    /// scrolls the list back to its top.
    pub browse_listing: u64,
    pub browse_more: bool,
    pub browse_status: String,
    /// Which mascot state the status line shows (design: mascot-preset-reference app-state
    /// combos): "" none, "loading", "error", "idle".
    pub browse_mood: &'static str,
    /// The search field (shown inside an open source).
    pub browse_search: bool,
    /// Rows drawn as a grid of tiles instead of a list (kept across listings).
    pub browse_grid: bool,
    /// A connected plugin offers browsing; without one, 작품 탐색 is not shown.
    pub has_browse_plugin: bool,
    /// At the end of an item, play the next one in the queue. Off until the user turns it
    /// on (field rules: auto next needs an explicit setting).
    pub auto_next: bool,
    pub skip_offer: bool, // Inside the marked skip span (controller/skip.rs).
}

impl Default for View {
    fn default() -> Self {
        Self {
            title: String::new(),
            work: String::new(),
            playlist: Vec::new(),
            current: None,
            has_media: false,
            playing: false,
            fullscreen: false,
            maximized: false,
            panel_open: false,
            menu_open: false,
            context_menu: None,
            controls_visible: true,
            position: 0.0,
            duration: 0.0,
            volume: 100.0,
            muted: false,
            speed: 1.0,
            status: String::new(),
            opening: false,
            notice: String::new(),
            subtitle: String::new(),
            browsing: false,
            browse_heading: String::new(),
            browse_rows: Vec::new(),
            browse_listing: 0,
            browse_more: false,
            browse_status: String::new(),
            browse_mood: "",
            browse_search: false,
            browse_grid: false,
            has_browse_plugin: false,
            auto_next: false,
            skip_offer: false,
        }
    }
}

/// Fixed buttons whose click counters the host watches (episode rows add `ep{i}`).
pub const BUTTONS: [&str; 45] = [
    "open",
    "browseSearch",
    "browse",
    "previous",
    "playPause",
    "next",
    "mute",
    "episodes",
    "fullscreen",
    "more",
    "minimize",
    "maximize",
    "close",
    "panelClose",
    "menuOpen",
    "menuEpisodes",
    "menuFullscreen",
    "browseBack",
    "browseMore",
    "browseLayout",
    "skipSegment",
    "clearSkip",
    "ctxOpen",
    "ctxBrowse",
    "ctxPlayPause",
    "ctxPrevious",
    "ctxNext",
    "ctxAutoNext",
    "ctxSpeedDown",
    "ctxSpeedUp",
    "ctxSpeedReset",
    "ctxSeekBack5",
    "ctxSeekFwd5",
    "ctxSeekBack30",
    "ctxSeekFwd30",
    "ctxSubtitle",
    "ctxAudio",
    "ctxVolumeUp",
    "ctxVolumeDown",
    "ctxMute",
    "ctxMarkSkipStart",
    "ctxMarkSkipEnd",
    "ctxSkipSegment",
    "ctxClearSkip",
    "ctxFullscreen",
];

#[derive(Debug, Clone, PartialEq)]
pub enum Prop {
    Text(String),
    Bool(bool),
}

pub fn props(view: &View) -> BTreeMap<String, Prop> {
    let t = |s: &str| Prop::Text(s.to_owned());
    let permille = if view.duration > 0.0 {
        (view.position / view.duration * 1000.0).clamp(0.0, 1000.0)
    } else {
        0.0
    };
    let context = if view.has_media {
        view.title.clone()
    } else {
        String::new()
    };
    let lines: Vec<String> = view
        .subtitle
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_owned)
        .collect();
    let mut out = BTreeMap::from([
        ("context".to_owned(), Prop::Text(context)),
        ("title".to_owned(), Prop::Text(view.title.clone())),
        ("work".to_owned(), Prop::Text(view.work.clone())),
        ("status".to_owned(), Prop::Text(view.status.clone())),
        (
            "statusClass".to_owned(),
            Prop::Text(mood_class(if view.opening { "loading" } else { "" })),
        ),
        ("notice".to_owned(), Prop::Text(view.notice.clone())),
        (
            "noNotice".to_owned(),
            Prop::Bool(!view.has_media || view.notice.is_empty()),
        ),
        (
            "noSkipOffer".to_owned(),
            Prop::Bool(!view.has_media || !view.skip_offer),
        ),
        ("hasMedia".to_owned(), Prop::Bool(view.has_media)),
        (
            "hideStart".to_owned(),
            Prop::Bool(view.has_media || view.browsing),
        ),
        ("noBrowse".to_owned(), Prop::Bool(!view.browsing)),
        (
            "noBrowsePlugin".to_owned(),
            Prop::Bool(!view.has_browse_plugin),
        ),
        // Kept laid out (visibility, not display) so the search field always has its
        // text editor: Meari creates it at layout and refuses an unlaid controlled input.
        (
            "browseClass".to_owned(),
            Prop::Text(if view.browsing {
                "aw-viewing-start kk-browse".into()
            } else {
                "aw-viewing-start kk-browse kk-off".into()
            }),
        ),
        (
            "browseHeading".to_owned(),
            Prop::Text(view.browse_heading.clone()),
        ),
        (
            "browseStatus".to_owned(),
            Prop::Text(view.browse_status.clone()),
        ),
        (
            "browseStatusClass".to_owned(),
            Prop::Text(mood_class(view.browse_mood)),
        ),
        (
            "noBrowseStatus".to_owned(),
            Prop::Bool(view.browse_status.is_empty()),
        ),
        ("noBrowseMore".to_owned(), Prop::Bool(!view.browse_more)),
        ("noBrowseSearch".to_owned(), Prop::Bool(!view.browse_search)),
        (
            "searchClass".to_owned(),
            Prop::Text(if view.browse_search {
                "kk-browse-search".into()
            } else {
                "kk-browse-search kk-collapsed".into()
            }),
        ),
        (
            "rowsClass".to_owned(),
            t(if view.browse_grid {
                "kk-grid"
            } else {
                "kk-list"
            }),
        ),
        (
            "layoutIcon".to_owned(),
            t(if view.browse_grid {
                "aw-icon aw-icon-episodes"
            } else {
                "aw-icon aw-icon-cards"
            }),
        ),
        (
            "layoutLabel".to_owned(),
            t(if view.browse_grid {
                "목록으로 보기"
            } else {
                "격자로 보기"
            }),
        ),
        ("noMedia".to_owned(), Prop::Bool(!view.has_media)),
        ("fullscreen".to_owned(), Prop::Bool(view.fullscreen)),
        ("panelOpen".to_owned(), Prop::Bool(view.panel_open)),
        ("noMenu".to_owned(), Prop::Bool(!view.menu_open)),
        (
            "noContextMenu".to_owned(),
            Prop::Bool(view.context_menu.is_none()),
        ),
        (
            "contextMenuStyle".to_owned(),
            t(&view
                .context_menu
                .map(|(x, y)| format!("left: {x:.0}px; top: {y:.0}px;"))
                .unwrap_or_default()),
        ),
        ("noWork".to_owned(), Prop::Bool(view.work.is_empty())),
        (
            "menuPanelLabel".to_owned(),
            Prop::Text(
                if view.panel_open {
                    "회차 목록 닫기"
                } else {
                    "회차 목록 열기"
                }
                .into(),
            ),
        ),
        (
            "subtitle1".to_owned(),
            Prop::Text(lines.first().cloned().unwrap_or_default()),
        ),
        (
            "subtitle2".to_owned(),
            Prop::Text(lines.get(1..).map(|l| l.join(" ")).unwrap_or_default()),
        ),
        ("noSubtitle2".to_owned(), Prop::Bool(lines.len() < 2)),
        (
            // OSD toast (#notice) shares the stage bottom centre with the text cue; hide
            // the cue while a notice is up so seek/volume/speed feedback stays readable.
            "noSubtitle".to_owned(),
            Prop::Bool(!view.has_media || lines.is_empty() || !view.notice.is_empty()),
        ),
        (
            "rootClass".to_owned(),
            // M1-V2: when the transport is shown, CSS lifts #subtitle/#notice above the
            // chrome with --aw-transport-height (skip-offer family). Hidden → no class.
            t(if view.controls_visible || !view.has_media {
                "kk-player-root aw-viewing-read-root aw-viewing-panel-closed kk-controls-up"
            } else {
                "kk-player-root aw-viewing-read-root aw-viewing-panel-closed"
            }),
        ),
        (
            "stageClass".to_owned(),
            t(if view.panel_open && !view.fullscreen {
                "aw-viewing-stage kk-docked"
            } else {
                "aw-viewing-stage"
            }),
        ),
        (
            "frameClass".to_owned(),
            t(if view.fullscreen {
                "aw-player-shell aw-player-fullscreen"
            } else {
                "aw-player-shell"
            }),
        ),
        (
            "transportClass".to_owned(),
            t(if view.controls_visible || !view.has_media {
                "aw-transport"
            } else {
                "aw-transport aw-controls-hidden"
            }),
        ),
        (
            "playIcon".to_owned(),
            t(if view.playing {
                "aw-icon aw-icon-pause"
            } else {
                "aw-icon aw-icon-play"
            }),
        ),
        (
            "playLabel".to_owned(),
            t(if view.playing {
                "일시 정지"
            } else {
                "재생"
            }),
        ),
        (
            "volumeIcon".to_owned(),
            t(if view.muted || view.volume <= 0.0 {
                "aw-icon aw-icon-volume-zero"
            } else {
                "aw-icon aw-icon-volume"
            }),
        ),
        (
            "maximizeIcon".to_owned(),
            t(if view.maximized {
                "aw-icon aw-icon-restore"
            } else {
                "aw-icon aw-icon-maximize"
            }),
        ),
        (
            "maximizeLabel".to_owned(),
            t(if view.maximized {
                "이전 크기로"
            } else {
                "최대화"
            }),
        ),
        (
            "fullscreenLabel".to_owned(),
            t(if view.fullscreen {
                "전체화면 종료"
            } else {
                "전체화면"
            }),
        ),
        ("position".to_owned(), Prop::Text(clock(view.position))),
        ("duration".to_owned(), Prop::Text(clock(view.duration))),
        (
            "seek".to_owned(),
            Prop::Text(format!("{}", permille.round() as u32)),
        ),
        (
            "volume".to_owned(),
            Prop::Text(format!("{}", view.volume.clamp(0.0, 100.0).round() as u32)),
        ),
    ]);
    for i in 0..BROWSE_ROWS {
        let row = view.browse_rows.get(i);
        out.insert(
            format!("brText{i}"),
            Prop::Text(row.cloned().unwrap_or_default()),
        );
        out.insert(format!("noBr{i}"), Prop::Bool(row.is_none()));
    }
    for i in 0..view.playlist.len() {
        let current = view.current == Some(i);
        out.insert(format!("cur{i}"), Prop::Bool(current));
        out.insert(
            format!("aria{i}"),
            t(if current { "true" } else { "false" }),
        );
    }
    out
}

/// The status line class; a mood shows the matching mascot above it.
fn mood_class(mood: &str) -> String {
    match mood {
        "" => "aw-connection-status".to_owned(),
        m => format!("aw-connection-status kk-mood-{m}"),
    }
}

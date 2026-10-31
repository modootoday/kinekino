//! Player view model → bounded Meari TSX source. Episode rows are fixed per mount; every
//! changing value is a typed prop, so playback ticks update props without a remount.
use crate::catalog::BROWSE_ROWS;
use std::collections::BTreeMap;

/// Product name shown in the title bar and on the start screen (mockup names are sample data).
pub const PRODUCT: &str = "kinekino";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Episode {
    pub number: String,
    pub title: String,
}

/// Everything the window shows. Times are seconds; volume is 0–100.
#[derive(Debug, Clone, PartialEq)]
pub struct View {
    pub title: String,
    pub work: String,
    pub episodes: Vec<Episode>,
    pub current: Option<usize>,
    pub has_media: bool,
    pub playing: bool,
    pub fullscreen: bool,
    pub maximized: bool,
    pub panel_open: bool,
    /// The ⋮ (더 보기) menu above the transport bar.
    pub menu_open: bool,
    pub controls_visible: bool,
    pub position: f64,
    pub duration: f64,
    pub volume: f64,
    pub muted: bool,
    pub status: String,
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
}

impl Default for View {
    fn default() -> Self {
        Self {
            title: String::new(),
            work: String::new(),
            episodes: Vec::new(),
            current: None,
            has_media: false,
            playing: false,
            fullscreen: false,
            maximized: false,
            panel_open: false,
            menu_open: false,
            controls_visible: true,
            position: 0.0,
            duration: 0.0,
            volume: 100.0,
            muted: false,
            status: String::new(),
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
        }
    }
}

/// Fixed buttons whose click counters the host watches (episode rows add `ep{i}`).
pub const BUTTONS: [&str; 20] = [
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
];

/// `m:ss`, or `h:mm:ss` from one hour.
pub fn clock(seconds: f64) -> String {
    let s = if seconds.is_finite() && seconds > 0.0 {
        seconds as u64
    } else {
        0
    };
    let (h, m, s) = (s / 3600, s / 60 % 60, s % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

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
        ("notice".to_owned(), Prop::Text(view.notice.clone())),
        (
            "noNotice".to_owned(),
            Prop::Bool(!view.has_media || view.notice.is_empty()),
        ),
        ("hasMedia".to_owned(), Prop::Bool(view.has_media)),
        (
            "hideStart".to_owned(),
            Prop::Bool(view.has_media || view.browsing),
        ),
        ("noBrowse".to_owned(), Prop::Bool(!view.browsing)),
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
            Prop::Text(if view.browse_mood.is_empty() {
                "aw-connection-status".to_owned()
            } else {
                format!("aw-connection-status kk-mood-{}", view.browse_mood)
            }),
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
            "menuPanelLabel".to_owned(),
            Prop::Text(
                if view.panel_open {
                    "재생목록 닫기"
                } else {
                    "재생목록 열기"
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
            "noSubtitle".to_owned(),
            Prop::Bool(!view.has_media || lines.is_empty()),
        ),
        (
            "rootClass".to_owned(),
            t(if view.panel_open && !view.fullscreen {
                "kk-player-root aw-viewing-read-root"
            } else {
                "kk-player-root aw-viewing-read-root aw-viewing-panel-closed"
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
    for i in 0..view.episodes.len() {
        let current = view.current == Some(i);
        out.insert(format!("cur{i}"), Prop::Bool(current));
        out.insert(
            format!("aria{i}"),
            t(if current { "true" } else { "false" }),
        );
    }
    out
}

fn quote(text: &str) -> String {
    format!("{text:?}")
}

/// Escape text for a JSX text child.
fn jsx_text(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '{' | '}' | '<' | '>' | '&' => format!("{{{}}}", quote(&c.to_string())),
            c => c.to_string(),
        })
        .collect()
}

fn icon_button(id: &str, label: &str, icon: &str, extra: &str) -> String {
    format!(
        "<button id=\"{id}\" className=\"aw-icon-button\" aria-label={label}{extra} onClick={{() => set{id}Count(p => p + 1)}}><span className={icon} aria-hidden=\"true\"></span></button>"
    )
}

/// TSX for a window with `episodes` fixed rows.
pub fn build(episodes: &[Episode]) -> String {
    let mut types = String::from(
        "context: string; title: string; work: string; status: string; notice: string; noNotice: boolean; hasMedia: boolean; noMedia: boolean; hideStart: boolean; noBrowse: boolean; browseClass: string; browseHeading: string; browseStatus: string; browseStatusClass: string; noBrowseStatus: boolean; noBrowseMore: boolean; noBrowseSearch: boolean; searchClass: string; rowsClass: string; layoutIcon: string; layoutLabel: string; fullscreen: boolean; panelOpen: boolean; noMenu: boolean; menuPanelLabel: string; subtitle1: string; subtitle2: string; noSubtitle: boolean; noSubtitle2: boolean; rootClass: string; frameClass: string; transportClass: string; playIcon: string; playLabel: string; volumeIcon: string; maximizeIcon: string; maximizeLabel: string; fullscreenLabel: string; position: string; duration: string; seek: string; volume: string;",
    );
    let mut state = String::new();
    let mut rows = String::new();
    let mut browse_rows = String::new();
    for i in 0..BROWSE_ROWS {
        types.push_str(&format!(" brText{i}: string; noBr{i}: boolean;"));
        state.push_str(&format!(
            "  const [br{i}Count, setbr{i}Count] = useState(0);\n"
        ));
        browse_rows.push_str(&format!(
            "<button id=\"br{i}\" className=\"aw-button aw-episode-row\" hidden={{props.noBr{i}}} onClick={{() => setbr{i}Count(p => p + 1)}}><span className=\"aw-episode-row-title\">{{props.brText{i}}}</span></button>\n"
        ));
    }
    for button in BUTTONS {
        state.push_str(&format!(
            "  const [{button}Count, set{button}Count] = useState(0);\n"
        ));
    }
    for (i, e) in episodes.iter().enumerate() {
        types.push_str(&format!(" cur{i}: boolean; aria{i}: string;"));
        state.push_str(&format!(
            "  const [ep{i}Count, setep{i}Count] = useState(0);\n"
        ));
        rows.push_str(&format!(
            "<button id=\"ep{i}\" role=\"radio\" className=\"aw-button aw-episode-row\" aria-checked={{props.aria{i}}} aria-current={{props.aria{i}}} onClick={{() => setep{i}Count(p => p + 1)}}><span className=\"aw-episode-row-title\">{} · {}</span><span className=\"aw-episode-current-marker\" hidden={{!props.cur{i}}}><span className=\"aw-icon aw-icon-play\" aria-hidden=\"true\"></span>재생 중</span></button>\n",
            jsx_text(&e.number),
            jsx_text(&e.title)
        ));
    }
    let click = |id: &str| format!("onClick={{() => set{id}Count(p => p + 1)}}");
    let caption = |id: &str, label: &str, icon: &str, class: &str| {
        format!(
            "<button id=\"{id}\" className=\"{class}\" aria-label={label} {}><span className={icon} aria-hidden=\"true\"></span></button>",
            click(id)
        )
    };
    format!(
        r#"import {{ useState }} from "react";
type Props = {{ {types} }};
export function KinekinoPlayer(props: Props) {{
{state}  const [seekAt, setSeekAt] = useState("");
  const [volumeAt, setVolumeAt] = useState("");
  const [queryAt, setQueryAt] = useState("");
  return <div id="player" className={{props.rootClass}}>
  <main className={{props.frameClass}}>
    <div className="aw-viewing-chrome" hidden={{props.fullscreen}}>
      <div className="aw-window-chrome" role="banner">
        <div className="aw-window-brand"><span className="aw-icon aw-icon-brand" aria-hidden="true"></span><span className="aw-product-name">{PRODUCT}</span><span id="context" className="aw-window-context" title={{props.context}}>{{props.context}}</span></div>
        <div className="aw-window-captions" aria-label="창 제어">
          {min}
          {max}
          {close}
        </div>
      </div>
    </div>
    <div className="aw-viewing-stage">
      <video id="video" className="aw-viewing-video" tabIndex="-1" aria-hidden="true" hidden={{props.noMedia}}></video>
      <p id="subtitle" className="kk-subtitle" aria-live="polite" hidden={{props.noSubtitle}}><span className="kk-subtitle-line">{{props.subtitle1}}</span><span className="kk-subtitle-line" hidden={{props.noSubtitle2}}>{{props.subtitle2}}</span></p>
      <p id="notice" className="kk-toast" role="status" hidden={{props.noNotice}}><span className="kk-toast-text">{{props.notice}}</span></p>
      <section id="start" className="aw-viewing-start" aria-label="시청 시작" hidden={{props.hideStart}}>
        <div className="aw-start-content">
          <span className="aw-start-mark" aria-hidden="true"></span>
          <h1 className="aw-start-title">{PRODUCT}</h1>
          <div className="aw-start-actions">
            <button id="open" className="aw-button aw-button-outline-accent" {c_open}><span className="aw-icon aw-icon-folder" aria-hidden="true"></span>파일 열기</button>
            <button id="browse" className="aw-button" {c_browse}><span className="aw-icon aw-icon-search" aria-hidden="true"></span>작품 탐색</button>
          </div>
          <p className="aw-start-shortcut">Ctrl+O로 파일 열기</p>
          <p id="status" className="aw-connection-status" role="status">{{props.status}}</p>
        </div>
      </section>
      <section id="browseView" className={{props.browseClass}} aria-label="작품 탐색" aria-hidden={{props.noBrowse}}>
        <div className="aw-catalog-browser kk-browse-panel">
          <div className="aw-episode-heading"><h2>{{props.browseHeading}}</h2><div className="kk-heading-actions">{browse_layout}{browse_back}</div></div>
          <div className={{props.searchClass}} role="search"><input id="browseQuery" className="aw-text-input" type="text" disabled={{props.noBrowseSearch}} aria-label="검색어" placeholder="검색어" value={{queryAt}} onChange={{event => setQueryAt(event.currentTarget.value)}}/><button id="browseSearch" className="aw-button" disabled={{props.noBrowseSearch}} {c_browse_search}><span className="aw-icon aw-icon-search" aria-hidden="true"></span>검색</button></div>
          <p id="browseStatus" className={{props.browseStatusClass}} role="status" hidden={{props.noBrowseStatus}}>{{props.browseStatus}}</p>
          <div id="browseLists" className="aw-catalog-lists"><div id="browseRows" className={{props.rowsClass}} role="list" aria-label="목록">
{browse_rows}          </div></div>
          <button id="browseMore" className="aw-button" hidden={{props.noBrowseMore}} {c_browse_more}>더 보기</button>
        </div>
      </section>
    </div>
    <section id="transport" className={{props.transportClass}} aria-label="재생 제어">
      <div className="aw-seek-range"><input id="seek" className="aw-range-input" type="range" data-meari-range-parts="compact" min="0" max="1000" step="1" value={{props.seek}} aria-label="재생 위치" disabled={{props.noMedia}} onChange={{event => setSeekAt(event.currentTarget.value)}}/></div>
      <div className="aw-transport-row">
        <div className="aw-transport-group">
          {prev}
          {play}
          {next}
          <span id="time" className="aw-time-label">{{props.position}} / {{props.duration}}</span>
        </div>
        <div className="aw-transport-group">
          {mute}
          <div className="aw-volume-range"><input id="volume" className="aw-range-input" type="range" data-meari-range-parts="compact" min="0" max="100" step="1" value={{props.volume}} aria-label="음량" onChange={{event => setVolumeAt(event.currentTarget.value)}}/></div>
          <span className="aw-divider"></span>
          {episodes}
          {fullscreen}
          <div className="kk-more">
            {more}
            <div id="menu" className="kk-menu" role="menu" aria-label="더 보기" hidden={{props.noMenu}}>
              <button id="menuOpen" className="aw-button kk-menu-item" role="menuitem" {c_menu_open}><span className="aw-icon aw-icon-folder" aria-hidden="true"></span>파일 열기…<span className="kk-menu-key">Ctrl+O</span></button>
              <button id="menuEpisodes" className="aw-button kk-menu-item" role="menuitem" {c_menu_episodes}><span className="aw-icon aw-icon-episodes" aria-hidden="true"></span>{{props.menuPanelLabel}}</button>
              <button id="menuFullscreen" className="aw-button kk-menu-item" role="menuitem" {c_menu_fullscreen}><span className="aw-icon aw-icon-fullscreen" aria-hidden="true"></span>{{props.fullscreenLabel}}<span className="kk-menu-key">F</span></button>
            </div>
          </div>
        </div>
      </div>
    </section>
  </main>
  <aside id="panel" className="aw-episode-panel" aria-label="재생목록" hidden={{!props.panelOpen}}>
    <div className="aw-catalog-browser">
      <div className="aw-episode-heading"><h2>재생목록</h2>{panel_close}</div>
      <p className="aw-episode-title">{{props.work}}</p>
      <div className="aw-episode-body aw-catalog-lists"><div role="radiogroup" aria-label="재생 항목">
{rows}      </div></div>
    </div>
  </aside>
  </div>;
}}
"#,
        min = caption(
            "minimize",
            "\"최소화\"",
            "\"aw-icon aw-icon-minimize\"",
            "aw-caption"
        ),
        max = caption(
            "maximize",
            "{props.maximizeLabel}",
            "{props.maximizeIcon}",
            "aw-caption"
        ),
        close = caption(
            "close",
            "\"닫기\"",
            "\"aw-icon aw-icon-close\"",
            "aw-caption aw-caption-close"
        ),
        c_open = click("open"),
        c_menu_open = click("menuOpen"),
        c_menu_episodes = click("menuEpisodes"),
        c_menu_fullscreen = click("menuFullscreen"),
        c_browse = click("browse"),
        c_browse_more = click("browseMore"),
        c_browse_search = click("browseSearch"),
        browse_layout = icon_button(
            "browseLayout",
            "{props.layoutLabel}",
            "{props.layoutIcon}",
            ""
        ),
        browse_back = icon_button("browseBack", "\"뒤로\"", "\"aw-icon aw-icon-close\"", ""),
        prev = icon_button(
            "previous",
            "\"이전 항목\"",
            "\"aw-icon aw-icon-previous\"",
            " disabled={props.noMedia}"
        ),
        play = icon_button(
            "playPause",
            "{props.playLabel}",
            "{props.playIcon}",
            " disabled={props.noMedia}"
        ),
        next = icon_button(
            "next",
            "\"다음 항목\"",
            "\"aw-icon aw-icon-next\"",
            " disabled={props.noMedia}"
        ),
        mute = icon_button("mute", "\"음소거\"", "{props.volumeIcon}", ""),
        episodes = icon_button(
            "episodes",
            "\"재생목록\"",
            "\"aw-icon aw-icon-episodes\"",
            ""
        ),
        fullscreen = icon_button(
            "fullscreen",
            "{props.fullscreenLabel}",
            "\"aw-icon aw-icon-fullscreen\"",
            ""
        ),
        more = icon_button("more", "\"더 보기\"", "\"aw-icon aw-icon-more\"", ""),
        panel_close = icon_button(
            "panelClose",
            "\"재생목록 닫기\"",
            "\"aw-icon aw-icon-close\"",
            ""
        ),
    )
}

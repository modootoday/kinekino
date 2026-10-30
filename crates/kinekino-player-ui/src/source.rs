//! Player view model → bounded Meari TSX source. Episode rows are fixed per mount; every
//! changing value is a typed prop, so playback ticks update props without a remount.
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
    /// Current subtitle cue from the engine (`sub-text`), drawn over the video.
    pub subtitle: String,
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
            subtitle: String::new(),
        }
    }
}

/// Fixed buttons whose click counters the host watches (episode rows add `ep{i}`).
pub const BUTTONS: [&str; 16] = [
    "open",
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
        ("hasMedia".to_owned(), Prop::Bool(view.has_media)),
        ("noMedia".to_owned(), Prop::Bool(!view.has_media)),
        ("fullscreen".to_owned(), Prop::Bool(view.fullscreen)),
        ("panelOpen".to_owned(), Prop::Bool(view.panel_open)),
        ("noMenu".to_owned(), Prop::Bool(!view.menu_open)),
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
        "context: string; title: string; work: string; status: string; hasMedia: boolean; noMedia: boolean; fullscreen: boolean; panelOpen: boolean; noMenu: boolean; menuPanelLabel: string; subtitle1: string; subtitle2: string; noSubtitle: boolean; noSubtitle2: boolean; rootClass: string; frameClass: string; transportClass: string; playIcon: string; playLabel: string; volumeIcon: string; maximizeIcon: string; maximizeLabel: string; fullscreenLabel: string; position: string; duration: string; seek: string; volume: string;",
    );
    let mut state = String::new();
    let mut rows = String::new();
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
      <section id="start" className="aw-viewing-start" aria-label="시청 시작" hidden={{props.hasMedia}}>
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
  <aside id="panel" className="aw-episode-panel" aria-label="회차 목록" hidden={{!props.panelOpen}}>
    <div className="aw-catalog-browser">
      <div className="aw-episode-heading"><h2>회차 목록</h2>{panel_close}</div>
      <p className="aw-episode-title">{{props.work}}</p>
      <div className="aw-episode-body aw-catalog-lists"><div role="radiogroup" aria-label="회차">
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
        prev = icon_button(
            "previous",
            "\"이전화\"",
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
            "\"다음화\"",
            "\"aw-icon aw-icon-next\"",
            " disabled={props.noMedia}"
        ),
        mute = icon_button("mute", "\"음소거\"", "{props.volumeIcon}", ""),
        episodes = icon_button(
            "episodes",
            "\"회차 목록\"",
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
            "\"회차 목록 닫기\"",
            "\"aw-icon aw-icon-close\"",
            ""
        ),
    )
}

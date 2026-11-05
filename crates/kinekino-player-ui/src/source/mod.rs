//! Player view model → bounded Meari TSX source. Playlist rows are fixed per mount; every
//! changing value is a typed prop, so playback ticks update props without a remount.
mod view;

pub use view::{BUTTONS, PRODUCT, PlaylistItem, Prop, View, props};

use crate::catalog::BROWSE_ROWS;

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
pub fn build(episodes: &[PlaylistItem]) -> String {
    let mut types = String::from(
        "context: string; title: string; work: string; status: string; statusClass: string; notice: string; noNotice: boolean; noSkipOffer: boolean; hasMedia: boolean; noMedia: boolean; hideStart: boolean; noBrowse: boolean; noBrowsePlugin: boolean; browseClass: string; browseHeading: string; browseStatus: string; browseStatusClass: string; noBrowseStatus: boolean; noBrowseMore: boolean; noBrowseSearch: boolean; searchClass: string; rowsClass: string; layoutIcon: string; layoutLabel: string; fullscreen: boolean; panelOpen: boolean; noMenu: boolean; menuPanelLabel: string; subtitle1: string; subtitle2: string; noSubtitle: boolean; noSubtitle2: boolean; rootClass: string; stageClass: string; frameClass: string; transportClass: string; playIcon: string; playLabel: string; volumeIcon: string; maximizeIcon: string; maximizeLabel: string; fullscreenLabel: string; position: string; duration: string; seek: string; volume: string;",
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
            "<button id=\"ep{i}\" role=\"radio\" className=\"aw-button aw-episode-row kk-pl-row\" aria-checked={{props.aria{i}}} aria-current={{props.aria{i}}} onClick={{() => setep{i}Count(p => p + 1)}}><span className=\"kk-pl-mark\"><span className=\"aw-icon aw-icon-play\" aria-hidden=\"true\" hidden={{!props.cur{i}}}></span></span><span className=\"kk-pl-num\">{}</span><span className=\"aw-episode-row-title\">{}</span></button>\n",
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
    <div className={{props.stageClass}}>
      <video id="video" className="aw-viewing-video" tabIndex="-1" aria-hidden="true" hidden={{props.noMedia}}></video>
      <p id="subtitle" className="kk-subtitle" aria-live="polite" hidden={{props.noSubtitle}}><span className="kk-subtitle-line">{{props.subtitle1}}</span><span className="kk-subtitle-line" hidden={{props.noSubtitle2}}>{{props.subtitle2}}</span></p>
      <p id="notice" className="kk-toast" role="status" hidden={{props.noNotice}}><span className="kk-toast-text">{{props.notice}}</span></p>
      <div id="skipOffer" className="kk-skip-offer" hidden={{props.noSkipOffer}}>
        <button id="skipSegment" className="aw-button" aria-label="건너뛰기" {c_skip}><span className="aw-icon aw-icon-next" aria-hidden="true"></span>건너뛰기</button>
        <button id="clearSkip" className="aw-button aw-button-outline-accent" aria-label="구간 지우기" {c_clear_skip}>구간 지우기</button>
      </div>
      <section id="start" className="aw-viewing-start" aria-label="시청 시작" hidden={{props.hideStart}}>
        <div className="aw-start-content">
          <span className="aw-start-mark" aria-hidden="true"></span>
          <h1 className="aw-start-title">{PRODUCT}</h1>
          <div className="aw-start-actions">
            <button id="open" className="aw-button aw-button-outline-accent" {c_open}><span className="aw-icon aw-icon-folder" aria-hidden="true"></span>파일 열기</button>
            <button id="browse" className="aw-button" hidden={{props.noBrowsePlugin}} {c_browse}><span className="aw-icon aw-icon-search" aria-hidden="true"></span>작품 탐색</button>
          </div>
          <p className="aw-start-shortcut">Ctrl+O로 파일 열기</p>
          <p id="status" className={{props.statusClass}} role="status">{{props.status}}</p>
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
      <aside id="panel" className="aw-episode-panel kk-playlist" aria-label="재생목록" hidden={{!props.panelOpen}}>
        <div className="aw-catalog-browser">
          <div className="aw-episode-heading"><h2>재생목록</h2>{panel_close}</div>
          <p className="aw-episode-title">{{props.work}}</p>
          <div className="aw-episode-body aw-catalog-lists"><div role="radiogroup" aria-label="재생 항목">
{rows}          </div></div>
        </div>
      </aside>
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
          <span className="aw-divider"></span>
          <span className="kk-engine-label" title="재생 엔진">네이티브</span>
        </div>
      </div>
    </section>
  </main>
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
        c_skip = click("skipSegment"),
        c_clear_skip = click("clearSkip"),
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

//! Studio (제작) design-review mockups: one shared frame (title bar, left navigation,
//! centre workspace, right settings, bottom job queue) and every studio draft drawn as a
//! fixed document. The host has none of these tools yet; nothing here runs a job.
use kinekino_player_ui::Mockup;

macro_rules! icons {
    ($($n:literal)*) => {
        &[$((concat!("assets/design/icons/", $n, ".svg"),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/design/icons/", $n, ".svg")).as_slice()),)*]
    };
}
const ICONS: &[(&str, &[u8])] = icons!("back" "segments" "subtitles" "subject" "clip" "image" "cards"
    "layers" "cube" "export" "model" "list" "play" "pause" "previous" "next" "volume" "fullscreen"
    "chevron-down" "plus" "minus" "delete" "undo" "redo" "merge" "info" "warning" "search" "folder"
    "eye" "lock" "grip" "microphone" "scan-text" "text-document" "music" "track-video"
    "track-subtitle" "track-audio" "selection-pointer" "brush-add" "eraser" "mask-exclude" "hand"
    "zoom-in" "move-view" "refresh" "file-image" "file-mask" "file-depth" "file-json" "copy"
    "more-vertical" "source-link" "forward");
const CSS: (&str, &str) = (
    "crates/kinekino-player-ui/examples/ui_mockups/studio.css",
    include_str!("studio.css"),
);

/// (screen, draft file under the design folder, what it shows)
pub const SCREENS: [(&str, &str, &str); 22] = [
    ("studio-overview", "v2/viewing/native-player-studio-v2.png", "제작 진입: 구간 감지 개요"),
    ("studio-skip", "v2/studio/skip-settings.png", "구간 감지: 오프닝·엔딩 건너뛰기 설정"),
    ("studio-subtitles", "v2/studio/subtitles.png", "자막: 음성 인식"),
    ("studio-ocr", "v2/studio/ocr-refined.png", "자막: 화면 OCR"),
    ("studio-correction", "v2/studio/correction.png", "자막: 교정"),
    ("studio-convert", "v2/studio/convert.png", "자막: 형식 변환"),
    ("studio-subject", "v2/studio/subject.png", "피사체: 탐지·분할·추적"),
    ("studio-short", "v2/studio/short.png", "숏폼: 수동 제작"),
    ("studio-short-recommend", "v2/studio/short-recommend-refined.png", "숏폼: AI 추천"),
    ("studio-short-batch", "v2/studio/short-batch.png", "숏폼: 회차 일괄"),
    ("studio-raw-frame", "v2/studio/raw-frame.png", "스냅샷: 원본 프레임"),
    ("studio-snapshot", "v2/studio/snapshot-refined.png", "스냅샷: 썸네일 편집"),
    ("studio-cards", "v2/studio/cards.png", "카드뉴스: 카드 편집"),
    ("studio-asset2d", "v2/studio/asset2d.png", "2D 에셋: 레이어와 마스크"),
    ("studio-projection", "v2/studio/projection3d-refined.png", "3D 투영: 2.5D 깊이"),
    ("studio-projection-package", "v2/studio/projection-package.png", "3D 투영: 투영 패키지"),
    ("studio-download", "v2/studio/download-refined.png", "영상 저장: 소스와 트랙"),
    ("studio-models", "v2/studio/models-refined.png", "모델 관리"),
    ("studio-queue", "v2/studio/queue-refined.png", "작업 대기열: 상태와 오류"),
    ("studio-export", "v2/studio/export.png", "내보내기: 파일 충돌과 저장"),
    ("viewing-skip", "v2/viewing/native-player-skip-v2.png", "시청: 오프닝 건너뛰기 안내"),
    ("viewing-web", "v2/viewing/native-player-web-v2.png", "시청: 호환 웹 탐색"),
];

// ---- small building blocks (TSX strings) ----
const GROW: &str = r#"<span className="ks-grow"></span>"#;
fn c(parts: &[&str]) -> String {
    parts.concat()
}
fn i(name: &str) -> String {
    format!(r#"<span className="ks-i ks-i-{name}" aria-hidden="true"></span>"#)
}
fn sel(text: &str) -> String {
    format!(r#"<span className="ks-select"><span>{text}</span>{}</span>"#, i("chevron-down"))
}
fn sel_c(class: &str, text: &str) -> String {
    format!(r#"<span className="ks-select {class}"><span>{text}</span>{}</span>"#, i("chevron-down"))
}
fn input(text: &str) -> String {
    format!(r#"<span className="ks-input">{text}</span>"#)
}
fn input_dim(text: &str) -> String {
    format!(r#"<span className="ks-input ks-dim">{text}</span>"#)
}
fn toggle(on: bool) -> String {
    format!(r#"<span className="ks-toggle{}"><i></i></span>"#, if on { " ks-on" } else { "" })
}
fn check(on: bool, label: &str) -> String {
    format!(r#"<span className="ks-check{}"><i></i>{label}</span>"#, if on { " ks-on" } else { "" })
}
fn radio(on: bool, label: &str) -> String {
    format!(r#"<span className="ks-radio{}"><i></i>{label}</span>"#, if on { " ks-on" } else { "" })
}
fn slider(pct: u32) -> String {
    format!(r#"<span className="ks-slider"><span style="width: {pct}%"></span><i style="left: {pct}%"></i></span>"#)
}
fn btn(class: &str, icon: &str, label: &str) -> String {
    let ic = if icon.is_empty() { String::new() } else { i(icon) };
    format!(r#"<span className="ks-btn {class}">{ic}{label}</span>"#)
}
/// Label + control row of the settings panel.
fn f(label: &str, value: &str) -> String {
    format!(r#"<div className="ks-f"><span className="ks-l">{label}</span><span className="ks-v">{value}</span></div>"#)
}
fn sec(title: &str, body: &str) -> String {
    let h = if title.is_empty() { String::new() } else { format!("<h3>{title}</h3>") };
    format!(r#"<section className="ks-sec">{h}{body}</section>"#)
}
fn note(kind: &str, icon: &str, text: &str) -> String {
    format!(r#"<div className="ks-note {kind}">{}<div className="ks-col" style="gap: 2px">{text}</div></div>"#, i(icon))
}
fn p(class: &str, text: &str) -> String {
    format!(r#"<span className="{class}">{text}</span>"#)
}
fn row(body: &str) -> String {
    format!(r#"<div className="ks-row">{body}</div>"#)
}
fn col(style: &str, body: &str) -> String {
    format!(r#"<div className="ks-col" style="{style}">{body}</div>"#)
}
fn style(pairs: &str) -> String {
    format!(r#"style="{pairs}""#)
}
fn tabs(items: &[(&str, &str)], on: usize) -> String {
    let mut s = String::from(r#"<div className="ks-tabs">"#);
    for (k, (icon, label)) in items.iter().enumerate() {
        let ic = if icon.is_empty() { String::new() } else { i(icon) };
        let c = if k == on { " ks-on" } else { "" };
        s.push_str(&format!(r#"<span className="ks-tab{c}">{ic}{label}</span>"#));
    }
    s + "</div>"
}
fn seg(items: &[&str], on: usize) -> String {
    let mut s = String::from(r#"<div className="ks-seg">"#);
    for (k, label) in items.iter().enumerate() {
        let c = if k == on { r#" className="ks-on""# } else { "" };
        s.push_str(&format!("<span{c}>{label}</span>"));
    }
    s + "</div>"
}
/// Painted placeholder picture (`kind`: "" night, "sunset", "day", "room", "checker").
fn pic(kind: &str, css: &str, inner: &str) -> String {
    let k = if kind.is_empty() { String::new() } else { format!(" ks-pic-{kind}") };
    format!(r#"<div className="ks-pic{k}" style="{css}">{inner}</div>"#)
}
/// The one live preview: the generated still from the engine, with overlays on top.
fn stage(css: &str, overlays: &str) -> String {
    format!(r#"<div className="ks-stage" style="{css}"><video id="video" tabIndex="-1" aria-hidden="true"></video>{overlays}</div>"#)
}
fn transport(time: &str, pct: u32, right: &[&str]) -> String {
    let mut r = String::new();
    for n in right {
        r.push_str(&i(n));
    }
    format!(r#"<div className="ks-transport">{}<span>{time}</span><span className="ks-bar"><span style="width: {pct}%"></span><i style="left: {pct}%"></i></span>{r}</div>"#, i("play"))
}
fn ruler(marks: &[&str]) -> String {
    let mut s = String::from(r#"<div className="ks-ruler">"#);
    for m in marks {
        s.push_str(&format!("<span>{m}</span>"));
    }
    s + "</div>"
}
/// Thumbnail strip with highlighted ranges (`ranges`: left%, width%, class).
fn strip(n: usize, kind: &str, ranges: &[(u32, u32, &str)], head: Option<u32>) -> String {
    let mut s = String::from(r#"<div className="ks-strip">"#);
    for _ in 0..n {
        s.push_str(&pic(kind, "", ""));
    }
    for (l, w, c) in ranges {
        s.push_str(&format!(r#"<span className="{c}" style="left: {l}%; width: {w}%"></span>"#));
    }
    if let Some(h) = head {
        s.push_str(&format!(r#"<span className="ks-head-line" style="left: {h}%"></span>"#));
    }
    s + "</div>"
}
/// Table: column widths (px, 0 = grow), header, rows; `on` = selected row.
fn table(widths: &[u32], head: &[&str], rows: &[Vec<String>], on: Option<usize>) -> String {
    let cell = |k: usize, v: &str| {
        let w = widths[k];
        let st = if w == 0 { r#"style="flex: 1""#.to_owned() } else { format!(r#"style="flex: 0 0 {w}px""#) };
        format!("<span {st}>{v}</span>")
    };
    let mut s = String::from(r#"<div className="ks-table"><div className="ks-tr ks-th">"#);
    for (k, h) in head.iter().enumerate() {
        s.push_str(&cell(k, h));
    }
    s.push_str("</div>");
    for (r, cells) in rows.iter().enumerate() {
        let c = if on == Some(r) { " ks-on" } else { "" };
        s.push_str(&format!(r#"<div className="ks-tr{c}">"#));
        for (k, v) in cells.iter().enumerate() {
            s.push_str(&cell(k, v));
        }
        s.push_str("</div>");
    }
    s + "</div>"
}
fn cells(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| (*s).to_owned()).collect()
}

const NAV: [(&str, &str); 10] = [
    ("segments", "구간 감지"),
    ("subtitles", "자막"),
    ("subject", "피사체"),
    ("clip", "숏폼"),
    ("image", "스냅샷"),
    ("cards", "카드뉴스"),
    ("layers", "2D 에셋"),
    ("cube", "3D 투영"),
    ("export", "다운로드"),
    ("model", "모델 관리"),
];

/// Bottom job queue: summary bar plus optional rows (title, detail, state).
fn queue(count: usize, rows: &[(&str, &str, &str)], eco: bool) -> String {
    let mut s = String::from(r#"<footer className="ks-queue">"#);
    s.push_str(&format!(
        r#"<div className="ks-queue-bar">{}<span>작업 대기열 ({count})</span><span className="ks-grow"></span><span className="ks-eco">{}절약 모드{}</span>{}</div>"#,
        i("list"),
        "",
        if eco { " 켜짐" } else { "" },
        toggle(eco)
    ));
    for (k, (t, d, st)) in rows.iter().enumerate() {
        s.push_str(&format!(
            r#"<div className="ks-qrow"><span className="ks-small">{}</span>{}<div className="ks-col" style="gap: 0px; flex: 0 0 220px"><span>{t}</span><span className="ks-small">{d}</span></div><span className="ks-bar" style="flex: 0 0 200px"><span style="width: 0%"></span></span><span className="ks-grow"></span><span className="ks-muted">{st}</span>{}</div>"#,
            k + 1,
            pic("", "flex: 0 0 44px; height: 24px", ""),
            i("more-vertical")
        ));
    }
    s + "</footer>"
}

/// The shared studio frame. `nav`: index into NAV (or usize::MAX for none).
fn frame(nav: usize, main: &str, insp: &str, queue_html: &str) -> String {
    let mut items = format!(
        r#"<span className="ks-nav-item ks-nav-back">{}시청으로 돌아가기</span>"#,
        i("back")
    );
    for (k, (icon, label)) in NAV.iter().enumerate() {
        let c = if k == nav { " ks-on" } else { "" };
        items.push_str(&format!(r#"<span className="ks-nav-item{c}">{}{label}</span>"#, i(icon)));
    }
    let insp = if insp.is_empty() { String::new() } else { format!(r#"<aside className="ks-insp">{insp}</aside>"#) };
    shell(&format!(
        r#"<div className="ks-body"><nav className="ks-nav">{items}</nav><main className="ks-main">{main}</main>{insp}</div>{queue_html}"#
    ), "스튜디오")
}

fn shell(body: &str, context: &str) -> String {
    format!(
        r#"export function KinekinoPlayer() {{
  return <div id="player" className="kk-player-root ks-root">
    <div className="aw-window-chrome" role="banner">
      <div className="aw-window-brand"><span className="aw-icon aw-icon-brand" aria-hidden="true"></span><span className="aw-product-name">kinekino</span><span className="aw-window-context">{context}</span></div>
      <div className="aw-window-captions" aria-label="창 제어">
        <button className="aw-caption" aria-label="최소화"><span className="aw-icon aw-icon-minimize" aria-hidden="true"></span></button>
        <button className="aw-caption" aria-label="최대화"><span className="aw-icon aw-icon-maximize" aria-hidden="true"></span></button>
        <button className="aw-caption aw-caption-close" aria-label="닫기"><span className="aw-icon aw-icon-close" aria-hidden="true"></span></button>
      </div>
    </div>
    {body}
  </div>;
}}
"#
    )
}

fn head(title: &str, about: &str) -> String {
    format!(r#"<div className="ks-head"><h1>{title}</h1><p>{about}</p></div>"#)
}
fn head_col(title: &str, about: &str) -> String {
    format!(r#"<div className="ks-headcol"><h1>{title}</h1><p>{about}</p></div>"#)
}

const SUB_TABS: [(&str, &str); 4] = [
    ("microphone", "음성 인식"),
    ("scan-text", "화면 OCR"),
    ("text-document", "교정"),
    ("refresh", "변환"),
];

include!("studio_screens.rs");

/// The mockup for `screen` and whether it shows the live preview (video element).
pub fn mockup(screen: &str) -> Option<(Mockup, bool)> {
    let tsx = screen_tsx(screen)?;
    let video = tsx.contains(r#"id="video""#);
    Some((
        Mockup {
            tsx,
            styles: vec![CSS],
            icons: ICONS.to_vec(),
        },
        video,
    ))
}

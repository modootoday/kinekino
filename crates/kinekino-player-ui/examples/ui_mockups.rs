//! Design-review mockups: the real player window (same document, styles and Meari window
//! host as `kinekino watch`) put into a fixed state with sample data, so each screen of the
//! design drafts can be compared with what the product draws today.
//!
//! Nothing here is product code: no plugin, no libmpv, no files are read. The video area
//! shows a generated picture, and every title is invented sample text.
//!
//! ```text
//! cargo run -p kinekino-player-ui --example ui_mockups -- <screen> [capture.png]
//! cargo run -p kinekino-player-ui --example ui_mockups -- --list
//! ```
use kinekino_core::playback::{RgbaFrame, VideoSurface};
use kinekino_core::port::error::PortResult;
use kinekino_player_ui::{Controller, Engine, PlaylistItem, Options, View, run};
use std::path::PathBuf;

#[path = "ui_mockups/studio.rs"]
mod studio;

/// Screens of the design drafts that the current player can already show.
const SCREENS: [(&str, &str); 11] = [
    ("start", "첫 화면: 파일 열기와 작품 탐색"),
    ("playback", "재생 중, 컨트롤 표시"),
    ("playlist", "재생 중, 플러그인 재생목록 패널 열림"),
    ("subtitle", "재생 중, 자막 두 줄"),
    ("menu", "재생 중, ⋮ 더 보기 메뉴"),
    ("fullscreen", "전체화면 레이아웃"),
    ("browse-sources", "작품 탐색: 소스 고르기"),
    ("browse-list", "작품 탐색: 검색창과 목록, 더 보기"),
    ("browse-grid", "작품 탐색: 격자 보기"),
    ("browse-empty", "작품 탐색: 설치된 소스 없음"),
    ("browse-error", "작품 탐색: 소스 오류"),
];

/// A still evening picture (sky gradient, hills, water) instead of decoded video.
/// `cue` is what the engine reports as `sub-text`; the controller copies it into the view.
struct StillEngine {
    cue: Option<String>,
}

impl VideoSurface for StillEngine {
    fn frame_ready(&mut self) -> PortResult<bool> {
        Ok(true)
    }
    fn render(&mut self, target: &mut RgbaFrame) -> PortResult<()> {
        let (w, h) = (target.width.max(1), target.height.max(1));
        for y in 0..h {
            for x in 0..w {
                let (fx, fy) = (x as f32 / w as f32, y as f32 / h as f32);
                let hill = 0.55 - 0.08 * (fx * 7.0).sin() - 0.05 * (fx * 13.0 + 1.0).sin();
                let rgb = if fy < hill {
                    let star = (x.wrapping_mul(7919) ^ y.wrapping_mul(104_729)) % 997 == 0;
                    if star {
                        [230.0, 230.0, 255.0]
                    } else {
                        [12.0 + 40.0 * fy, 18.0 + 50.0 * fy, 48.0 + 90.0 * fy]
                    }
                } else if fy < 0.62 {
                    [10.0, 22.0, 30.0]
                } else {
                    let glint = ((fx * 40.0).sin() * (fy * 90.0).sin()).max(0.0) * 30.0;
                    [20.0 + glint, 34.0 + glint, 70.0 + glint]
                };
                let i = ((y * w + x) * 4) as usize;
                target.pixels[i..i + 4].copy_from_slice(&[
                    rgb[0] as u8,
                    rgb[1] as u8,
                    rgb[2] as u8,
                    255,
                ]);
            }
        }
        Ok(())
    }
}

impl Engine for StillEngine {
    fn load(&mut self, _: &str, _: f64) -> PortResult<()> {
        Ok(())
    }
    fn command(&mut self, _: &[&str]) -> PortResult<()> {
        Ok(())
    }
    fn property(&mut self, name: &str) -> Option<f64> {
        match name {
            "time-pos" => Some(504.0),
            "duration" => Some(1440.0),
            _ => None,
        }
    }
    fn text(&mut self, name: &str) -> Option<String> {
        (name == "sub-text").then(|| self.cue.clone()).flatten()
    }
}

fn playing(view: &mut View) {
    view.title = "샘플 재생목록 · 03 별이 머무는 밤".into();
    view.work = "샘플 재생목록".into();
    view.playlist = [
        "새로운 시작",
        "낯선 길",
        "별이 머무는 밤",
        "다음 만남",
        "약속",
        "여행의 끝",
    ]
    .iter()
    .enumerate()
    .map(|(i, t)| PlaylistItem {
        number: format!("{:02}", i + 1),
        title: (*t).into(),
    })
    .collect();
    view.current = Some(2);
    view.has_media = true;
    view.playing = true;
    view.position = 504.0;
    view.duration = 1440.0;
    view.volume = 70.0;
}

fn browsing(view: &mut View, heading: &str, rows: usize) {
    view.browsing = true;
    view.has_browse_plugin = true;
    view.browse_heading = heading.into();
    view.browse_rows = (1..=rows).map(|i| format!("예시 항목 {i:02}")).collect();
}

fn view_for(screen: &str) -> Option<View> {
    let mut v = View::default();
    match screen {
        "start" | "start-no-plugin" => {}
        "start-with-plugin" => v.has_browse_plugin = true,
        "playback" => playing(&mut v),
        "playlist" => {
            playing(&mut v);
            v.panel_open = true;
        }
        "subtitle" => {
            playing(&mut v);
            v.subtitle =
                "호숫가에 불이 하나둘 켜지기 시작했다.\n오늘 밤은 별이 잘 보일 거야.".into();
        }
        "menu" => {
            playing(&mut v);
            v.menu_open = true;
        }
        "fullscreen" => {
            playing(&mut v);
            v.fullscreen = true;
        }
        "browse-sources" => {
            browsing(&mut v, "작품 탐색", 0);
            v.browse_rows = vec!["샘플 소스 A".into(), "샘플 소스 B".into()];
        }
        "browse-list" => {
            browsing(&mut v, "샘플 소스 A", 12);
            v.browse_search = true;
            v.browse_more = true;
        }
        "browse-grid" => {
            browsing(&mut v, "샘플 소스 A", 12);
            v.browse_search = true;
            v.browse_grid = true;
            v.browse_more = true;
        }
        "browse-empty" => {
            browsing(&mut v, "작품 탐색", 0);
            v.browse_status = "설치된 소스 플러그인이 없습니다. 플러그인 폴더에 소스 플러그인을 설치하면 작품을 탐색할 수 있습니다.".into();
            v.browse_mood = "idle";
        }
        "browse-error" => {
            browsing(&mut v, "샘플 소스 B", 0);
            v.browse_search = true;
            v.browse_status = "소스에서 목록을 받지 못했습니다: 연결 시간이 초과되었습니다.".into();
            v.browse_mood = "error";
        }
        _ => return None,
    }
    Some(v)
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(screen) = args.first().filter(|a| *a != "--list") else {
        for (name, about) in SCREENS {
            println!("{name:16} {about}");
        }
        for (name, draft, about) in studio::SCREENS {
            println!("{name:26} {about} ({draft})");
        }
        return Ok(());
    };
    let studio = studio::mockup(screen);
    let view = match &studio {
        Some((_, video)) => {
            let mut v = View::default();
            v.has_media = *video;
            v.playing = *video;
            Some(v)
        }
        None => view_for(screen),
    }
    .ok_or_else(|| format!("unknown screen {screen}; see --list"))?;
    let mut controller = Controller::new(StillEngine {
        cue: (!view.subtitle.is_empty()).then(|| view.subtitle.clone()),
    });
    controller.view = view;
    let capture = args.get(1).map(PathBuf::from);
    let report = run(
        controller,
        Options {
            smoke: capture.is_some(),
            capture,
            mockup: studio.map(|(m, _)| m),
        },
    )?;
    println!(
        "{screen}: frames {} captured {}",
        report.frames, report.captured
    );
    Ok(())
}

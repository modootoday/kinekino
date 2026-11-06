//! Playlist panel ↔ mockup episode-list pattern (ADR 20261106105936 / M2-4):
//! header label, current-item props, docking, fullscreen hide, Esc/P, empty work.
use super::folder;
use super::support::FakeEngine;
use kinekino_player_ui::{
    Command, Controller, Effect, PlaylistItem, Session, View, meari_props, shortcut,
};
use meari_native::application::Renderer;
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};

fn prop(view: &View, key: &str) -> String {
    format!("{:?}", meari_props(view)[key])
}

fn hidden(doc: &meari_native::infrastructure::document::InteractiveDocument, sel: &str) -> bool {
    let base = &*doc.renderer.document;
    let id = base.query_selector(sel).unwrap().unwrap();
    let node = base.get_node(id).unwrap();
    node.attr(blitz_dom::local_name!("hidden")).is_some()
}

fn all_text(doc: &meari_native::infrastructure::document::InteractiveDocument) -> String {
    doc.elements()
        .iter()
        .flat_map(|e| e.text_runs.iter().map(|r| r.text.clone()))
        .collect::<Vec<_>>()
        .join("")
}

#[test]
fn panel_chrome_uses_the_mockup_episode_list_heading() {
    let view = View {
        has_media: true,
        panel_open: true,
        work: "예시 작품".into(),
        playlist: vec![
            PlaylistItem {
                number: "01".into(),
                title: "첫 회".into(),
            },
            PlaylistItem {
                number: "02".into(),
                title: "둘째 회".into(),
            },
        ],
        current: Some(1),
        ..View::default()
    };
    let (_, mut doc) = Session::mount(&view).unwrap();
    let viewport = doc.viewport.clone();
    doc.update_module_props(meari_props(&view)).unwrap();
    doc.renderer.rasterize(&viewport).unwrap();
    let text = all_text(&doc);
    assert!(
        text.contains("회차 목록"),
        "mockup header missing in {text:?}"
    );
    assert!(
        !text.contains("재생목록"),
        "legacy playlist label still shown in {text:?}"
    );
    assert!(!hidden(&doc, "#panel"), "open panel is visible");
    let base = &*doc.renderer.document;
    assert!(
        base.query_selector("#panelClose").unwrap().is_some(),
        "close control missing"
    );
}

#[test]
fn current_item_props_mark_only_the_playing_row() {
    let mut view = View {
        has_media: true,
        panel_open: true,
        playlist: vec![
            PlaylistItem {
                number: "01".into(),
                title: "a".into(),
            },
            PlaylistItem {
                number: "02".into(),
                title: "b".into(),
            },
            PlaylistItem {
                number: "03".into(),
                title: "c".into(),
            },
        ],
        current: Some(2),
        ..View::default()
    };
    assert!(prop(&view, "cur2").contains("true"));
    assert!(prop(&view, "cur0").contains("false"));
    assert!(prop(&view, "aria2").contains("true"));
    assert!(prop(&view, "aria1").contains("false"));
    view.current = Some(0);
    assert!(prop(&view, "cur0").contains("true"));
    assert!(prop(&view, "cur2").contains("false"));
}

#[test]
fn open_panel_docks_the_stage_until_fullscreen() {
    let mut view = View {
        has_media: true,
        panel_open: true,
        playlist: vec![PlaylistItem {
            number: "01".into(),
            title: "a".into(),
        }],
        current: Some(0),
        ..View::default()
    };
    let stage = prop(&view, "stageClass");
    assert!(
        stage.contains("kk-docked"),
        "windowed open panel docks: {stage}"
    );
    view.panel_open = false;
    let closed = prop(&view, "stageClass");
    assert!(
        !closed.contains("kk-docked"),
        "closed panel undocks: {closed}"
    );
    view.panel_open = true;
    view.fullscreen = true;
    let full = prop(&view, "stageClass");
    assert!(!full.contains("kk-docked"), "fullscreen drops dock: {full}");
}

#[test]
fn fullscreen_hides_the_panel_but_keeps_open_state() {
    let (dir, files) = folder("pl-fs");
    let mut c = Controller::new(FakeEngine::default());
    c.open(&files[0]);
    c.apply(Command::TogglePlaylist);
    assert!(c.view.panel_open);
    let (_session, mut doc) = Session::mount(&c.view).unwrap();
    let viewport = doc.viewport.clone();
    doc.update_module_props(meari_props(&c.view)).unwrap();
    doc.renderer.rasterize(&viewport).unwrap();
    assert!(!hidden(&doc, "#panel"));
    c.view.fullscreen = true;
    doc.update_module_props(meari_props(&c.view)).unwrap();
    doc.renderer.rasterize(&viewport).unwrap();
    assert!(c.view.panel_open, "fullscreen must not clear panel_open");
    assert!(
        hidden(&doc, "#panel"),
        "fullscreen hides the panel (mockup)"
    );
    c.view.fullscreen = false;
    doc.update_module_props(meari_props(&c.view)).unwrap();
    doc.renderer.rasterize(&viewport).unwrap();
    assert!(!hidden(&doc, "#panel"), "leaving fullscreen restores it");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn escape_closes_the_panel_before_leaving_fullscreen() {
    let (dir, files) = folder("pl-esc");
    let mut c = Controller::new(FakeEngine::default());
    c.open(&files[0]);
    c.apply(Command::TogglePlaylist);
    assert!(c.view.panel_open);
    assert_eq!(
        c.apply(Command::ExitFullscreen),
        None,
        "windowed: Esc closes the panel only"
    );
    assert!(!c.view.panel_open);
    c.apply(Command::TogglePlaylist);
    c.view.fullscreen = true;
    assert_eq!(
        c.apply(Command::ExitFullscreen),
        None,
        "fullscreen + open panel: Esc closes the panel first"
    );
    assert!(!c.view.panel_open && c.view.fullscreen);
    assert_eq!(
        c.apply(Command::ExitFullscreen),
        Some(Effect::Fullscreen(false)),
        "next Esc leaves fullscreen"
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn p_toggles_the_episode_list_panel() {
    assert_eq!(
        shortcut(
            ModifiersState::empty(),
            PhysicalKey::Code(KeyCode::KeyP),
            false
        ),
        Some(Command::TogglePlaylist)
    );
    assert_eq!(
        shortcut(
            ModifiersState::empty(),
            PhysicalKey::Code(KeyCode::KeyP),
            true
        ),
        None,
        "auto-repeat ignored"
    );
    let mut c = Controller::new(FakeEngine::default());
    c.apply(Command::TogglePlaylist);
    assert!(c.view.panel_open);
    c.apply(Command::TogglePlaylist);
    assert!(!c.view.panel_open);
}

#[test]
fn empty_work_title_is_hidden_in_the_panel() {
    let mut view = View {
        has_media: true,
        panel_open: true,
        work: String::new(),
        playlist: vec![PlaylistItem {
            number: "01".into(),
            title: "alone".into(),
        }],
        current: Some(0),
        ..View::default()
    };
    assert!(
        prop(&view, "noWork").contains("true"),
        "empty work → noWork"
    );
    let (_session, mut doc) = Session::mount(&view).unwrap();
    let viewport = doc.viewport.clone();
    doc.update_module_props(meari_props(&view)).unwrap();
    doc.renderer.rasterize(&viewport).unwrap();
    assert!(hidden(&doc, "#panelWork"), "empty work row hidden");
    view.work = "작품".into();
    doc.update_module_props(meari_props(&view)).unwrap();
    doc.renderer.rasterize(&viewport).unwrap();
    assert!(!hidden(&doc, "#panelWork"), "non-empty work row shown");
    assert!(prop(&view, "noWork").contains("false"));
}

#[test]
fn selecting_a_row_moves_the_current_marker() {
    let (dir, files) = folder("pl-sel");
    let mut c = Controller::new(FakeEngine::default());
    c.open(&files[1]); // 01.mkv → current 0 after sort
    assert_eq!(c.view.current, Some(0));
    c.apply(Command::TogglePlaylist);
    c.apply(Command::Item(2));
    assert_eq!(c.view.current, Some(2));
    assert!(prop(&c.view, "cur2").contains("true"));
    assert!(prop(&c.view, "cur0").contains("false"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn playlist_panel_css_uses_shared_surface_tokens() {
    // M2-V3: mockup hex must not reappear in live rules; menu/toast/browse share tokens.
    let css = include_str!("../../src/player.css");
    assert!(
        css.contains(".kk-playlist") && css.contains("background: var(--aw-surface-panel)"),
        "playlist panel bg must use --aw-surface-panel"
    );
    assert!(
        css.contains("border-left: var(--aw-border-thin) solid var(--aw-border)"),
        "playlist border must use --aw-border"
    );
    assert!(
        css.contains("background: var(--aw-selected-surface)"),
        "current row fill must use --aw-selected-surface"
    );
    assert!(
        !css.contains("background: #1b1e22"),
        "mockup panel hex must not be a live background"
    );
    assert!(
        !css.contains("background: #2f2a23"),
        "mockup selected hex must not be a live background"
    );
    assert!(
        !css.contains("solid #34363a"),
        "mockup border hex must not be a live border color"
    );
    // Chrome label / dock contracts still hold alongside the color swap.
    let view = View {
        has_media: true,
        panel_open: true,
        work: "예시".into(),
        playlist: vec![PlaylistItem {
            number: "01".into(),
            title: "a".into(),
        }],
        current: Some(0),
        ..View::default()
    };
    let (_, mut doc) = Session::mount(&view).unwrap();
    let viewport = doc.viewport.clone();
    doc.update_module_props(meari_props(&view)).unwrap();
    doc.renderer.rasterize(&viewport).unwrap();
    let text = all_text(&doc);
    assert!(text.contains("회차 목록"), "header label regress: {text:?}");
    assert!(
        !text.contains("재생목록"),
        "legacy playlist label must stay gone: {text:?}"
    );
    assert!(
        prop(&view, "stageClass").contains("kk-docked"),
        "dock class regress"
    );
}

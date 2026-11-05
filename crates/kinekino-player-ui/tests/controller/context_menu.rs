//! Right-click menu (ADR 20261105161614): open at the pointer, move, close rules,
//! groups wired to M1 commands (M2-3), placement inside the window.
use super::folder;
use super::support::FakeEngine;
use kinekino_player_ui::{
    CONTEXT_MENU_SIZE, Command, Controller, Effect, SEEK_STEP_1, SEEK_STEP_2, Session, activate,
    context_anchor, meari_props,
};
use meari_native::application::Renderer;
use std::time::{Duration, Instant};

#[test]
fn right_click_opens_moves_and_other_commands_close_it() {
    let mut c = Controller::new(FakeEngine::default());
    assert_eq!(c.view.context_menu, None);
    c.view.menu_open = true;
    assert_eq!(c.apply(Command::ContextMenu(100.0, 80.0)), None);
    assert_eq!(c.view.context_menu, Some((100.0, 80.0)));
    assert!(!c.view.menu_open, "the ⋮ menu closes");
    c.apply(Command::ContextMenu(300.0, 200.0));
    assert_eq!(
        c.view.context_menu,
        Some((300.0, 200.0)),
        "a second right-click moves it"
    );
    c.apply(Command::Mute);
    assert_eq!(c.view.context_menu, None, "any other command closes it");
    c.apply(Command::ContextMenu(10.0, 10.0));
    c.apply(Command::More);
    assert!(
        c.view.menu_open && c.view.context_menu.is_none(),
        "⋮ replaces it"
    );
    c.apply(Command::ContextMenu(10.0, 10.0));
    c.apply(Command::CloseMenu);
    assert!(!c.view.menu_open && c.view.context_menu.is_none());
    c.apply(Command::More);
    c.apply(Command::CloseMenu);
    assert!(!c.view.menu_open, "CloseMenu closes the ⋮ menu too");
}

#[test]
fn esc_closes_only_the_context_menu_first() {
    let (dir, files) = folder("ctx-esc");
    let mut c = Controller::new(FakeEngine::default());
    c.open(&files[0]);
    c.view.fullscreen = true;
    c.apply(Command::ContextMenu(50.0, 50.0));
    assert_eq!(c.apply(Command::ExitFullscreen), None);
    assert!(c.view.fullscreen && c.view.context_menu.is_none());
    assert_eq!(
        c.apply(Command::ExitFullscreen),
        Some(Effect::Fullscreen(false))
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn an_open_context_menu_keeps_fullscreen_controls_visible() {
    let (dir, files) = folder("ctx-idle");
    let mut c = Controller::new(FakeEngine::default());
    c.open(&files[0]);
    c.apply(Command::ToggleFullscreen);
    c.view.playing = true;
    c.view.status.clear();
    c.apply(Command::ContextMenu(50.0, 50.0));
    let later = Instant::now() + Duration::from_secs(60);
    c.idle(later);
    assert!(c.view.controls_visible, "no auto-hide under an open menu");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn the_anchor_stays_inside_the_window() {
    let (w, h) = CONTEXT_MENU_SIZE;
    let view = (1280.0, 900.0);
    assert_eq!(context_anchor((100.0, 50.0), view), (100.0, 50.0));
    assert_eq!(
        context_anchor((1200.0, 800.0), view),
        (1200.0 - w, 800.0 - h),
        "near the bottom-right corner it opens up and to the left"
    );
    assert_eq!(
        context_anchor((100.0, 100.0), (200.0, 200.0)),
        (0.0, 0.0),
        "a window smaller than the menu pins it to the corner"
    );
}

#[test]
fn context_items_map_to_existing_commands_and_lay_out_at_the_pointer() {
    let (dir, files) = folder("ctx-items");
    let mut c = Controller::new(FakeEngine::default());
    let (mut session, mut doc) = Session::mount(&c.view).unwrap();
    let viewport = doc.viewport.clone();
    let hidden = |doc: &meari_native::infrastructure::document::InteractiveDocument, sel: &str| {
        let base = &*doc.renderer.document;
        let id = base.query_selector(sel).unwrap().unwrap();
        let node = base.get_node(id).unwrap();
        node.attr(blitz_dom::local_name!("hidden")).is_some()
    };
    let disabled = |doc: &meari_native::infrastructure::document::InteractiveDocument,
                    sel: &str| {
        let base = &*doc.renderer.document;
        let id = base.query_selector(sel).unwrap().unwrap();
        let node = base.get_node(id).unwrap();
        node.attr(blitz_dom::local_name!("disabled")).is_some()
    };
    assert!(hidden(&doc, "#contextMenu"), "closed at start");
    c.apply(Command::ContextMenu(120.0, 90.0));
    doc.update_module_props(meari_props(&c.view)).unwrap();
    doc.renderer.rasterize(&viewport).unwrap();
    assert!(!hidden(&doc, "#contextMenu"));
    for id in ["#ctxPlayPause", "#ctxSubtitle", "#ctxAudio"] {
        assert!(disabled(&doc, id), "{id} needs media");
    }
    assert!(!disabled(&doc, "#ctxOpen") && !disabled(&doc, "#ctxFullscreen"));
    {
        let base = &*doc.renderer.document;
        let id = base.query_selector("#contextMenu").unwrap().unwrap();
        let node = base.get_node(id).unwrap();
        let l = *node.final_layout();
        let (mut x, mut y) = (l.location.x, l.location.y);
        let mut parent = node.layout_parent.get();
        while let Some(p) = parent {
            let pn = base.get_node(p).unwrap();
            x += pn.final_layout().location.x;
            y += pn.final_layout().location.y;
            parent = pn.layout_parent.get();
        }
        eprintln!("context menu {x},{y} {}x{}", l.size.width, l.size.height);
        assert!(
            (x - 120.0).abs() <= 1.0 && (y - 90.0).abs() <= 1.0,
            "at the pointer"
        );
        let (w, h) = CONTEXT_MENU_SIZE;
        assert!(l.size.width <= w as f32 + 0.5 && l.size.height <= h as f32 + 0.5);
        assert!(l.size.height >= 6.0 * 32.0, "room for six groups");
        let groups: Vec<String> = base
            .query_selector_all("#contextMenu .kk-menu-heading")
            .unwrap()
            .into_iter()
            .map(|n| base.get_node(n).unwrap().text_content())
            .collect();
        assert_eq!(groups, ["열기", "재생", "구간", "자막", "오디오", "화면"]);
    }
    assert_eq!(
        click(&mut session, &mut doc, &mut c, "ctxOpen"),
        [Command::Open]
    );
    assert!(c.view.context_menu.is_none(), "an item closes the menu");
    // Media adds playlist rows, which are fixed per mount.
    c.open(&files[0]);
    let (mut session, mut doc) = Session::mount(&c.view).unwrap();
    for (id, command) in [
        ("ctxPlayPause", Command::PlayPause),
        ("ctxSubtitle", Command::CycleSubtitle),
        ("ctxAudio", Command::CycleAudio),
        ("ctxFullscreen", Command::ToggleFullscreen),
    ] {
        c.apply(Command::ContextMenu(10.0, 10.0));
        doc.update_module_props(meari_props(&c.view)).unwrap();
        assert_eq!(click(&mut session, &mut doc, &mut c, id), [command], "{id}");
        assert!(c.view.context_menu.is_none());
    }
    std::fs::remove_dir_all(dir).unwrap();
}

/// Activates `#id`, runs its commands through `c` and refreshes the props; the commands.
fn click(
    session: &mut Session,
    doc: &mut meari_native::infrastructure::document::InteractiveDocument,
    c: &mut Controller<FakeEngine>,
    id: &str,
) -> Vec<Command> {
    activate(doc, id).unwrap();
    let commands = session.commands(doc).unwrap();
    for command in commands.iter().cloned() {
        c.apply(command);
    }
    doc.update_module_props(meari_props(&c.view)).unwrap();
    commands
}

#[test]
fn m1_commands_map_from_context_menu_items() {
    let (dir, files) = folder("ctx-m1");
    let mut c = Controller::new(FakeEngine::default());
    c.open(&files[0]);
    let (mut session, mut doc) = Session::mount(&c.view).unwrap();
    let cases: &[(&str, Command)] = &[
        ("ctxPrevious", Command::Previous),
        ("ctxNext", Command::Next),
        ("ctxAutoNext", Command::ToggleAutoNext),
        ("ctxSpeedDown", Command::SpeedDown),
        ("ctxSpeedUp", Command::SpeedUp),
        ("ctxSpeedReset", Command::SpeedReset),
        ("ctxSeekBack5", Command::SeekBy(-SEEK_STEP_1)),
        ("ctxSeekFwd5", Command::SeekBy(SEEK_STEP_1)),
        ("ctxSeekBack30", Command::SeekBy(-SEEK_STEP_2)),
        ("ctxSeekFwd30", Command::SeekBy(SEEK_STEP_2)),
        ("ctxVolumeUp", Command::VolumeUp),
        ("ctxVolumeDown", Command::VolumeDown),
        ("ctxMute", Command::Mute),
        ("ctxMarkSkipStart", Command::MarkSkipStart),
        ("ctxMarkSkipEnd", Command::MarkSkipEnd),
        ("ctxSkipSegment", Command::SkipSegment),
        ("ctxClearSkip", Command::ClearSkip),
    ];
    for (id, command) in cases {
        c.apply(Command::ContextMenu(10.0, 10.0));
        doc.update_module_props(meari_props(&c.view)).unwrap();
        assert_eq!(
            click(&mut session, &mut doc, &mut c, id),
            [command.clone()],
            "{id}"
        );
        assert!(c.view.context_menu.is_none(), "{id} closes the menu");
    }
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn m1_media_items_are_disabled_without_media_and_headings_include_skip() {
    let mut c = Controller::new(FakeEngine::default());
    let (_session, mut doc) = Session::mount(&c.view).unwrap();
    let viewport = doc.viewport.clone();
    let disabled = |doc: &meari_native::infrastructure::document::InteractiveDocument,
                    sel: &str| {
        let base = &*doc.renderer.document;
        let id = base.query_selector(sel).unwrap().unwrap();
        let node = base.get_node(id).unwrap();
        node.attr(blitz_dom::local_name!("disabled")).is_some()
    };
    c.apply(Command::ContextMenu(40.0, 40.0));
    doc.update_module_props(meari_props(&c.view)).unwrap();
    doc.renderer.rasterize(&viewport).unwrap();
    for id in [
        "#ctxPrevious",
        "#ctxNext",
        "#ctxSpeedDown",
        "#ctxSpeedUp",
        "#ctxSpeedReset",
        "#ctxSeekBack5",
        "#ctxSeekFwd5",
        "#ctxSeekBack30",
        "#ctxSeekFwd30",
        "#ctxMarkSkipStart",
        "#ctxMarkSkipEnd",
        "#ctxSkipSegment",
        "#ctxClearSkip",
        "#ctxPlayPause",
        "#ctxSubtitle",
        "#ctxAudio",
    ] {
        assert!(disabled(&doc, id), "{id} needs media");
    }
    for id in [
        "#ctxMute",
        "#ctxVolumeUp",
        "#ctxVolumeDown",
        "#ctxAutoNext",
        "#ctxFullscreen",
    ] {
        assert!(!disabled(&doc, id), "{id} stays enabled");
    }
    {
        let base = &*doc.renderer.document;
        let id = base.query_selector("#contextMenu").unwrap().unwrap();
        let node = base.get_node(id).unwrap();
        let l = *node.final_layout();
        eprintln!(
            "m1 context menu {}x{} (reservation {:?})",
            l.size.width, l.size.height, CONTEXT_MENU_SIZE
        );
        let (w, h) = CONTEXT_MENU_SIZE;
        assert!(
            l.size.width <= w as f32 + 0.5 && l.size.height <= h as f32 + 0.5,
            "laid-out menu must fit the reservation"
        );
        assert!(
            l.size.height > 360.0,
            "M2-3 grows past the M2-2 360px skeleton"
        );
        let groups: Vec<String> = base
            .query_selector_all("#contextMenu .kk-menu-heading")
            .unwrap()
            .into_iter()
            .map(|n| base.get_node(n).unwrap().text_content())
            .collect();
        assert_eq!(
            groups,
            ["열기", "재생", "구간", "자막", "오디오", "화면"],
            "flat groups including 구간"
        );
    }
}

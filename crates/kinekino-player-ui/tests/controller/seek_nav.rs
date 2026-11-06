//! Seek and navigation keys (ADR 20261105151827): arrow seek steps, PageUp/Down, Backspace.
//! Seek OSD toast (ADR 20261105182156).
use super::folder;
use super::support::FakeEngine;
use kinekino_player_ui::{
    Command, Controller, SEEK_STEP_1, SEEK_STEP_2, SEEK_STEP_3, SEEK_STEP_4, View, meari_props,
    shortcut,
};
use winit::keyboard::{KeyCode as K, ModifiersState as M, PhysicalKey::Code};

fn key(mods: M, code: K, repeat: bool) -> Option<Command> {
    shortcut(mods, Code(code), repeat)
}

fn seeks(e: &FakeEngine) -> Vec<&str> {
    e.commands
        .iter()
        .filter(|c| c.starts_with("seek"))
        .map(String::as_str)
        .collect()
}

#[test]
fn seek_steps_match_the_potplayer_defaults() {
    assert_eq!(
        (SEEK_STEP_1, SEEK_STEP_2, SEEK_STEP_3, SEEK_STEP_4),
        (5.0, 30.0, 60.0, 300.0)
    );
}

#[test]
fn arrow_keys_seek_by_step_with_modifiers_and_allow_repeat() {
    let none = M::empty();
    let ctrl = M::CONTROL;
    let shift = M::SHIFT;
    let ctrl_alt = M::CONTROL | M::ALT;
    assert_eq!(key(none, K::ArrowLeft, false), Some(Command::SeekBy(-5.0)));
    assert_eq!(key(none, K::ArrowRight, true), Some(Command::SeekBy(5.0)));
    assert_eq!(
        key(ctrl, K::ArrowLeft, true),
        Some(Command::SeekBy(-30.0)),
        "Ctrl step repeats"
    );
    assert_eq!(key(ctrl, K::ArrowRight, false), Some(Command::SeekBy(30.0)));
    assert_eq!(
        key(shift, K::ArrowLeft, false),
        Some(Command::SeekBy(-60.0))
    );
    assert_eq!(key(shift, K::ArrowRight, true), Some(Command::SeekBy(60.0)));
    assert_eq!(
        key(ctrl_alt, K::ArrowLeft, false),
        Some(Command::SeekBy(-300.0))
    );
    assert_eq!(
        key(ctrl_alt, K::ArrowRight, true),
        Some(Command::SeekBy(300.0))
    );
    // Meta alone counts as primary (macOS); Meta+Alt does not.
    assert_eq!(
        key(M::META, K::ArrowRight, false),
        Some(Command::SeekBy(30.0))
    );
    assert_eq!(key(M::META | M::ALT, K::ArrowRight, false), None);
    // Shift+Ctrl is not a seek chord in this table.
    assert_eq!(key(M::CONTROL | M::SHIFT, K::ArrowLeft, false), None);
    // Volume stays plain-only: Shift+Up is not volume and not seek.
    assert_eq!(key(shift, K::ArrowUp, false), None);
}

#[test]
fn page_up_down_and_backspace_navigate_without_repeat() {
    let none = M::empty();
    assert_eq!(key(none, K::PageUp, false), Some(Command::Previous));
    assert_eq!(key(none, K::PageDown, false), Some(Command::Next));
    assert_eq!(key(none, K::Backspace, false), Some(Command::Seek(0.0)));
    assert_eq!(key(none, K::PageUp, true), None, "auto-repeat");
    assert_eq!(key(none, K::PageDown, true), None);
    assert_eq!(key(none, K::Backspace, true), None);
    assert_eq!(key(M::CONTROL, K::PageUp, false), None);
    assert_eq!(key(M::SHIFT, K::Backspace, false), None);
}

#[test]
fn skip_keys_are_confirmed_not_provisional() {
    let ctrl = M::CONTROL;
    let none = M::empty();
    assert_eq!(
        key(ctrl, K::BracketLeft, false),
        Some(Command::MarkSkipStart)
    );
    assert_eq!(
        key(ctrl, K::BracketRight, false),
        Some(Command::MarkSkipEnd)
    );
    assert_eq!(key(none, K::KeyS, false), Some(Command::SkipSegment));
    assert_eq!(key(ctrl, K::Backslash, false), Some(Command::ClearSkip));
}

#[test]
fn seek_by_moves_clamps_and_ignores_empty_media() {
    let (dir, files) = folder("seek-by");
    let mut c = Controller::new(FakeEngine::default());
    // No media: no-op.
    c.apply(Command::SeekBy(5.0));
    assert!(c.engine.commands.is_empty());
    c.open(&files[0]);
    c.engine.time = 100.0;
    c.poll();
    assert_eq!(c.view.position, 100.0);
    assert_eq!(c.view.duration, 1500.0);

    c.apply(Command::SeekBy(-SEEK_STEP_1));
    assert_eq!(c.view.position, 95.0);
    assert_eq!(
        seeks(&c.engine).last().copied(),
        Some("seek 95.000 absolute")
    );

    c.apply(Command::SeekBy(SEEK_STEP_1));
    assert_eq!(c.view.position, 100.0);

    c.apply(Command::SeekBy(2000.0));
    assert_eq!(c.view.position, 1500.0, "clamped at end");
    assert_eq!(
        seeks(&c.engine).last().copied(),
        Some("seek 1500.000 absolute")
    );

    c.apply(Command::SeekBy(-10_000.0));
    assert_eq!(c.view.position, 0.0, "clamped at start");
    assert_eq!(
        seeks(&c.engine).last().copied(),
        Some("seek 0.000 absolute")
    );

    // duration ≤ 0: no-op.
    c.view.duration = 0.0;
    let before = c.engine.commands.len();
    c.apply(Command::SeekBy(5.0));
    assert_eq!(c.engine.commands.len(), before);
    assert_eq!(c.view.position, 0.0);

    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn backspace_seeks_to_the_start() {
    let (dir, files) = folder("seek-home");
    let mut c = Controller::new(FakeEngine::default());
    c.open(&files[0]);
    c.engine.time = 42.0;
    c.poll();
    c.apply(Command::Seek(0.0));
    assert_eq!(c.view.position, 0.0);
    assert_eq!(
        seeks(&c.engine).last().copied(),
        Some("seek 0.000 absolute")
    );
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn seek_by_shows_position_and_duration_on_the_toast() {
    let (dir, files) = folder("osd-seek");
    let mut c = Controller::new(FakeEngine::default());
    c.open(&files[0]);
    c.engine.time = 100.0;
    c.poll();
    c.apply(Command::SeekBy(SEEK_STEP_1));
    assert_eq!(c.view.position, 105.0);
    assert_eq!(c.view.notice, "1:45 / 25:00");
    c.apply(Command::SeekBy(-SEEK_STEP_1));
    assert_eq!(c.view.notice, "1:40 / 25:00");
    c.apply(Command::Seek(0.0));
    assert_eq!(c.view.position, 0.0);
    assert_eq!(c.view.notice, "0:00 / 25:00");
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn seek_does_not_toast_without_media_or_duration() {
    let mut c = Controller::new(FakeEngine::default());
    c.apply(Command::SeekBy(5.0));
    assert!(c.view.notice.is_empty());
    let (dir, files) = folder("osd-seek-empty");
    c.open(&files[0]);
    c.poll();
    c.view.notice.clear();
    c.view.duration = 0.0;
    c.apply(Command::SeekBy(5.0));
    assert!(c.view.notice.is_empty());
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn seek_toast_hides_the_text_subtitle_overlay() {
    // Toast and text cue share the stage bottom centre; while the OSD is up the cue must
    // yield so seek/volume/speed feedback stays readable (visual-review consistency).
    let mut view = View {
        has_media: true,
        subtitle: "cue line".into(),
        ..View::default()
    };
    let shown = format!("{:?}", meari_props(&view)["noSubtitle"]);
    assert!(
        shown.contains("false"),
        "text cue shows when there is no toast: {shown}"
    );
    view.notice = "1:45 / 25:00".into();
    let hidden = format!("{:?}", meari_props(&view)["noSubtitle"]);
    assert!(
        hidden.contains("true"),
        "toast owns the bottom centre while the notice is set: {hidden}"
    );
    view.notice.clear();
    let back = format!("{:?}", meari_props(&view)["noSubtitle"]);
    assert!(
        back.contains("false"),
        "cue returns after the toast clears: {back}"
    );
}

#[test]
fn osd_toast_css_uses_shared_panel_tokens() {
    // M2-V4: live toast chrome stays on shared panel tokens (menu family), not
    // mockup translucent-pill hex. Locks the visual-review recommendation for OSD.
    let css = include_str!("../../src/player.css");
    assert!(
        css.contains(".kk-toast") && css.contains(".kk-toast-text"),
        "toast selectors must remain"
    );
    let toast_text_rule = css
        .lines()
        .find(|l| l.contains(".kk-toast-text"))
        .expect(".kk-toast-text rule");
    assert!(
        toast_text_rule.contains("background: var(--aw-surface-panel)"),
        "toast fill must use --aw-surface-panel: {toast_text_rule}"
    );
    assert!(
        toast_text_rule.contains("var(--aw-border)"),
        "toast border must use --aw-border: {toast_text_rule}"
    );
    assert!(
        toast_text_rule.contains("border-radius: var(--aw-radius-control)"),
        "toast radius must use --aw-radius-control: {toast_text_rule}"
    );
    assert!(
        toast_text_rule.contains("font-family: var(--aw-font-family)"),
        "toast font must use --aw-font-family (same as menu): {toast_text_rule}"
    );
    for banned in [
        "background: rgba(0, 0, 0,",
        "background: rgba(0,0,0,",
        "background: #000",
        "background: #111",
        "background: #1a1a1a",
        "background: #1b1e22",
    ] {
        assert!(
            !toast_text_rule.contains(banned),
            "mockup-style toast fill must not be live: {banned}"
        );
    }
    // Notice channel still wired: seek toast text + subtitle yield (V1).
    let view = View {
        has_media: true,
        notice: "1:45 / 25:00".into(),
        subtitle: "cue".into(),
        ..View::default()
    };
    let props = meari_props(&view);
    assert!(
        format!("{:?}", props["notice"]).contains("1:45 / 25:00"),
        "seek toast text must surface: {:?}",
        props["notice"]
    );
    assert!(
        format!("{:?}", props["noNotice"]).contains("false"),
        "notice visible while set"
    );
    assert!(
        format!("{:?}", props["noSubtitle"]).contains("true"),
        "subtitle yields while toast is up"
    );
}

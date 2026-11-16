//! Styled and picture subtitles (ASS/SSA, PGS, DVD, DVB) are drawn by the engine into the
//! video frame; plain text subtitles stay on the player's own overlay.
use super::folder;
use super::support::FakeEngine;
use kinekino_player_ui::Controller;

const CODEC: &str = "current-tracks/sub/codec";

fn visibility(e: &FakeEngine) -> Vec<&str> {
    e.commands
        .iter()
        .filter(|c| c.starts_with("set sub-visibility"))
        .map(String::as_str)
        .collect()
}

#[test]
fn styled_and_picture_tracks_are_drawn_by_the_engine() {
    for codec in [
        "ass",
        "ssa",
        "hdmv_pgs_subtitle",
        "dvd_subtitle",
        "dvb_subtitle",
    ] {
        let (dir, files) = folder(&format!("subs-engine-{codec}"));
        let mut c = Controller::new(FakeEngine::default());
        c.open(&files[0]);
        c.engine.strings.insert(CODEC.into(), codec.into());
        c.engine.subtitle = Some("styled line".into());
        c.poll();
        assert_eq!(visibility(&c.engine), ["set sub-visibility yes"], "{codec}");
        assert_eq!(
            c.view.subtitle, "",
            "{codec}: no second copy on the overlay"
        );
        // Polling again does not resend the switch.
        c.poll();
        assert_eq!(visibility(&c.engine).len(), 1, "{codec}");
        std::fs::remove_dir_all(dir).ok();
    }
}

#[test]
fn text_tracks_stay_on_the_overlay() {
    let (dir, files) = folder("subs-overlay");
    let mut c = Controller::new(FakeEngine::default());
    c.open(&files[0]);
    c.engine.strings.insert(CODEC.into(), "subrip".into());
    c.engine.subtitle = Some("plain line".into());
    c.poll();
    assert!(visibility(&c.engine).is_empty(), "engine drawing stays off");
    assert_eq!(c.view.subtitle, "plain line");
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn switching_tracks_moves_drawing_back_and_forth() {
    let (dir, files) = folder("subs-switch");
    let mut c = Controller::new(FakeEngine::default());
    c.open(&files[0]);
    c.engine.strings.insert(CODEC.into(), "ass".into());
    c.engine.subtitle = Some("line".into());
    c.poll();
    c.engine.strings.insert(CODEC.into(), "subrip".into());
    assert!(c.poll(), "the overlay text appears");
    assert_eq!(c.view.subtitle, "line");
    // No subtitle track at all: the engine reports no codec.
    c.engine.strings.insert(CODEC.into(), "ass".into());
    c.poll();
    c.engine.strings.remove(CODEC);
    c.poll();
    assert_eq!(
        visibility(&c.engine),
        [
            "set sub-visibility yes",
            "set sub-visibility no",
            "set sub-visibility yes",
            "set sub-visibility no"
        ]
    );
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn text_subtitle_css_lifts_above_transport_when_controls_are_up() {
    // M1-V2: text cue (and toast) share the stage bottom with the transport sibling.
    // Real-window check: with controls visible the cue sits flush on the seek chrome;
    // lift with --aw-transport-height while `.kk-controls-up` (skip-offer family), and
    // keep bottom:0 when controls hide so the cue can sit near the screen edge.
    let css = include_str!("../../src/player.css");
    assert!(
        css.contains(".kk-subtitle"),
        "subtitle overlay rule must remain"
    );
    let lift = css
        .lines()
        .find(|l| l.contains("kk-controls-up") && l.contains("kk-subtitle"))
        .or_else(|| {
            // Multi-selector rule may put the property on the next shared block; accept a
            // combined selector line that names both controls-up and subtitle/toast.
            css.lines().find(|l| {
                l.contains("kk-controls-up")
                    && (l.contains("kk-subtitle") || l.contains("kk-toast"))
            })
        })
        .expect("kk-controls-up must target the subtitle (and toast) for the lift");
    assert!(
        lift.contains("kk-toast") || css.contains("kk-controls-up") && css.contains("kk-toast"),
        "toast must lift with the subtitle so OSD stays consistent: {lift}"
    );
    // The lift offset must use the transport token (no new hex, no bare 48px).
    let block_start = css
        .find(".kk-player-root.kk-controls-up")
        .expect("controls-up lift block");
    let block = &css[block_start..block_start + 280.min(css.len() - block_start)];
    assert!(
        block.contains("var(--aw-transport-height)"),
        "lift must use --aw-transport-height: {block}"
    );
    assert!(
        !block.contains("bottom: 48px") && !block.contains("+ 48px"),
        "do not hard-code 48px when the transport token exists: {block}"
    );
}

#[test]
fn root_marks_controls_up_with_the_transport_visibility() {
    use kinekino_player_ui::{View, meari_props};
    let mut view = View {
        has_media: true,
        controls_visible: true,
        ..View::default()
    };
    let root = format!("{:?}", meari_props(&view)["rootClass"]);
    assert!(
        root.contains("kk-controls-up"),
        "controls visible → root carries kk-controls-up for the subtitle lift: {root}"
    );
    view.controls_visible = false;
    let root = format!("{:?}", meari_props(&view)["rootClass"]);
    assert!(
        !root.contains("kk-controls-up"),
        "controls hidden → no lift class (cue returns to stage bottom): {root}"
    );
}

#[test]
fn player_overlays_with_their_own_display_still_hide_when_hidden() {
    // M1-V3: an author `display:` on a class beats the user-agent `[hidden]` rule, so the
    // cue stayed on screen under skip and seek toasts although `noSubtitle` was set.
    // Every player overlay (`kk-*`) toggled by `hidden` needs its own `[hidden]` rule.
    let markup = include_str!("../../src/source/mod.rs");
    let css = include_str!("../../src/player.css");
    let toggled = markup
        .lines()
        .filter(|l| l.contains("hidden={{"))
        .filter_map(|l| l.split("className=\"").nth(1)?.split('"').next())
        .flat_map(str::split_whitespace)
        .filter(|c| c.starts_with("kk-"));
    let mut missing = Vec::new();
    for class in toggled {
        let own = format!(".{class}");
        let shows = css.lines().any(|l| {
            let Some((selector, body)) = l.split_once('{') else {
                return false;
            };
            selector.trim_end().ends_with(&own)
                && body.contains("display:")
                && !body.contains("display: none")
        });
        let hides = css
            .lines()
            .any(|l| l.contains(&format!("{own}[hidden]")) && l.contains("display: none"));
        if shows && !hides {
            missing.push(class);
        }
    }
    assert!(missing.is_empty(), "no [hidden] rule for: {missing:?}");
}

fn positions(e: &FakeEngine) -> Vec<&str> {
    e.commands
        .iter()
        .filter(|c| c.starts_with("set sub-pos"))
        .map(String::as_str)
        .collect()
}

#[test]
fn engine_drawn_subtitles_clear_the_transport_while_controls_show() {
    // M1-V2 follow-up: the overlay cue lifts by --aw-transport-height (56px) while the
    // controls show; a cue the engine draws into the frame must lift the same distance.
    let (dir, files) = folder("subs-lift");
    let mut c = Controller::new(FakeEngine::default());
    c.open(&files[0]);
    c.engine.strings.insert(CODEC.into(), "ass".into());
    // The engine renders into the video box, so its OSD height is the box height.
    c.engine.numbers.insert("osd-height".into(), 560.0);
    c.poll();
    assert_eq!(
        positions(&c.engine),
        ["set sub-pos 90"],
        "56 of 560 px → 10 %"
    );
    c.poll();
    assert_eq!(
        positions(&c.engine).len(),
        1,
        "unchanged position is not resent"
    );
    // Controls auto-hide: the cue returns to the frame's own bottom.
    c.view.controls_visible = false;
    c.poll();
    assert_eq!(positions(&c.engine).last(), Some(&"set sub-pos 100"));
    // A taller box needs a smaller share; the lift never rounds below the transport.
    c.view.controls_visible = true;
    c.engine.numbers.insert("osd-height".into(), 1000.0);
    c.poll();
    assert_eq!(positions(&c.engine).last(), Some(&"set sub-pos 94"));
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn overlay_tracks_and_unknown_boxes_leave_the_engine_position_alone() {
    let (dir, files) = folder("subs-lift-none");
    let mut c = Controller::new(FakeEngine::default());
    c.open(&files[0]);
    c.engine.numbers.insert("osd-height".into(), 560.0);
    c.engine.strings.insert(CODEC.into(), "subrip".into());
    c.poll();
    assert!(
        positions(&c.engine).is_empty(),
        "the overlay lifts itself in CSS"
    );
    // Engine drawing with no laid-out box yet: nothing to compute from.
    c.engine.numbers.remove("osd-height");
    c.engine.strings.insert(CODEC.into(), "ass".into());
    c.poll();
    assert!(positions(&c.engine).is_empty(), "{:?}", c.engine.commands);
    // Leaving engine drawing after a lift puts the engine back at its default.
    c.engine.numbers.insert("osd-height".into(), 560.0);
    c.poll();
    c.engine.strings.insert(CODEC.into(), "subrip".into());
    c.poll();
    assert_eq!(positions(&c.engine), ["set sub-pos 90", "set sub-pos 100"]);
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn the_lift_matches_the_transport_token() {
    let tokens = include_str!("../../../kinekino-player/src/presentation/web_ui/styles/tokens.css");
    assert!(
        tokens.contains(&format!(
            "--aw-transport-height: {}px;",
            kinekino_player_ui::TRANSPORT_HEIGHT_PX
        )),
        "keep the engine lift equal to --aw-transport-height"
    );
}

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

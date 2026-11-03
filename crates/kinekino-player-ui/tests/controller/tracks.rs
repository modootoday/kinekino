//! Audio and subtitle track keys: Alt+A / Alt+L cycle the engine's tracks and a toast
//! names the one now playing.
use super::folder;
use super::support::FakeEngine;
use kinekino_player_ui::{Command, Controller};

/// Gives the engine a track list of `kinds` ("audio", "sub", "video").
fn tracks(e: &mut FakeEngine, kinds: &[&str]) {
    e.numbers
        .insert("track-list/count".into(), kinds.len() as f64);
    for (i, kind) in kinds.iter().enumerate() {
        e.strings
            .insert(format!("track-list/{i}/type"), (*kind).into());
    }
}

/// The engine's state after a cycle: selection `id` and the selected track's names.
fn selected(e: &mut FakeEngine, kind: &str, id: &str, title: &str, lang: &str) {
    let sel = if kind == "audio" { "aid" } else { "sid" };
    e.strings.insert(sel.into(), id.into());
    e.strings
        .insert(format!("current-tracks/{kind}/title"), title.into());
    e.strings
        .insert(format!("current-tracks/{kind}/lang"), lang.into());
}

#[test]
fn alt_a_and_alt_l_cycle_audio_and_subtitle_tracks() {
    use kinekino_player_ui::shortcut;
    use winit::keyboard::{KeyCode as K, ModifiersState as M, PhysicalKey::Code};
    let key = |m, k, r| shortcut(m, Code(k), r);
    assert_eq!(key(M::ALT, K::KeyA, false), Some(Command::CycleAudio));
    assert_eq!(key(M::ALT, K::KeyL, false), Some(Command::CycleSubtitle));
    assert_eq!(key(M::ALT, K::KeyA, true), None, "auto-repeat");
    assert_eq!(key(M::ALT, K::KeyL, true), None, "auto-repeat");
    for mods in [
        M::ALT | M::SHIFT,
        M::ALT | M::CONTROL,
        M::ALT | M::META,
        M::CONTROL,
        M::SHIFT,
        M::empty(),
    ] {
        assert_eq!(key(mods, K::KeyA, false), None, "{mods:?}+A");
        assert_eq!(key(mods, K::KeyL, false), None, "{mods:?}+L");
    }
    // Other Alt combos still never fire.
    assert_eq!(key(M::ALT, K::KeyM, false), None);
    assert_eq!(key(M::ALT, K::Space, false), None);
}

#[test]
fn cycling_audio_sends_the_command_and_names_the_track() {
    let (dir, files) = folder("tracks-audio");
    let mut c = Controller::new(FakeEngine::default());
    c.open(&files[0]);
    tracks(&mut c.engine, &["video", "audio", "audio", "sub"]);

    selected(&mut c.engine, "audio", "2", "", "");
    c.apply(Command::CycleAudio);
    assert_eq!(c.engine.commands.last().unwrap(), "cycle aid");
    assert_eq!(c.view.notice, "오디오 트랙 2", "no name");

    selected(&mut c.engine, "audio", "1", "", "kor");
    c.apply(Command::CycleAudio);
    assert_eq!(c.view.notice, "오디오 트랙 1 · kor", "language only");

    selected(&mut c.engine, "audio", "2", "Commentary", "eng");
    c.apply(Command::CycleAudio);
    assert_eq!(c.view.notice, "오디오 트랙 2 · Commentary", "title wins");
    let cycles = c.engine.commands.iter().filter(|s| *s == "cycle aid");
    assert_eq!(cycles.count(), 3);
    std::fs::remove_dir_all(dir).unwrap();
}

/// Queues what the engine selects after each following command: `(id, title, lang)`
/// per command, so the selection and the selected track's names change together.
fn answers(e: &mut FakeEngine, kind: &str, steps: &[(&str, &str, &str)]) {
    let sel = if kind == "audio" { "aid" } else { "sid" };
    for (id, title, lang) in steps {
        e.after_command.push_back(vec![
            (sel.into(), (*id).into()),
            (format!("current-tracks/{kind}/title"), (*title).into()),
            (format!("current-tracks/{kind}/lang"), (*lang).into()),
        ]);
    }
}

const OFF: (&str, &str, &str) = ("no", "", "");

#[test]
fn cycling_audio_skips_off_and_wraps_to_the_first_track() {
    // libmpv cycles `aid` 1 → 2 → no → 1; PotPlayer's audio cycle never turns audio off.
    let (dir, files) = folder("tracks-audio-wrap");
    let mut c = Controller::new(FakeEngine::default());
    c.open(&files[0]);
    tracks(&mut c.engine, &["video", "audio", "audio", "sub"]);
    selected(&mut c.engine, "audio", "2", "Commentary", "eng");
    answers(&mut c.engine, "audio", &[OFF, ("1", "", "kor")]);
    c.apply(Command::CycleAudio);
    let cycles = c.engine.commands.iter().filter(|s| *s == "cycle aid");
    assert_eq!(cycles.count(), 2, "{:?}", c.engine.commands);
    assert_eq!(c.engine.strings["aid"], "1");
    assert_eq!(
        c.view.notice, "오디오 트랙 1 · kor",
        "names read after the last cycle"
    );

    // Only one extra step: an engine still at 'no' is not cycled forever.
    c.engine.commands.clear();
    answers(&mut c.engine, "audio", &[OFF, OFF, ("1", "", "kor")]);
    c.apply(Command::CycleAudio);
    assert_eq!(c.engine.commands, ["cycle aid", "cycle aid"]);
    assert_eq!(c.view.notice, "오디오 트랙 끔", "the engine stayed off");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cycling_a_single_audio_track_comes_back_to_it() {
    // libmpv cycles a lone `aid` 1 → no → 1.
    let (dir, files) = folder("tracks-audio-single");
    let mut c = Controller::new(FakeEngine::default());
    c.open(&files[0]);
    tracks(&mut c.engine, &["video", "audio"]);
    selected(&mut c.engine, "audio", "1", "", "jpn");
    answers(&mut c.engine, "audio", &[OFF, ("1", "", "jpn")]);
    c.apply(Command::CycleAudio);
    let cycles = c.engine.commands.iter().filter(|s| *s == "cycle aid");
    assert_eq!(cycles.count(), 2, "{:?}", c.engine.commands);
    assert_eq!(c.view.notice, "오디오 트랙 1 · jpn");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cycling_audio_without_audio_tracks_or_subtitles_to_off_does_not_retry() {
    let (dir, files) = folder("tracks-no-retry");
    let mut c = Controller::new(FakeEngine::default());
    c.open(&files[0]);
    tracks(&mut c.engine, &["video"]);
    answers(&mut c.engine, "audio", &[OFF, OFF]);
    c.apply(Command::CycleAudio);
    assert_eq!(c.engine.commands.last().unwrap(), "cycle aid");
    let cycles = c.engine.commands.iter().filter(|s| *s == "cycle aid");
    assert_eq!(cycles.count(), 1, "no audio tracks: one cycle");
    assert_eq!(c.view.notice, "오디오 트랙 없음");

    tracks(&mut c.engine, &["video", "audio", "sub"]);
    c.engine.after_command.clear();
    answers(&mut c.engine, "sub", &[OFF, ("1", "", "kor")]);
    c.apply(Command::CycleSubtitle);
    let cycles = c.engine.commands.iter().filter(|s| *s == "cycle sid");
    assert_eq!(cycles.count(), 1, "subtitles keep 'off'");
    assert_eq!(c.view.notice, "자막 끔");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cycling_subtitles_names_the_track_or_says_off() {
    let (dir, files) = folder("tracks-sub");
    let mut c = Controller::new(FakeEngine::default());
    c.open(&files[0]);
    tracks(&mut c.engine, &["video", "audio", "sub", "sub"]);

    selected(&mut c.engine, "sub", "1", "", "");
    c.apply(Command::CycleSubtitle);
    assert_eq!(c.engine.commands.last().unwrap(), "cycle sid");
    assert_eq!(c.view.notice, "자막 1");

    selected(&mut c.engine, "sub", "2", "", "kor");
    c.apply(Command::CycleSubtitle);
    assert_eq!(c.view.notice, "자막 2 · kor");

    selected(&mut c.engine, "sub", "1", "Signs", "eng");
    c.apply(Command::CycleSubtitle);
    assert_eq!(c.view.notice, "자막 1 · Signs", "title wins");

    selected(&mut c.engine, "sub", "no", "", "");
    c.apply(Command::CycleSubtitle);
    assert_eq!(c.view.notice, "자막 끔", "cycled past the last one");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cycling_with_no_tracks_of_that_kind_says_so() {
    let (dir, files) = folder("tracks-none");
    let mut c = Controller::new(FakeEngine::default());
    c.open(&files[0]);
    tracks(&mut c.engine, &["video"]);
    c.engine.strings.insert("aid".into(), "no".into());
    c.engine.strings.insert("sid".into(), "no".into());
    c.apply(Command::CycleAudio);
    assert_eq!(c.view.notice, "오디오 트랙 없음");
    c.apply(Command::CycleSubtitle);
    assert_eq!(c.view.notice, "자막 없음");

    // An engine that reports no track list at all is the same as an empty one.
    let mut c = Controller::new(FakeEngine::default());
    c.open(&files[0]);
    c.apply(Command::CycleSubtitle);
    assert_eq!(c.view.notice, "자막 없음");
    c.apply(Command::CycleAudio);
    assert_eq!(c.view.notice, "오디오 트랙 없음");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn track_toast_does_not_cover_the_opening_line() {
    let (dir, files) = folder("tracks-opening");
    let mut c = Controller::new(FakeEngine::default());
    c.open(&files[0]);
    tracks(&mut c.engine, &["video", "audio", "audio", "sub"]);
    selected(&mut c.engine, "audio", "2", "", "kor");
    selected(&mut c.engine, "sub", "1", "", "kor");
    c.view.opening = true;
    c.view.notice = "여는 중…".into();
    c.apply(Command::CycleAudio);
    assert_eq!(c.view.notice, "여는 중…");
    c.apply(Command::CycleSubtitle);
    assert_eq!(c.view.notice, "여는 중…");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn track_keys_do_nothing_before_media() {
    let mut c = Controller::new(FakeEngine::default());
    tracks(&mut c.engine, &["audio", "sub"]);
    selected(&mut c.engine, "audio", "1", "", "");
    selected(&mut c.engine, "sub", "1", "", "");
    assert_eq!(c.apply(Command::CycleAudio), None);
    assert_eq!(c.apply(Command::CycleSubtitle), None);
    assert!(c.engine.commands.is_empty(), "{:?}", c.engine.commands);
    assert!(c.view.notice.is_empty(), "{}", c.view.notice);
}

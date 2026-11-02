//! Volume and speed keys: steps, ranges, mute, and what the bar and toast show.
use super::folder;
use super::support::FakeEngine;
use kinekino_player_ui::{Command, Controller, Session, meari_props};

#[test]
fn arrow_keys_step_the_volume_and_m_mutes() {
    use kinekino_player_ui::shortcut;
    use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};
    let none = ModifiersState::empty();
    let key = |k| PhysicalKey::Code(k);
    assert_eq!(
        shortcut(none, key(KeyCode::ArrowUp), false),
        Some(Command::VolumeUp)
    );
    assert_eq!(
        shortcut(none, key(KeyCode::ArrowDown), false),
        Some(Command::VolumeDown)
    );
    // Holding an arrow keeps stepping; holding M must not flicker mute on and off.
    assert_eq!(
        shortcut(none, key(KeyCode::ArrowUp), true),
        Some(Command::VolumeUp),
        "auto-repeat steps"
    );
    assert_eq!(
        shortcut(none, key(KeyCode::KeyM), false),
        Some(Command::Mute)
    );
    assert_eq!(
        shortcut(none, key(KeyCode::KeyM), true),
        None,
        "auto-repeat"
    );
    for mods in [
        ModifiersState::CONTROL,
        ModifiersState::ALT,
        ModifiersState::SHIFT,
    ] {
        assert_eq!(shortcut(mods, key(KeyCode::ArrowUp), false), None);
        assert_eq!(shortcut(mods, key(KeyCode::KeyM), false), None);
    }
}

#[test]
fn volume_steps_are_five_and_stay_within_0_to_100() {
    use kinekino_player_ui::VOLUME_STEP;
    assert_eq!(VOLUME_STEP, 5.0);
    let mut c = Controller::new(FakeEngine::default());
    assert_eq!(c.view.volume, 100.0, "starts full");
    c.apply(Command::VolumeUp);
    assert_eq!(c.view.volume, 100.0, "no boost past 100");
    for _ in 0..3 {
        c.apply(Command::VolumeDown);
    }
    assert_eq!(c.view.volume, 85.0);
    assert_eq!(c.engine.commands.last().unwrap(), "set volume 85");
    c.apply(Command::Volume(3.0));
    c.apply(Command::VolumeDown);
    assert_eq!(c.view.volume, 0.0, "clamped at silence");
    c.apply(Command::VolumeDown);
    assert_eq!(c.view.volume, 0.0);
    c.apply(Command::Volume(98.0));
    c.apply(Command::VolumeUp);
    assert_eq!(c.view.volume, 100.0, "clamped at full");
    // A slider value off the step grid moves by one step from where it is.
    c.apply(Command::Volume(42.0));
    c.apply(Command::VolumeUp);
    assert_eq!(c.view.volume, 47.0);
    // Out-of-range values never reach the engine.
    c.apply(Command::Volume(250.0));
    assert_eq!(c.view.volume, 100.0);
    c.apply(Command::Volume(-4.0));
    assert_eq!(c.view.volume, 0.0);
    assert!(
        c.engine
            .commands
            .iter()
            .filter(|s| s.starts_with("set volume"))
            .all(|s| {
                let v: f64 = s["set volume ".len()..].parse().unwrap();
                (0.0..=100.0).contains(&v)
            })
    );
}

#[test]
fn volume_keys_unmute_upward_and_show_on_the_bar() {
    let (dir, files) = folder("volume");
    let mut c = Controller::new(FakeEngine::default());
    c.open(&files[0]);
    let prop = |c: &Controller<FakeEngine>, k: &str| format!("{:?}", meari_props(&c.view)[k]);
    c.apply(Command::VolumeDown);
    assert!(prop(&c, "volume").contains("95"), "{}", prop(&c, "volume"));
    c.apply(Command::Mute);
    assert!(c.view.muted);
    assert!(prop(&c, "volumeIcon").contains("volume-zero"));
    assert!(prop(&c, "volume").contains("95"), "muting keeps the level");
    c.apply(Command::VolumeDown);
    assert!(c.view.muted, "turning down while muted stays muted");
    assert_eq!(c.view.volume, 90.0);
    c.apply(Command::VolumeUp);
    assert!(!c.view.muted, "turning up unmutes");
    assert_eq!(c.view.volume, 95.0);
    assert_eq!(c.engine.commands.last().unwrap(), "set mute no");
    assert!(!prop(&c, "volumeIcon").contains("volume-zero"));
    // The mounted bar follows: the slider shows the new level.
    let (_, mut doc) = Session::mount(&c.view).unwrap();
    c.apply(Command::VolumeDown);
    doc.update_module_props(meari_props(&c.view)).unwrap();
    let slider = doc.elements().iter().find(|e| e.id == "volume").unwrap();
    assert_eq!(
        slider.attributes.get("value").map(String::as_str),
        Some("90")
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn c_x_z_slow_down_speed_up_and_reset_the_speed() {
    use kinekino_player_ui::shortcut;
    use winit::keyboard::{KeyCode as K, ModifiersState as M, PhysicalKey::Code};
    let key = |m, k, r| shortcut(m, Code(k), r);
    assert_eq!(key(M::empty(), K::KeyC, false), Some(Command::SpeedDown));
    assert_eq!(
        key(M::empty(), K::KeyX, true),
        Some(Command::SpeedUp),
        "repeat steps"
    );
    assert_eq!(key(M::empty(), K::KeyZ, false), Some(Command::SpeedReset));
    assert_eq!(key(M::empty(), K::KeyZ, true), None, "auto-repeat");
    assert_eq!(key(M::CONTROL, K::KeyX, false), None);
}

#[test]
fn speed_steps_by_a_tenth_within_the_engine_range_and_shows_a_toast() {
    use kinekino_player_ui::{SPEED_MAX, SPEED_MIN, SPEED_STEP};
    assert_eq!((SPEED_STEP, SPEED_MIN, SPEED_MAX), (0.1, 0.25, 4.0));
    let (dir, files) = folder("speed");
    let mut c = Controller::new(FakeEngine::default());
    assert_eq!(c.view.speed, 1.0, "starts at normal speed");
    c.open(&files[0]);
    c.apply(Command::SpeedUp);
    c.apply(Command::SpeedUp);
    assert_eq!(
        (c.view.speed, c.engine.commands.last().unwrap().as_str()),
        (1.2, "set speed 1.20")
    );
    assert_eq!(c.view.notice, "재생 속도 1.2x");
    (0..12).for_each(|_| _ = c.apply(Command::SpeedDown));
    assert_eq!(
        (c.view.speed, c.view.notice.as_str()),
        (0.25, "재생 속도 0.25x"),
        "floor"
    );
    (0..50).for_each(|_| _ = c.apply(Command::SpeedUp));
    assert_eq!(c.view.speed, 4.0, "ceiling");
    c.apply(Command::SpeedReset);
    assert_eq!(
        (c.view.speed, c.view.notice.as_str()),
        (1.0, "재생 속도 1.0x")
    );
    let sent = c
        .engine
        .commands
        .iter()
        .filter_map(|s| s.strip_prefix("set speed "));
    assert!(
        sent.map(|v| v.parse::<f64>().unwrap())
            .all(|v| (0.25..=4.0).contains(&v))
    );
    c.apply(Command::SpeedUp);
    c.apply(Command::Next);
    assert_eq!(c.view.speed, 1.1, "the next item keeps the chosen speed");
    std::fs::remove_dir_all(dir).unwrap();
}

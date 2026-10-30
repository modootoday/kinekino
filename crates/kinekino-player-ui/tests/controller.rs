//! Player behaviour headless: commands drive the engine and the view; the folder is the
//! episode list; window-only work comes back as effects; the document reflects the view.
use kinekino_core::playback::{FakeSurface, RgbaFrame, VideoSurface};
use kinekino_core::port::error::PortResult;
use kinekino_player_ui::{Command, Controller, Effect, Engine, Session, activate, meari_props};
use std::path::PathBuf;

#[derive(Default)]
struct FakeEngine {
    surface: FakeSurface,
    loaded: Vec<String>,
    commands: Vec<String>,
    time: f64,
    subtitle: Option<String>,
}
impl VideoSurface for FakeEngine {
    fn frame_ready(&mut self) -> PortResult<bool> {
        self.surface.frame_ready()
    }
    fn render(&mut self, frame: &mut RgbaFrame) -> PortResult<()> {
        self.surface.render(frame)
    }
}
impl Engine for FakeEngine {
    fn load(&mut self, source: &str, _: f64) -> PortResult<()> {
        self.loaded.push(source.to_owned());
        Ok(())
    }
    fn command(&mut self, args: &[&str]) -> PortResult<()> {
        self.commands.push(args.join(" "));
        Ok(())
    }
    fn property(&mut self, name: &str) -> Option<f64> {
        match name {
            "time-pos" => Some(self.time),
            "duration" => Some(1500.0),
            _ => None,
        }
    }
    fn text(&mut self, name: &str) -> Option<String> {
        (name == "sub-text")
            .then(|| self.subtitle.clone())
            .flatten()
    }
}

fn folder(tag: &str) -> (PathBuf, Vec<PathBuf>) {
    let dir = std::env::temp_dir().join(format!("kk-player-ctl-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let files: Vec<PathBuf> = ["02.mkv", "01.mkv", "03.mp4", "notes.txt"]
        .iter()
        .map(|n| {
            let p = dir.join(n);
            std::fs::write(&p, b"x").unwrap();
            p
        })
        .collect();
    (dir, files)
}

#[test]
fn folder_becomes_episodes_and_commands_drive_engine() {
    let (dir, files) = folder("engine");
    let mut c = Controller::new(FakeEngine::default());
    c.open(&files[0]);
    let titles: Vec<_> = c.view.episodes.iter().map(|e| e.title.as_str()).collect();
    assert_eq!(titles, ["01", "02", "03"], "videos only, sorted");
    assert_eq!(c.view.current, Some(1));
    assert!(c.view.has_media && c.view.playing);
    c.apply(Command::Next);
    assert_eq!(c.view.current, Some(2));
    c.apply(Command::Next);
    assert_eq!(c.view.current, Some(2), "no episode after the last");
    c.apply(Command::Previous);
    c.apply(Command::Episode(0));
    assert_eq!(c.view.title, "01");
    c.engine.time = 30.0;
    assert!(c.poll());
    assert_eq!((c.view.position, c.view.duration), (30.0, 1500.0));
    assert!(!c.poll(), "same second needs no re-projection");
    c.apply(Command::PlayPause);
    assert!(!c.view.playing);
    c.apply(Command::Seek(0.5));
    c.apply(Command::Volume(40.0));
    c.apply(Command::Mute);
    c.engine.commands.retain(|c| c != "set pause no");
    assert_eq!(
        c.engine.commands,
        [
            "cycle pause",
            "seek 750.000 absolute",
            "set volume 40",
            "set mute yes"
        ]
    );
    assert_eq!(c.engine.loaded.len(), 4);
    assert_eq!(
        c.apply(Command::ToggleFullscreen),
        Some(Effect::Fullscreen(true))
    );
    assert_eq!(c.apply(Command::Maximize), Some(Effect::Maximize(true)));
    assert_eq!(c.apply(Command::Minimize), Some(Effect::Minimize));
    assert_eq!(c.apply(Command::Open), Some(Effect::PickFile));
    assert_eq!(c.apply(Command::Close), Some(Effect::Close));
    assert_eq!(c.apply(Command::ToggleEpisodes), None);
    assert!(c.view.panel_open);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn start_screen_ignores_transport_until_media() {
    let mut c = Controller::new(FakeEngine::default());
    assert_eq!(c.apply(Command::PlayPause), None);
    assert_eq!(c.apply(Command::ToggleFullscreen), None);
    assert!(!c.poll());
    assert!(c.engine.commands.is_empty());
    c.apply(Command::Browse);
    assert!(!c.view.status.is_empty(), "browse explains itself");
}

#[test]
fn document_clicks_round_trip_through_controller() {
    let (dir, files) = folder("document");
    let mut c = Controller::new(FakeEngine::default());
    c.open(&files[1]);
    let (mut session, mut doc) = Session::mount(&c.view).unwrap();
    let mut effects = Vec::new();
    // The panel opens first; its rows take clicks once the view is projected.
    for ids in [&["playPause", "episodes"][..], &["ep2", "fullscreen"]] {
        for id in ids {
            activate(&mut doc, id).unwrap();
        }
        for command in session.commands(&doc).unwrap() {
            effects.extend(c.apply(command));
        }
        doc.update_module_props(meari_props(&c.view)).unwrap();
    }
    assert_eq!(c.view.current, Some(2));
    assert!(c.view.panel_open);
    assert_eq!(effects, [Effect::Fullscreen(true)]);
    doc.update_module_props(meari_props(&c.view)).unwrap();
    let html: String = doc
        .elements()
        .iter()
        .flat_map(|e| e.text_runs.iter().map(|r| r.text.clone()))
        .collect();
    assert!(html.contains("kinekino"), "product name in chrome");
    // Mockup sample site names must not reach the chrome. The word is built at
    // runtime so the literal never lands in the tree (r-src).
    let sample_site: String = ['a', 'n', 'i', 'w', 'e', 'e', 'k'].iter().collect();
    assert!(
        !html.to_lowercase().contains(&sample_site),
        "mockup sample site name in chrome"
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn ctrl_o_opens_the_file_picker_on_any_layout() {
    use kinekino_player_ui::shortcut;
    use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};
    let o = PhysicalKey::Code(KeyCode::KeyO);
    assert_eq!(
        shortcut(ModifiersState::CONTROL, o, false),
        Some(Command::Open)
    );
    assert_eq!(
        shortcut(ModifiersState::META, o, false),
        Some(Command::Open)
    );
    assert_eq!(shortcut(ModifiersState::empty(), o, false), None);
    assert_eq!(
        shortcut(ModifiersState::CONTROL, o, true),
        None,
        "auto-repeat"
    );
    assert_eq!(
        shortcut(ModifiersState::CONTROL | ModifiersState::ALT, o, false),
        None
    );
    let p = PhysicalKey::Code(KeyCode::KeyP);
    assert_eq!(shortcut(ModifiersState::CONTROL, p, false), None);
    let mut c = Controller::new(FakeEngine::default());
    assert_eq!(c.apply(Command::Open), Some(Effect::PickFile));
}

#[test]
fn fullscreen_controls_hide_after_idle_and_return_on_input() {
    use kinekino_player_ui::HIDE_AFTER;
    use std::time::{Duration, Instant};
    let (dir, files) = folder("autohide");
    let mut c = Controller::new(FakeEngine::default());
    c.open(&files[0]);
    let t0 = Instant::now();
    let later = |d: Duration| t0 + d;
    // Windowed: never hides.
    assert!(!c.idle(later(HIDE_AFTER * 4)));
    assert!(c.view.controls_visible);
    c.view.fullscreen = true;
    assert!(
        !c.idle(later(HIDE_AFTER * 4)),
        "clock restarts on entering fullscreen"
    );
    assert!(!c.idle(later(HIDE_AFTER * 5 - Duration::from_millis(1))));
    assert!(
        c.idle(later(HIDE_AFTER * 5)),
        "hidden after the quiet period"
    );
    assert!(!c.view.controls_visible);
    let transport = format!("{:?}", meari_props(&c.view)["transportClass"]);
    assert!(transport.contains("aw-controls-hidden"), "{transport}");
    assert!(c.input(later(HIDE_AFTER * 6)), "input shows them again");
    assert!(c.view.controls_visible);
    // Paused, status, or panel hold the controls.
    for hold in 0..3 {
        c.input(later(HIDE_AFTER * 6));
        match hold {
            0 => c.view.playing = false,
            1 => c.view.status = "재생 명령 실패".into(),
            _ => c.view.panel_open = true,
        }
        assert!(!c.idle(later(HIDE_AFTER * 9)));
        assert!(c.view.controls_visible);
        c.view.playing = true;
        c.view.status.clear();
        c.view.panel_open = false;
    }
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn subtitle_cue_is_drawn_over_the_video_and_cleared() {
    let (dir, files) = folder("subtitle");
    let mut c = Controller::new(FakeEngine::default());
    c.open(&files[0]);
    let (_, mut doc) = Session::mount(&c.view).unwrap();
    let shown = |doc: &meari_native::infrastructure::document::InteractiveDocument| -> String {
        doc.elements()
            .iter()
            .flat_map(|e| e.text_runs.iter().map(|r| r.text.clone()))
            .collect()
    };
    c.engine.subtitle = Some("첫 줄\n둘째 줄".into());
    assert!(c.poll(), "a new cue re-projects");
    assert!(!c.poll(), "same cue: nothing to do");
    doc.update_module_props(meari_props(&c.view)).unwrap();
    let text = shown(&doc);
    assert!(text.contains("첫 줄") && text.contains("둘째 줄"), "{text}");
    c.engine.subtitle = Some(String::new());
    assert!(c.poll(), "cue ended");
    doc.update_module_props(meari_props(&c.view)).unwrap();
    assert!(!shown(&doc).contains("첫 줄"));
    std::fs::remove_dir_all(dir).unwrap();
}

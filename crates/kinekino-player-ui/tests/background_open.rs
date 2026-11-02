//! KB-003: opening a slow source must not hold the UI thread. The open returns at once,
//! the view shows that it is opening, and a later choice or Esc drops the earlier open.
use kinekino_core::playback::{FakeSurface, RgbaFrame, VideoSurface};
use kinekino_core::port::error::{ErrorCode, PortError, PortResult};
use kinekino_player_ui::{Command, Controller, Engine};
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// A slow source: the last open becomes ready (or fails) only after `delay`.
/// `stop` is recorded but does not end the open, so a controller that kept waiting on a
/// cancelled open would still see it become ready.
struct SlowEngine {
    surface: FakeSurface,
    delay: Duration,
    fail: bool,
    opened: Option<(String, Instant)>,
    loaded: Vec<String>,
    commands: Vec<String>,
}

impl SlowEngine {
    fn new(delay: Duration) -> Self {
        Self {
            surface: FakeSurface::default(),
            delay,
            fail: false,
            opened: None,
            loaded: Vec::new(),
            commands: Vec::new(),
        }
    }
}

impl VideoSurface for SlowEngine {
    fn frame_ready(&mut self) -> PortResult<bool> {
        self.surface.frame_ready()
    }
    fn render(&mut self, frame: &mut RgbaFrame) -> PortResult<()> {
        self.surface.render(frame)
    }
}

impl Engine for SlowEngine {
    fn begin_load(&mut self, source: &str, _: f64) -> PortResult<()> {
        self.loaded.push(source.to_owned());
        self.opened = Some((source.to_owned(), Instant::now()));
        Ok(())
    }
    fn load_ready(&mut self) -> PortResult<bool> {
        let Some((_, at)) = &self.opened else {
            return Ok(true);
        };
        if at.elapsed() < self.delay {
            return Ok(false);
        }
        if self.fail {
            return Err(PortError::new(
                ErrorCode::Timeout,
                "no frame from the source",
            ));
        }
        Ok(true)
    }
    fn command(&mut self, args: &[&str]) -> PortResult<()> {
        self.commands.push(args.join(" "));
        Ok(())
    }
    fn property(&mut self, name: &str) -> Option<f64> {
        match name {
            "time-pos" => Some(0.0),
            "duration" => Some(60.0),
            "eof-reached" => Some(0.0),
            _ => None,
        }
    }
}

fn folder(tag: &str) -> (PathBuf, Vec<PathBuf>) {
    let dir = std::env::temp_dir().join(format!("kk-player-bgopen-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let files = ["01.mkv", "02.mkv", "03.mkv"]
        .iter()
        .map(|n| {
            let p = dir.join(n);
            std::fs::write(&p, b"x").unwrap();
            p
        })
        .collect();
    (dir, files)
}

/// Polls like the window tick until `done` holds or 3 s pass.
fn poll_until(c: &mut Controller<SlowEngine>, done: impl Fn(&Controller<SlowEngine>) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while !done(c) && Instant::now() < deadline {
        c.poll();
        std::thread::sleep(Duration::from_millis(10));
    }
}

const SLOW: Duration = Duration::from_millis(400);

#[test]
fn a_slow_open_returns_at_once_and_shows_that_it_is_opening() {
    let (dir, files) = folder("start");
    let mut c = Controller::new(SlowEngine::new(SLOW));
    let started = Instant::now();
    c.open(&files[1]);
    assert!(
        started.elapsed() < Duration::from_millis(150),
        "open held the caller for {:?}",
        started.elapsed()
    );
    assert!(c.view.opening, "the view must say it is opening");
    assert!(!c.view.playing && !c.view.has_media);
    assert!(
        !c.view.status.is_empty(),
        "the start screen shows the opening state"
    );
    assert_eq!(
        c.view.playlist.len(),
        3,
        "the queue is known before the open ends"
    );
    poll_until(&mut c, |c| c.view.playing);
    assert!(c.view.playing && c.view.has_media && !c.view.opening);
    assert_eq!((c.view.current, c.view.title.as_str()), (Some(1), "02"));
    assert!(c.view.status.is_empty(), "the opening line goes away");
    assert!(c.engine.commands.contains(&"set pause no".to_owned()));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn opening_over_a_playing_video_shows_a_notice_and_the_last_choice_wins() {
    let (dir, files) = folder("switch");
    let mut c = Controller::new(SlowEngine::new(Duration::ZERO));
    c.open(&files[0]);
    c.finish();
    assert!(c.view.playing && c.view.current == Some(0));
    c.engine.delay = SLOW;
    c.apply(Command::Item(1));
    assert!(c.view.opening && !c.view.notice.is_empty());
    std::thread::sleep(Duration::from_millis(100));
    c.apply(Command::Item(2));
    poll_until(&mut c, |c| !c.view.opening);
    assert_eq!((c.view.current, c.view.title.as_str()), (Some(2), "03"));
    assert!(c.view.notice.is_empty(), "the opening notice goes away");
    assert_eq!(c.engine.loaded.len(), 3);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn esc_cancels_an_open_and_its_late_result_is_dropped() {
    let (dir, files) = folder("cancel");
    let mut c = Controller::new(SlowEngine::new(SLOW));
    c.open(&files[0]);
    assert!(c.view.opening);
    c.apply(Command::ExitFullscreen);
    assert!(!c.view.opening && c.view.status.is_empty());
    assert!(c.engine.commands.contains(&"stop".to_owned()));
    std::thread::sleep(SLOW + Duration::from_millis(50));
    for _ in 0..5 {
        c.poll();
    }
    assert!(!c.view.playing && !c.view.has_media && c.view.current.is_none());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn a_failed_slow_open_says_why_and_stops_opening() {
    let (dir, files) = folder("fail");
    let mut engine = SlowEngine::new(SLOW);
    engine.fail = true;
    let mut c = Controller::new(engine);
    c.open(&files[0]);
    poll_until(&mut c, |c| !c.view.opening);
    assert!(!c.view.opening && !c.view.playing && !c.view.has_media);
    assert!(
        c.view.status.contains("no frame from the source"),
        "{}",
        c.view.status
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn finish_waits_for_a_slow_open() {
    let (dir, files) = folder("finish");
    let mut c = Controller::new(SlowEngine::new(SLOW));
    c.open(&files[2]);
    c.finish();
    assert!(c.view.playing && !c.view.opening && c.view.current == Some(2));
    std::fs::remove_dir_all(dir).unwrap();
}

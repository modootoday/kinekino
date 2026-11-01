//! The end of a file with the real libmpv engine (not the fake): the view stops playing and
//! auto-next opens the following item. Missing libmpv/ffmpeg fails, never skips.
use kinekino_libmpv::LibMpvSurface;
use kinekino_player_ui::{Command, Controller};
use std::path::{Path, PathBuf};
use std::process::Command as Process;
use std::time::{Duration, Instant};

/// A folder with two one-second clips, `01.mkv` and `02.mkv`.
fn clips(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("kk-player-end-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for name in ["01.mkv", "02.mkv"] {
        let status = Process::new("ffmpeg")
            .args(["-y", "-loglevel", "error", "-f", "lavfi"])
            .args(["-i", "color=c=gray:s=64x36:r=25:d=1"])
            .args(["-c:v", "libx264", "-preset", "ultrafast"])
            .arg(dir.join(name))
            .status()
            .expect("ffmpeg is required");
        assert!(status.success());
    }
    dir
}

fn controller(first: &Path) -> Controller<LibMpvSurface> {
    let engine = LibMpvSurface::new(None, true)
        .expect("libmpv is required ($KINEKINO_LIBMPV or soname on the loader path)");
    let mut c = Controller::new(engine);
    c.open(first);
    assert!(c.view.playing, "{}", c.view.status);
    c
}

/// Polls until `done` holds or 10 s pass.
fn poll_until(c: &mut Controller<LibMpvSurface>, done: impl Fn(&Controller<LibMpvSurface>) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !done(c) && Instant::now() < deadline {
        c.poll();
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn the_end_of_a_file_stops_the_view() {
    let dir = clips("stop");
    let mut c = controller(&dir.join("02.mkv"));
    poll_until(&mut c, |c| !c.view.playing);
    assert!(!c.view.playing, "the play button still shows pause at the end");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn auto_next_opens_the_following_file_at_the_end() {
    let dir = clips("next");
    let mut c = controller(&dir.join("01.mkv"));
    c.apply(Command::ToggleAutoNext);
    poll_until(&mut c, |c| c.view.current == Some(1));
    assert_eq!(c.view.current, Some(1), "auto-next never fired");
    assert!(c.view.playing);
    std::fs::remove_dir_all(dir).unwrap();
}

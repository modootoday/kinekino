//! Headless mpv tests (`--vo=null --ao=null`) on a clip generated locally with ffmpeg.
//! Requires `mpv` and `ffmpeg` (see docs/_src mpv install notes); missing tools fail, never skip.
use super::*;
use kinekino_core::playback::check_backend;
use std::sync::OnceLock;

fn clip() -> &'static Path {
    static CLIP: OnceLock<PathBuf> = OnceLock::new();
    CLIP.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("kinekino-mpv-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let out = dir.join("clip.mkv");
        let status = Command::new("ffmpeg")
            .args([
                "-y",
                "-loglevel",
                "error",
                "-f",
                "lavfi",
                "-i",
                "testsrc=size=160x120:rate=25:duration=4",
            ])
            .args([
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:duration=4",
                "-shortest",
            ])
            .args([
                "-c:v",
                "libx264",
                "-preset",
                "ultrafast",
                "-g",
                "5",
                "-c:a",
                "aac",
            ])
            .arg(&out)
            .status()
            .expect("ffmpeg is required for mpv tests");
        assert!(status.success(), "ffmpeg failed to generate the test clip");
        out
    })
}

fn backend() -> MpvBackend {
    MpvBackend::spawn(&MpvOptions {
        binary: None,
        headless: true,
    })
    .expect("mpv is required (PATH or $KINEKINO_MPV)")
}

#[test]
fn mpv_meets_backend_contract() {
    check_backend(&mut backend(), clip().to_str().unwrap()).unwrap();
}

#[test]
fn reports_duration_and_plays_to_eof() {
    let mut b = backend();
    b.open(clip().to_str().unwrap(), 3_500_000).unwrap();
    let s = b.status().unwrap();
    assert!(
        s.duration_us.unwrap().abs_diff(4_000_000) < 200_000,
        "{s:?}"
    );
    assert!(s.source.unwrap().ends_with("clip.mkv"));
    b.control(&PlaybackControl::Play).unwrap();
    let start = Instant::now();
    while !b.status().unwrap().eof {
        assert!(start.elapsed() < TIMEOUT, "never reached eof");
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn rejects_missing_source_and_use_before_open() {
    let mut b = backend();
    assert_eq!(b.status().unwrap_err().code, ErrorCode::Unavailable);
    assert_eq!(
        b.control(&PlaybackControl::Play).unwrap_err().code,
        ErrorCode::Unavailable
    );
    let missing = std::env::temp_dir().join("kinekino-mpv-does-not-exist.mkv");
    assert_eq!(
        b.open(missing.to_str().unwrap(), 0).unwrap_err().code,
        ErrorCode::InvalidInput
    );
    assert_eq!(b.open("", 0).unwrap_err().code, ErrorCode::InvalidInput);
}

#[test]
fn explicit_binary_path_is_checked() {
    let bogus = Path::new("/nonexistent/mpv");
    assert_eq!(
        find_mpv(Some(bogus)).unwrap_err().code,
        ErrorCode::Unavailable
    );
    let found = find_mpv(None).unwrap();
    assert_eq!(find_mpv(Some(&found)).unwrap(), found);
}

#[test]
fn resume_point_round_trips_through_mpv_and_file_store() {
    use kinekino_core::playback::{FileResumeStore, ResumePolicy, ResumeStore, ResumingPlayback};
    use kinekino_media_contracts::api_values::DecimalU64;
    let file = clip().with_file_name("resume.json");
    let policy = ResumePolicy {
        min_position_us: 500_000,
        finished_tail_us: 500_000,
        finished_fraction: 0.9,
        rewind_us: 1_000_000,
    };
    let mut p = ResumingPlayback::new(backend(), FileResumeStore::at(&file), policy);
    assert_eq!(
        p.open("synthetic-clip", clip().to_str().unwrap()).unwrap(),
        0
    );
    p.control(&PlaybackControl::Seek {
        position_us: DecimalU64(2_500_000),
    })
    .unwrap();
    p.stop().unwrap();
    let saved = FileResumeStore::at(&file)
        .get("synthetic-clip")
        .unwrap()
        .unwrap();
    assert!(saved.position_us.abs_diff(2_500_000) < 100_000, "{saved:?}");
    let mut again = ResumingPlayback::new(backend(), FileResumeStore::at(&file), policy);
    let start = again
        .open("synthetic-clip", clip().to_str().unwrap())
        .unwrap();
    assert!(start.abs_diff(1_500_000) < 100_000);
    assert!(again.status().unwrap().position_us.abs_diff(1_500_000) < 150_000);
}

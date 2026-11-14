//! Headless libmpv software-render tests on an ffmpeg-generated clip (no display, `ao=null`).
//! Missing libmpv/ffmpeg fails, never skips.
use super::*;
use kinekino_core::playback::check_surface;
use std::path::Path;
use std::process::Command;
use std::sync::OnceLock;

/// Left half pure red, right half pure blue, 4 s.
fn clip() -> &'static Path {
    static CLIP: OnceLock<PathBuf> = OnceLock::new();
    CLIP.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("kinekino-libmpv-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let out = dir.join("halves.mkv");
        let status = Command::new("ffmpeg")
            .args([
                "-y",
                "-loglevel",
                "error",
                "-f",
                "lavfi",
                "-i",
                "color=c=red:s=64x36:r=25:d=4",
            ])
            .args(["-f", "lavfi", "-i", "color=c=blue:s=64x36:r=25:d=4"])
            .args([
                "-filter_complex",
                "[0][1]hstack",
                "-c:v",
                "libx264",
                "-preset",
                "ultrafast",
                "-qp",
                "0",
                "-pix_fmt",
                "yuv444p",
            ])
            .arg(&out)
            .status()
            .expect("ffmpeg is required");
        assert!(status.success());
        out
    })
}

fn surface() -> LibMpvSurface {
    let mut s = LibMpvSurface::new(None, true)
        .expect("libmpv is required ($KINEKINO_LIBMPV or soname on the loader path)");
    s.open(clip().to_str().unwrap(), 1.0).unwrap();
    s
}

#[test]
fn libmpv_surface_meets_contract() {
    check_surface(&mut surface()).unwrap();
}

#[test]
fn renders_decoded_colours_and_letterboxes() {
    let mut s = surface();
    let mut f = RgbaFrame::new(128, 36).unwrap();
    s.render(&mut f).unwrap();
    let near = |p: [u8; 4], want: [u8; 3]| p[..3].iter().zip(want).all(|(a, b)| a.abs_diff(b) < 40);
    assert!(near(f.pixel(10, 18), [255, 0, 0]), "{:?}", f.pixel(10, 18));
    assert!(
        near(f.pixel(117, 18), [0, 0, 255]),
        "{:?}",
        f.pixel(117, 18)
    );
    // Taller target than the 128x36 source: top/bottom bands must be black.
    let mut tall = RgbaFrame::new(128, 128).unwrap();
    s.render(&mut tall).unwrap();
    assert!(near(tall.pixel(64, 2), [0, 0, 0]) && near(tall.pixel(10, 64), [255, 0, 0]));
}

#[test]
fn bad_source_and_bad_library_are_errors() {
    let mut s = LibMpvSurface::new(None, true).unwrap();
    let missing = std::env::temp_dir().join("kinekino-libmpv-missing.mkv");
    assert_eq!(
        s.open(missing.to_str().unwrap(), 0.0).unwrap_err().code,
        ErrorCode::InvalidInput
    );
    let err = LibMpvSurface::new(Some("/nonexistent/libmpv.so".into()), true)
        .err()
        .unwrap();
    assert_eq!(err.code, ErrorCode::Unavailable);
}

/// Repeated create/render/drop on parallel threads: teardown must not unload libmpv
/// under helper threads still finishing (former intermittent SIGSEGV).
#[test]
fn surfaces_tear_down_repeatedly_across_threads() {
    let workers: Vec<_> = (0..3)
        .map(|_| {
            std::thread::spawn(|| {
                for _ in 0..3 {
                    let mut s = surface();
                    let mut f = RgbaFrame::new(32, 18).unwrap();
                    s.render(&mut f).unwrap();
                }
            })
        })
        .collect();
    for w in workers {
        w.join().unwrap();
    }
}

/// A source that accepts connections and never answers (a slow stream server).
struct SilentServer {
    url: String,
    stop: std::sync::mpsc::Sender<()>,
    worker: Option<std::thread::JoinHandle<()>>,
}

impl SilentServer {
    fn start() -> Self {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!("http://{}/slow.mp4", listener.local_addr().unwrap());
        let (stop, stopped) = std::sync::mpsc::channel::<()>();
        let worker = std::thread::spawn(move || {
            let mut held = Vec::new();
            while stopped.try_recv().is_err() {
                if let Ok((conn, _)) = listener.accept() {
                    held.push(conn);
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        });
        Self {
            url,
            stop,
            worker: Some(worker),
        }
    }
}

impl Drop for SilentServer {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        if let Some(w) = self.worker.take() {
            let _ = w.join();
        }
    }
}

/// KB-003: starting an open returns at once; a later open replaces a slow one and only the
/// later source reports ready.
#[test]
fn begin_open_returns_before_a_slow_source_answers() {
    let server = SilentServer::start();
    let mut s = LibMpvSurface::new(None, true).unwrap();
    let started = Instant::now();
    s.begin_open(&server.url, 0.0).unwrap();
    assert!(
        started.elapsed() < Duration::from_millis(500),
        "begin_open held the caller for {:?}",
        started.elapsed()
    );
    let quiet = Instant::now() + Duration::from_millis(800);
    while Instant::now() < quiet {
        assert!(!s.open_ready().unwrap(), "a silent source cannot be ready");
        std::thread::sleep(Duration::from_millis(20));
    }
    let clip = clip().to_str().unwrap();
    s.begin_open(clip, 1.0).unwrap();
    let deadline = Instant::now() + Duration::from_secs(15);
    while !s.open_ready().unwrap() {
        assert!(
            Instant::now() < deadline,
            "the replacing open never became ready"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(s.property_string("path").as_deref(), Some(clip));
    let mut f = RgbaFrame::new(64, 36).unwrap();
    s.render(&mut f).unwrap();
}

#[test]
fn begin_open_of_a_missing_file_fails_later_not_at_once() {
    let mut s = LibMpvSurface::new(None, true).unwrap();
    let missing = std::env::temp_dir().join("kinekino-libmpv-missing-bg.mkv");
    s.begin_open(missing.to_str().unwrap(), 0.0).unwrap();
    let deadline = Instant::now() + Duration::from_secs(15);
    let err = loop {
        match s.open_ready() {
            Ok(false) => {
                assert!(Instant::now() < deadline, "no answer for a missing file");
                std::thread::sleep(Duration::from_millis(10));
            }
            Ok(true) => panic!("a missing file cannot be ready"),
            Err(e) => break e,
        }
    };
    assert_eq!(err.code, ErrorCode::InvalidInput);
}

/// M1-8: an ASS track reports its codec as `ass`, stays off the frame by default and is
/// drawn into the frame once `sub-visibility` is on (a vector box needs no fonts).
#[test]
fn ass_track_is_drawn_into_the_frame_only_when_visible() {
    let mut s = surface();
    let dir = clip().parent().unwrap();
    let ass = dir.join("box.ass");
    std::fs::write(
        &ass,
        "[Script Info]\nScriptType: v4.00+\nPlayResX: 128\nPlayResY: 36\n\n\
         [V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, BorderStyle, Outline, Shadow, Alignment\n\
         Style: Default,sans,10,&H00FFFFFF,1,0,0,7\n\n\
         [Events]\nFormat: Layer, Start, End, Style, Text\n\
         Dialogue: 0,0:00:00.00,0:00:10.00,Default,{\\pos(0,0)\\p1}m 0 0 l 64 0 64 36 0 36{\\p0}\n",
    )
    .unwrap();
    s.command(&["sub-add", ass.to_str().unwrap(), "select"])
        .unwrap();
    assert_eq!(
        s.property_string("current-tracks/sub/codec").as_deref(),
        Some("ass")
    );
    let white = |p: [u8; 4]| p[..3].iter().all(|v| *v > 200);
    let mut f = RgbaFrame::new(128, 36).unwrap();
    s.render(&mut f).unwrap();
    assert!(
        !white(f.pixel(10, 18)),
        "hidden by default: {:?}",
        f.pixel(10, 18)
    );
    s.command(&["set", "sub-visibility", "yes"]).unwrap();
    s.render(&mut f).unwrap();
    assert!(
        white(f.pixel(10, 18)),
        "drawn when visible: {:?}",
        f.pixel(10, 18)
    );
    assert!(!white(f.pixel(117, 18)), "only where the box is");
}

#[test]
fn windows_load_flags_search_the_dll_folder_only_for_absolute_paths() {
    let absolute = std::env::temp_dir().join("mpv").join("libmpv-2.dll");
    assert!(absolute.is_absolute());
    assert_eq!(crate::ffi::windows_load_flags(&absolute), 0x1100);
    assert_eq!(crate::ffi::windows_load_flags(Path::new("libmpv-2.dll")), 0);
    assert_eq!(
        crate::ffi::windows_load_flags(Path::new("mpv/libmpv-2.dll")),
        0
    );
}

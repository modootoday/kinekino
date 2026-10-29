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

//! VideoSurface → Meari texture, verified on the headless raster: frames land inside the
//! video box, re-upload only when a new frame is ready, and real libmpv frames show up.
//! Missing libmpv/ffmpeg fails, never skips.
use kinekino_core::playback::{FakeSurface, VideoSurface};
use kinekino_player_ui::{Session, VideoTexture, View};
use meari_native::application::Renderer;
use meari_native::infrastructure::document::InteractiveDocument;
use std::path::PathBuf;
use std::process::Command;

fn playing() -> View {
    View {
        title: "sample.mkv".into(),
        has_media: true,
        playing: true,
        duration: 4.0,
        ..View::default()
    }
}

fn raster(doc: &mut InteractiveDocument) -> Vec<u8> {
    let viewport = doc.viewport.clone();
    doc.renderer.rasterize(&viewport).unwrap()
}

fn px(rgba: &[u8], x: u32, y: u32) -> [u8; 3] {
    let i = (y * kinekino_player_ui::VIEWPORT.width + x) as usize * 4;
    [rgba[i], rgba[i + 1], rgba[i + 2]]
}

fn near(p: [u8; 3], want: [u8; 3]) -> bool {
    p.iter().zip(want).all(|(a, b)| a.abs_diff(b) < 40)
}

/// Lays out, uploads one frame and returns the raster and the video box size.
fn frame(
    doc: &mut InteractiveDocument,
    tex: &mut VideoTexture,
    s: &mut dyn VideoSurface,
) -> Vec<u8> {
    raster(doc);
    assert!(tex.upload(doc, s).unwrap(), "first upload");
    raster(doc)
}

#[test]
fn fake_frames_paint_the_video_box_and_follow_readiness() {
    let (_, mut doc) = Session::mount(&playing()).unwrap();
    let mut tex = VideoTexture::default();
    let mut fake = FakeSurface::default();
    fake.push([200, 30, 30, 255]);
    let rgba = frame(&mut doc, &mut tex, &mut fake);
    let (w, h) = VideoTexture::target_size(&doc).unwrap().unwrap();
    assert!(w > 600 && h > 300, "video box {w}x{h}");
    assert!(
        near(px(&rgba, w / 2, h / 2), [200, 30, 30]),
        "{:?}",
        px(&rgba, w / 2, h / 2)
    );
    // No new frame: nothing pulled.
    assert!(!tex.upload(&mut doc, &mut fake).unwrap());
    assert_eq!(fake.renders, 1);
    fake.push([30, 200, 30, 255]);
    assert!(tex.upload(&mut doc, &mut fake).unwrap());
    let rgba = raster(&mut doc);
    assert!(
        near(px(&rgba, w / 2, h / 2), [30, 200, 30]),
        "{:?}",
        px(&rgba, w / 2, h / 2)
    );
    assert_eq!(tex.uploads(), 2);
}

#[test]
fn start_screen_has_no_video_box() {
    let (_, mut doc) = Session::mount(&View::default()).unwrap();
    raster(&mut doc);
    assert_eq!(VideoTexture::target_size(&doc).unwrap(), None);
    let mut fake = FakeSurface::default();
    fake.push([1, 2, 3, 255]);
    assert!(!VideoTexture::default().upload(&mut doc, &mut fake).unwrap());
}

/// Left half red, right half blue, 4 s (ffmpeg lavfi; no network, no fixture file).
fn clip() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("kinekino-player-ui-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let out = dir.join("halves.mkv");
    let status = Command::new("ffmpeg")
        .args(["-y", "-loglevel", "error", "-f", "lavfi", "-i"])
        .arg("color=c=red:s=64x36:r=25:d=4")
        .args(["-f", "lavfi", "-i", "color=c=blue:s=64x36:r=25:d=4"])
        .args(["-filter_complex", "[0][1]hstack", "-c:v", "libx264"])
        .args(["-preset", "ultrafast", "-qp", "0", "-pix_fmt", "yuv444p"])
        .arg(&out)
        .status()
        .expect("ffmpeg is required");
    assert!(status.success());
    out
}

#[test]
fn libmpv_frames_reach_the_meari_raster() {
    let mut mpv = kinekino_libmpv::LibMpvSurface::new(None, true)
        .expect("libmpv is required ($KINEKINO_LIBMPV or soname on the loader path)");
    mpv.open(clip().to_str().unwrap(), 1.0).unwrap();
    let (_, mut doc) = Session::mount(&playing()).unwrap();
    let mut tex = VideoTexture::default();
    let rgba = frame(&mut doc, &mut tex, &mut mpv);
    let (w, h) = VideoTexture::target_size(&doc).unwrap().unwrap();
    // 128x36 source letterboxed into the box: red left, blue right, black bands.
    let cy = h / 2;
    assert!(
        near(px(&rgba, w / 4, cy), [255, 0, 0]),
        "{:?}",
        px(&rgba, w / 4, cy)
    );
    assert!(
        near(px(&rgba, w * 3 / 4, cy), [0, 0, 255]),
        "{:?}",
        px(&rgba, w * 3 / 4, cy)
    );
    assert!(
        near(px(&rgba, w / 2, 4), [0, 0, 0]),
        "{:?}",
        px(&rgba, w / 2, 4)
    );
}

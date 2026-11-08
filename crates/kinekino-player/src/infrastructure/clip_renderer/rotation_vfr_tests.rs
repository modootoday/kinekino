//! Short-form rotation and variable-frame-rate first step on synthetic local media (roadmap M4-6).
//! Source inspection reports a display-matrix rotation and the candidate encoder refuses it as
//! unsupported before any encode; an unrotated variable-frame-rate source keeps its original
//! frame timestamps for membership and still encodes to the preset's constant rate. Uses the
//! installed FFmpeg/FFprobe pinned by their current hash; skipped when they are not installed.
use super::*;
use crate::{
    application::frame_capture::FrameCapturePort, infrastructure::frame_capture::FfmpegFrameCapture,
};
use sha2::{Digest, Sha256};
struct Tools {
    ffmpeg: PathBuf,
    ffprobe: PathBuf,
}
fn tools() -> Option<Tools> {
    kinekino_platform::install();
    let capability = FfmpegFrameCapture::discover().capability();
    if !capability.available {
        eprintln!("FFmpeg/FFprobe not installed; rotation/VFR clip fixture skipped");
        return None;
    }
    Some(Tools {
        ffmpeg: capability.ffmpeg_path?.into(),
        ffprobe: capability.ffprobe_path?.into(),
    })
}
fn sha(path: &std::path::Path) -> String {
    kinekino_media_contracts::hex_lower(Sha256::digest(fs::read(path).unwrap()))
}
fn run(command: &mut Command) -> Vec<u8> {
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}
struct Scratch(PathBuf);
impl Scratch {
    fn new(tag: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("kk-clip-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
/// One-second segment from the start, no audio, full-frame crop, builtin preset.
fn short_document(source: &ClipSourcePin) -> ClipDocument {
    let mut doc = document(source);
    doc.segments[0].range.start_us = DecimalU64(0);
    doc.segments[0].range.end_us = DecimalU64(1_000_000);
    doc.segments[0].crop_keys[0].source_us = DecimalU64(0);
    doc.segments[0].audio = AudioSelection::Disabled;
    doc
}
struct Render {
    error: Option<ApiError>,
    included_frames: u64,
    first_excluded_pts: String,
    rotation: Option<i32>,
    output: Option<Vec<u8>>,
}
fn render(tools: &Tools, scratch: &Scratch, source: PathBuf) -> Render {
    let sources = FfmpegClipSources::new(
        BTreeMap::from([(
            "source".into(),
            FixedClipSource {
                path: source,
                container: ClipContainer::Matroska,
            },
        )]),
        &tools.ffprobe,
        &sha(&tools.ffprobe),
        crate::domain::frame_capture::MAX_MEDIA_BYTES,
        Duration::from_secs(60),
    )
    .unwrap();
    let initial = sources.inspect_source("source", &Control).unwrap();
    let rotation = initial.facts().rotation_degrees;
    let doc = short_document(&initial.facts().source);
    doc.validate().unwrap();
    let store_path = scratch.0.join("projects");
    fs::create_dir(&store_path).unwrap();
    let store = ClipProjectFiles::open(&store_path).unwrap();
    let saved = clip_projects::create(&store, &doc).unwrap();
    let media = prepare_media(
        &store,
        SavedClipProjectPin {
            document_id: doc.document_id.clone(),
            publication_revision: DecimalU64(1),
            publication_sha256: saved.publication_sha256,
        },
        &sources,
        &Control,
    )
    .unwrap();
    let encoder = prepare_encoder(
        &doc,
        &crate::application::clip_render::presets::BuiltinClipPresets,
        &FixedClipEncoder {
            path: tools.ffmpeg.clone(),
            sha256: sha(&tools.ffmpeg),
            inventory_timeout: Duration::from_secs(30),
        },
        &Control,
    )
    .unwrap();
    let boundary = media.sources()[0]
        .observe_segment(&doc.segments[0], Duration::from_secs(30), &Control)
        .unwrap();
    let boundaries = &boundary.observation().boundaries;
    let color = DeclaredBt709Limited {
        source_sha256: initial.facts().source.sha256.clone(),
    };
    let (error, output) = match encode_candidate(
        &media,
        &encoder,
        &boundary,
        &color,
        Duration::from_secs(120),
        64 * 1024 * 1024,
        &Control,
    ) {
        Ok(candidate) => (None, Some(candidate.bytes().to_vec())),
        Err(error) => (Some(error), None),
    };
    Render {
        error,
        included_frames: boundaries.included_frames.0,
        first_excluded_pts: boundaries.first_excluded_pts.clone(),
        rotation,
        output,
    }
}
fn synthetic(tools: &Tools, path: &std::path::Path, filter: &str) {
    run(Command::new(&tools.ffmpeg)
        .args(["-v", "error", "-nostdin", "-f", "lavfi", "-i"])
        .arg("testsrc2=size=96x64:rate=12:duration=2")
        .args(["-vf", filter, "-an", "-c:v", "libx264", "-bf", "0"])
        .args([
            "-pix_fmt",
            "yuv420p",
            "-fps_mode",
            "passthrough",
            "-threads",
            "2",
        ])
        .arg(path));
}
#[test]
fn rotated_source_is_reported_and_refused_before_encoding() {
    let Some(tools) = tools() else { return };
    let scratch = Scratch::new("rotation");
    let base = scratch.0.join("base.mkv");
    synthetic(&tools, &base, "null");
    for (rotation, reported) in [("90", 90), ("270", -90), ("180", -180)] {
        let source = scratch.0.join(format!("rotated-{rotation}.mkv"));
        run(Command::new(&tools.ffmpeg)
            .args([
                "-v",
                "error",
                "-nostdin",
                "-display_rotation",
                rotation,
                "-i",
            ])
            .arg(&base)
            .args(["-c", "copy"])
            .arg(&source));
        let before = sha(&source);
        let case = Scratch::new(&format!("rotation-{rotation}"));
        let result = render(&tools, &case, source.clone());
        assert_eq!(result.rotation, Some(reported), "{rotation}");
        let error = result.error.expect("rotated source must not encode");
        assert_eq!(error.code, ErrorCode::Unsupported, "{rotation}: {error:?}");
        assert!(error.message.contains("rotation"), "{}", error.message);
        assert!(result.output.is_none());
        assert_eq!(sha(&source), before, "source must stay untouched");
    }
    // The unrotated twin passes the same path, so the refusal is caused by rotation alone.
    let case = Scratch::new("rotation-none");
    let plain = render(&tools, &case, base);
    assert_eq!(plain.rotation, None);
    assert!(plain.error.is_none(), "{:?}", plain.error);
}
#[test]
fn variable_frame_rate_source_keeps_original_pts_membership_and_encodes_constant_rate() {
    let Some(tools) = tools() else { return };
    let scratch = Scratch::new("vfr");
    let source = scratch.0.join("vfr.mkv");
    // Dropping every fifth frame of a 12 fps source leaves uneven frame intervals.
    synthetic(&tools, &source, "select=not(eq(mod(n\\,5)\\,1))");
    let probe: Value = serde_json::from_slice(&run(Command::new(&tools.ffprobe)
        .args(["-v", "error", "-select_streams", "v:0", "-show_frames"])
        .args(["-show_entries", "frame=pts:stream=time_base", "-of", "json"])
        .arg(&source)))
    .unwrap();
    let (num, den) = probe["streams"][0]["time_base"]
        .as_str()
        .unwrap()
        .split_once('/')
        .unwrap();
    let (num, den): (u64, u64) = (num.parse().unwrap(), den.parse().unwrap());
    let ticks: Vec<u64> = probe["frames"]
        .as_array()
        .unwrap()
        .iter()
        .map(|frame| frame["pts"].as_u64().unwrap())
        .collect();
    let gaps: std::collections::BTreeSet<_> = ticks.windows(2).map(|w| w[1] - w[0]).collect();
    assert!(
        gaps.len() > 1,
        "fixture must contain variable frame intervals"
    );
    // Half-open [0, 1s): count original frames, not 12 assumed constant-rate frames.
    let end = |pts: &u64| pts * num * 1_000_000 >= 1_000_000 * den;
    let included = ticks.iter().take_while(|pts| !end(pts)).count() as u64;
    assert_ne!(
        included, 12,
        "fixture must drop frames inside the first second"
    );
    let case = Scratch::new("vfr-render");
    let result = render(&tools, &case, source);
    assert_eq!(result.rotation, None);
    assert_eq!(result.included_frames, included);
    assert_eq!(
        result.first_excluded_pts,
        ticks.iter().find(|pts| end(pts)).unwrap().to_string()
    );
    assert!(result.error.is_none(), "{:?}", result.error);
    let output = case.0.join("candidate.mp4");
    fs::write(&output, result.output.unwrap()).unwrap();
    let metadata: Value = serde_json::from_slice(&run(Command::new(&tools.ffprobe)
        .args(["-v", "error", "-count_frames", "-select_streams", "v:0"])
        .args([
            "-show_entries",
            "stream=nb_read_frames,r_frame_rate",
            "-of",
            "json",
        ])
        .arg(&output)))
    .unwrap();
    let video = &metadata["streams"][0];
    assert_eq!(video["r_frame_rate"], "30/1");
    assert_eq!(video["nb_read_frames"], "30");
}

use kinekino_player::application::api::PlaybackClient;
#[path = "support/cli_exe.rs"]
mod cli_exe;
use kinekino_player::{
    application::{
        api::{ApplicationClientHandle, PlayerSdk, Request, RequestContext},
        frame_capture::FrameCapturePort,
    },
    domain::{api_values::DecimalU64, frame_capture::CaptureFrame},
    infrastructure::{frame_capture::FfmpegFrameCapture, subtitle_files::SubtitleFiles},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "frame-contract-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
    fn request(&self) -> Request {
        let mut request = Request::new("production.capture_frame");
        request.payload = json!({"source_path":self.path("input.mp4"),"source_pts_us":"500000","output_path":self.path("output.png")});
        request
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn cli(request: &Request) -> (i32, Value) {
    let mut child = Command::new(crate::cli_exe::kinekino_cli_exe())
        .args(["--input", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(request).unwrap())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.stderr.is_empty());
    (
        output.status.code().unwrap(),
        serde_json::from_slice(&output.stdout).unwrap(),
    )
}
fn generate(fixture: &Fixture) {
    let port = FfmpegFrameCapture::discover();
    let capability = port.capability();
    assert!(
        capability.available,
        "Native integration test requires installed FFmpeg/FFprobe"
    );
    let status = Command::new(capability.ffmpeg_path.unwrap())
        .args([
            "-v",
            "error",
            "-nostdin",
            "-f",
            "lavfi",
            "-i",
            "testsrc=size=64x48:rate=10:duration=2",
            "-an",
            "-c:v",
            "mpeg4",
            "-threads",
            "2",
        ])
        .arg(fixture.path("input.mp4"))
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
#[ignore = "Requires installed FFmpeg/FFprobe; executed explicitly in capture validation"]
fn local_capture_png_pts_and_typed_client_match() {
    kinekino_platform::install();
    let fixture = Fixture::new();
    generate(&fixture);
    let original = fs::read(fixture.path("input.mp4")).unwrap();
    let (code, result) = cli(&fixture.request());
    assert_eq!(code, 0, "{result}");
    assert_eq!(result["data"]["width"], 64);
    assert_eq!(result["data"]["height"], 48);
    assert_eq!(result["data"]["actual_source_pts_us"], "500000");
    let bytes = fs::read(fixture.path("output.png")).unwrap();
    assert_eq!(
        result["data"]["output_sha256"],
        format!("{:x}", Sha256::digest(&bytes))
    );
    let mut decoder = png::Decoder::new(std::io::Cursor::new(&bytes))
        .read_info()
        .unwrap();
    let mut pixels = vec![0; decoder.output_buffer_size().unwrap()];
    decoder.next_frame(&mut pixels).unwrap();
    decoder.finish().unwrap();
    let handle: ApplicationClientHandle = std::sync::Arc::new(
        PlayerSdk::new(SubtitleFiles).with_frame_capture(FfmpegFrameCapture::discover()),
    );
    let frame = handle
        .capture_frame(
            RequestContext::default(),
            CaptureFrame {
                source_path: fixture.path("input.mp4").to_string_lossy().into(),
                source_pts_us: DecimalU64(500000),
                output_path: fixture.path("typed.png").to_string_lossy().into(),
            },
        )
        .unwrap();
    assert_eq!(frame.actual_source_pts_us.0, 500000);
    assert_eq!(
        frame.output_sha256.as_deref(),
        result["data"]["output_sha256"].as_str()
    );
    assert_eq!(fs::read(fixture.path("typed.png")).unwrap(), bytes);
    assert_eq!(fs::read(fixture.path("input.mp4")).unwrap(), original);
}

#[test]
#[ignore = "Requires installed FFmpeg/FFprobe; executed explicitly in capture validation"]
fn errors_timeout_and_overwrite_never_publish_or_modify() {
    kinekino_platform::install();
    let fixture = Fixture::new();
    generate(&fixture);
    let mut request = fixture.request();
    request.payload["source_pts_us"] = json!("9000000");
    assert_eq!(cli(&request).0, 2);
    assert!(!fixture.path("output.png").exists());
    request = fixture.request();
    request.context.timeout_ms = Some(1);
    assert_eq!(cli(&request).0, 10);
    assert!(!fixture.path("output.png").exists());
    fs::write(fixture.path("output.png"), "KEEP").unwrap();
    assert_eq!(cli(&fixture.request()).0, 5);
    assert_eq!(fs::read(fixture.path("output.png")).unwrap(), b"KEEP");
    fs::hard_link(fixture.path("input.mp4"), fixture.path("alias.png")).unwrap();
    request = fixture.request();
    request.payload["output_path"] = json!(fixture.path("alias.png"));
    assert_eq!(cli(&request).0, 5);
}

#[test]
fn protocols_playlists_bad_signature_and_negative_or_numeric_pts_are_rejected() {
    kinekino_platform::install();
    let fixture = Fixture::new();
    let sdk = PlayerSdk::new(SubtitleFiles).with_frame_capture(FfmpegFrameCapture::discover());
    for source in [
        "https://example.com/input.mp4",
        "file:///C:/input.mp4",
        r"\\server\share\input.mp4",
        "relative.mp4",
        "concat:one.mp4|two.mp4",
    ] {
        let mut request = fixture.request();
        request.payload["source_path"] = json!(source);
        let response = sdk.execute(request);
        assert!(!response.ok);
    }
    fs::write(
        fixture.path("input.mp4"),
        "#EXTM3U\nhttps://example.com/live\n",
    )
    .unwrap();
    let response = sdk.execute(fixture.request());
    assert!(!response.ok);
    for pts in [json!("-1"), json!(1)] {
        let mut request = fixture.request();
        request.payload["source_pts_us"] = pts;
        assert!(!sdk.execute(request).ok);
    }
}

#[test]
#[ignore = "Requires installed FFmpeg/FFprobe; executed explicitly in capture validation"]
fn concurrent_png_exports_and_oversized_media_are_rejected_safely() {
    kinekino_platform::install();
    let fixture = Fixture::new();
    generate(&fixture);
    let request = fixture.request();
    let (first, second) = std::thread::scope(|scope| {
        let a = scope.spawn(|| cli(&request).0);
        let b = scope.spawn(|| cli(&request).0);
        (a.join().unwrap(), b.join().unwrap())
    });
    assert!(matches!((first, second), (0, 5) | (5, 0)));
    assert_eq!(fs::read_dir(&fixture.0).unwrap().count(), 2);
    let oversized = fixture.path("large.mp4");
    let file = fs::File::create(&oversized).unwrap();
    file.set_len(kinekino_player::domain::frame_capture::MAX_MEDIA_BYTES + 1)
        .unwrap();
    drop(file);
    let mut request = fixture.request();
    request.payload["source_path"] = json!(oversized);
    request.payload["output_path"] = json!(fixture.path("large.png"));
    assert_eq!(cli(&request).0, 7);
    assert!(!fixture.path("large.png").exists());
}

#[test]
fn missing_adapter_reports_unsupported_and_schema_matches_payload() {
    kinekino_platform::install();
    let fixture = Fixture::new();
    let sdk = PlayerSdk::new(SubtitleFiles);
    assert_eq!(sdk.execute(fixture.request()).error.unwrap().exit_code(), 3);
    let mut request = Request::new("schema");
    request.payload = json!({"command":"production.capture_frame"});
    let schema = sdk.execute(request).data.unwrap();
    assert_eq!(schema["status"], "unsupported");
    assert_eq!(
        schema["request"]["properties"]["payload"]["properties"]["source_pts_us"]["type"],
        "string"
    );
}

#[test]
#[ignore = "Requires original local runtime/ai-spike/source/Sintel.2010.720p.mkv and installed FFmpeg"]
fn actual_local_sintel_frame_and_response_are_saved_for_review() {
    kinekino_platform::install();
    let source = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .join("runtime/ai-spike/source/Sintel.2010.720p.mkv");
    let output_dir = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .join("runtime/capture-validation");
    fs::create_dir_all(&output_dir).unwrap();
    let output = output_dir.join(format!(
        "sintel-{}-{}.png",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let mut request = Request::new("production.capture_frame");
    request.payload = json!({"source_path":source,"source_pts_us":"52000000","output_path":output});
    let (code, response) = cli(&request);
    fs::write(
        output.with_extension("json"),
        serde_json::to_vec_pretty(&response).unwrap(),
    )
    .unwrap();
    assert_eq!(code, 0, "{response}");
    assert_eq!(response["data"]["width"], 1280);
    assert_eq!(response["data"]["height"], 544);
    assert!(output.exists());
}

fn native_output(command: &mut Command) -> Vec<u8> {
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

fn png_pixels(bytes: &[u8]) -> Vec<u8> {
    let mut reader = png::Decoder::new(std::io::Cursor::new(bytes))
        .read_info()
        .unwrap();
    let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut pixels).unwrap();
    pixels.truncate(info.buffer_size());
    reader.finish().unwrap();
    pixels
}

#[test]
#[ignore = "Requires installed FFmpeg/FFprobe; short synthetic source-PTS seek validation"]
fn seek_matches_independently_enumerated_first_frame_and_pixels() {
    kinekino_platform::install();
    let fixture = Fixture::new();
    let port = FfmpegFrameCapture::discover();
    let capability = port.capability();
    assert!(capability.available);
    let ffmpeg = capability.ffmpeg_path.unwrap();
    let ffprobe = capability.ffprobe_path.unwrap();
    for (name, filter, offset, variable, gop, x264) in [
        ("cfr.mp4", "null", 0, false, "12", "open-gop=0"),
        (
            "shifted.mp4",
            "setpts=PTS+5/TB",
            5_000_000,
            false,
            "12",
            "open-gop=0",
        ),
        (
            "shifted.mkv",
            "setpts=PTS+5/TB",
            5_000_000,
            false,
            "12",
            "open-gop=0",
        ),
        (
            "vfr.mkv",
            "select=not(eq(mod(n\\,5)\\,1))",
            0,
            true,
            "12",
            "open-gop=0",
        ),
        ("long-gop.mp4", "null", 0, false, "48", "open-gop=0"),
        ("open-gop.mp4", "null", 0, false, "12", "open-gop=1"),
    ] {
        let source = fixture.path(name);
        native_output(
            Command::new(&ffmpeg)
                .args([
                    "-v",
                    "error",
                    "-nostdin",
                    "-filter_threads",
                    "1",
                    "-f",
                    "lavfi",
                    "-i",
                    "testsrc2=size=96x64:rate=12:duration=5",
                    "-vf",
                    filter,
                    "-an",
                    "-c:v",
                    "libx264",
                    "-preset",
                    "fast",
                    "-g",
                    gop,
                    "-keyint_min",
                    "12",
                    "-sc_threshold",
                    "0",
                    "-x264-params",
                    x264,
                    "-bf",
                    "2",
                    "-threads",
                    "2",
                    "-pix_fmt",
                    "yuv420p",
                    "-fps_mode",
                    "passthrough",
                ])
                .arg(&source),
        );
        let probe: Value = serde_json::from_slice(&native_output(
            Command::new(&ffprobe)
                .args([
                    "-v",
                    "error",
                    "-select_streams",
                    "v:0",
                    "-show_frames",
                    "-show_streams",
                    "-show_entries",
                    "frame=pts,pict_type,key_frame:stream=time_base,start_time",
                    "-of",
                    "json",
                ])
                .arg(&source),
        ))
        .unwrap();
        let base = probe["streams"][0]["time_base"].as_str().unwrap();
        let (num, den) = base.split_once('/').unwrap();
        let num: i128 = num.parse().unwrap();
        let den: i128 = den.parse().unwrap();
        let frames = probe["frames"].as_array().unwrap();
        assert!(
            frames.iter().any(|frame| frame["pict_type"] == "B"),
            "{name}"
        );
        assert!(
            frames
                .iter()
                .filter(|frame| frame["key_frame"] == 1)
                .count()
                >= 2
        );
        if name == "cfr.mp4" {
            let packets: Value = serde_json::from_slice(&native_output(
                Command::new(&ffprobe)
                    .args([
                        "-v",
                        "error",
                        "-select_streams",
                        "v:0",
                        "-show_entries",
                        "packet=dts",
                        "-of",
                        "json",
                    ])
                    .arg(&source),
            ))
            .unwrap();
            assert!(
                packets["packets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|packet| packet["dts"].as_i64().unwrap() < 0),
                "B-frame fixture must preserve negative DTS with nonnegative presentation timing"
            );
        }
        let ticks: Vec<i128> = frames
            .iter()
            .map(|frame| i128::from(frame["pts"].as_i64().unwrap()))
            .collect();
        assert!(ticks.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(ticks[0] * num * 1_000_000 / den, i128::from(offset));
        if variable {
            let gaps: std::collections::BTreeSet<_> =
                ticks.windows(2).map(|pair| pair[1] - pair[0]).collect();
            assert!(
                gaps.len() > 1,
                "fixture must contain variable frame intervals"
            );
        }
        for relative in [
            0u64, 250_000, 583_333, 583_334, 3_000_000, 4_250_000, 4_500_001,
        ] {
            let requested = offset + relative;
            let index = ticks
                .iter()
                .position(|pts| pts * num * 1_000_000 >= i128::from(requested) * den)
                .unwrap();
            let output_path = fixture.path(&format!("{name}-{requested}.png"));
            let result = port
                .capture(
                    &CaptureFrame {
                        source_path: source.to_string_lossy().into_owned(),
                        source_pts_us: DecimalU64(requested),
                        output_path: output_path.to_string_lossy().into_owned(),
                    },
                    30_000,
                )
                .unwrap_or_else(|error| panic!("{name} {requested}: {error:?}"));
            assert_eq!(
                result.source_pts_ticks,
                ticks[index].to_string(),
                "{name} {requested}"
            );
            assert_eq!(result.source_time_base, base, "{name} {requested}");
            assert_eq!(
                u128::from(result.actual_source_pts_us.0),
                (ticks[index] * num * 1_000_000 / den) as u128
            );
            // Reference decodes from the beginning and chooses by ffprobe frame index, without timestamp selection or seek.
            let reference = native_output(
                Command::new(&ffmpeg)
                    .args([
                        "-v",
                        "error",
                        "-nostdin",
                        "-filter_threads",
                        "1",
                        "-threads",
                        "2",
                        "-copyts",
                        "-noautorotate",
                        "-i",
                    ])
                    .arg(&source)
                    .args([
                        "-map",
                        "0:v:0",
                        "-an",
                        "-sn",
                        "-dn",
                        "-vf",
                        &format!("select=eq(n\\,{index})"),
                        "-frames:v",
                        "1",
                        "-fps_mode",
                        "passthrough",
                        "-c:v",
                        "png",
                        "-pix_fmt",
                        "rgb24",
                        "-threads",
                        "2",
                        "-f",
                        "image2pipe",
                        "pipe:1",
                    ]),
            );
            assert_eq!(
                png_pixels(&fs::read(output_path).unwrap()),
                png_pixels(&reference),
                "{name} {requested} pixels"
            );
            if relative == 4_500_001 {
                let cli_output = fixture.path(&format!("cli-{name}.png"));
                let mut request = Request::new("production.capture_frame");
                request.payload = json!({"source_path":source,"source_pts_us":requested.to_string(),"output_path":cli_output});
                let (code, response) = cli(&request);
                assert_eq!(code, 0, "{name}: {response}");
                assert_eq!(
                    response["data"]["source_pts_ticks"],
                    result.source_pts_ticks
                );
                assert_eq!(
                    response["data"]["source_time_base"],
                    result.source_time_base
                );
                assert_eq!(
                    png_pixels(&fs::read(cli_output).unwrap()),
                    png_pixels(&reference)
                );
            }
        }
    }
}

#[test]
#[ignore = "Requires installed FFmpeg/FFprobe; short synthetic negative timing rejection"]
fn negative_source_start_is_rejected_without_publishing() {
    kinekino_platform::install();
    let fixture = Fixture::new();
    let port = FfmpegFrameCapture::discover();
    let capability = port.capability();
    assert!(capability.available);
    let source = fixture.path("negative.mp4");
    native_output(
        Command::new(capability.ffmpeg_path.unwrap())
            .args([
                "-v",
                "error",
                "-nostdin",
                "-f",
                "lavfi",
                "-i",
                "testsrc2=size=96x64:rate=12:duration=3",
                "-vf",
                "null",
                "-c:v",
                "libx264",
                "-threads",
                "2",
                "-bf",
                "2",
                "-fps_mode",
                "passthrough",
                "-avoid_negative_ts",
                "disabled",
                "-use_editlist",
                "0",
            ])
            .arg(&source),
    );
    // MOV muxing normalizes encoder timestamps. Construct signed composition offsets
    // in this local synthetic fixture, then require ffprobe to observe negative PTS.
    let mut bytes = fs::read(&source).unwrap();
    let ctts = bytes.windows(4).position(|atom| atom == b"ctts").unwrap();
    let atom_size = u32::from_be_bytes(bytes[ctts - 4..ctts].try_into().unwrap()) as usize;
    assert_eq!(bytes[ctts + 4], 0);
    let entries = u32::from_be_bytes(bytes[ctts + 8..ctts + 12].try_into().unwrap()) as usize;
    assert_eq!(atom_size, 16 + entries * 8);
    bytes[ctts + 4] = 1;
    for entry in 0..entries {
        let offset = ctts + 16 + entry * 8;
        let old = u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap());
        let shifted = i32::try_from(old).unwrap().checked_sub(12288).unwrap();
        bytes[offset..offset + 4].copy_from_slice(&shifted.to_be_bytes());
    }
    fs::write(&source, bytes).unwrap();
    let probe: Value = serde_json::from_slice(&native_output(
        Command::new(capability.ffprobe_path.unwrap())
            .args([
                "-v",
                "error",
                "-select_streams",
                "v:0",
                "-show_entries",
                "stream=start_time:frame=pts",
                "-show_frames",
                "-of",
                "json",
            ])
            .arg(&source),
    ))
    .unwrap();
    assert!(
        probe["streams"][0]["start_time"]
            .as_str()
            .unwrap()
            .starts_with('-'),
        "fixture must retain a negative stream start"
    );
    assert!(
        probe["frames"]
            .as_array()
            .unwrap()
            .iter()
            .any(|frame| frame["pts"].as_i64().unwrap() < 0)
    );
    let output = fixture.path("negative.png");
    let error = port
        .capture(
            &CaptureFrame {
                source_path: source.to_string_lossy().into_owned(),
                source_pts_us: DecimalU64(0),
                output_path: output.to_string_lossy().into_owned(),
            },
            30_000,
        )
        .unwrap_err();
    assert_eq!(
        error.code,
        kinekino_player::application::api::ErrorCode::Unsupported
    );
    assert!(!output.exists());
}

#[test]
fn legacy_capture_receipt_without_png_identity_remains_explicitly_unbound() {
    kinekino_platform::install();
    let old = json!({"output_path":"legacy.png","requested_source_pts_us":"500000","actual_source_pts_us":"520000","source_pts_ticks":"52","source_time_base":"1/100","width":64,"height":48,"output_bytes":100,"source_color_space":null,"source_color_transfer":null,"rotation_applied":false,"source_rotation_degrees":null,"subtitle_mode":"separate_tracks_excluded_burn_in_preserved","limitations":[],"dependency":{"available":false,"adapter":"legacy","ffmpeg_path":null,"ffprobe_path":null,"ffmpeg_version":null,"ffprobe_version":null,"unavailable_reason":null}});
    let receipt: kinekino_player::application::frame_capture::CapturedFrame =
        serde_json::from_value(old).unwrap();
    assert_eq!(receipt.output_sha256, None);
    assert_eq!(receipt.actual_source_pts_us.0, 520000);
    assert_eq!(receipt.requested_source_pts_us.0, 500000);
}

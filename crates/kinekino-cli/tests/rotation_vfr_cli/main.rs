//! Rotation and variable-frame-rate first step through `kinekino --input -` (roadmap M4-6).
//! Frame capture keeps decoding with `-noautorotate`: a display-matrix rotation is reported in
//! `source_rotation_degrees` but never applied, so the pixels match the unrotated twin. On a
//! variable-frame-rate source the CLI returns the first original frame at or after the request,
//! taken from the source timestamps rather than an assumed constant rate. Fixtures are generated
//! locally with the installed FFmpeg; without it the tests only confirm capture is unsupported.
use serde_json::{Value, json};
use std::{
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

struct Fixture(PathBuf);

impl Fixture {
    fn new(tag: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("kk-rotvfr-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }
    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn call(request: &Value) -> (Option<i32>, Value) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kinekino"))
        .args(["--input", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(request.to_string().as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    (
        output.status.code(),
        serde_json::from_slice(&output.stdout).unwrap(),
    )
}

fn request(command: &str, payload: Value) -> Value {
    json!({"command":command,"context":{"request_id":"rotation-vfr","api_version":"1"},"payload":payload})
}

/// Installed FFmpeg/FFprobe paths reported by the CLI, or `None` when capture is unavailable.
fn tools() -> Option<(String, String)> {
    let (code, capabilities) = call(&request("capabilities", json!({})));
    assert_eq!(code, Some(0), "{capabilities}");
    let dependency = &capabilities["data"]["frame_capture"]["dependency"];
    if dependency["available"] != true {
        return None;
    }
    Some((
        dependency["ffmpeg_path"].as_str()?.to_owned(),
        dependency["ffprobe_path"].as_str()?.to_owned(),
    ))
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

fn capture(source: &Path, pts_us: u64, output: &Path) -> (Option<i32>, Value) {
    call(&request(
        "production.capture_frame",
        json!({"source_path":source,"source_pts_us":pts_us.to_string(),"output_path":output}),
    ))
}

fn assert_unsupported_without_tools(fixture: &Fixture) {
    let output = fixture.path("out.png");
    let (code, response) = capture(&fixture.path("input.mp4"), 0, &output);
    assert_ne!(code, Some(0), "{response}");
    assert_eq!(response["error"]["code"], "unsupported", "{response}");
    assert!(!output.exists());
}

#[test]
fn display_rotation_is_reported_but_not_applied_to_captured_pixels() {
    let fixture = Fixture::new("rotation");
    let Some((ffmpeg, ffprobe)) = tools() else {
        return assert_unsupported_without_tools(&fixture);
    };
    let base = fixture.path("base.mp4");
    run(Command::new(&ffmpeg)
        .args(["-v", "error", "-nostdin", "-f", "lavfi", "-i"])
        .arg("testsrc2=size=96x64:rate=12:duration=2")
        .args(["-an", "-c:v", "libx264", "-bf", "0", "-pix_fmt", "yuv420p"])
        .args(["-threads", "2"])
        .arg(&base));
    let (code, plain) = capture(&base, 500_000, &fixture.path("base.png"));
    assert_eq!(code, Some(0), "{plain}");
    assert_eq!(plain["data"]["source_rotation_degrees"], Value::Null);
    assert_eq!(plain["data"]["rotation_applied"], false);

    // FFprobe reports the stored display-matrix angle; 270 is stored as -90.
    for (rotation, reported, name) in [
        ("90", 90, "rotated-90.mp4"),
        ("90", 90, "rotated-90.mkv"),
        ("180", -180, "rotated-180.mkv"),
        ("270", -90, "rotated-270.mp4"),
    ] {
        let source = fixture.path(name);
        run(Command::new(&ffmpeg)
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
        let probe: Value = serde_json::from_slice(&run(Command::new(&ffprobe)
            .args(["-v", "error", "-select_streams", "v:0", "-show_entries"])
            .args(["stream_side_data=rotation", "-of", "json"])
            .arg(&source)))
        .unwrap();
        assert_eq!(
            probe["streams"][0]["side_data_list"][0]["rotation"], reported,
            "{name}: fixture must carry a display matrix"
        );
        let output = fixture.path(&format!("{name}.png"));
        let (code, response) = capture(&source, 500_000, &output);
        assert_eq!(code, Some(0), "{name}: {response}");
        let data = &response["data"];
        assert_eq!(data["source_rotation_degrees"], reported, "{name}");
        assert_eq!(data["rotation_applied"], false, "{name}");
        // Dimensions and pixels stay in stored (unrotated) orientation.
        assert_eq!(
            (data["width"].clone(), data["height"].clone()),
            (json!(96), json!(64))
        );
        assert_eq!(
            data["output_sha256"], plain["data"]["output_sha256"],
            "{name}"
        );
        assert_eq!(data["actual_source_pts_us"], "500000", "{name}");
        assert!(
            data["limitations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|line| line
                    .as_str()
                    .is_some_and(|text| text.contains("rotation is not applied"))),
            "{name}: {response}"
        );
        assert!(output.exists(), "{name}");
    }
}

#[test]
fn variable_frame_rate_capture_selects_first_original_frame_at_or_after_request() {
    let fixture = Fixture::new("vfr");
    let Some((ffmpeg, ffprobe)) = tools() else {
        return assert_unsupported_without_tools(&fixture);
    };
    let source = fixture.path("vfr.mkv");
    // Dropping every fifth frame of a 12 fps source leaves uneven frame intervals.
    run(Command::new(&ffmpeg)
        .args(["-v", "error", "-nostdin", "-f", "lavfi", "-i"])
        .arg("testsrc2=size=96x64:rate=12:duration=2")
        .args(["-vf", "select=not(eq(mod(n\\,5)\\,1))", "-an"])
        .args(["-c:v", "libx264", "-bf", "0", "-pix_fmt", "yuv420p"])
        .args(["-fps_mode", "passthrough", "-threads", "2"])
        .arg(&source));
    let probe: Value = serde_json::from_slice(&run(Command::new(&ffprobe)
        .args(["-v", "error", "-select_streams", "v:0", "-show_frames"])
        .args(["-show_entries", "frame=pts:stream=time_base", "-of", "json"])
        .arg(&source)))
    .unwrap();
    let time_base = probe["streams"][0]["time_base"]
        .as_str()
        .unwrap()
        .to_owned();
    let (num, den) = time_base.split_once('/').unwrap();
    let (num, den): (u128, u128) = (num.parse().unwrap(), den.parse().unwrap());
    let ticks: Vec<u128> = probe["frames"]
        .as_array()
        .unwrap()
        .iter()
        .map(|frame| frame["pts"].as_u64().unwrap().into())
        .collect();
    let gaps: std::collections::BTreeSet<_> = ticks.windows(2).map(|w| w[1] - w[0]).collect();
    assert!(
        gaps.len() > 1,
        "fixture must contain variable frame intervals"
    );

    let mut skipped_gap = false;
    for requested in [
        0u64, 83_334, 100_000, 166_000, 400_000, 500_000, 1_333_334, 1_750_000,
    ] {
        let index = ticks
            .iter()
            .position(|pts| pts * num * 1_000_000 >= u128::from(requested) * den)
            .unwrap();
        // A constant 12 fps assumption would pick frame round_up(requested * 12 / 1e6).
        let constant_index = (u128::from(requested) * 12).div_ceil(1_000_000) as usize;
        skipped_gap |= index != constant_index;
        let output = fixture.path(&format!("frame-{requested}.png"));
        let (code, response) = capture(&source, requested, &output);
        assert_eq!(code, Some(0), "{requested}: {response}");
        let data = &response["data"];
        assert_eq!(data["source_time_base"], time_base, "{requested}");
        assert_eq!(
            data["source_pts_ticks"],
            ticks[index].to_string(),
            "{requested}"
        );
        let actual = ticks[index] * num * 1_000_000 / den;
        assert_eq!(
            data["actual_source_pts_us"],
            actual.to_string(),
            "{requested}"
        );
        assert!(actual >= u128::from(requested), "{requested}");
        assert!(
            index == 0 || ticks[index - 1] * num * 1_000_000 < u128::from(requested) * den,
            "{requested}: an earlier original frame also satisfies the request"
        );
        assert!(output.exists(), "{requested}");
    }
    assert!(skipped_gap, "requests must exercise a dropped-frame gap");
}

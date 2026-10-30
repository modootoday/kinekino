#![cfg(windows)]
#[path = "support/cli_exe.rs"]
mod cli_exe;
#[path = "sprite_capture_cli/custody.rs"]
mod custody;
#[path = "sprite_capture_cli/process.rs"]
mod process;
use custody::Guard;
use process::call;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};
fn request(command: &str, payload: Value, timeout: Option<u64>) -> Value {
    json!({"context":{"request_id":"sprite-capture-technical","api_version":"1","expected_revision":null,"idempotency_key":null,"timeout_ms":timeout},"command":command,"payload":payload})
}
#[test]
#[ignore = "Requires fixed existing Sprite media and explicit central offline CPU media allocation"]
fn fixed_sprite_cli_capture_matches_observed_source_ticks_and_preserves_original() {
    kinekino_platform::install();
    let root = std::env::temp_dir().join(format!("sprite-capture-cli-{}", std::process::id()));
    fs::create_dir(&root).unwrap();
    let source = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .join("runtime/ai-spike/source/sprite-fright-sprite-source-r1/source.mp4");
    let original = Guard::acquire(&source);
    assert_eq!(
        original.sha256,
        "7cfc8b8c6eeb19685048a289aca208a01356f5082b470fb9f4d933fe4b2087ef"
    );
    let cli = Guard::acquire(Path::new(crate::cli_exe::kinekino_cli_exe()));
    let caps = call(
        &root,
        "00-capabilities",
        &request("capabilities", json!({}), None),
        &cli,
        true,
        None,
    );
    let dependency = &caps["data"]["frame_capture"]["dependency"];
    assert_eq!(dependency["available"], true);
    let ffmpeg = Guard::acquire(Path::new(dependency["ffmpeg_path"].as_str().unwrap()));
    let ffprobe = Guard::acquire(Path::new(dependency["ffprobe_path"].as_str().unwrap()));
    fs::write(root.join("INPUT.json"),serde_json::to_vec_pretty(&json!({"source":source,"original":original.evidence(),"cli":cli.evidence(),"ffmpeg":ffmpeg.evidence(),"ffprobe":ffprobe.evidence(),"dependency":dependency,"model_output":false,"actual9_binding":false,"quality_approved":false})).unwrap()).unwrap();
    for (label, micros, ticks) in [
        ("0s", 0u64, "0"),
        ("17s", 17_000_000u64, "208896"),
        ("25-5s", 25_500_000, "313344"),
    ] {
        let output = root.join(format!("{label}.png"));
        let command = request(
            "production.capture_frame",
            json!({"source_path":source,"source_pts_us":micros.to_string(),"output_path":output}),
            Some(30000),
        );
        let captured = call(&root, label, &command, &cli, true, None);
        let data = &captured["data"];
        assert_eq!(data["source_pts_ticks"], ticks);
        assert_eq!(data["actual_source_pts_us"], micros.to_string());
        let base = kinekino_player::domain::media_timeline::TimeBase::parse(
            data["source_time_base"].as_str().unwrap(),
        )
        .unwrap();
        assert_eq!(
            base,
            kinekino_player::domain::media_timeline::TimeBase::new(1, 12288).unwrap()
        );
        let bytes = fs::read(&output).unwrap();
        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
        let sha = format!("{:x}", Sha256::digest(&bytes));
        assert_eq!(data["output_sha256"], sha);
        call(
            &root,
            &format!("{label}-overwrite"),
            &command,
            &cli,
            false,
            Some("revision_conflict"),
        );
        assert_eq!(
            format!("{:x}", Sha256::digest(fs::read(&output).unwrap())),
            sha
        );
        original.assert_unchanged();
        ffmpeg.assert_unchanged();
        ffprobe.assert_unchanged();
    }
    let absent = root.join("outside.png");
    call(
        &root,
        "outside-source-range",
        &request(
            "production.capture_frame",
            json!({"source_path":source,"source_pts_us":"630000000","output_path":absent}),
            Some(30000),
        ),
        &cli,
        false,
        Some("invalid_argument"),
    );
    assert!(!absent.exists());
    original.assert_unchanged();
    ffmpeg.assert_unchanged();
    ffprobe.assert_unchanged();
    println!(
        "Preserved actual SDK/CLI capture evidence: {}",
        root.display()
    );
}

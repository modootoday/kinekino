use super::*;
use std::fs::{self, OpenOptions};
struct Control;
impl MutationControl for Control {
    fn checkpoint(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
#[test]
#[ignore = "Explicit pinned installed FFmpeg and fresh evidence; inventory only, no encoding"]
fn actual_pinned_encoder_inventory_retains_guard_build_license_and_required_tables() {
    let root =
        PathBuf::from(std::env::var_os("CLIP_ENCODER_EVIDENCE_DIR").expect("Fresh evidence"));
    assert!(root.is_absolute());
    fs::create_dir(&root).unwrap();
    let path = PathBuf::from(std::env::var_os("CLIP_ENCODER_EXE").expect("Pinned FFmpeg"));
    let sha = std::env::var("CLIP_ENCODER_SHA256").unwrap().to_lowercase();
    let encoder = PinnedClipEncoder::open(&path, &sha, Duration::from_secs(30), &Control).unwrap();
    assert!(OpenOptions::new().write(true).open(&path).is_err());
    assert_eq!(encoder.path().file_name().unwrap(), "ffmpeg.exe");
    let inventory = encoder.inventory();
    assert_eq!(inventory.tool_sha256, sha);
    assert!(inventory.encoders.contains("libx264"));
    assert!(inventory.encoders.contains("aac"));
    assert!(inventory.muxers.contains("mp4"));
    for name in [
        "trim",
        "setpts",
        "crop",
        "scale",
        "pad",
        "format",
        "fps",
        "atrim",
        "asetpts",
        "volume",
        "subtitles",
    ] {
        assert!(inventory.filters.contains(name), "missing {name}");
    }
    fs::write(
        root.join("inventory.json"),
        serde_json::to_vec_pretty(inventory).unwrap(),
    )
    .unwrap();
    for observation in encoder.observations() {
        fs::write(
            root.join(format!("{}.stdout", observation.stage)),
            &observation.stdout,
        )
        .unwrap();
        fs::write(
            root.join(format!("{}.stderr", observation.stage)),
            &observation.stderr,
        )
        .unwrap();
    }
    let reopened = PinnedClipEncoder::open(&path, &sha, Duration::from_secs(30), &Control).unwrap();
    assert_eq!(
        reopened.inventory().inventory_sha256,
        inventory.inventory_sha256
    );
    let mut wrong = sha.clone();
    wrong.replace_range(0..1, if sha.starts_with('0') { "1" } else { "0" });
    match PinnedClipEncoder::open(&path, &wrong, Duration::from_secs(30), &Control) {
        Err(error) => assert_eq!(error.code, ErrorCode::SourceExpired),
        Ok(_) => panic!("wrong encoder pin accepted"),
    }
    fs::write(root.join("verification.json"),serde_json::to_vec_pretty(&serde_json::json!({"tool_sha256":sha,"inventory_sha256":inventory.inventory_sha256,"six_observation_stages":encoder.observations().len(),"stable_across_reopen":true,"wrong_tool_pin_rejected":true,"write_denied_while_retained":true,"encoded":false,"playback_verified":false,"redistribution_approved":false,"quality_approved":false})).unwrap()).unwrap();
}

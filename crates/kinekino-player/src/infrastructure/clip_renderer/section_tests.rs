//! Actual pinned filter/encoder pixel transition; synthetic media, not source custody or quality approval.
use super::*;
#[test]
#[ignore = "Pinned FFmpeg and fresh evidence; synthetic red/blue source, no GUI/models/network"]
fn pinned_section_crop_switches_exactly_between_decoded_frame_14_and_15() {
    let root =
        PathBuf::from(std::env::var_os("CLIP_SECTION_EVIDENCE_DIR").expect("Fresh evidence"));
    fs::create_dir(&root).unwrap();
    let tool = PathBuf::from(std::env::var_os("CLIP_RENDER_ENCODER").unwrap());
    let pin = std::env::var("CLIP_RENDER_ENCODER_SHA256").unwrap();
    let (tool, _guards) = crate::infrastructure::frame_capture::tools::open(&tool, &pin).unwrap();
    let mut raw = Vec::new();
    for _ in 0..31 {
        for _ in 0..32 {
            for x in 0..64 {
                raw.extend(if x < 32 { [255, 0, 0] } else { [0, 0, 255] });
            }
        }
    }
    let source_raw = root.join("source.rgb");
    fs::write(&source_raw, raw).unwrap();
    let mut command = Command::new(&tool);
    command
        .args([
            "-hide_banner",
            "-nostdin",
            "-v",
            "error",
            "-threads",
            "2",
            "-f",
            "rawvideo",
            "-pixel_format",
            "rgb24",
            "-video_size",
            "64x32",
            "-framerate",
            "30",
            "-i",
        ])
        .arg(&source_raw)
        .args([
            "-frames:v",
            "31",
            "-c:v",
            "ffv1",
            "-threads:v",
            "1",
            "-f",
            "matroska",
            "-",
        ]);
    let source = process::run_with_recovery(
        &mut command,
        Instant::now() + Duration::from_secs(30),
        2 * 1024 * 1024,
        Some(&Control),
        None,
    )
    .unwrap();
    let source_path = root.join("source.mkv");
    fs::write(&source_path, &source.stdout).unwrap();
    let plan = ManualClipPlan {
        document_sha256: "a".repeat(64),
        source_media_id: "synthetic".into(),
        source_sha256: format!("{:x}", Sha256::digest(&source.stdout)),
        source_observation_sha256: "b".repeat(64),
        segment_id: "one".into(),
        video_stream_index: 0,
        start_pts: 0,
        end_pts: 1000,
        crop_sections: vec![
            ManualClipCropSection {
                source_us: DecimalU64(0),
                threshold_pts: "0".into(),
                frame_offset: DecimalU64(0),
                crop: ClipPixelCrop {
                    left: 0,
                    top: 0,
                    width: 32,
                    height: 32,
                },
                fit: FitMode::Fit,
            },
            ManualClipCropSection {
                source_us: DecimalU64(500000),
                threshold_pts: "500".into(),
                frame_offset: DecimalU64(15),
                crop: ClipPixelCrop {
                    left: 32,
                    top: 0,
                    width: 32,
                    height: 32,
                },
                fit: FitMode::Fill,
            },
        ],
        width: 32,
        height: 32,
        fps_numerator: 30,
        fps_denominator: 1,
        duration_us: 1_000_000,
        input_seek_start_us: 0,
        audio: None,
    };
    let args = command::arguments(&source_path, "matroska", &plan).unwrap();
    fs::write(
        root.join("argv.json"),
        serde_json::to_vec_pretty(
            &args
                .iter()
                .map(|arg| arg.to_str().unwrap())
                .collect::<Vec<_>>(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut command = Command::new(&tool);
    command.args(args);
    let encoded = process::run_with_recovery(
        &mut command,
        Instant::now() + Duration::from_secs(60),
        2 * 1024 * 1024,
        Some(&Control),
        None,
    )
    .unwrap();
    let path = root.join("section.mp4");
    fs::write(&path, &encoded.stdout).unwrap();
    let mut command = Command::new(&tool);
    command
        .args([
            "-hide_banner",
            "-nostdin",
            "-v",
            "error",
            "-threads",
            "2",
            "-f",
            "mov",
            "-enable_drefs",
            "0",
            "-i",
        ])
        .arg(&path)
        .args([
            "-map",
            "0:v:0",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgb24",
            "-threads:v",
            "1",
            "-",
        ]);
    let decoded = process::run_with_recovery(
        &mut command,
        Instant::now() + Duration::from_secs(30),
        2 * 1024 * 1024,
        Some(&Control),
        None,
    )
    .unwrap();
    fs::write(root.join("decoded.rgb"), &decoded.stdout).unwrap();
    assert_eq!(decoded.stdout.len(), 30 * 32 * 32 * 3);
    let mut centers = Vec::new();
    for (frame, pixels) in decoded
        .stdout
        .as_chunks::<{ 32 * 32 * 3 }>()
        .0
        .iter()
        .enumerate()
    {
        let rgb = &pixels[(16 * 32 + 16) * 3..][..3];
        centers.push(rgb.to_vec());
        if frame < 15 {
            assert!(rgb[0] > 180 && rgb[2] < 80, "frame {frame}: {rgb:?}");
        } else {
            assert!(rgb[2] > 180 && rgb[0] < 80, "frame {frame}: {rgb:?}");
        }
    }
    fs::write(root.join("verification.json"), serde_json::to_vec_pretty(&json!({"synthetic_source":true,"pinned_engine_sha256":pin,"frames":30,"red_frames":15,"blue_frames":15,"centers":centers,"original_pts_policy_tested_separately":true,"jobs_or_cli_verified":false,"quality_approved":false,"source_color_accuracy_verified":false})).unwrap()).unwrap();
}

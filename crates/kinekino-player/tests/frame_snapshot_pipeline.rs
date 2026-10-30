#![cfg(windows)]
//! Existing local media only. Two existing commands, not a new atomic workflow API.
use kinekino_player::application::api::PlaybackClient;
#[path = "support/cli_exe.rs"]
mod cli_exe;
#[path = "support/snapshot_cli_evidence.rs"]
pub mod snapshot_cli_evidence;
use kinekino_player::{
    application::{
        api::{PlayerSdk, Request, RequestContext},
        frame_capture::CapturedFrame,
        snapshot_export::{ExportSnapshot, PixelCrop},
    },
    domain::frame_capture::{MAX_FRAME_PIXELS, MAX_MEDIA_BYTES},
    infrastructure::{content_digest, snapshot_export::PngSnapshotExport},
};
use serde_json::json;
use snapshot_cli_evidence::{CliFailure, CliOptions, cli_recorded, evidence_call, hash, save_json};
use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
    time::Duration,
};

fn input_path(variable: &str, relative: &str) -> PathBuf {
    let path = std::env::var_os(variable)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")).join(relative)
        });
    assert!(
        path.is_absolute() && path.is_file(),
        "Missing existing local fixture: {}",
        path.display()
    );
    path
}
fn media_identity(path: &Path) -> (u64, String) {
    content_digest::sha256(&mut fs::File::open(path).unwrap(), MAX_MEDIA_BYTES).unwrap()
}
fn decode_rgb(bytes: &[u8]) -> (u32, u32, usize, Vec<u8>) {
    let mut options = png::DecodeOptions::default();
    options.set_ignore_crc(false);
    options.set_skip_ancillary_crc_failures(false);
    options.set_ignore_adler32(false);
    let mut reader = png::Decoder::new_with_options(Cursor::new(bytes), options)
        .read_info()
        .unwrap();
    let size = reader
        .output_buffer_size()
        .filter(|n| *n as u64 <= MAX_FRAME_PIXELS * 4)
        .unwrap();
    let mut pixels = vec![0; size];
    let info = reader.next_frame(&mut pixels).unwrap();
    reader.finish().unwrap();
    assert_eq!(info.color_type, png::ColorType::Rgb);
    assert_eq!(info.bit_depth, png::BitDepth::Eight);
    pixels.truncate(info.buffer_size());
    (info.width, info.height, info.line_size, pixels)
}
fn assert_crop(source: &[u8], output: &[u8], crop: PixelCrop) {
    let (width, height, stride, pixels) = decode_rgb(source);
    crop.validate_bounds(width, height).unwrap();
    let (out_width, out_height, out_stride, out) = decode_rgb(output);
    assert_eq!((out_width, out_height), (crop.width, crop.height));
    for row in 0..crop.height as usize {
        let begin = (crop.y as usize + row) * stride + crop.x as usize * 3;
        assert_eq!(
            &out[row * out_stride..(row + 1) * out_stride],
            &pixels[begin..begin + crop.width as usize * 3]
        );
    }
}
fn png_options() -> CliOptions {
    CliOptions::default()
}
fn capture_options() -> CliOptions {
    // FFmpeg discovery and product operation deadline are distinct from test observation.
    CliOptions {
        wall_timeout: Duration::from_secs(45),
        clear_path: false,
    }
}

#[test]
#[ignore = "Requires existing pinned local Sintel, reference PNG, source SHA and trusted FFmpeg/FFprobe; central media slot only"]
fn original_capture_receipt_identity_drives_cli_crop_and_preserves_partial_artifacts()
-> Result<(), Box<CliFailure>> {
    kinekino_platform::install();
    let source = input_path(
        "KINEKINO_FRAME_SNAPSHOT_SOURCE",
        "runtime/ai-spike/source/Sintel.2010.720p.mkv",
    );
    let reference = input_path(
        "KINEKINO_FRAME_SNAPSHOT_REFERENCE",
        ".spec/artifacts/spikes/2026-10-18/fixtures/ocr/sintel-credits-820-seek-r1.png",
    );
    let expected_media = std::env::var("KINEKINO_FRAME_SNAPSHOT_SOURCE_SHA256")
        .expect("Pin the existing local media SHA256 before the explicit media gate");
    assert!(kinekino_player::domain::identifiers::valid_sha256(
        &expected_media
    ));
    let media_before = media_identity(&source);
    assert_eq!(media_before.1, expected_media);
    let reference_bytes = fs::read(reference).unwrap();
    assert_eq!(
        hash(&reference_bytes),
        "836cb932003eb058aac3905e660b191a8c8c76a3e66e67f2a64d7c8250704943"
    );
    let folder = evidence_call();
    let original = folder.join("original.png");
    let cropped = folder.join("crop.png");
    let rejected_crop = folder.join("rejected-crop.png");
    let mut capture = Request::new("production.capture_frame");
    capture.context.request_id = "pipeline-original".into();
    capture.context.timeout_ms = Some(30_000);
    capture.payload =
        json!({"source_path":source,"source_pts_us":"820000000","output_path":original});
    let captured =
        cli_recorded(&capture, None, capture_options()).map_err(|failure| failure.closeout())?;
    save_json(
        &folder,
        "capture-stage.json",
        &json!({"raw_call":captured.evidence,"response":captured.response,"original_exists":original.exists(),"downstream_calls":0,"atomic_workflow":false}),
    );
    assert_eq!(
        captured.code,
        0,
        "Capture failed; retained evidence={}",
        folder.display()
    );
    let capture_receipt: CapturedFrame =
        serde_json::from_value(captured.response["data"].clone()).unwrap();
    let original_bytes = fs::read(&original).unwrap();
    let original_sha = capture_receipt
        .output_sha256
        .as_ref()
        .expect("New producer must return the actual PNG SHA256");
    assert_eq!(original_sha, &hash(&original_bytes));
    assert_eq!(capture_receipt.output_bytes, original_bytes.len() as u64);
    assert_eq!(capture_receipt.actual_source_pts_us.0, 820000000);
    assert_eq!(capture_receipt.source_pts_ticks, "820000");
    assert_eq!(capture_receipt.source_time_base, "1/1000");
    assert_eq!((capture_receipt.width, capture_receipt.height), (1280, 544));
    // Existing observed artifact regression, not an independent source frame-index oracle.
    let (width, height, stride, pixels) = decode_rgb(&original_bytes);
    let (reference_width, reference_height, reference_stride, reference_pixels) =
        decode_rgb(&reference_bytes);
    assert_eq!(
        (width, height, stride),
        (reference_width, reference_height, reference_stride)
    );
    assert_eq!(hash(&pixels), hash(&reference_pixels));

    let mut crop = Request::new("production.export_snapshot");
    crop.context.request_id = "pipeline-invalid-crop".into();
    crop.payload = json!({"source_path":original,"expected_source_sha256":original_sha,"output_path":rejected_crop,"crop":{"x":1280,"y":0,"width":1,"height":1}});
    let rejected =
        cli_recorded(&crop, None, png_options()).map_err(|failure| failure.closeout())?;
    save_json(
        &folder,
        "partial-stage.json",
        &json!({"state":"original_saved_crop_failed","capture_receipt":captured.response,"crop_error":rejected.response,"raw_capture":captured.evidence,"raw_crop":rejected.evidence,"original_sha256":hash(&fs::read(&original).unwrap()),"crop_exists":rejected_crop.exists(),"automatic_rollback":false,"automatic_retry":false}),
    );
    assert_ne!(rejected.code, 0);
    assert_eq!(rejected.response["error"]["code"], "invalid_argument");
    assert!(!rejected_crop.exists());
    assert_eq!(fs::read(&original).unwrap(), original_bytes);

    let rectangle = PixelCrop {
        x: 160,
        y: 2,
        width: 960,
        height: 540,
    };
    crop.context.request_id = "pipeline-explicit-crop".into();
    crop.payload["output_path"] = json!(cropped);
    crop.payload["crop"] = serde_json::to_value(rectangle).unwrap();
    let exported =
        cli_recorded(&crop, None, png_options()).map_err(|failure| failure.closeout())?;
    assert_eq!(exported.code, 0, "{0}", exported.response);
    let cropped_bytes = fs::read(&cropped).unwrap();
    assert_crop(&original_bytes, &cropped_bytes, rectangle);
    assert_eq!(exported.response["data"]["source_sha256"], *original_sha);
    assert_eq!(
        exported.response["data"]["output_sha256"],
        hash(&cropped_bytes)
    );
    assert!(
        exported.response["data"]
            .get("actual_source_pts_us")
            .is_none()
    );

    crop.context.request_id = "pipeline-overwrite-rejected".into();
    let overwrite =
        cli_recorded(&crop, None, png_options()).map_err(|failure| failure.closeout())?;
    assert_ne!(overwrite.code, 0);
    assert_eq!(overwrite.response["error"]["code"], "revision_conflict");
    assert_eq!(fs::read(&cropped).unwrap(), cropped_bytes);
    assert_eq!(fs::read(&original).unwrap(), original_bytes);

    let typed_path = folder.join("typed-crop.png");
    let typed = PlayerSdk::default()
        .with_snapshot_export(PngSnapshotExport)
        .export_snapshot(
            RequestContext::default(),
            ExportSnapshot {
                source_path: original.to_string_lossy().into(),
                expected_source_sha256: original_sha.clone(),
                output_path: typed_path.to_string_lossy().into(),
                crop: rectangle,
            },
        )
        .unwrap();
    assert_crop(&original_bytes, &fs::read(&typed_path).unwrap(), rectangle);
    assert_eq!(typed.output_sha256, hash(&fs::read(&typed_path).unwrap()));

    let failed_original = folder.join("invalid-original.png");
    capture.context.request_id = "pipeline-invalid-capture".into();
    capture.payload["source_pts_us"] = json!("invalid");
    capture.payload["output_path"] = json!(failed_original);
    let failed =
        cli_recorded(&capture, None, capture_options()).map_err(|failure| failure.closeout())?;
    assert_ne!(failed.code, 0);
    assert_eq!(failed.response["error"]["code"], "invalid_argument");
    assert!(!failed_original.exists());
    let media_after = media_identity(&source);
    assert_eq!(media_after, media_before);
    save_json(
        &folder,
        "workflow.json",
        &json!({"state":"original_and_crop_saved","source_media":{"path":source,"bytes":media_before.0,"sha256":media_before.1,"unchanged":true},"capture_receipt":captured.response,"crop_receipt":exported.response,"typed_crop_receipt":typed,"capture_call":captured.evidence,"crop_call":exported.evidence,"partial_call":rejected.evidence,"overwrite_call":overwrite.evidence,"capture_failure_call":failed.evidence,"capture_failure_downstream_calls":0,"original_png":original,"original_png_sha256":original_sha,"crop_png":cropped,"crop_png_sha256":hash(&cropped_bytes),"atomic_workflow":false,"automatic_rollback":false,"automatic_retry":false,"native_frame_time_claim":false}),
    );
    Ok(())
}

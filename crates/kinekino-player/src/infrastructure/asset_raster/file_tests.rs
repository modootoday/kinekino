//! Existing pinned PNG plus explicit synthetic manual mask; no inference or review approval.
use super::*;
use crate::infrastructure::{
    frame_capture::{process, tools},
    immutable_storage_paths as paths,
};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, OpenOptions},
    io::Read,
    path::PathBuf,
    process::Command,
    time::{Duration, Instant},
};
struct Control;
impl MutationControl for Control {
    fn checkpoint(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn write_png(path: &std::path::Path, width: u32, height: u32, color: png::ColorType, bytes: &[u8]) {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    {
        let mut encoder = png::Encoder::new(&mut file, width, height);
        encoder.set_color(color);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(bytes).unwrap();
        writer.finish().unwrap();
    }
    file.sync_all().unwrap();
}
#[test]
#[ignore = "Pinned existing PNG/tool and fresh evidence; explicit manual rectangle mask, no model/GUI/user quality approval"]
fn existing_png_manual_mask_outputs_roundtrip_and_independent_probe_without_mutating_source() {
    let source = PathBuf::from(std::env::var_os("ASSET_SOURCE_PNG").expect("Pinned existing PNG"));
    let expected = std::env::var("ASSET_SOURCE_PNG_SHA256").unwrap();
    let root =
        PathBuf::from(std::env::var_os("ASSET_RASTER_EVIDENCE_DIR").expect("Fresh evidence"));
    fs::create_dir(&root).unwrap();
    let (_, _source_root) = paths::root(source.parent().unwrap()).unwrap();
    let mut source_guard = paths::file(&source).unwrap();
    let mut source_bytes = Vec::new();
    (&mut source_guard)
        .take(crate::domain::frame_capture::MAX_FRAME_PNG_BYTES + 1)
        .read_to_end(&mut source_bytes)
        .unwrap();
    assert_eq!(hash(&source_bytes), expected);
    let (width, height) =
        crate::infrastructure::snapshot_export::inspect_static_png(&source_bytes).unwrap();
    assert!(width >= 8 && height >= 8);
    let rgba = image::load_from_memory_with_format(&source_bytes, image::ImageFormat::Png)
        .unwrap()
        .into_rgba8()
        .into_raw();
    let mut mask = vec![0; pixel_count(width, height).unwrap()];
    for y in height / 4..height * 3 / 4 {
        for x in width / 4..width * 3 / 4 {
            mask[y as usize * width as usize + x as usize] = 255;
        }
    }
    let mask_sha = hash(&mask);
    let transform = AssetTransform {
        trim: true,
        mask_threshold: 0,
        feather_radius: 1,
        scale: PixelScale {
            numerator: 1,
            denominator: 2,
        },
        pivot: NormalizedPivot {
            x: NORMALIZED_SCALE / 2,
            y: NORMALIZED_SCALE,
        },
        alpha_mode: AlphaMode::Straight,
        color: AssetColor::Srgb,
    };
    let output = render(
        SourcePixels {
            width,
            height,
            rgba: &rgba,
            mask: &mask,
        },
        transform,
        LayerKind::Foreground,
        &Control,
    )
    .unwrap();
    let png_path = root.join("foreground.png");
    let mask_path = root.join("mask.png");
    write_png(
        &png_path,
        output.geometry.output_width,
        output.geometry.output_height,
        png::ColorType::Rgba,
        &output.rgba,
    );
    write_png(
        &mask_path,
        output.geometry.output_width,
        output.geometry.output_height,
        png::ColorType::Grayscale,
        &output.mask,
    );
    let output_bytes = fs::read(&png_path).unwrap();
    let mask_bytes = fs::read(&mask_path).unwrap();
    assert_eq!(
        crate::infrastructure::snapshot_export::inspect_static_png(&output_bytes).unwrap(),
        (output.geometry.output_width, output.geometry.output_height)
    );
    assert_eq!(
        image::load_from_memory_with_format(&output_bytes, image::ImageFormat::Png)
            .unwrap()
            .into_rgba8()
            .into_raw(),
        output.rgba
    );
    assert_eq!(
        image::load_from_memory_with_format(&mask_bytes, image::ImageFormat::Png)
            .unwrap()
            .into_luma8()
            .into_raw(),
        output.mask
    );
    let probe = PathBuf::from(std::env::var_os("ASSET_PNG_PROBE").expect("Pinned ffprobe"));
    let probe_sha = std::env::var("ASSET_PNG_PROBE_SHA256").unwrap();
    let (probe, _probe_guards) = tools::open(&probe, &probe_sha).unwrap();
    for (name, path, pixel_format) in [
        ("foreground", &png_path, "rgba"),
        ("mask", &mask_path, "gray"),
    ] {
        let mut command = Command::new(&probe);
        command
            .args([
                "-v",
                "error",
                "-protocol_whitelist",
                "file",
                "-show_entries",
                "stream=codec_name,width,height,pix_fmt",
                "-of",
                "json",
            ])
            .arg(path);
        let observed = process::run_with_recovery(
            &mut command,
            Instant::now() + Duration::from_secs(20),
            65536,
            Some(&Control),
            None,
        )
        .unwrap();
        fs::write(root.join(format!("{name}-probe.json")), &observed.stdout).unwrap();
        let value: serde_json::Value = serde_json::from_slice(&observed.stdout).unwrap();
        let stream = &value["streams"][0];
        assert_eq!(stream["codec_name"], "png");
        assert_eq!(stream["width"], output.geometry.output_width);
        assert_eq!(stream["height"], output.geometry.output_height);
        assert_eq!(stream["pix_fmt"], pixel_format);
    }
    assert_eq!(hash(&fs::read(&source).unwrap()), expected);
    assert_eq!(hash(&mask), mask_sha);
    fs::write(root.join("verification.json"),serde_json::to_vec_pretty(&serde_json::json!({"source_png_sha256":expected,"source_width":width,"source_height":height,"mask_origin":"explicit_synthetic_manual_rectangle","mask_pixels_sha256":mask_sha,"source_pts":null,"source_color_interpretation":"declared_srgb_not_verified_colorimetry","transform":transform,"geometry":output.geometry,"foreground_png_sha256":hash(&output_bytes),"mask_png_sha256":hash(&mask_bytes),"pixel_roundtrip_verified":true,"independent_ffprobe_png_dimensions_and_pixel_formats_verified":true,"source_bytes_unchanged":true,"quality_approved":false,"sdk_cli_verified":false,"gui_visual_verified":false})).unwrap()).unwrap();
}

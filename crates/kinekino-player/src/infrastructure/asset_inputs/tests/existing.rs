//! Reuse fixed R212 output PNGs as new static inputs; no original-video PTS claim.
use super::*;
#[test]
#[ignore = "Pinned existing PNG pair, fresh evidence, application/native guards; no inference/GUI/user review approval"]
fn existing_png_pair_stays_bound_and_guarded_through_shared_application_render() {
    let image_path =
        PathBuf::from(std::env::var_os("ASSET_INPUT_IMAGE").expect("Pinned existing image"));
    let mask_path =
        PathBuf::from(std::env::var_os("ASSET_INPUT_MASK").expect("Pinned existing mask"));
    let image_sha = std::env::var("ASSET_INPUT_IMAGE_SHA256").unwrap();
    let mask_sha = std::env::var("ASSET_INPUT_MASK_SHA256").unwrap();
    let root = PathBuf::from(std::env::var_os("ASSET_INPUT_EVIDENCE_DIR").expect("Fresh evidence"));
    fs::create_dir(&root).unwrap();
    let image_bytes = fs::read(&image_path).unwrap();
    let mask_bytes = fs::read(&mask_path).unwrap();
    assert_eq!(hash(&image_bytes), image_sha);
    assert_eq!(hash(&mask_bytes), mask_sha);
    let (width, height) = snapshot_export::inspect_static_png(&image_bytes).unwrap();
    assert_eq!(
        snapshot_export::inspect_static_png(&mask_bytes).unwrap(),
        (width, height)
    );
    let image = AssetImagePin {
        blob: AssetBlobPin {
            artifact_id: "existing-r212-image".into(),
            revision: DecimalU64(1),
            sha256: image_sha.clone(),
        },
        width,
        height,
        origin: AssetFrameOrigin::StaticPng,
        declared_color: AssetColor::Srgb,
    };
    let mask = AssetMaskPin {
        blob: AssetBlobPin {
            artifact_id: "existing-r212-mask".into(),
            revision: DecimalU64(1),
            sha256: mask_sha.clone(),
        },
        source: image.clone(),
        encoding: AssetMaskEncoding::Gray8Coverage,
        origin: AssetMaskOrigin::ImportedUnverified,
    };
    let ports = FixedAssetInputs::new(
        BTreeMap::from([(
            image.blob.artifact_id.clone(),
            FixedAssetImage {
                pin: image.clone(),
                path: image_path.clone(),
            },
        )]),
        BTreeMap::from([(
            mask.blob.artifact_id.clone(),
            FixedAssetMask {
                pin: mask.clone(),
                path: mask_path.clone(),
            },
        )]),
    )
    .unwrap();
    let request = AssetFrameRequest {
        schema_version: "asset-frame-build/1".into(),
        image,
        review: AssetMaskReview::UnreviewedAcknowledged {
            mask: mask.blob.clone(),
        },
        mask,
        layer: LayerKind::Foreground,
        transform: AssetTransform {
            trim: false,
            mask_threshold: 0,
            feather_radius: 0,
            scale: PixelScale {
                numerator: 1,
                denominator: 1,
            },
            pivot: NormalizedPivot {
                x: 500000,
                y: 1000000,
            },
            alpha_mode: AlphaMode::Straight,
            color: AssetColor::Srgb,
        },
    };
    let admitted = asset_generation::prepare(&request, &ports, &Control).unwrap();
    drop(ports);
    assert!(OpenOptions::new().write(true).open(&image_path).is_err());
    assert!(OpenOptions::new().write(true).open(&mask_path).is_err());
    let rendered = admitted.render(&RustAssetRaster, &Control).unwrap();
    assert!(rendered.unreviewed_mask_warning());
    assert_eq!(rendered.request(), &request);
    let source = image::load_from_memory_with_format(&image_bytes, image::ImageFormat::Png)
        .unwrap()
        .into_rgba8();
    let coverage = image::load_from_memory_with_format(&mask_bytes, image::ImageFormat::Png)
        .unwrap()
        .into_luma8();
    for (pixel, (original, mask)) in rendered
        .pixels()
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .zip(source.pixels().zip(coverage.pixels()))
    {
        let alpha = ((u32::from(original[3]) * u32::from(mask[0]) + 127) / 255) as u8;
        assert_eq!(pixel[3], alpha);
        if alpha > 0 {
            assert_eq!(&pixel[..3], &original.0[..3]);
        } else {
            assert_eq!(&pixel[..3], &[0, 0, 0]);
        }
    }
    let output = root.join("rendered.png");
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&output)
        .unwrap();
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&rendered.pixels().rgba).unwrap();
    writer.finish().unwrap();
    let output_bytes = fs::read(&output).unwrap();
    assert_eq!(
        image::load_from_memory_with_format(&output_bytes, image::ImageFormat::Png)
            .unwrap()
            .into_rgba8()
            .into_raw(),
        rendered.pixels().rgba
    );
    fs::write(
        root.join("request.json"),
        serde_json::to_vec_pretty(&request).unwrap(),
    )
    .unwrap();
    fs::write(root.join("verification.json"),serde_json::to_vec_pretty(&serde_json::json!({"source_image_sha256":image_sha,"source_mask_sha256":mask_sha,"output_png_sha256":hash(&output_bytes),"width":width,"height":height,"geometry":rendered.pixels().geometry,"original_catalog_dropped_before_render":true,"original_source_and_mask_write_denied_while_result_owned":true,"application_mask_alpha_and_pixel_roundtrip_verified":true,"unreviewed_mask_warning":rendered.unreviewed_mask_warning(),"source_pts":null,"source_origin":"existing_r212_pngs_reused_as_static_inputs","quality_approved":false,"sdk_cli_verified":false,"gui_visual_verified":false})).unwrap()).unwrap();
    assert_eq!(hash(&fs::read(&image_path).unwrap()), image_sha);
    assert_eq!(hash(&fs::read(&mask_path).unwrap()), mask_sha);
    drop(rendered);
    assert!(OpenOptions::new().write(true).open(&image_path).is_ok());
    assert!(OpenOptions::new().write(true).open(&mask_path).is_ok());
}

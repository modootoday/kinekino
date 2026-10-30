use super::*;
use crate::{
    application::asset_generation, domain::api_values::DecimalU64,
    infrastructure::asset_raster::RustAssetRaster,
};
use std::{
    fs::{self, OpenOptions},
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Control;
impl MutationControl for Control {
    fn checkpoint(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn png(path: &Path, color: png::ColorType, bytes: &[u8], gamma: bool, srgb: bool, trns: bool) {
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    let mut encoder = png::Encoder::new(file, 2, 1);
    encoder.set_color(color);
    encoder.set_depth(png::BitDepth::Eight);
    if gamma {
        encoder.set_source_gamma(png::ScaledFloat::new(1.0));
    }
    if srgb {
        encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
    }
    if trns {
        encoder.set_trns(vec![0, 255, 0, 0, 0, 0]);
    }
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(bytes).unwrap();
    writer.finish().unwrap();
}
struct Fixture {
    root: PathBuf,
    request: AssetFrameRequest,
    ports: FixedAssetInputs,
}
impl Fixture {
    fn new(mask_alpha: bool, gamma: bool, srgb: bool, trns: bool) -> Self {
        let root = std::env::temp_dir().join(format!(
            "asset-input-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir(&root).unwrap();
        let image = root.join("source.png");
        let mask = root.join("mask.png");
        png(
            &image,
            png::ColorType::Rgb,
            &[255, 0, 0, 0, 0, 255],
            gamma,
            srgb,
            trns,
        );
        if mask_alpha {
            png(
                &mask,
                png::ColorType::Rgba,
                &[100, 200, 50, 64, 255, 0, 0, 255],
                false,
                false,
                false,
            );
        } else {
            png(
                &mask,
                png::ColorType::Grayscale,
                &[64, 255],
                false,
                false,
                false,
            );
        }
        let image = AssetImagePin {
            blob: AssetBlobPin {
                artifact_id: "source".into(),
                revision: DecimalU64(1),
                sha256: hash(&fs::read(&image).unwrap()),
            },
            width: 2,
            height: 1,
            origin: AssetFrameOrigin::StaticPng,
            declared_color: AssetColor::Srgb,
        };
        let mask = AssetMaskPin {
            blob: AssetBlobPin {
                artifact_id: "mask".into(),
                revision: DecimalU64(1),
                sha256: hash(&fs::read(&mask).unwrap()),
            },
            source: image.clone(),
            encoding: if mask_alpha {
                AssetMaskEncoding::Rgba8Alpha
            } else {
                AssetMaskEncoding::Gray8Coverage
            },
            origin: AssetMaskOrigin::ImportedUnverified,
        };
        let ports = FixedAssetInputs::new(
            BTreeMap::from([(
                "source".into(),
                FixedAssetImage {
                    pin: image.clone(),
                    path: root.join("source.png"),
                },
            )]),
            BTreeMap::from([(
                "mask".into(),
                FixedAssetMask {
                    pin: mask.clone(),
                    path: root.join("mask.png"),
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
        Self {
            root,
            request,
            ports,
        }
    }
}
#[test]
fn original_gray_and_rgba_alpha_mask_leases_survive_catalog_drop_and_keep_warning() {
    for alpha in [false, true] {
        let fixture = Fixture::new(alpha, false, false, false);
        let prepared =
            asset_generation::prepare(&fixture.request, &fixture.ports, &Control).unwrap();
        assert!(prepared.unreviewed_mask_warning());
        drop(fixture.ports);
        let rendered = prepared.render(&RustAssetRaster, &Control).unwrap();
        assert_eq!(rendered.pixels().rgba, [255, 0, 0, 64, 0, 0, 255, 255]);
        assert_eq!(rendered.pixels().mask, [64, 255]);
        assert_eq!(rendered.request(), &fixture.request);
        assert!(rendered.unreviewed_mask_warning());
        assert_eq!(
            OpenOptions::new()
                .write(true)
                .open(fixture.root.join("source.png"))
                .is_err(),
            crate::infrastructure::os_policy::has_mandatory_share_modes()
        );
        assert_eq!(
            OpenOptions::new()
                .write(true)
                .open(fixture.root.join("mask.png"))
                .is_err(),
            crate::infrastructure::os_policy::has_mandatory_share_modes()
        );
        drop(rendered);
        assert!(
            OpenOptions::new()
                .write(true)
                .open(fixture.root.join("source.png"))
                .is_ok()
        );
        assert!(
            OpenOptions::new()
                .write(true)
                .open(fixture.root.join("mask.png"))
                .is_ok()
        );
    }
}
#[test]
fn full_frame_binding_rejects_reusing_same_mask_bytes_for_another_image_identity() {
    let mut fixture = Fixture::new(false, false, false, false);
    let mut other = fixture.request.image.clone();
    other.blob.artifact_id = "other".into();
    fixture.ports.images.insert(
        "other".into(),
        FixedAssetImage {
            pin: other.clone(),
            path: fixture.root.join("source.png"),
        },
    );
    fixture.request.image = other.clone();
    fixture.request.mask.source = other;
    assert_eq!(
        asset_generation::prepare(&fixture.request, &fixture.ports, &Control)
            .err()
            .unwrap()
            .code,
        ErrorCode::SourceExpired
    );
}
#[test]
fn measured_png_digest_rejects_a_matching_request_and_catalog_with_false_hash() {
    let mut fixture = Fixture::new(false, false, false, false);
    fixture.request.image.blob.sha256 = "a".repeat(64);
    fixture.request.mask.source = fixture.request.image.clone();
    fixture.ports.images.get_mut("source").unwrap().pin = fixture.request.image.clone();
    fixture.ports.masks.get_mut("mask").unwrap().pin = fixture.request.mask.clone();
    assert_eq!(
        asset_generation::prepare(&fixture.request, &fixture.ports, &Control)
            .err()
            .unwrap()
            .code,
        ErrorCode::SourceExpired
    );
}
#[test]
fn request_flags_cannot_create_review_or_reuse_old_mask_acknowledgement() {
    let fixture = Fixture::new(false, false, false, false);
    let mut request = fixture.request.clone();
    if let AssetMaskReview::UnreviewedAcknowledged { mask } = &mut request.review {
        mask.revision.0 += 1;
    }
    assert!(request.validate().is_err());
    let mut json = serde_json::to_value(&fixture.request).unwrap();
    json["reviewed"] = serde_json::json!(true);
    assert!(serde_json::from_value::<AssetFrameRequest>(json).is_err());
    request = fixture.request.clone();
    request.review = AssetMaskReview::Reference {
        record: AssetBlobPin {
            artifact_id: "claimed-review".into(),
            revision: DecimalU64(1),
            sha256: "a".repeat(64),
        },
    };
    assert_eq!(
        asset_generation::prepare(&request, &fixture.ports, &Control)
            .err()
            .unwrap()
            .code,
        ErrorCode::Unsupported
    );
    assert!(
        OpenOptions::new()
            .write(true)
            .open(fixture.root.join("source.png"))
            .is_ok()
    );
}
#[test]
fn srgb_tag_is_recognized_but_untagged_gamma_requires_explicit_conversion() {
    let source = Fixture::new(false, true, false, false);
    assert_eq!(
        asset_generation::prepare(&source.request, &source.ports, &Control)
            .err()
            .unwrap()
            .code,
        ErrorCode::Unsupported
    );
    let source = Fixture::new(false, true, true, false);
    assert!(asset_generation::prepare(&source.request, &source.ports, &Control).is_ok());
}
#[test]
fn rgb_transparency_is_expanded_without_turning_transparent_red_opaque() {
    let fixture = Fixture::new(false, false, false, true);
    let rendered = asset_generation::prepare(&fixture.request, &fixture.ports, &Control)
        .unwrap()
        .render(&RustAssetRaster, &Control)
        .unwrap();
    assert_eq!(rendered.pixels().rgba, [0, 0, 0, 0, 0, 0, 255, 255]);
    assert_eq!(rendered.pixels().mask, [0, 255]);
}
#[test]
fn observed_video_values_preserve_large_signed_ticks_but_need_original_observation_resolver() {
    let mut fixture = Fixture::new(false, false, false, false);
    fixture.request.image.origin = AssetFrameOrigin::ObservedVideo {
        source_sha256: "b".repeat(64),
        stream_index: 2,
        pts: "9007199254740993".into(),
        clock: crate::domain::clip_document::SourceClock {
            origin_pts: "-1000".into(),
            time_base_numerator: 1,
            time_base_denominator: 90000,
            observation_sha256: "c".repeat(64),
        },
        rotation_degrees: 90,
    };
    fixture.request.mask.source = fixture.request.image.clone();
    assert!(fixture.request.validate().is_ok());
    assert_eq!(
        asset_generation::prepare(&fixture.request, &fixture.ports, &Control)
            .err()
            .unwrap()
            .code,
        ErrorCode::Unsupported
    );
    if let AssetFrameOrigin::ObservedVideo { pts, .. } = &mut fixture.request.image.origin {
        *pts = "1.5".into();
    }
    fixture.request.mask.source = fixture.request.image.clone();
    assert!(fixture.request.validate().is_err());
}
#[test]
fn complete_png_stream_and_explicit_mask_encoding_are_not_replaced_by_matching_hashes() {
    let mut fixture = Fixture::new(false, false, false, false);
    let mut bytes = fs::read(fixture.root.join("source.png")).unwrap();
    bytes.push(0);
    fs::write(fixture.root.join("trailing.png"), &bytes).unwrap();
    fixture.request.image.blob.sha256 = hash(&bytes);
    fixture.request.mask.source = fixture.request.image.clone();
    let entry = fixture.ports.images.get_mut("source").unwrap();
    entry.pin = fixture.request.image.clone();
    entry.path = fixture.root.join("trailing.png");
    fixture.ports.masks.get_mut("mask").unwrap().pin = fixture.request.mask.clone();
    assert_eq!(
        asset_generation::prepare(&fixture.request, &fixture.ports, &Control)
            .err()
            .unwrap()
            .code,
        ErrorCode::InvalidArgument
    );
    let mut fixture = Fixture::new(false, false, false, false);
    fixture.request.mask.encoding = AssetMaskEncoding::Rgba8Alpha;
    fixture.ports.masks.get_mut("mask").unwrap().pin = fixture.request.mask.clone();
    assert_eq!(
        asset_generation::prepare(&fixture.request, &fixture.ports, &Control)
            .err()
            .unwrap()
            .code,
        ErrorCode::Unsupported
    );
}
#[test]
fn application_rejects_raster_alpha_mismatch_and_keeps_first_cancellation_error() {
    struct Bad;
    impl AssetRasterPort for Bad {
        fn render(
            &self,
            input: AssetRasterInput<'_>,
            transform: AssetTransform,
            layer: LayerKind,
            control: &dyn MutationControl,
        ) -> Result<AssetRasterOutput, ApiError> {
            let mut output = RustAssetRaster.render(input, transform, layer, control)?;
            output.mask[0] ^= 1;
            Ok(output)
        }
    }
    let fixture = Fixture::new(false, false, false, false);
    let prepared = asset_generation::prepare(&fixture.request, &fixture.ports, &Control).unwrap();
    assert_eq!(
        prepared.render(&Bad, &Control).err().unwrap().code,
        ErrorCode::SourceExpired
    );
    struct Cancel;
    impl MutationControl for Cancel {
        fn checkpoint(&self) -> Result<(), ApiError> {
            let mut error = ApiError::new(
                ErrorCode::Cancelled,
                "Original input admission cancellation",
            );
            error.details = serde_json::json!({"first":true});
            Err(error)
        }
    }
    let error = asset_generation::prepare(&fixture.request, &fixture.ports, &Cancel)
        .err()
        .unwrap();
    assert_eq!(error.code, ErrorCode::Cancelled);
    assert_eq!(error.details, serde_json::json!({"first":true}));
    assert!(
        OpenOptions::new()
            .write(true)
            .open(fixture.root.join("source.png"))
            .is_ok()
    );
}
#[test]
fn ignored_invalid_icc_payload_remains_an_explicit_unsupported_source_profile() {
    let mut fixture = Fixture::new(false, false, false, false);
    let path = fixture.root.join("invalid-icc.png");
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .unwrap();
    let mut encoder = png::Encoder::new(file, 2, 1);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer
        .write_chunk(
            png::chunk::iCCP,
            b"unusable\0\0not a compressed ICC profile",
        )
        .unwrap();
    writer.write_image_data(&[255, 0, 0, 0, 0, 255]).unwrap();
    writer.finish().unwrap();
    let bytes = fs::read(&path).unwrap();
    assert_eq!(snapshot_export::inspect_static_png(&bytes).unwrap(), (2, 1));
    fixture.request.image.blob.sha256 = hash(&bytes);
    fixture.request.mask.source = fixture.request.image.clone();
    let entry = fixture.ports.images.get_mut("source").unwrap();
    entry.path = path;
    entry.pin = fixture.request.image.clone();
    fixture.ports.masks.get_mut("mask").unwrap().pin = fixture.request.mask.clone();
    assert_eq!(
        asset_generation::prepare(&fixture.request, &fixture.ports, &Control)
            .err()
            .unwrap()
            .code,
        ErrorCode::Unsupported
    );
}
mod existing;
mod review;

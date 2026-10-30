use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
struct Control;
impl MutationControl for Control {
    fn checkpoint(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
fn transform() -> AssetTransform {
    AssetTransform {
        trim: true,
        mask_threshold: 0,
        feather_radius: 0,
        scale: PixelScale {
            numerator: 1,
            denominator: 1,
        },
        pivot: NormalizedPivot {
            x: 0,
            y: NORMALIZED_SCALE,
        },
        alpha_mode: AlphaMode::Straight,
        color: AssetColor::Srgb,
    }
}
#[test]
fn source_alpha_trim_offset_uv_and_unclamped_pivot_preserve_original_pixels() {
    let mut rgba = vec![0_u8; 4 * 3 * 4];
    rgba[5 * 4..6 * 4].copy_from_slice(&[255, 0, 0, 128]);
    rgba[6 * 4..7 * 4].copy_from_slice(&[0, 0, 255, 255]);
    let original = rgba.clone();
    let mut mask = vec![0_u8; 12];
    mask[5] = 128;
    mask[6] = 255;
    let result = render(
        SourcePixels {
            width: 4,
            height: 3,
            rgba: &rgba,
            mask: &mask,
        },
        transform(),
        LayerKind::Foreground,
        &Control,
    )
    .unwrap();
    assert_eq!(result.rgba, [255, 0, 0, 64, 0, 0, 255, 255]);
    assert_eq!(result.mask, [64, 255]);
    assert_eq!(
        result.geometry.source_bounds,
        PixelCrop {
            x: 1,
            y: 1,
            width: 2,
            height: 1
        }
    );
    assert_eq!(
        result.geometry.source_uv,
        SourceUv {
            left: 1,
            top: 1,
            right: 3,
            bottom: 2,
            denominator_x: 4,
            denominator_y: 3
        }
    );
    assert_eq!(result.geometry.pivot_x.numerator, "-2000000");
    assert_eq!(result.geometry.pivot_x.denominator.0, 2000000);
    assert_eq!(result.geometry.pivot_y.numerator, "2000000");
    assert_eq!(rgba, original);
    let premult = render(
        SourcePixels {
            width: 4,
            height: 3,
            rgba: &rgba,
            mask: &mask,
        },
        AssetTransform {
            alpha_mode: AlphaMode::Premultiplied,
            ..transform()
        },
        LayerKind::Foreground,
        &Control,
    )
    .unwrap();
    assert_eq!(premult.rgba, [64, 0, 0, 64, 0, 0, 255, 255]);
    assert_eq!(premult.geometry, result.geometry);
}
#[test]
fn complementary_mattes_keep_source_alpha_without_inventing_background_pixels() {
    let rgba = [20, 40, 60, 255, 80, 100, 120, 128];
    let mask = [64, 128];
    let input = || SourcePixels {
        width: 2,
        height: 1,
        rgba: &rgba,
        mask: &mask,
    };
    let foreground = render(input(), transform(), LayerKind::Foreground, &Control).unwrap();
    let background = render(input(), transform(), LayerKind::Background, &Control).unwrap();
    assert_eq!(foreground.mask, [64, 64]);
    assert_eq!(background.mask, [191, 64]);
    assert_eq!(&foreground.rgba[..3], &rgba[..3]);
    assert_eq!(&background.rgba[..3], &rgba[..3]);
    for index in 0..2 {
        assert_eq!(
            u16::from(foreground.mask[index]) + u16::from(background.mask[index]),
            u16::from(rgba[index * 4 + 3])
        );
    }
}
#[test]
fn associated_linear_lanczos_does_not_bleed_hidden_transparent_red_into_blue() {
    let rgba = [255, 0, 0, 0, 0, 0, 255, 255];
    let mask = [255, 255];
    let options = AssetTransform {
        trim: false,
        scale: PixelScale {
            numerator: 2,
            denominator: 1,
        },
        ..transform()
    };
    for mode in [AlphaMode::Straight, AlphaMode::Premultiplied] {
        let result = render(
            SourcePixels {
                width: 2,
                height: 1,
                rgba: &rgba,
                mask: &mask,
            },
            AssetTransform {
                alpha_mode: mode,
                ..options
            },
            LayerKind::Foreground,
            &Control,
        )
        .unwrap();
        assert_eq!(
            (result.geometry.output_width, result.geometry.output_height),
            (4, 2)
        );
        assert!(result.mask.iter().any(|alpha| *alpha > 0 && *alpha < 255));
        for pixel in result.rgba.as_chunks::<4>().0 {
            assert_eq!(&pixel[..2], &[0, 0]);
            assert_eq!(
                pixel[2],
                if pixel[3] == 0 {
                    0
                } else if mode == AlphaMode::Straight {
                    255
                } else {
                    pixel[3]
                }
            );
        }
    }
}
#[test]
fn black_white_downsample_uses_linear_light_and_preserves_full_source_uv() {
    let result = render(
        SourcePixels {
            width: 2,
            height: 1,
            rgba: &[0, 0, 0, 255, 255, 255, 255, 255],
            mask: &[255, 255],
        },
        AssetTransform {
            trim: false,
            scale: PixelScale {
                numerator: 1,
                denominator: 2,
            },
            ..transform()
        },
        LayerKind::Foreground,
        &Control,
    )
    .unwrap();
    assert_eq!(
        (result.geometry.output_width, result.geometry.output_height),
        (1, 1)
    );
    assert!((185..=190).contains(&result.rgba[0]), "{:?}", result.rgba);
    assert_eq!(&result.rgba[..3], &[result.rgba[0]; 3]);
    assert_eq!(result.rgba[3], 255);
    assert_eq!(result.geometry.source_uv.right, 2);
}
#[test]
fn threshold_precedes_zero_extended_box_feather_in_source_pixel_coordinates() {
    let rgba = vec![255; 3 * 3 * 4];
    let mut mask = vec![64; 9];
    mask[4] = 128;
    let result = render(
        SourcePixels {
            width: 3,
            height: 3,
            rgba: &rgba,
            mask: &mask,
        },
        AssetTransform {
            mask_threshold: 127,
            feather_radius: 1,
            ..transform()
        },
        LayerKind::Foreground,
        &Control,
    )
    .unwrap();
    assert_eq!(result.mask, vec![14; 9]);
    assert_eq!(
        result.geometry.source_bounds,
        PixelCrop {
            x: 0,
            y: 0,
            width: 3,
            height: 3
        }
    );
    assert_eq!(mask[4], 128);
}
#[test]
fn identity_path_roundtrips_straight_rgb_and_every_nonzero_source_alpha() {
    let mut rgba = Vec::new();
    for alpha in 0..=255_u8 {
        rgba.extend([alpha, 255 - alpha, 17, alpha]);
    }
    let result = render(
        SourcePixels {
            width: 256,
            height: 1,
            rgba: &rgba,
            mask: &[255; 256],
        },
        AssetTransform {
            trim: false,
            ..transform()
        },
        LayerKind::Foreground,
        &Control,
    )
    .unwrap();
    assert_eq!(&result.rgba[..4], &[0, 0, 0, 0]);
    assert_eq!(&result.rgba[4..], &rgba[4..]);
}
#[test]
fn invalid_budget_empty_mask_and_shape_mismatch_fail_before_output() {
    assert!(pixel_count(u32::MAX, u32::MAX).is_err());
    let bounds = PixelCrop {
        x: 0,
        y: 0,
        width: 3840,
        height: 2160,
    };
    assert!(
        geometry(
            AssetTransform {
                scale: PixelScale {
                    numerator: 2,
                    denominator: 1
                },
                ..transform()
            },
            3840,
            2160,
            bounds
        )
        .is_err()
    );
    assert!(
        geometry(
            AssetTransform {
                trim: false,
                ..transform()
            },
            4,
            3,
            PixelCrop {
                x: 1,
                y: 1,
                width: 1,
                height: 1
            }
        )
        .is_err()
    );
    assert!(
        AssetTransform {
            feather_radius: 65,
            ..transform()
        }
        .validate()
        .is_err()
    );
    let error = render(
        SourcePixels {
            width: 1,
            height: 1,
            rgba: &[255; 4],
            mask: &[0],
        },
        transform(),
        LayerKind::Foreground,
        &Control,
    )
    .err()
    .unwrap();
    assert_eq!(error.code, ErrorCode::NeedsInput);
    let error = render(
        SourcePixels {
            width: 1,
            height: 1,
            rgba: &[255; 3],
            mask: &[255],
        },
        transform(),
        LayerKind::Foreground,
        &Control,
    )
    .err()
    .unwrap();
    assert_eq!(error.code, ErrorCode::InvalidArgument);
}
#[test]
fn every_exposed_checkpoint_keeps_first_error_and_original_inputs_unchanged() {
    struct Fault {
        count: AtomicUsize,
        fail_at: usize,
    }
    impl MutationControl for Fault {
        fn checkpoint(&self) -> Result<(), ApiError> {
            if self.count.fetch_add(1, Ordering::SeqCst) == self.fail_at {
                let mut error = ApiError::new(ErrorCode::Cancelled, "Original caller cancellation");
                error.details = serde_json::json!({"original":true});
                Err(error)
            } else {
                Ok(())
            }
        }
    }
    let rgba = vec![255; 36];
    let mask = vec![128; 9];
    let options = AssetTransform {
        feather_radius: 1,
        scale: PixelScale {
            numerator: 2,
            denominator: 1,
        },
        ..transform()
    };
    let baseline = Fault {
        count: AtomicUsize::new(0),
        fail_at: usize::MAX,
    };
    render(
        SourcePixels {
            width: 3,
            height: 3,
            rgba: &rgba,
            mask: &mask,
        },
        options,
        LayerKind::Foreground,
        &baseline,
    )
    .unwrap();
    for fail_at in 0..baseline.count.load(Ordering::SeqCst) {
        let fault = Fault {
            count: AtomicUsize::new(0),
            fail_at,
        };
        let error = render(
            SourcePixels {
                width: 3,
                height: 3,
                rgba: &rgba,
                mask: &mask,
            },
            options,
            LayerKind::Foreground,
            &fault,
        )
        .err()
        .unwrap();
        assert_eq!(error.code, ErrorCode::Cancelled);
        assert_eq!(error.details, serde_json::json!({"original":true}));
    }
    assert_eq!(rgba, vec![255; 36]);
    assert_eq!(mask, vec![128; 9]);
}

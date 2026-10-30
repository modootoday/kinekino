use super::*;
use crate::domain::{asset_generation::*, snapshot_export::PixelCrop};
struct Control(bool);
impl MutationControl for Control {
    fn checkpoint(&self) -> Result<(), ApiError> {
        if self.0 {
            Err(ApiError::new(ErrorCode::Cancelled, "Cancelled codec"))
        } else {
            Ok(())
        }
    }
}
fn pixels() -> AssetRasterOutput {
    let transform = AssetTransform {
        trim: false,
        mask_threshold: 0,
        feather_radius: 0,
        scale: PixelScale {
            numerator: 1,
            denominator: 1,
        },
        pivot: NormalizedPivot { x: 0, y: 0 },
        alpha_mode: AlphaMode::Premultiplied,
        color: AssetColor::Srgb,
    };
    AssetRasterOutput {
        geometry: geometry(
            transform,
            2,
            1,
            PixelCrop {
                x: 0,
                y: 0,
                width: 2,
                height: 1,
            },
        )
        .unwrap(),
        rgba: vec![64, 32, 0, 128, 0, 0, 0, 0],
        mask: vec![128, 0],
        alpha_mode: AlphaMode::Premultiplied,
        color: AssetColor::Srgb,
    }
}
#[test]
fn premultiplied_raw_is_preserved_and_png_is_straight_while_coverage_is_untagged() {
    let original = pixels();
    let encoded = RustAssetLayerEncoder
        .encode(&original, &Control(false))
        .unwrap();
    assert_eq!(original.rgba, vec![64, 32, 0, 128, 0, 0, 0, 0]);
    assert_eq!(
        image::load_from_memory_with_format(&encoded.rgba_png, image::ImageFormat::Png)
            .unwrap()
            .into_rgba8()
            .into_raw(),
        vec![128, 64, 0, 128, 0, 0, 0, 0]
    );
    assert_eq!(
        image::load_from_memory_with_format(&encoded.mask_png, image::ImageFormat::Png)
            .unwrap()
            .into_luma8()
            .into_raw(),
        vec![128, 0]
    );
    let rgba = png::Decoder::new(std::io::Cursor::new(&encoded.rgba_png))
        .read_info()
        .unwrap();
    assert!(rgba.info().srgb.is_some());
    let mask = png::Decoder::new(std::io::Cursor::new(&encoded.mask_png))
        .read_info()
        .unwrap();
    assert!(mask.info().srgb.is_none());
    assert!(mask.info().gama_chunk.is_none());
}
#[test]
fn codec_rejects_invalid_lengths_and_honors_original_cancellation() {
    let mut original = pixels();
    assert_eq!(
        RustAssetLayerEncoder
            .encode(&original, &Control(true))
            .err()
            .unwrap()
            .code,
        ErrorCode::Cancelled
    );
    original.mask.pop();
    assert_eq!(
        RustAssetLayerEncoder
            .encode(&original, &Control(false))
            .err()
            .unwrap()
            .code,
        ErrorCode::EngineFailed
    );
}

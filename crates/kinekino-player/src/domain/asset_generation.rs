//! Pixel geometry and explicit transforms. Values confer no source or review admission.
mod document;
mod inputs;
use crate::domain::{
    api_values::DecimalU64, frame_capture::MAX_FRAME_PIXELS, snapshot_export::PixelCrop,
};
pub use document::*;
pub use inputs::*;
use serde::{Deserialize, Serialize};
pub const NORMALIZED_SCALE: u32 = 1_000_000;
pub const MAX_FEATHER_RADIUS: u32 = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlphaMode {
    Straight,
    Premultiplied,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayerKind {
    Foreground,
    Background,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetColor {
    Srgb,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PixelScale {
    pub numerator: u32,
    pub denominator: u32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NormalizedPivot {
    pub x: u32,
    pub y: u32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetTransform {
    pub trim: bool,
    /// Mask values at or below this threshold become zero before feathering.
    pub mask_threshold: u8,
    pub feather_radius: u32,
    pub scale: PixelScale,
    pub pivot: NormalizedPivot,
    pub alpha_mode: AlphaMode,
    /// Explicit declared interpretation; the file admission adapter must check it.
    pub color: AssetColor,
}
impl AssetTransform {
    pub fn validate(self) -> Result<(), &'static str> {
        if self.feather_radius > MAX_FEATHER_RADIUS
            || self.scale.numerator == 0
            || self.scale.denominator == 0
            || self.scale.numerator > 16
            || self.scale.denominator > 16
            || self.pivot.x > NORMALIZED_SCALE
            || self.pivot.y > NORMALIZED_SCALE
        {
            return Err("Invalid asset feather, scale or normalized pivot");
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignedPixelRatio {
    pub numerator: String,
    pub denominator: DecimalU64,
}
/// Exact top-left UV fractions; right/bottom are exclusive source edges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceUv {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub denominator_x: u32,
    pub denominator_y: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetGeometry {
    pub source_width: u32,
    pub source_height: u32,
    pub source_bounds: PixelCrop,
    pub output_width: u32,
    pub output_height: u32,
    pub source_uv: SourceUv,
    /// May lie outside the trimmed output. Never clamp or recenter the source pivot.
    pub pivot_x: SignedPixelRatio,
    pub pivot_y: SignedPixelRatio,
}
pub fn pixel_count(width: u32, height: u32) -> Result<usize, &'static str> {
    let count = u64::from(width) * u64::from(height);
    if count == 0 || count > MAX_FRAME_PIXELS {
        return Err("Asset pixel budget exceeded");
    }
    usize::try_from(count).map_err(|_| "Asset pixel budget exceeds address space")
}
pub fn geometry(
    transform: AssetTransform,
    width: u32,
    height: u32,
    bounds: PixelCrop,
) -> Result<AssetGeometry, &'static str> {
    transform.validate()?;
    pixel_count(width, height)?;
    bounds.validate_bounds(width, height)?;
    if !transform.trim
        && bounds
            != (PixelCrop {
                x: 0,
                y: 0,
                width,
                height,
            })
    {
        return Err("Untrimmed assets must preserve the original canvas");
    }
    let scaled = |value: u32| -> Result<u32, &'static str> {
        let value = (u64::from(value) * u64::from(transform.scale.numerator))
            .div_ceil(u64::from(transform.scale.denominator));
        u32::try_from(value).map_err(|_| "Scaled asset dimension overflow")
    };
    let (output_width, output_height) = (scaled(bounds.width)?, scaled(bounds.height)?);
    pixel_count(output_width, output_height)?;
    let pivot =
        |normalized: u32, original: u32, offset: u32, extent: u32, output: u32| SignedPixelRatio {
            numerator: ((i128::from(normalized) * i128::from(original)
                - i128::from(offset) * i128::from(NORMALIZED_SCALE))
                * i128::from(output))
            .to_string(),
            denominator: DecimalU64(u64::from(NORMALIZED_SCALE) * u64::from(extent)),
        };
    Ok(AssetGeometry {
        source_width: width,
        source_height: height,
        source_bounds: bounds,
        output_width,
        output_height,
        source_uv: SourceUv {
            left: bounds.x,
            top: bounds.y,
            right: bounds.x + bounds.width,
            bottom: bounds.y + bounds.height,
            denominator_x: width,
            denominator_y: height,
        },
        pivot_x: pivot(
            transform.pivot.x,
            width,
            bounds.x,
            bounds.width,
            output_width,
        ),
        pivot_y: pivot(
            transform.pivot.y,
            height,
            bounds.y,
            bounds.height,
            output_height,
        ),
    })
}

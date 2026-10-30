//! Declared sRGB straight RGBA + source-aligned A8 mask → retained output pixels.
//! This kernel neither opens files nor authenticates source/review/PTS provenance.
mod feather;
#[cfg(all(test, windows))]
mod file_tests;
#[cfg(test)]
mod tests;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        review_source::MutationControl,
    },
    domain::{asset_generation::*, snapshot_export::PixelCrop},
};
use image::{
    ImageBuffer, Rgba,
    imageops::{FilterType, resize},
};

pub use crate::application::asset_generation::{
    AssetRasterInput as SourcePixels, AssetRasterOutput as AssetPixels,
};
pub struct RustAssetRaster;
impl crate::application::asset_generation::AssetRasterPort for RustAssetRaster {
    fn render(
        &self,
        input: SourcePixels<'_>,
        transform: AssetTransform,
        layer: LayerKind,
        control: &dyn MutationControl,
    ) -> Result<AssetPixels, ApiError> {
        render(input, transform, layer, control)
    }
}
fn invalid(message: &str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
fn linear(value: u8) -> f32 {
    let value = f32::from(value) / 255.0;
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}
fn encoded(value: f32) -> u8 {
    let value = value.clamp(0.0, 1.0);
    let value = if value <= 0.0031308 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    };
    (value * 255.0).round().clamp(0.0, 255.0) as u8
}
pub fn render(
    source: SourcePixels<'_>,
    transform: AssetTransform,
    kind: LayerKind,
    control: &dyn MutationControl,
) -> Result<AssetPixels, ApiError> {
    transform.validate().map_err(invalid)?;
    let count = pixel_count(source.width, source.height).map_err(invalid)?;
    if source.rgba.len() != count * 4 || source.mask.len() != count {
        return Err(invalid("Source RGBA/mask dimensions differ"));
    }
    control.checkpoint()?;
    let mut mask = Vec::with_capacity(count);
    for row in source.mask.chunks_exact(source.width as usize) {
        control.checkpoint()?;
        mask.extend(row.iter().map(|value| {
            if *value <= transform.mask_threshold {
                0
            } else {
                *value
            }
        }));
    }
    if transform.feather_radius > 0 {
        mask = feather::apply(
            &mask,
            source.width,
            source.height,
            transform.feather_radius,
            control,
        )?;
    }
    let mut min_x = source.width;
    let mut min_y = source.height;
    let mut max_x = 0;
    let mut max_y = 0;
    for (y, row) in mask.chunks_exact_mut(source.width as usize).enumerate() {
        control.checkpoint()?;
        for (x, value) in row.iter_mut().enumerate() {
            if kind == LayerKind::Background {
                *value = 255 - *value;
            }
            let index = y * source.width as usize + x;
            *value =
                ((u32::from(*value) * u32::from(source.rgba[index * 4 + 3]) + 127) / 255) as u8;
            if *value > 0 {
                min_x = min_x.min(x as u32);
                min_y = min_y.min(y as u32);
                max_x = max_x.max(x as u32 + 1);
                max_y = max_y.max(y as u32 + 1);
            }
        }
    }
    if min_x >= max_x || min_y >= max_y {
        return Err(ApiError::new(
            ErrorCode::NeedsInput,
            "Selected asset layer is entirely transparent",
        ));
    }
    let bounds = if transform.trim {
        PixelCrop {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
        }
    } else {
        PixelCrop {
            x: 0,
            y: 0,
            width: source.width,
            height: source.height,
        }
    };
    let geometry = geometry(transform, source.width, source.height, bounds).map_err(invalid)?;
    let mut lut = [0_f32; 256];
    for (value, entry) in lut.iter_mut().enumerate() {
        *entry = linear(value as u8);
    }
    let mut associated = Vec::with_capacity(bounds.width as usize * bounds.height as usize * 4);
    for y in bounds.y..bounds.y + bounds.height {
        control.checkpoint()?;
        for x in bounds.x..bounds.x + bounds.width {
            let index = y as usize * source.width as usize + x as usize;
            let alpha = f32::from(mask[index]) / 255.0;
            for channel in &source.rgba[index * 4..index * 4 + 3] {
                associated.push(lut[*channel as usize] * alpha);
            }
            associated.push(alpha);
        }
    }
    let associated =
        ImageBuffer::<Rgba<f32>, Vec<f32>>::from_raw(bounds.width, bounds.height, associated)
            .ok_or_else(|| invalid("Associated asset buffer dimensions differ"))?;
    control.checkpoint()?;
    // The bounded library call has no interior cancellation hook. Its result stays
    // local and unpublished until the next checkpoint; do not acknowledge early.
    let scaled = if (bounds.width, bounds.height) == (geometry.output_width, geometry.output_height)
    {
        associated
    } else {
        resize(
            &associated,
            geometry.output_width,
            geometry.output_height,
            FilterType::Lanczos3,
        )
    };
    control.checkpoint()?;
    let output_count =
        pixel_count(geometry.output_width, geometry.output_height).map_err(invalid)?;
    let mut rgba = Vec::with_capacity(output_count * 4);
    let mut output_mask = Vec::with_capacity(output_count);
    for row in scaled
        .as_raw()
        .chunks_exact(geometry.output_width as usize * 4)
    {
        control.checkpoint()?;
        for pixel in row.as_chunks::<4>().0 {
            let alpha = pixel[3].clamp(0.0, 1.0);
            let alpha_byte = (alpha * 255.0).round() as u8;
            for channel in &pixel[..3] {
                let straight = if alpha_byte == 0 {
                    0
                } else {
                    encoded(channel.clamp(0.0, alpha) / alpha)
                };
                rgba.push(match transform.alpha_mode {
                    AlphaMode::Straight => straight,
                    AlphaMode::Premultiplied => {
                        ((u32::from(straight) * u32::from(alpha_byte) + 127) / 255) as u8
                    }
                });
            }
            rgba.push(alpha_byte);
            output_mask.push(alpha_byte);
        }
    }
    control.checkpoint()?;
    Ok(AssetPixels {
        geometry,
        rgba,
        mask: output_mask,
        alpha_mode: transform.alpha_mode,
        color: transform.color,
    })
}

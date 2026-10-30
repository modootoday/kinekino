//! PNG is straight sRGB8; mask is untagged coverage, original raw alpha bytes stay separate.
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        asset_bundle::{AssetEncodedBuffers, AssetLayerEncoder},
        asset_generation::AssetRasterOutput,
        review_source::MutationControl,
    },
    domain::asset_generation::{AlphaMode, pixel_count},
};
pub struct RustAssetLayerEncoder;
#[cfg(test)]
mod tests;
fn failed(message: impl std::fmt::Display) -> ApiError {
    ApiError::new(ErrorCode::EngineFailed, message.to_string())
}
fn encode(
    bytes: &[u8],
    width: u32,
    height: u32,
    color: png::ColorType,
    srgb: bool,
) -> Result<Vec<u8>, ApiError> {
    let mut output = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut output, width, height);
        encoder.set_color(color);
        encoder.set_depth(png::BitDepth::Eight);
        if srgb {
            encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
        }
        let mut writer = encoder.write_header().map_err(failed)?;
        writer.write_image_data(bytes).map_err(failed)?;
        writer.finish().map_err(failed)?;
    }
    Ok(output)
}
impl AssetLayerEncoder for RustAssetLayerEncoder {
    fn encode(
        &self,
        pixels: &AssetRasterOutput,
        control: &dyn MutationControl,
    ) -> Result<AssetEncodedBuffers, ApiError> {
        control.checkpoint()?;
        let width = pixels.geometry.output_width;
        let height = pixels.geometry.output_height;
        let count = pixel_count(width, height).map_err(failed)?;
        if pixels.rgba.len() != count * 4 || pixels.mask.len() != count {
            return Err(failed("Invalid asset raster buffers"));
        }
        let mut straight = pixels.rgba.clone();
        if pixels.alpha_mode == AlphaMode::Premultiplied {
            for row in straight.chunks_exact_mut(width as usize * 4) {
                control.checkpoint()?;
                for pixel in row.as_chunks_mut::<4>().0 {
                    let alpha = u32::from(pixel[3]);
                    for channel in &mut pixel[..3] {
                        *channel = (u32::from(*channel) * 255 + alpha / 2)
                            .checked_div(alpha)
                            .unwrap_or(0)
                            .min(255) as u8;
                    }
                }
            }
        }
        control.checkpoint()?;
        let rgba_png = encode(&straight, width, height, png::ColorType::Rgba, true)?;
        control.checkpoint()?;
        let mask_png = encode(
            &pixels.mask,
            width,
            height,
            png::ColorType::Grayscale,
            false,
        )?;
        control.checkpoint()?;
        if image::load_from_memory_with_format(&rgba_png, image::ImageFormat::Png)
            .map_err(failed)?
            .into_rgba8()
            .into_raw()
            != straight
            || image::load_from_memory_with_format(&mask_png, image::ImageFormat::Png)
                .map_err(failed)?
                .into_luma8()
                .into_raw()
                != pixels.mask
        {
            return Err(failed(
                "Asset PNG round trip differs from actual output buffers",
            ));
        }
        control.checkpoint()?;
        Ok(AssetEncodedBuffers { rgba_png, mask_png })
    }
}

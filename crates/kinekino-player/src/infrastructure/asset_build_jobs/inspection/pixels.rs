use super::*;
use crate::{
    application::asset_bundle::AssetLayerOutput,
    domain::asset_generation::{AlphaMode, pixel_count},
};
use std::io::BufReader;
fn decoder(
    file: &File,
    layer: &AssetLayerOutput,
    mask: bool,
) -> Result<png::Reader<BufReader<File>>, ApiError> {
    let mut file = file
        .try_clone()
        .map_err(kinekino_core::file_publication::io_error)?;
    file.seek(SeekFrom::Start(0))
        .map_err(kinekino_core::file_publication::io_error)?;
    let mut options = png::DecodeOptions::default();
    options.set_ignore_adler32(false);
    options.set_ignore_crc(false);
    options.set_skip_ancillary_crc_failures(false);
    let mut decoder = png::Decoder::new_with_options(BufReader::new(file), options);
    decoder.set_limits(png::Limits {
        bytes: crate::domain::frame_capture::MAX_FRAME_PNG_BYTES as usize,
    });
    let reader = decoder.read_info().map_err(|_| changed())?;
    let info = reader.info();
    if info.width != layer.geometry.output_width
        || info.height != layer.geometry.output_height
        || info.bit_depth != png::BitDepth::Eight
        || info.color_type
            != if mask {
                png::ColorType::Grayscale
            } else {
                png::ColorType::Rgba
            }
        || info.interlaced
        || info.animation_control.is_some()
        || info.icc_profile.is_some()
        || info.coding_independent_code_points.is_some()
        || if mask {
            info.srgb.is_some() || info.gama_chunk.is_some() || info.chrm_chunk.is_some()
        } else {
            info.srgb != Some(png::SrgbRenderingIntent::Perceptual)
        }
    {
        return Err(changed());
    }
    Ok(reader)
}
/// Decode row by row from original held files; no whole-sequence pixel allocation.
pub(super) fn verify(
    layer: &AssetLayerOutput,
    raw: &File,
    png: &File,
    mask: &File,
    control: &dyn MutationControl,
) -> Result<(), ApiError> {
    pixel_count(layer.geometry.output_width, layer.geometry.output_height)
        .map_err(|_| changed())?;
    let mut rgba = decoder(png, layer, false)?;
    let mut coverage = decoder(mask, layer, true)?;
    let mut raw = raw
        .try_clone()
        .map_err(kinekino_core::file_publication::io_error)?;
    raw.seek(SeekFrom::Start(0))
        .map_err(kinekino_core::file_publication::io_error)?;
    let mut row = vec![0; layer.geometry.output_width as usize * 4];
    for _ in 0..layer.geometry.output_height {
        control.checkpoint()?;
        raw.read_exact(&mut row)
            .map_err(kinekino_core::file_publication::io_error)?;
        let decoded = rgba
            .next_row()
            .map_err(|_| changed())?
            .ok_or_else(changed)?;
        let mask = coverage
            .next_row()
            .map_err(|_| changed())?
            .ok_or_else(changed)?;
        if decoded.data().len() != row.len() || mask.data().len() != row.len() / 4 {
            return Err(changed());
        }
        for ((raw, png), mask) in row
            .as_chunks::<4>()
            .0
            .iter()
            .zip(decoded.data().as_chunks::<4>().0.iter())
            .zip(mask.data())
        {
            let alpha = u32::from(raw[3]);
            if png[3] != raw[3] || *mask != raw[3] {
                return Err(changed());
            }
            for channel in 0..3 {
                let expected = match layer.raw_alpha_mode {
                    AlphaMode::Straight => raw[channel],
                    AlphaMode::Premultiplied => {
                        if u32::from(raw[channel]) > alpha {
                            return Err(changed());
                        }
                        ((u32::from(raw[channel]) * 255 + alpha / 2)
                            .checked_div(alpha)
                            .unwrap_or(0)
                            .min(255)) as u8
                    }
                };
                if png[channel] != expected {
                    return Err(changed());
                }
            }
        }
    }
    if rgba.next_row().map_err(|_| changed())?.is_some()
        || coverage.next_row().map_err(|_| changed())?.is_some()
    {
        return Err(changed());
    }
    rgba.finish().map_err(|_| changed())?;
    coverage.finish().map_err(|_| changed())?;
    control.checkpoint()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "Pinned existing asset manifest and exclusive fresh temporary files required"]
    fn real_png_rows_match_raw_and_reject_reencoded_pixel_or_mask_changes() {
        let root = std::env::temp_dir().join(format!(
            "asset-pixel-readback-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let original = PathBuf::from(
            std::env::var_os("ASSET_PIXEL_MANIFEST").expect("Pinned existing manifest required"),
        );
        let manifest: crate::application::asset_bundle::AssetBundleManifest =
            serde_json::from_slice(&fs::read(original).unwrap()).unwrap();
        let mut layer = manifest.layers[0].clone();
        layer.geometry.output_width = 2;
        layer.geometry.output_height = 1;
        let write_png = |name: &str, data: &[u8], mask: bool| {
            let file = File::create(root.join(name)).unwrap();
            let mut encoder = png::Encoder::new(file, 2, 1);
            encoder.set_depth(png::BitDepth::Eight);
            encoder.set_color(if mask {
                png::ColorType::Grayscale
            } else {
                png::ColorType::Rgba
            });
            if !mask {
                encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
            }
            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(data).unwrap();
            writer.finish().unwrap();
        };
        for mode in [AlphaMode::Straight, AlphaMode::Premultiplied] {
            layer.raw_alpha_mode = mode;
            let raw = if mode == AlphaMode::Straight {
                vec![90, 40, 10, 128, 0, 0, 0, 0]
            } else {
                vec![45, 20, 5, 128, 0, 0, 0, 0]
            };
            let expected = if mode == AlphaMode::Straight {
                raw.clone()
            } else {
                vec![90, 40, 10, 128, 0, 0, 0, 0]
            };
            fs::write(root.join("raw"), &raw).unwrap();
            write_png("rgba", &expected, false);
            write_png("mask", &[128, 0], true);
            let verify_now = || {
                verify(
                    &layer,
                    &File::open(root.join("raw")).unwrap(),
                    &File::open(root.join("rgba")).unwrap(),
                    &File::open(root.join("mask")).unwrap(),
                    &Observe,
                )
            };
            verify_now().unwrap();
            write_png("mask", &[127, 0], true);
            assert!(verify_now().is_err());
            write_png("mask", &[128, 0], true);
            let mut wrong = expected.clone();
            wrong[0] ^= 1;
            write_png("rgba", &wrong, false);
            assert!(verify_now().is_err());
        }
    }
}

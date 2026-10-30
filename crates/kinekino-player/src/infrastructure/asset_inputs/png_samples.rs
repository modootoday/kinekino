use super::*;
use std::io::Cursor;
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::InvalidArgument,
        "Asset PNG dimensions or sample encoding differ",
    )
}
// Invalid iCCP payloads are ignored by the PNG decoder. Inspect chunk presence
// too, so a missing decoded profile cannot silently become declared sRGB.
fn reject_profile_chunks(bytes: &[u8], control: &dyn MutationControl) -> Result<(), ApiError> {
    let mut offset = 8_usize;
    while offset < bytes.len() {
        control.checkpoint()?;
        let header = bytes
            .get(offset..offset.checked_add(8).ok_or_else(invalid)?)
            .ok_or_else(invalid)?;
        let length = u32::from_be_bytes(header[..4].try_into().map_err(|_| invalid())?) as usize;
        let end = offset
            .checked_add(length)
            .and_then(|value| value.checked_add(12))
            .filter(|end| *end <= bytes.len())
            .ok_or_else(invalid)?;
        if &header[4..] == b"iCCP" || &header[4..] == b"cICP" {
            return Err(unsupported(
                "Asset PNG contains an ICC/cICP profile requiring explicit supported conversion",
            ));
        }
        offset = end;
    }
    Ok(())
}
fn decode(
    bytes: &[u8],
    width: u32,
    height: u32,
    source_rgb: bool,
    control: &dyn MutationControl,
) -> Result<(png::OutputInfo, Vec<u8>), ApiError> {
    control.checkpoint()?;
    if snapshot_export::inspect_static_png(bytes)? != (width, height) {
        return Err(changed());
    }
    if source_rgb {
        reject_profile_chunks(bytes, control)?;
    }
    control.checkpoint()?;
    let mut options = png::DecodeOptions::default();
    options.set_ignore_adler32(false);
    options.set_ignore_crc(false);
    options.set_skip_ancillary_crc_failures(false);
    let mut decoder = png::Decoder::new_with_options(Cursor::new(bytes), options);
    decoder.set_limits(png::Limits {
        bytes: MAX_FRAME_PNG_BYTES as usize,
    });
    decoder.set_transformations(png::Transformations::EXPAND);
    let mut reader = decoder.read_info().map_err(|_| invalid())?;
    let info = reader.info();
    if info.bit_depth != png::BitDepth::Eight {
        return Err(unsupported(
            "Asset input currently requires explicit 8-bit samples; no silent precision reduction",
        ));
    }
    if source_rgb
        && (info.icc_profile.is_some()
            || info.coding_independent_code_points.is_some()
            || (info.srgb.is_none() && (info.gama_chunk.is_some() || info.chrm_chunk.is_some())))
    {
        return Err(unsupported(
            "Asset input requires a supported explicit color profile conversion; declared sRGB cannot override PNG metadata",
        ));
    }
    let count = pixel_count(width, height).map_err(|_| invalid())?;
    let size = reader
        .output_buffer_size()
        .filter(|size| *size <= count * 4)
        .ok_or_else(invalid)?;
    let mut pixels = vec![0; size];
    control.checkpoint()?;
    let info = reader.next_frame(&mut pixels).map_err(|_| invalid())?;
    reader.finish().map_err(|_| invalid())?;
    pixels.truncate(info.buffer_size());
    control.checkpoint()?;
    Ok((info, pixels))
}
pub(super) fn image(
    bytes: &[u8],
    width: u32,
    height: u32,
    control: &dyn MutationControl,
) -> Result<Vec<u8>, ApiError> {
    let (info, pixels) = decode(bytes, width, height, true, control)?;
    match info.color_type {
        png::ColorType::Rgba => Ok(pixels),
        png::ColorType::Rgb => {
            let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
            for row in pixels.chunks_exact(width as usize * 3) {
                control.checkpoint()?;
                for pixel in row.as_chunks::<3>().0 {
                    rgba.extend_from_slice(pixel);
                    rgba.push(255);
                }
            }
            Ok(rgba)
        }
        _ => Err(unsupported("Asset source requires RGB8 or RGBA8 PNG")),
    }
}
pub(super) fn mask(
    bytes: &[u8],
    width: u32,
    height: u32,
    encoding: AssetMaskEncoding,
    control: &dyn MutationControl,
) -> Result<Vec<u8>, ApiError> {
    let (info, pixels) = decode(bytes, width, height, false, control)?;
    match (encoding, info.color_type) {
        (AssetMaskEncoding::Gray8Coverage, png::ColorType::Grayscale) => Ok(pixels),
        (AssetMaskEncoding::Rgba8Alpha, png::ColorType::Rgba) => {
            let mut coverage = Vec::with_capacity(width as usize * height as usize);
            for row in pixels.chunks_exact(width as usize * 4) {
                control.checkpoint()?;
                coverage.extend(row.as_chunks::<4>().0.iter().map(|pixel| pixel[3]));
            }
            Ok(coverage)
        }
        _ => Err(unsupported(
            "Mask PNG differs from its explicit coverage/alpha encoding",
        )),
    }
}

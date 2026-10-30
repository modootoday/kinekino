use crate::{
    application::api::{ApiError, ErrorCode},
    domain::{
        frame_capture::{MAX_FRAME_PIXELS, MAX_FRAME_PNG_BYTES},
        snapshot_export::PixelCrop,
    },
};
use std::io::Cursor;

pub(super) struct CroppedPng {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
}
pub(super) fn budget() -> ApiError {
    ApiError::new(
        ErrorCode::ResourceExhausted,
        "PNG exceeds the shared frame byte or pixel budget",
    )
}
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::InvalidArgument,
        "Source must be a complete, CRC-valid static PNG with no trailing data",
    )
}
fn decoding(error: png::DecodingError) -> ApiError {
    match error {
        png::DecodingError::LimitsExceeded => budget(),
        _ => invalid(),
    }
}

/// Require exactly one complete PNG stream; png::Reader::finish validates CRC/chunks.
fn complete_stream(bytes: &[u8]) -> Result<(), ApiError> {
    if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Err(invalid());
    }
    let mut offset = 8usize;
    loop {
        let header = bytes
            .get(offset..offset.checked_add(8).ok_or_else(invalid)?)
            .ok_or_else(invalid)?;
        let length = u32::from_be_bytes(header[..4].try_into().map_err(|_| invalid())?) as usize;
        let end = offset
            .checked_add(length)
            .and_then(|n| n.checked_add(12))
            .ok_or_else(invalid)?;
        if end > bytes.len() {
            return Err(invalid());
        }
        if &header[4..] == b"IEND" {
            return if length == 0 && end == bytes.len() {
                Ok(())
            } else {
                Err(invalid())
            };
        }
        offset = end;
    }
}
struct DecodedPng {
    width: u32,
    height: u32,
    info: png::OutputInfo,
    pixels: Vec<u8>,
    decoded_limit: usize,
}
fn decode_static(
    bytes: &[u8],
    before_pixels: impl FnOnce(u32, u32) -> Result<(), ApiError>,
) -> Result<DecodedPng, ApiError> {
    if bytes.len() as u64 > MAX_FRAME_PNG_BYTES {
        return Err(budget());
    }
    complete_stream(bytes)?;
    let decoded_limit = MAX_FRAME_PIXELS
        .checked_mul(4)
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(budget)?;
    let mut options = png::DecodeOptions::default();
    options.set_ignore_adler32(false);
    options.set_ignore_crc(false);
    options.set_skip_ancillary_crc_failures(false);
    let mut decoder = png::Decoder::new_with_options(Cursor::new(bytes), options);
    decoder.set_limits(png::Limits {
        bytes: usize::try_from(MAX_FRAME_PNG_BYTES).map_err(|_| budget())?,
    });
    decoder.set_transformations(png::Transformations::EXPAND);
    let mut reader = decoder.read_info().map_err(decoding)?;
    let (width, height) = (reader.info().width, reader.info().height);
    if reader.info().animation_control.is_some() {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Animated PNG snapshots are unsupported",
        ));
    }
    if width == 0
        || height == 0
        || u64::from(width)
            .checked_mul(u64::from(height))
            .is_none_or(|n| n > MAX_FRAME_PIXELS)
    {
        return Err(budget());
    }
    before_pixels(width, height)?;
    let size = reader
        .output_buffer_size()
        .filter(|size| *size <= decoded_limit)
        .ok_or_else(budget)?;
    let mut decoded = vec![0; size];
    let info = reader.next_frame(&mut decoded).map_err(decoding)?;
    reader.finish().map_err(decoding)?;
    Ok(DecodedPng {
        width,
        height,
        info,
        pixels: decoded,
        decoded_limit,
    })
}
pub(super) fn inspect_static_png(bytes: &[u8]) -> Result<(u32, u32), ApiError> {
    let decoded = decode_static(bytes, |_, _| Ok(()))?;
    if !matches!(
        decoded.info.bit_depth,
        png::BitDepth::Eight | png::BitDepth::Sixteen
    ) {
        return Err(invalid());
    }
    Ok((decoded.width, decoded.height))
}
pub(super) fn crop(bytes: &[u8], crop: PixelCrop) -> Result<CroppedPng, ApiError> {
    let decoded = decode_static(bytes, |width, height| {
        crop.validate_bounds(width, height)
            .map_err(|message| ApiError::new(ErrorCode::InvalidArgument, message))
    })?;
    let DecodedPng {
        width,
        height,
        info,
        pixels: decoded,
        decoded_limit,
    } = decoded;
    let sample_bytes = match info.bit_depth {
        png::BitDepth::Eight => 1usize,
        png::BitDepth::Sixteen => 2,
        _ => return Err(invalid()),
    };
    let pixel_bytes = info
        .color_type
        .samples()
        .checked_mul(sample_bytes)
        .ok_or_else(budget)?;
    let row_bytes = (crop.width as usize)
        .checked_mul(pixel_bytes)
        .ok_or_else(budget)?;
    let capacity = row_bytes
        .checked_mul(crop.height as usize)
        .filter(|n| *n <= decoded_limit)
        .ok_or_else(budget)?;
    let mut pixels = Vec::with_capacity(capacity);
    for row in crop.y..crop.y.checked_add(crop.height).ok_or_else(invalid)? {
        let start = (row as usize)
            .checked_mul(info.line_size)
            .and_then(|n| {
                (crop.x as usize)
                    .checked_mul(pixel_bytes)
                    .and_then(|x| n.checked_add(x))
            })
            .ok_or_else(budget)?;
        let end = start.checked_add(row_bytes).ok_or_else(budget)?;
        pixels.extend_from_slice(decoded.get(start..end).ok_or_else(invalid)?);
    }
    let mut output = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut output, crop.width, crop.height);
        encoder.set_color(info.color_type);
        encoder.set_depth(info.bit_depth);
        let mut writer = encoder.write_header().map_err(|_| invalid())?;
        writer.write_image_data(&pixels).map_err(|_| invalid())?;
        writer.finish().map_err(|_| invalid())?;
    }
    if output.len() as u64 > MAX_FRAME_PNG_BYTES {
        return Err(budget());
    }
    Ok(CroppedPng {
        bytes: output,
        width,
        height,
    })
}

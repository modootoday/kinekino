use crate::{
    application::api::{ApiError, ErrorCode},
    domain::frame_capture::MAX_FRAME_PIXELS,
};
use std::io::Cursor;

pub fn validate(bytes: &[u8]) -> Result<(u32, u32), ApiError> {
    let decoder = png::Decoder::new(Cursor::new(bytes));
    let mut reader = decoder.read_info().map_err(|_| invalid())?;
    let width = reader.info().width;
    let height = reader.info().height;
    if width == 0 || height == 0 || u64::from(width) * u64::from(height) > MAX_FRAME_PIXELS {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Decoded PNG dimensions exceed pixel budget",
        ));
    }
    let size = reader
        .output_buffer_size()
        .filter(|size| *size <= MAX_FRAME_PIXELS as usize * 4)
        .ok_or_else(invalid)?;
    let mut decoded = vec![0; size];
    reader.next_frame(&mut decoded).map_err(|_| invalid())?;
    reader.finish().map_err(|_| invalid())?;
    Ok((width, height))
}
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::EngineFailed,
        "Decoder output is not a complete verified PNG",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_truncation_and_crc_corruption_before_publication() {
        let mut encoded = Vec::new();
        {
            let encoder = png::Encoder::new(&mut encoded, 1, 1);
            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(&[127]).unwrap();
        }
        assert_eq!(validate(&encoded).unwrap(), (1, 1));
        assert!(validate(&encoded[..encoded.len() / 2]).is_err());
        encoded[29] ^= 1;
        assert!(validate(&encoded).is_err());
    }
}

use crate::application::api::{ApiError, ErrorCode};
use kinekino_card_render::{
    application::ports::PngInspection,
    domain::{RenderLimits, Size},
};
pub(super) struct StrictPng;
impl PngInspection for StrictPng {
    type Error = ApiError;
    fn inspect_static_png(&self, bytes: &[u8], limits: RenderLimits) -> Result<Size, ApiError> {
        if bytes.len() > limits.max_png_bytes {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "PNG exceeds caller byte limit",
            ));
        }
        let (width, height) = crate::infrastructure::snapshot_export::inspect_static_png(bytes)?;
        if u64::from(width) * u64::from(height) > limits.max_pixels {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "PNG exceeds caller pixel limit",
            ));
        }
        Ok(Size { width, height })
    }
}

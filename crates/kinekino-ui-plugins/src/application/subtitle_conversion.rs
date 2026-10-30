use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_subtitles::application::convert::{ConversionResult, ConvertSubtitles};
use serde_json::Value;

/// The host owns ApplicationClient and constructs its canonical request context.
/// This adapter neither owns a registry nor evaluates conversion/storage policy.
pub struct SubtitleConversionAdapter<F> {
    invoke: F,
}

impl<F> SubtitleConversionAdapter<F>
where
    F: Fn(ConvertSubtitles) -> Result<ConversionResult, ApiError>,
{
    pub fn new(invoke: F) -> Self {
        Self { invoke }
    }

    pub fn dispatch(&self, payload: Value) -> Result<Value, ApiError> {
        let request = serde_json::from_value::<ConvertSubtitles>(payload).map_err(|_| {
            ApiError::new(
                ErrorCode::InvalidArgument,
                "Expected the canonical subtitle conversion request",
            )
        })?;
        let result = (self.invoke)(request)?;
        serde_json::to_value(result).map_err(|_| {
            ApiError::new(
                ErrorCode::Internal,
                "Subtitle conversion result encoding failed",
            )
        })
    }
}

/// Used only when the host has no compatible SDK transport for this target.
pub fn unavailable_conversion(_: ConvertSubtitles) -> Result<ConversionResult, ApiError> {
    Err(ApiError::new(
        ErrorCode::Unsupported,
        "Subtitle conversion SDK transport is unavailable for this host",
    ))
}

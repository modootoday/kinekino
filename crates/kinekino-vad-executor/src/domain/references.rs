use kinekino_media_contracts::errors::{ApiError, ErrorCode};
/// Private feature lookup key, never a path/URI/executable selector.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpaqueReference(String);
impl OpaqueReference {
    pub fn new(value: String) -> Result<Self, ApiError> {
        if value.is_empty()
            || value.len() > 128
            || !value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
        {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid opaque VAD reference",
            ));
        };
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpectedDigest(String);
impl ExpectedDigest {
    pub fn new(value: String) -> Result<Self, ApiError> {
        if !kinekino_jobs::domain::valid_sha256(&value) {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid expected VAD digest",
            ));
        };
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

//! Bounded pinned startup bytes. Configuration is not a timeline authority.
use crate::{
    application::api::{ApiError, ErrorCode},
    domain::identifiers::valid_sha256,
    infrastructure::{content_digest, immutable_storage_paths},
};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};
pub struct RetainedReviewStartupSettings {
    bytes: Vec<u8>,
    _guards: Vec<File>,
}
impl RetainedReviewStartupSettings {
    pub fn open(path: &Path, expected: &str) -> Result<Self, ApiError> {
        if !valid_sha256(expected) {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Settings SHA256 required",
            ));
        }
        let (_, mut guards) = immutable_storage_paths::root(path.parent().ok_or_else(|| {
            ApiError::new(
                ErrorCode::InvalidArgument,
                "Absolute settings file required",
            )
        })?)?;
        let mut file = immutable_storage_paths::file(path)?;
        if content_digest::sha256(&mut file, 65536)?.1 != expected.to_ascii_lowercase() {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Review startup settings digest differs",
            ));
        }
        file.seek(SeekFrom::Start(0)).map_err(io_error)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).map_err(io_error)?;
        guards.push(file);
        Ok(Self {
            bytes,
            _guards: guards,
        })
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}
fn io_error(error: std::io::Error) -> ApiError {
    ApiError::new(ErrorCode::EngineFailed, error.to_string())
}

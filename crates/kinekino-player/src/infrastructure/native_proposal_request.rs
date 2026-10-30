//! Bounded local request bytes; decoding and launch policy belong to application.
use super::immutable_storage_paths;
use crate::application::{
    api::{ApiError, ErrorCode},
    launch::native_proposal::MAX_NATIVE_PROPOSAL_REQUEST_BYTES,
};
use std::{io::Read, path::Path};

pub fn read_native_proposal_request(path: &Path) -> Result<Vec<u8>, ApiError> {
    let parent = path
        .parent()
        .filter(|_| path.is_absolute())
        .ok_or_else(|| {
            ApiError::new(
                ErrorCode::InvalidArgument,
                "Native proposal request requires an absolute file path",
            )
        })?;
    let name = path.file_name().ok_or_else(|| {
        ApiError::new(
            ErrorCode::InvalidArgument,
            "Native proposal request requires a file name",
        )
    })?;
    let (root, _guards) = immutable_storage_paths::root(parent)?;
    let file = immutable_storage_paths::file(&root.join(name))?;
    let mut bytes = Vec::new();
    file.take(MAX_NATIVE_PROPOSAL_REQUEST_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| {
            ApiError::new(
                ErrorCode::EngineFailed,
                format!("Native proposal request read failed: {error}"),
            )
        })?;
    if bytes.len() > MAX_NATIVE_PROPOSAL_REQUEST_BYTES {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Native proposal launch request exceeds 64 KiB",
        ));
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests;

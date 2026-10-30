//! Local host configuration only; never derived from media metadata or browser messages.
use crate::{
    application::api::{ApiError, ErrorCode},
    domain::identifiers::valid_sha256,
    infrastructure::{content_digest, immutable_storage_paths},
};
use std::{
    fs::File,
    path::{Path, PathBuf},
};

pub struct TrustedMpvExecutable {
    path: PathBuf,
    sha256: String,
    _guards: Vec<File>,
}

impl TrustedMpvExecutable {
    pub fn open(path: &Path, expected_sha256: &str) -> Result<Self, ApiError> {
        if !valid_sha256(expected_sha256)
            || path.file_name().and_then(|name| name.to_str()) != Some("mpv.exe")
        {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Native host requires a fixed mpv.exe path and SHA256",
            ));
        }
        let parent = path.parent().ok_or_else(|| {
            ApiError::new(
                ErrorCode::InvalidArgument,
                "Missing native executable parent",
            )
        })?;
        let (parent, mut guards) = immutable_storage_paths::root(parent)?;
        let path = parent.join("mpv.exe");
        let mut file = immutable_storage_paths::file(&path)?;
        let (_, sha256) = content_digest::sha256(&mut file, 512 * 1024 * 1024)?;
        if sha256 != expected_sha256 {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Native executable SHA256 does not match the explicit host configuration",
            ));
        }
        guards.push(file);
        Ok(Self {
            path,
            sha256,
            _guards: guards,
        })
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
}

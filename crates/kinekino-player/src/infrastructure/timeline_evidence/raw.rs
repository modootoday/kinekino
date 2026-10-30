use crate::{
    application::api::{ApiError, ErrorCode},
    domain::identifiers::valid_sha256,
    infrastructure::{content_digest, immutable_storage_paths as rooted},
};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};
const MAX_BYTES: u64 = 12 * 1024 * 1024;
pub(super) struct RetainedRaw {
    file: File,
    sha256: String,
    _directories: Vec<File>,
    max_bytes: u64,
}
impl RetainedRaw {
    pub fn open(path: &Path, expected: &str) -> Result<Self, ApiError> {
        Self::open_bounded(path, expected, MAX_BYTES)
    }
    pub fn open_bounded(path: &Path, expected: &str, max_bytes: u64) -> Result<Self, ApiError> {
        if max_bytes == 0 || max_bytes > 64 * 1024 * 1024 {
            return Err(invalid("Invalid observation byte bound"));
        }
        if !valid_sha256(expected) {
            return Err(invalid("Invalid observation SHA256"));
        }
        let (parent, directories) = rooted::root(
            path.parent()
                .ok_or_else(|| invalid("Observation parent required"))?,
        )?;
        let name = path
            .file_name()
            .ok_or_else(|| invalid("Observation file required"))?;
        let mut file = rooted::file(&parent.join(name))?;
        let (_, sha256) = content_digest::sha256(&mut file, max_bytes)?;
        if sha256 != expected {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Raw observation content differs",
            ));
        }
        Ok(Self {
            file,
            sha256,
            _directories: directories,
            max_bytes,
        })
    }
    pub fn read(&mut self) -> Result<String, ApiError> {
        String::from_utf8(self.read_bytes()?).map_err(|_| invalid("Observation requires UTF-8"))
    }
    pub fn read_bytes(&mut self) -> Result<Vec<u8>, ApiError> {
        self.file.seek(SeekFrom::Start(0)).map_err(io_error)?;
        let mut raw = Vec::new();
        Read::by_ref(&mut self.file)
            .take(self.max_bytes + 1)
            .read_to_end(&mut raw)
            .map_err(io_error)?;
        if raw.len() as u64 > self.max_bytes {
            return Err(invalid("Observation exceeds byte limit"));
        }
        Ok(raw)
    }
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
}
fn invalid(message: &str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
fn io_error(error: std::io::Error) -> ApiError {
    ApiError::new(
        ErrorCode::EngineFailed,
        format!("Observation read: {error}"),
    )
}

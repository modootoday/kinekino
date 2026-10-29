use crate::application::storage::{MAX_SUBTITLE_BYTES, SubtitleStorage};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use std::{fs::File, io::Read};

pub struct SubtitleFiles;

impl SubtitleStorage for SubtitleFiles {
    fn read_utf8(&self, path: &str) -> Result<String, ApiError> {
        let file = File::open(path).map_err(io_error)?;
        let metadata = file.metadata().map_err(io_error)?;
        if !metadata.is_file() {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Input must be a regular file",
            ));
        }
        if metadata.len() > MAX_SUBTITLE_BYTES as u64 {
            return Err(too_large());
        }
        let mut bytes = Vec::new();
        file.take(MAX_SUBTITLE_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(io_error)?;
        if bytes.len() > MAX_SUBTITLE_BYTES {
            return Err(too_large());
        }
        String::from_utf8(bytes)
            .map_err(|_| ApiError::new(ErrorCode::InvalidArgument, "Input must use UTF-8 encoding"))
    }

    fn publish_new(&self, path: &str, contents: &str) -> Result<(), ApiError> {
        kinekino_core::file_publication::publish(path, contents.as_bytes())
    }
}

fn too_large() -> ApiError {
    ApiError::new(ErrorCode::ResourceExhausted, "Subtitle input exceeds 8 MiB")
}
fn io_error(error: std::io::Error) -> ApiError {
    match error.kind() {
        std::io::ErrorKind::AlreadyExists => ApiError::new(
            ErrorCode::RevisionConflict,
            "Output already exists; choose a new path",
        ),
        std::io::ErrorKind::NotFound => ApiError::new(
            ErrorCode::NotFound,
            "Input file or output directory not found",
        ),
        std::io::ErrorKind::PermissionDenied => {
            ApiError::new(ErrorCode::PermissionDenied, "File access denied")
        }
        _ => ApiError::new(ErrorCode::EngineFailed, "Subtitle file operation failed"),
    }
}

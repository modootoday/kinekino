//! Existing PNG to new PNG; no decoder subprocess, source recapture or fabricated PTS.
#[cfg(test)]
mod inspection_tests;
mod raster;
/// Internal strict inspection reuses the same static stream decoder as crop.
pub(crate) fn inspect_static_png(
    bytes: &[u8],
) -> Result<(u32, u32), crate::application::api::ApiError> {
    raster::inspect_static_png(bytes)
}
#[cfg(test)]
mod tests;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        snapshot_export::{ExportedSnapshot, SnapshotExportPort},
    },
    domain::{frame_capture::MAX_FRAME_PNG_BYTES, snapshot_export::ExportSnapshot},
    infrastructure::content_digest,
};
use std::{
    fs::File,
    io::{Cursor, Read},
    path::Path,
};

pub struct PngSnapshotExport;
impl SnapshotExportPort for PngSnapshotExport {
    fn export(&self, request: &ExportSnapshot) -> Result<ExportedSnapshot, ApiError> {
        request
            .validate()
            .map_err(|message| ApiError::new(ErrorCode::InvalidArgument, message))?;
        let source = Path::new(&request.source_path)
            .canonicalize()
            .map_err(kinekino_core::file_publication::io_error)?;
        if source.to_string_lossy().starts_with(r"\\?\UNC\") {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Network PNG sources are unsupported",
            ));
        }
        if Path::new(&request.output_path).symlink_metadata().is_ok() {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Output already exists; choose a new path",
            ));
        }
        let output_parent = Path::new(&request.output_path)
            .parent()
            .ok_or_else(|| {
                ApiError::new(
                    ErrorCode::InvalidArgument,
                    "Output must name a new PNG file",
                )
            })?
            .canonicalize()
            .map_err(kinekino_core::file_publication::io_error)?;
        if output_parent.to_string_lossy().starts_with(r"\\?\UNC\") {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Network PNG outputs are unsupported",
            ));
        }
        let file = File::open(source).map_err(kinekino_core::file_publication::io_error)?;
        if !file
            .metadata()
            .map_err(kinekino_core::file_publication::io_error)?
            .is_file()
        {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Source must be a regular PNG file",
            ));
        }
        let mut bytes = Vec::new();
        file.take(MAX_FRAME_PNG_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(kinekino_core::file_publication::io_error)?;
        if bytes.len() as u64 > MAX_FRAME_PNG_BYTES {
            return Err(raster::budget());
        }
        let (_, source_sha256) =
            content_digest::sha256(&mut Cursor::new(&bytes), MAX_FRAME_PNG_BYTES)?;
        if source_sha256 != request.expected_source_sha256 {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Source PNG no longer matches the expected SHA256",
            ));
        }
        let result = raster::crop(&bytes, request.crop)?;
        let (output_bytes, output_sha256) =
            content_digest::sha256(&mut Cursor::new(&result.bytes), MAX_FRAME_PNG_BYTES)?;
        kinekino_core::file_publication::publish(&request.output_path, &result.bytes)?;
        Ok(ExportedSnapshot {
            output_path: request.output_path.clone(),
            source_sha256,
            output_sha256,
            source_width: result.width,
            source_height: result.height,
            crop: request.crop,
            output_bytes,
            metadata_preserved: false,
        })
    }
}

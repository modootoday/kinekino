use crate::{
    application::api::{ApiError, ErrorCode},
    domain::frame_capture::MAX_FRAME_PNG_BYTES,
};
use std::{
    fs::File,
    io::{Cursor, Read},
    path::{Path, PathBuf},
};
pub(super) fn local_png_path(path: &str) -> Result<PathBuf, ApiError> {
    let p = Path::new(path);
    if !p.is_absolute()
        || p.extension()
            .and_then(|x| x.to_str())
            .is_none_or(|x| !x.eq_ignore_ascii_case("png"))
        || !matches!(p.components().next(),Some(std::path::Component::Prefix(x)) if matches!(x.kind(),std::path::Prefix::Disk(_)|std::path::Prefix::VerbatimDisk(_)))
    {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Absolute local disk PNG path required",
        ));
    }
    Ok(p.to_path_buf())
}
pub(super) fn read(path: &str, sha: &str) -> Result<Vec<u8>, ApiError> {
    let source = local_png_path(path)?
        .canonicalize()
        .map_err(kinekino_core::file_publication::io_error)?;
    local_png_path(
        source
            .to_str()
            .ok_or_else(|| ApiError::new(ErrorCode::InvalidArgument, "Non-UTF8 frame path"))?,
    )?;
    let file = File::open(source).map_err(kinekino_core::file_publication::io_error)?;
    if !file
        .metadata()
        .map_err(kinekino_core::file_publication::io_error)?
        .is_file()
    {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Frame must be a regular file",
        ));
    }
    let mut bytes = Vec::new();
    file.take(MAX_FRAME_PNG_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(kinekino_core::file_publication::io_error)?;
    let (_, actual) = crate::infrastructure::content_digest::sha256(
        &mut Cursor::new(&bytes),
        MAX_FRAME_PNG_BYTES,
    )?;
    if actual != sha {
        return Err(ApiError::new(
            ErrorCode::RevisionConflict,
            "Frame differs from the explicitly pinned SHA256",
        ));
    }
    Ok(bytes)
}
pub(super) fn validate_output(path: &str) -> Result<(), ApiError> {
    let p = local_png_path(path)?;
    let parent = p
        .parent()
        .ok_or_else(|| ApiError::new(ErrorCode::InvalidArgument, "Output parent required"))?
        .canonicalize()
        .map_err(kinekino_core::file_publication::io_error)?;
    if !matches!(parent.components().next(),Some(std::path::Component::Prefix(x)) if matches!(x.kind(),std::path::Prefix::Disk(_)|std::path::Prefix::VerbatimDisk(_)))
    {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Network PNG output unsupported",
        ));
    }
    Ok(())
}

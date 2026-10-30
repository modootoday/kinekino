//! Complete-file publication from the original handle, without replacing a target.
use crate::application::api::{ApiError, ErrorCode};
use std::{
    fs::File,
    io::Write,
    path::{Component, Path},
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
/// Caller retains the trusted root/ancestor guards and its original writer lease.
/// Failed staging files are intentionally retained for inspection.
pub(crate) fn publish(directory: &Path, name: &str, bytes: &[u8]) -> Result<(), ApiError> {
    publish_inner(directory, name, bytes, false).map(drop)
}
/// Same original writer/rename, retained for readback and identity checks.
pub(crate) fn publish_retained(
    directory: &Path,
    name: &str,
    bytes: &[u8],
) -> Result<File, ApiError> {
    publish_inner(directory, name, bytes, true)
}
fn publish_inner(
    directory: &Path,
    name: &str,
    bytes: &[u8],
    readable: bool,
) -> Result<File, ApiError> {
    if name.is_empty()
        || name.encode_utf16().count() > 255
        || name.ends_with(['.', ' '])
        || name
            .chars()
            .any(|c| c.is_control() || matches!(c, ':' | '/' | '\\'))
        || !matches!(
            Path::new(name).components().collect::<Vec<_>>().as_slice(),
            [Component::Normal(_)]
        )
    {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Invalid immutable file name",
        ));
    }
    let _guard = super::immutable_storage_paths::directory(directory)?;
    let stage = directory.join(format!(
        ".pending-{}-{}.json",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = crate::infrastructure::os_policy::create_publication_stage(&stage, readable)
        .map_err(kinekino_core::file_publication::io_error)?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(kinekino_core::file_publication::io_error)?;
    rename_new(&file, &stage, &directory.join(name))?;
    Ok(file)
}
fn rename_new(file: &File, stage: &Path, target: &Path) -> Result<(), ApiError> {
    crate::infrastructure::os_policy::rename_no_replace(file, stage, target).map_err(|error| {
        if error.kind() == std::io::ErrorKind::InvalidFilename {
            ApiError::new(
                ErrorCode::InvalidArgument,
                "Immutable file path is too long",
            )
        } else {
            kinekino_core::file_publication::io_error(error)
        }
    })
}

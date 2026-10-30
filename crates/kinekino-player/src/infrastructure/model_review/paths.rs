use crate::application::api::{ApiError, ErrorCode};
use crate::infrastructure::os_policy::{self, OpenOptionsPolicy, ShareMode};
use std::{
    fs::{self, File, OpenOptions},
    path::{Component, Path, PathBuf},
};
fn denied() -> ApiError {
    ApiError::new(
        ErrorCode::PermissionDenied,
        "Review storage must remain in its local non-reparse root",
    )
}
fn ordinary(path: &Path) -> Result<(), ApiError> {
    let metadata = fs::symlink_metadata(path).map_err(super::io_error)?;
    if os_policy::is_reparse_point(&metadata) {
        return Err(denied());
    }
    Ok(())
}
pub fn absent(path: &Path) -> Result<bool, ApiError> {
    match fs::symlink_metadata(path) {
        Ok(_) => {
            ordinary(path)?;
            Ok(false)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(true),
        Err(e) => Err(super::io_error(e)),
    }
}
pub fn lock_directory(path: &Path) -> Result<File, ApiError> {
    ordinary(path)?;
    let file = OpenOptions::new()
        .read(true)
        .share(ShareMode::ReadWrite)
        .flags(os_policy::NO_FOLLOW_DIRECTORY)
        .open(path)
        .map_err(super::io_error)?;
    if !file.metadata().map_err(super::io_error)?.is_dir()
        || os_policy::is_reparse_point(&file.metadata().map_err(super::io_error)?)
    {
        return Err(denied());
    }
    Ok(file)
}
pub fn root(path: &Path) -> Result<(PathBuf, Vec<File>), ApiError> {
    // A canonicalized `\\?\C:\..` root is the same local volume; strip the verbatim prefix
    // first so the fixed-volume check (UNC/device still denied) sees the ordinary form.
    let normalized = os_policy::normalize_local_path(path).ok_or_else(denied)?;
    let path = normalized.as_path();
    if !path.is_absolute()
        || !os_policy::is_local_fixed_path(path)
        || path.components().any(|c| matches!(c, Component::ParentDir))
    {
        return Err(denied());
    }
    let mut ancestors: Vec<_> = path.ancestors().collect();
    ancestors.reverse();
    let mut guards = Vec::new();
    for directory in ancestors {
        guards.push(lock_directory(directory)?);
    }
    let canonical = path.canonicalize().map_err(super::io_error)?;
    Ok((canonical, guards))
}
pub fn file(path: &Path) -> Result<File, ApiError> {
    ordinary(path)?;
    let file = OpenOptions::new()
        .read(true)
        .share(ShareMode::ReadOnly)
        .flags(os_policy::NO_FOLLOW)
        .open(path)
        .map_err(super::io_error)?;
    let metadata = file.metadata().map_err(super::io_error)?;
    if !metadata.is_file() || os_policy::is_reparse_point(&metadata) {
        return Err(denied());
    }
    Ok(file)
}
pub fn create_locked(path: &Path) -> Result<File, ApiError> {
    match fs::create_dir(path) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(e) => return Err(super::io_error(e)),
    }
    lock_directory(path)
}

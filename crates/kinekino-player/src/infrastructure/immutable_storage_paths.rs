use crate::infrastructure::os_policy::{self, OpenFlags, ShareMode};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use std::{
    fs::{self, File, OpenOptions},
    path::{Component, Path, PathBuf},
};
const NO_FOLLOW: OpenFlags = OpenFlags {
    no_follow: true,
    directory_handle: false,
    delete_on_close: false,
    delete_access: false,
};
fn denied() -> ApiError {
    ApiError::new(
        ErrorCode::PermissionDenied,
        "Immutable storage requires ordinary local files within its fixed root",
    )
}
pub fn directory(path: &Path) -> Result<File, ApiError> {
    let metadata = fs::symlink_metadata(path).map_err(io_error)?;
    if os_policy::is_reparse_point(&metadata) || !metadata.is_dir() {
        return Err(denied());
    }
    let mut options = OpenOptions::new();
    options.read(true);
    os_policy::apply_share_mode(&mut options, ShareMode::ReadWrite);
    os_policy::apply_open_flags(
        &mut options,
        OpenFlags {
            directory_handle: true,
            ..NO_FOLLOW
        },
    );
    let f = options.open(path).map_err(io_error)?;
    if os_policy::is_reparse_point(&f.metadata().map_err(io_error)?) {
        return Err(denied());
    }
    Ok(f)
}
pub fn root(path: &Path) -> Result<(PathBuf, Vec<File>), ApiError> {
    if !os_policy::is_local_fixed_path(path)
        || path.components().any(|p| matches!(p, Component::ParentDir))
    {
        return Err(denied());
    }
    let mut ancestors: Vec<_> = path.ancestors().collect();
    ancestors.reverse();
    let guards = ancestors
        .into_iter()
        .map(directory)
        .collect::<Result<Vec<_>, _>>()?;
    Ok((path.canonicalize().map_err(io_error)?, guards))
}
pub fn file(path: &Path) -> Result<File, ApiError> {
    let mut options = OpenOptions::new();
    options.read(true);
    os_policy::apply_share_mode(&mut options, ShareMode::ReadOnly);
    os_policy::apply_open_flags(&mut options, NO_FOLLOW);
    let f = options.open(path).map_err(io_error)?;
    let m = f.metadata().map_err(io_error)?;
    if !m.is_file() || os_policy::is_reparse_point(&m) {
        return Err(denied());
    }
    Ok(f)
}
pub fn lock(path: &Path) -> Result<File, ApiError> {
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    os_policy::apply_share_mode(&mut options, ShareMode::Exclusive);
    os_policy::apply_open_flags(&mut options, NO_FOLLOW);
    let held = |e: std::io::Error| {
        if os_policy::is_sharing_violation(&e) || e.kind() == std::io::ErrorKind::WouldBlock {
            ApiError::new(
                ErrorCode::RevisionConflict,
                "Immutable storage writer is active; inspect and retry",
            )
        } else {
            io_error(e)
        }
    };
    let f = options.open(path).map_err(held)?;
    os_policy::try_lock_exclusive(&f).map_err(held)?;
    if os_policy::is_reparse_point(&f.metadata().map_err(io_error)?) {
        return Err(denied());
    }
    Ok(f)
}
fn io_error(error: std::io::Error) -> ApiError {
    ApiError::new(
        match error.kind() {
            std::io::ErrorKind::NotFound => ErrorCode::NotFound,
            std::io::ErrorKind::AlreadyExists => ErrorCode::RevisionConflict,
            std::io::ErrorKind::PermissionDenied => ErrorCode::PermissionDenied,
            _ => ErrorCode::EngineFailed,
        },
        format!("Immutable storage I/O: {error}"),
    )
}

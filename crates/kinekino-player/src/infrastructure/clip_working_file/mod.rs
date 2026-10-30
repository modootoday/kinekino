//! Newly created ephemeral bytes only. Original-handle deletion; no pathname removal API.
mod release;
use crate::infrastructure::os_policy::{
    OpenFlags, ShareMode, apply_open_flags, apply_share_mode, is_reparse_point,
};
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        review_source::MutationControl,
    },
    infrastructure::{
        content_digest, immutable_storage_paths as paths,
        storage_directory_identity::StorageDirectoryIdentity,
    },
};
pub(crate) use release::ClipWorkingFileRelease;
use serde::Serialize;
use std::{
    fs::{File, OpenOptions},
    io::{Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
#[derive(Debug, Serialize)]
pub(crate) struct WorkingFileFacts {
    pub schema_version: String,
    pub identity: StorageDirectoryIdentity,
    pub sha256: String,
    pub bytes: u64,
    pub delete_on_close_armed: bool,
}
/// Not serializable/reconstructible; owns CREATE_NEW and the original DELETE access.
pub(crate) struct ClipWorkingFile {
    file: File,
    path: PathBuf,
    identity: StorageDirectoryIdentity,
    maximum: usize,
    sealed: bool,
    attempted: bool,
    _guards: Vec<File>,
}
impl ClipWorkingFile {
    pub(crate) fn into_release(self) -> ClipWorkingFileRelease {
        ClipWorkingFileRelease::new(self)
    }
    pub(crate) fn create(
        root: &Path,
        maximum: usize,
        control: &dyn MutationControl,
    ) -> Result<Self, ApiError> {
        control.checkpoint()?;
        if maximum == 0 || maximum > 128 * 1024 * 1024 {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Bounded clip working-file bytes required",
            ));
        }
        let (root, guards) = paths::root(root)?;
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| {
                ApiError::new(
                    ErrorCode::EngineFailed,
                    "Cannot create working-file namespace",
                )
            })?
            .as_nanos();
        let path = root.join(format!(
            "clip-working-{}-{stamp}-{}.tmp",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        // READ | WRITE | DELETE; readers must share original write/delete access.
        // OPEN_REPARSE_POINT | DELETE_ON_CLOSE. Only this newly created object is removed.
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        apply_share_mode(&mut options, ShareMode::ReadOnly);
        apply_open_flags(
            &mut options,
            OpenFlags {
                no_follow: true,
                delete_on_close: true,
                delete_access: true,
                ..OpenFlags::default()
            },
        );
        let file = options
            .open(&path)
            .map_err(kinekino_core::file_publication::io_error)?;
        let metadata = file
            .metadata()
            .map_err(kinekino_core::file_publication::io_error)?;
        if !metadata.is_file() || is_reparse_point(&metadata) {
            return Err(ApiError::new(
                ErrorCode::PermissionDenied,
                "Working file must be a new ordinary local file",
            ));
        }
        let identity = StorageDirectoryIdentity::observe(&file)?;
        Ok(Self {
            file,
            path,
            identity,
            maximum,
            sealed: false,
            attempted: false,
            _guards: guards,
        })
    }
    pub(crate) fn seal(
        &mut self,
        bytes: &[u8],
        control: &dyn MutationControl,
    ) -> Result<WorkingFileFacts, ApiError> {
        control.checkpoint()?;
        if self.attempted {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Working file write was already attempted",
            ));
        }
        if bytes.len() > self.maximum {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Clip working bytes exceed admission",
            ));
        }
        self.attempted = true;
        for chunk in bytes.chunks(65536) {
            control.checkpoint()?;
            self.file
                .write_all(chunk)
                .map_err(kinekino_core::file_publication::io_error)?;
        }
        self.file
            .sync_all()
            .map_err(kinekino_core::file_publication::io_error)?;
        self.file
            .seek(SeekFrom::Start(0))
            .map_err(kinekino_core::file_publication::io_error)?;
        let (length, sha256) =
            content_digest::sha256_with_checkpoint(&mut self.file, self.maximum as u64, || {
                control.checkpoint()
            })
            .map_err(|e| e.into_api_error())?;
        self.check()?;
        self.sealed = true;
        Ok(WorkingFileFacts {
            schema_version: "clip-working-file/1".into(),
            identity: StorageDirectoryIdentity::observe(&self.file)?,
            sha256,
            bytes: length,
            delete_on_close_armed: true,
        })
    }
    fn check(&self) -> Result<(), ApiError> {
        if StorageDirectoryIdentity::observe(&self.file)? != self.identity {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Original clip working file identity changed",
            ));
        }
        Ok(())
    }
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
    /// Parent retains this owner while the existing family supervisor consumes a read-only
    /// independent cursor. Original read/write/delete handle is never inherited or exposed.
    pub(crate) fn with_reader<T>(
        &self,
        read: impl FnOnce(File) -> Result<T, ApiError>,
    ) -> Result<T, ApiError> {
        if !self.sealed {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Working file must be sealed before native read",
            ));
        }
        self.check()?;
        let mut options = OpenOptions::new();
        options.read(true);
        apply_share_mode(&mut options, ShareMode::All);
        apply_open_flags(
            &mut options,
            OpenFlags {
                no_follow: true,
                ..OpenFlags::default()
            },
        );
        let file = options
            .open(&self.path)
            .map_err(kinekino_core::file_publication::io_error)?;
        if StorageDirectoryIdentity::observe(&file)? != self.identity {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Working input reader differs from original owner",
            ));
        }
        read(file)
    }
}
#[cfg(test)]
mod tests;

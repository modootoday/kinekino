//! Windows file-open policy adapter.
use kinekino_core::platform::{FileIdentity, FileIdentity128, OpenFlags, ShareMode};
use std::fs::{File, Metadata, OpenOptions};
use std::io;
use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
use std::os::windows::io::AsRawHandle;
use std::path::Path;

const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
const FILE_FLAG_DELETE_ON_CLOSE: u32 = 0x0400_0000;
const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
const ERROR_SHARING_VIOLATION: i32 = 32;

/// Applies the Win32 share mode for `mode`.
pub fn apply_share_mode(options: &mut OpenOptions, mode: ShareMode) {
    options.share_mode(match mode {
        ShareMode::Exclusive => 0,
        ShareMode::ReadOnly => 1,
        ShareMode::ReadWrite => 3,
        ShareMode::All => 7,
    });
}

/// Applies Win32 `FILE_FLAG_*` equivalents of `flags`.
pub fn apply_open_flags(options: &mut OpenOptions, flags: OpenFlags) {
    let mut bits = 0;
    if flags.no_follow {
        bits |= FILE_FLAG_OPEN_REPARSE_POINT;
    }
    if flags.directory_handle {
        bits |= FILE_FLAG_BACKUP_SEMANTICS;
    }
    if flags.delete_on_close {
        bits |= FILE_FLAG_DELETE_ON_CLOSE;
    }
    options.custom_flags(bits);
    if flags.delete_access {
        // GENERIC_READ | GENERIC_WRITE | DELETE
        options.access_mode(0xc001_0000);
    }
}

/// Opens an existing file for reading while excluding writers, without following reparse points.
pub fn open_reserved_read(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    apply_share_mode(&mut options, ShareMode::ReadOnly);
    apply_open_flags(
        &mut options,
        OpenFlags {
            no_follow: true,
            ..OpenFlags::default()
        },
    );
    options.open(path)
}

/// True when metadata describes a reparse point (symlink, junction, ...).
pub fn is_reparse_point(metadata: &Metadata) -> bool {
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

/// True when `error` means another handle's share mode blocked the open.
pub fn is_sharing_violation(error: &io::Error) -> bool {
    error.raw_os_error() == Some(ERROR_SHARING_VIOLATION)
}

/// Volume serial + file index of an open handle.
pub fn file_identity(file: &File) -> io::Result<FileIdentity> {
    use windows::Win32::{
        Foundation::HANDLE,
        Storage::FileSystem::{BY_HANDLE_FILE_INFORMATION, GetFileInformationByHandle},
    };
    let mut info = BY_HANDLE_FILE_INFORMATION::default();
    // SAFETY: live borrowed handle and fixed-size output buffer.
    unsafe {
        GetFileInformationByHandle(HANDLE(file.as_raw_handle()), &mut info)
            .map_err(io::Error::other)?;
    }
    Ok(FileIdentity {
        volume: u64::from(info.dwVolumeSerialNumber),
        index: (u64::from(info.nFileIndexHigh) << 32) | u64::from(info.nFileIndexLow),
    })
}

/// `FILE_ID_INFO` (volume serial + 128-bit file id) from the open handle.
pub fn file_identity_128(file: &File) -> io::Result<FileIdentity128> {
    use windows::Win32::{
        Foundation::HANDLE,
        Storage::FileSystem::{FILE_ID_INFO, FileIdInfo, GetFileInformationByHandleEx},
    };
    let mut info = FILE_ID_INFO::default();
    // SAFETY: FileIdInfo fills this exact repr(C) object from the live borrowed handle.
    unsafe {
        GetFileInformationByHandleEx(
            HANDLE(file.as_raw_handle()),
            FileIdInfo,
            (&mut info as *mut FILE_ID_INFO).cast(),
            std::mem::size_of::<FILE_ID_INFO>() as u32,
        )
    }
    .map_err(|error| io::Error::from_raw_os_error((error.code().0 as u32 & 0xffff) as i32))?;
    Ok(FileIdentity128 {
        volume: info.VolumeSerialNumber,
        file: info.FileId.Identifier,
    })
}

/// Absolute drive-letter path (`C:\...`); UNC and device paths are not local fixed volumes.
pub fn is_local_fixed_path(path: &Path) -> bool {
    use std::path::{Component, Prefix};
    path.is_absolute()
        && matches!(path.components().next(), Some(Component::Prefix(p)) if matches!(p.kind(), Prefix::Disk(_)))
}

/// `C:\..` as-is, `\\?\C:\..` without its verbatim prefix; UNC, device and relative paths fail.
pub fn normalize_local_path(path: &Path) -> Option<std::path::PathBuf> {
    use std::path::{Component, Prefix};
    match path.components().next() {
        Some(Component::Prefix(prefix)) => match prefix.kind() {
            Prefix::Disk(_) => Some(path.to_path_buf()),
            Prefix::VerbatimDisk(_) => path
                .to_str()
                .and_then(|path| path.strip_prefix("\\\\?\\"))
                .map(std::path::PathBuf::from),
            _ => None,
        },
        _ => None,
    }
    .filter(|path| path.is_absolute())
}

/// No-op: callers open lock files with `ShareMode::Exclusive`, which already excludes handles.
pub fn try_lock_exclusive(_file: &File) -> io::Result<()> {
    Ok(())
}
pub fn try_lock_shared(_file: &File) -> io::Result<()> {
    Ok(())
}

/// Creates a file symlink at `link` pointing to `target` (fixtures for link-rejection checks).
/// Requires the symlink privilege or Developer Mode; failure stays a real error.
pub fn symlink_file(target: &Path, link: &Path) -> io::Result<()> {
    std::os::windows::fs::symlink_file(target, link)
}

#[cfg(test)]
mod normalize_tests {
    use super::normalize_local_path;
    use std::path::{Path, PathBuf};

    #[test]
    fn verbatim_disk_is_normalized_and_unc_or_relative_paths_are_rejected() {
        assert_eq!(
            normalize_local_path(Path::new(r"\\?\C:\workspaces\player-cli.exe")),
            Some(PathBuf::from(r"C:\workspaces\player-cli.exe"))
        );
        for value in [
            r"\\server\share\player.exe",
            r"\\?\UNC\server\share\player.exe",
            "relative.exe",
        ] {
            assert_eq!(normalize_local_path(Path::new(value)), None);
        }
    }
}

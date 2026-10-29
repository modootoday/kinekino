//! Unix file-open policy adapter. POSIX has no mandatory share modes, so share policy is a no-op.
use kinekino_core::platform::{FileIdentity, FileIdentity128, OpenFlags, ShareMode};
use std::fs::{File, Metadata, OpenOptions};
use std::io;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::Path;

/// No-op on Unix: there are no mandatory share modes.
pub fn apply_share_mode(_options: &mut OpenOptions, _mode: ShareMode) {}

/// Maps `no_follow` to `O_NOFOLLOW`; other flags have no POSIX open equivalent.
pub fn apply_open_flags(options: &mut OpenOptions, flags: OpenFlags) {
    if flags.no_follow {
        options.custom_flags(libc::O_NOFOLLOW);
    }
}

/// Opens an existing file for reading without following a final symlink (writers are not excluded).
pub fn open_reserved_read(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    apply_open_flags(
        &mut options,
        OpenFlags {
            no_follow: true,
            ..OpenFlags::default()
        },
    );
    options.open(path)
}

/// True for symlinks (the POSIX reparse-point analogue).
pub fn is_reparse_point(metadata: &Metadata) -> bool {
    metadata.file_type().is_symlink()
}

/// POSIX opens never fail on share modes.
pub fn is_sharing_violation(_error: &io::Error) -> bool {
    false
}

/// Device + inode of an open file.
pub fn file_identity(file: &File) -> io::Result<FileIdentity> {
    let info = file.metadata()?;
    Ok(FileIdentity {
        volume: info.dev(),
        index: info.ino(),
    })
}

/// Device + inode (inode little-endian in the low 8 bytes).
pub fn file_identity_128(file: &File) -> io::Result<FileIdentity128> {
    let info = file.metadata()?;
    let mut id = [0; 16];
    id[..8].copy_from_slice(&info.ino().to_le_bytes());
    Ok(FileIdentity128 {
        volume: info.dev(),
        file: id,
    })
}

/// Any absolute path; POSIX has no volume prefixes.
pub fn is_local_fixed_path(path: &Path) -> bool {
    path.is_absolute()
}

/// Any absolute path is an ordinary local path; POSIX has no verbatim or UNC prefixes.
pub fn normalize_local_path(path: &Path) -> Option<std::path::PathBuf> {
    path.is_absolute().then(|| path.to_path_buf())
}

/// Advisory `flock(LOCK_EX | LOCK_NB)`: POSIX opens cannot exclude other handles.
/// `flock` conflicts follow the open file description, so a concurrent `fork` (e.g. a
/// `pre_exec` spawn elsewhere in this process) briefly keeps a released lock's description
/// alive until the child's `exec` closes it (CLOEXEC). Retry `WouldBlock` for a bounded
/// window so that transient inheritance is not reported as an active writer.
pub fn try_lock_exclusive(file: &File) -> io::Result<()> {
    const WINDOW: std::time::Duration = std::time::Duration::from_millis(250);
    let start = std::time::Instant::now();
    loop {
        match file.try_lock() {
            Ok(()) => return Ok(()),
            Err(std::fs::TryLockError::Error(error)) => return Err(error),
            Err(std::fs::TryLockError::WouldBlock) if start.elapsed() < WINDOW => {
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            Err(std::fs::TryLockError::WouldBlock) => {
                return Err(io::Error::from(io::ErrorKind::WouldBlock));
            }
        }
    }
}

pub fn try_lock_shared(file: &File) -> io::Result<()> {
    match file.try_lock_shared() {
        Ok(()) => Ok(()),
        Err(std::fs::TryLockError::Error(error)) => Err(error),
        Err(std::fs::TryLockError::WouldBlock) => Err(io::Error::from(io::ErrorKind::WouldBlock)),
    }
}

/// Creates a file symlink at `link` pointing to `target` (fixtures for link-rejection checks).
pub fn symlink_file(target: &Path, link: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_is_stable_and_reserved_read_rejects_symlink() {
        let dir = std::env::temp_dir().join(format!("kk-platform-unix-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir(&dir).unwrap();
        let file = dir.join("a");
        std::fs::write(&file, b"x").unwrap();
        let a = file_identity(&File::open(&file).unwrap()).unwrap();
        let b = file_identity(&open_reserved_read(&file).unwrap()).unwrap();
        assert_eq!(a, b);
        let link = dir.join("l");
        std::os::unix::fs::symlink(&file, &link).unwrap();
        assert!(open_reserved_read(&link).is_err());
        assert!(is_reparse_point(&std::fs::symlink_metadata(&link).unwrap()));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn exclusive_lock_conflicts_and_identity_128_is_stable() {
        let path =
            std::env::temp_dir().join(format!("kk-platform-unix-lock-{}", std::process::id()));
        std::fs::write(&path, b"x").unwrap();
        let first = File::open(&path).unwrap();
        let second = File::open(&path).unwrap();
        try_lock_exclusive(&first).unwrap();
        assert_eq!(
            try_lock_exclusive(&second).unwrap_err().kind(),
            io::ErrorKind::WouldBlock
        );
        assert_eq!(
            file_identity_128(&first).unwrap(),
            file_identity_128(&second).unwrap()
        );
        assert!(is_local_fixed_path(&path) && !is_local_fixed_path(Path::new("rel")));
        drop((first, second));
        std::fs::remove_file(&path).unwrap();
    }
}

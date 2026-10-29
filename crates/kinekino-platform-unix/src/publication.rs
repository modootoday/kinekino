//! Unix complete-file publication: `link(2)` for no-replace rename, `flock` for transient locks.
use std::fs::{File, OpenOptions};
use std::io;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

/// New staging file opened without following a final symlink. POSIX rename needs no handle right.
pub fn create_stage(path: &Path, readable: bool) -> io::Result<File> {
    OpenOptions::new()
        .read(readable)
        .write(true)
        .create_new(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
}

/// `link` fails with `EEXIST` rather than replacing; the staged name is then removed.
pub fn rename_no_replace(_file: &File, staged: &Path, target: &Path) -> io::Result<()> {
    std::fs::hard_link(staged, target)?;
    std::fs::remove_file(staged)
}

/// A persistent lock file guarded by advisory `flock`; contention maps to `AlreadyExists`.
pub fn create_transient_lock(path: &Path) -> io::Result<File> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)?;
    match file.try_lock() {
        Ok(()) => Ok(file),
        Err(std::fs::TryLockError::WouldBlock) => {
            Err(io::Error::from(io::ErrorKind::AlreadyExists))
        }
        Err(std::fs::TryLockError::Error(error)) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rename_refuses_existing_target_and_lock_is_exclusive() {
        let dir = std::env::temp_dir().join(format!("kk-pub-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let staged = dir.join("stage");
        let target = dir.join("target");
        let _ = std::fs::remove_file(&target);
        let file = create_stage(&staged, false).unwrap();
        rename_no_replace(&file, &staged, &target).unwrap();
        assert!(!staged.exists() && target.exists());
        let again = create_stage(&staged, true).unwrap();
        let error = rename_no_replace(&again, &staged, &target).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
        let lock = dir.join(".lock");
        let held = create_transient_lock(&lock).unwrap();
        assert_eq!(
            create_transient_lock(&lock).unwrap_err().kind(),
            io::ErrorKind::AlreadyExists
        );
        drop(held);
        create_transient_lock(&lock).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
    }
}

//! Pinned tool execution on POSIX (ADR 20261116151137).
//!
//! A pinned tool was hashed through `verified`. POSIX sharing is advisory, so the bytes at its
//! path can still change after that check. Linux runs the verified inode itself through
//! `/proc/self/fd/<n>`. Every POSIX host also requires that only the user or root can change the
//! file or any directory above it, and that the path still names the verified inode.
use std::fs::{self, File, Metadata};
use std::io;
use std::os::unix::fs::{FileExt, MetadataExt};
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};

const GROUP_OR_OTHER_WRITE: u32 = 0o022;
const STICKY: u32 = 0o1000;

/// True where the kernel can run an open file handle, so a renamed or rewritten path has no
/// effect on what runs.
pub fn has_handle_execution() -> bool {
    cfg!(target_os = "linux")
}

/// Program path to spawn for the pinned tool at `path`, whose open handle `verified` was hashed.
///
/// On Linux the result is `/proc/self/fd/<n>` and is valid only while `verified` stays open.
/// Elsewhere it is the canonical path. Errors:
/// - `InvalidInput`: `path` is relative, or the file starts with `#!` (a script's interpreter
///   would reopen the path instead of the verified handle);
/// - `PermissionDenied`: the file or a directory above it can be changed by another user;
/// - `InvalidData`: `path` no longer names the verified file.
pub fn pinned_program(verified: &File, path: &Path) -> io::Result<PathBuf> {
    if !path.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "pinned tool path must be absolute",
        ));
    }
    let canonical = fs::canonicalize(path)?;
    // SAFETY: geteuid has no preconditions and cannot fail.
    let uid = unsafe { libc::geteuid() };
    require_owner_only(&canonical, uid)?;
    let held = verified.metadata()?;
    let named = fs::metadata(&canonical)?;
    if !held.is_file() || held.dev() != named.dev() || held.ino() != named.ino() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "pinned tool path no longer names the verified file",
        ));
    }
    let mut magic = [0u8; 2];
    if verified.read_at(&mut magic, 0)? == 2 && &magic == b"#!" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "pinned tool must be a native executable, not a script",
        ));
    }
    if has_handle_execution() {
        Ok(PathBuf::from(format!(
            "/proc/self/fd/{}",
            verified.as_raw_fd()
        )))
    } else {
        Ok(canonical)
    }
}

fn require_owner_only(file: &Path, uid: u32) -> io::Result<()> {
    let denied = |what: &str| {
        io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "pinned tool {what} must be owned by the user or root and not writable by group or others"
            ),
        )
    };
    let meta = fs::metadata(file)?;
    if !trusted_owner(&meta, uid) || writable_by_others(&meta) {
        return Err(denied("file"));
    }
    let mut ancestors = file.ancestors().skip(1);
    if let Some(parent) = ancestors.next() {
        let meta = fs::metadata(parent)?;
        if !trusted_owner(&meta, uid) || writable_by_others(&meta) {
            return Err(denied("parent directory"));
        }
    }
    for dir in ancestors {
        let meta = fs::metadata(dir)?;
        // A sticky directory (such as /tmp) lets others add entries but not replace ours.
        if !trusted_owner(&meta, uid) || (writable_by_others(&meta) && meta.mode() & STICKY == 0) {
            return Err(denied("ancestor directory"));
        }
    }
    Ok(())
}

fn trusted_owner(meta: &Metadata, uid: u32) -> bool {
    meta.uid() == uid || meta.uid() == 0
}

fn writable_by_others(meta: &Metadata) -> bool {
    meta.mode() & GROUP_OR_OTHER_WRITE != 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::process::Command;

    struct Scratch(PathBuf);
    impl Scratch {
        fn new(tag: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("kk-pinned-{tag}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir(&dir).unwrap();
            fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
            Self(fs::canonicalize(dir).unwrap())
        }
        fn tool(&self, name: &str, from: &str) -> PathBuf {
            let path = self.0.join(name);
            fs::copy(from, &path).unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
            path
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn native(name: &str) -> String {
        ["/bin", "/usr/bin"]
            .iter()
            .map(|dir| format!("{dir}/{name}"))
            .find(|path| Path::new(path).is_file())
            .expect("a native true/false executable")
    }

    #[test]
    fn owner_only_tool_runs_the_verified_file() {
        let scratch = Scratch::new("ok");
        let path = scratch.tool("tool", &native("true"));
        let verified = File::open(&path).unwrap();
        let program = pinned_program(&verified, &path).unwrap();
        if has_handle_execution() {
            assert_eq!(
                program,
                PathBuf::from(format!("/proc/self/fd/{}", verified.as_raw_fd()))
            );
        } else {
            assert_eq!(program, path);
        }
        assert!(Command::new(&program).status().unwrap().success());
    }

    #[test]
    fn handle_execution_ignores_a_replaced_path() {
        if !has_handle_execution() {
            return;
        }
        let scratch = Scratch::new("swap");
        let path = scratch.tool("tool", &native("true"));
        let verified = File::open(&path).unwrap();
        let program = pinned_program(&verified, &path).unwrap();
        let other = scratch.tool("other", &native("false"));
        fs::rename(&other, &path).unwrap();
        // The verified inode still runs, not the file now at the path.
        assert!(Command::new(&program).status().unwrap().success());
        // A fresh request sees that the path names another file.
        let error = pinned_program(&verified, &path).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn writable_file_or_parent_is_refused() {
        let scratch = Scratch::new("mode");
        let path = scratch.tool("tool", &native("true"));
        let verified = File::open(&path).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o775)).unwrap();
        let error = pinned_program(&verified, &path).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        assert!(error.to_string().contains("pinned tool file"));
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        // A sticky world-writable parent is still refused: only ancestors above it may be sticky.
        fs::set_permissions(&scratch.0, fs::Permissions::from_mode(0o1777)).unwrap();
        let error = pinned_program(&verified, &path).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        assert!(error.to_string().contains("parent directory"));
    }

    #[test]
    fn writable_ancestor_is_refused_unless_sticky() {
        let scratch = Scratch::new("ancestor");
        let inner = scratch.0.join("bin");
        fs::create_dir(&inner).unwrap();
        fs::set_permissions(&inner, fs::Permissions::from_mode(0o755)).unwrap();
        let path = inner.join("tool");
        fs::copy(native("true"), &path).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        let verified = File::open(&path).unwrap();
        fs::set_permissions(&scratch.0, fs::Permissions::from_mode(0o777)).unwrap();
        let error = pinned_program(&verified, &path).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        assert!(error.to_string().contains("ancestor directory"));
        fs::set_permissions(&scratch.0, fs::Permissions::from_mode(0o1777)).unwrap();
        assert!(pinned_program(&verified, &path).is_ok());
        fs::set_permissions(&scratch.0, fs::Permissions::from_mode(0o700)).unwrap();
    }

    #[test]
    fn scripts_and_relative_paths_are_refused() {
        let scratch = Scratch::new("script");
        let path = scratch.0.join("tool.sh");
        fs::write(&path, b"#!/bin/sh\nexit 0\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        let verified = File::open(&path).unwrap();
        let error = pinned_program(&verified, &path).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert!(error.to_string().contains("not a script"));
        let error = pinned_program(&verified, Path::new("tool.sh")).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    }
}

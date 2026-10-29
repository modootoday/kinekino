use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);

pub fn publish(path: &str, bytes: &[u8]) -> Result<(), ApiError> {
    publish_before(path, bytes, None)
}

pub fn publish_before(
    path: &str,
    bytes: &[u8],
    deadline: Option<std::time::Instant>,
) -> Result<(), ApiError> {
    publish_guarded(path, bytes, || {
        if deadline.is_some_and(|deadline| std::time::Instant::now() >= deadline) {
            return Err(io_error(std::io::ErrorKind::TimedOut.into()));
        }
        Ok(())
    })
}

/// Check the original lease/cancellation after write+sync, immediately before
/// the create-only link. An error here proves this call did not publish.
pub fn publish_guarded(
    path: &str,
    bytes: &[u8],
    before_publish: impl FnOnce() -> Result<(), ApiError>,
) -> Result<(), ApiError> {
    let output = Path::new(path);
    let name = output
        .file_name()
        .ok_or_else(|| ApiError::new(ErrorCode::InvalidArgument, "Output must name a new file"))?;
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .canonicalize()
        .map_err(io_error)?;
    let output = parent.join(name);
    let temp = parent.join(format!(
        ".export-{}-{}.tmp",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(io_error)?;
    let result = file
        .write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(io_error);
    drop(file);
    let result = result.and_then(|()| {
        before_publish()?;
        fs::hard_link(&temp, &output).map_err(io_error)
    });
    let _ = fs::remove_file(&temp);
    result
}
pub fn io_error(error: std::io::Error) -> ApiError {
    match error.kind() {
        std::io::ErrorKind::TimedOut => ApiError::new(
            ErrorCode::Timeout,
            "Export deadline expired before output publication",
        ),
        std::io::ErrorKind::AlreadyExists => ApiError::new(
            ErrorCode::RevisionConflict,
            "Output already exists; choose a new path",
        ),
        std::io::ErrorKind::NotFound => ApiError::new(
            ErrorCode::NotFound,
            "Input file or output directory not found",
        ),
        std::io::ErrorKind::PermissionDenied => {
            ApiError::new(ErrorCode::PermissionDenied, "File access denied")
        }
        _ => ApiError::new(ErrorCode::EngineFailed, "File operation failed"),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn root() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "publication-guard-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        path
    }
    #[test]
    fn guard_observes_complete_temp_bytes_and_cancellation_leaves_no_publication() {
        let root = root();
        let target = root.join("output.json");
        let error = publish_guarded(target.to_str().unwrap(), b"complete bytes", || {
            assert!(!target.exists());
            let temp = fs::read_dir(&root).unwrap().next().unwrap().unwrap().path();
            assert_eq!(fs::read(temp).unwrap(), b"complete bytes");
            Err(ApiError::new(ErrorCode::Cancelled, "cancelled"))
        })
        .unwrap_err();
        assert_eq!(error.code, ErrorCode::Cancelled);
        assert!(!target.exists());
        assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
        publish(target.to_str().unwrap(), b"retry").unwrap();
        assert_eq!(fs::read(&target).unwrap(), b"retry");
        let expired = root.join("expired.json");
        assert_eq!(
            publish_before(
                expired.to_str().unwrap(),
                b"late",
                Some(std::time::Instant::now())
            )
            .unwrap_err()
            .code,
            ErrorCode::Timeout
        );
        assert!(!expired.exists());
        assert_eq!(
            publish(target.to_str().unwrap(), b"replace")
                .unwrap_err()
                .code,
            ErrorCode::RevisionConflict
        );
        assert_eq!(fs::read(&target).unwrap(), b"retry");
    }
}

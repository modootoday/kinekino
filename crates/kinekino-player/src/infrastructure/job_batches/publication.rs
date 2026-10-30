//! Sync complete bytes, then create-only rename using the original owned file handle.
use super::*;
use std::{
    io::Write,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
pub(super) fn publish(directory: &Path, record: &records::Record) -> Result<(), ApiError> {
    if record.revision.0 > records::MAX_EVENTS as u64 {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Batch journal event count limit exceeded",
        ));
    }
    let bytes = serde_json::to_vec(record).map_err(|_| corrupt())?;
    if bytes.len() > records::MAX_EVENT_BYTES {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Batch journal event byte limit exceeded",
        ));
    }
    let sha = hash(&bytes);
    let target = directory.join(format!("r-{:020}-{sha}.json", record.revision.0));
    let stage = directory.join(format!(
        ".pending-{}-{}.json",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file =
        crate::infrastructure::os_policy::create_publication_stage(&stage, false).map_err(io)?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(io)?;
    rename_new(&file, &stage, &target)
}
fn rename_new(file: &File, stage: &Path, target: &Path) -> Result<(), ApiError> {
    crate::infrastructure::os_policy::rename_no_replace(file, stage, target).map_err(|error| {
        if error.kind() == std::io::ErrorKind::InvalidFilename {
            ApiError::new(ErrorCode::InvalidArgument, "Batch event path is too long")
        } else {
            io(error)
        }
    })
}

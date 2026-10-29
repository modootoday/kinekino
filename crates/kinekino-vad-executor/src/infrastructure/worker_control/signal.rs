//! Publish complete marker bytes without overwriting an earlier signal. The
//! original transport retains the read guard; marker publication is not ACK.
use super::*;
pub(in crate::infrastructure) fn publish_cancellation(root: &Path) -> Result<File, ApiError> {
    let pending = root.join("cancel.signal.pending");
    let destination = root.join("cancel.signal");
    let mut file = read_shared(OpenOptions::new().create_new(true).write(true))
        .open(&pending)
        .map_err(io)?;
    file.write_all(b"cancel\n").map_err(io)?;
    file.sync_all().map_err(io)?;
    drop(file);
    // The destination only appears after the exact source was synced. Unlike
    // rename-with-replace, hard_link refuses an existing destination. Unsupported
    // filesystems fail; no in-place partial marker fallback is permitted.
    std::fs::hard_link(&pending, &destination).map_err(io)?;
    let mut guard = no_follow(read_shared(OpenOptions::new().read(true)))
        .open(&destination)
        .map_err(io)?;
    let metadata = guard.metadata().map_err(io)?;
    if !metadata.is_file()
        || kinekino_core::platform::os().is_reparse_point(&metadata)
        || metadata.len() != 7
    {
        return Err(ApiError::new(
            ErrorCode::NeedsInput,
            "Original cancellation marker changed",
        ));
    }
    let mut bytes = Vec::new();
    Read::by_ref(&mut guard)
        .take(8)
        .read_to_end(&mut bytes)
        .map_err(io)?;
    if bytes != b"cancel\n" {
        return Err(ApiError::new(
            ErrorCode::NeedsInput,
            "Original cancellation marker bytes changed",
        ));
    }
    Ok(guard)
}

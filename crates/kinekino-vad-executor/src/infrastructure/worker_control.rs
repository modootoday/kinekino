//! Parent writes a separate immutable control sidecar under its original root.
#[path = "worker_control/clock.rs"]
mod clock;
#[path = "worker_control/signal.rs"]
mod signal;
use crate::application::{MAX_WORKER_CONTROL_BYTES, RunControl, prepare_worker_cutoff};
use kinekino_core::platform::{OpenFlags, ShareMode};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_process::domain::deadline::Deadline;
use sha2::{Digest, Sha256};
pub(super) use signal::publish_cancellation;
use std::{
    ffi::OsString,
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub(super) struct PreparedWorkerControl {
    path: PathBuf,
    sha256: String,
    bytes: usize,
    _guard: File,
}
impl PreparedWorkerControl {
    pub(super) fn arguments(&self) -> Vec<OsString> {
        vec![
            "--worker-control-v1".into(),
            self.path.as_os_str().to_owned(),
            "--worker-control-sha256".into(),
            self.sha256.clone().into(),
            "--worker-control-bytes".into(),
            self.bytes.to_string().into(),
        ]
    }
}
/// Later handles may only read (Windows `FILE_SHARE_READ`; advisory elsewhere).
fn read_shared(options: &mut OpenOptions) -> &mut OpenOptions {
    kinekino_core::platform::os().apply_share_mode(options, ShareMode::ReadOnly);
    options
}
/// Opens a final link itself so a replaced link is detected, never followed.
fn no_follow(options: &mut OpenOptions) -> &mut OpenOptions {
    kinekino_core::platform::os().apply_open_flags(
        options,
        OpenFlags {
            no_follow: true,
            ..OpenFlags::default()
        },
    );
    options
}
fn io(error: std::io::Error) -> ApiError {
    ApiError::new(
        ErrorCode::EngineFailed,
        format!("Worker control storage: {error}"),
    )
}
pub(super) fn prepare(
    root: &Path,
    original: Deadline,
    caller: &dyn RunControl,
    request: &[u8],
) -> Result<PreparedWorkerControl, ApiError> {
    let request_sha256 = format!("{:x}", Sha256::digest(request));
    let cutoff = prepare_worker_cutoff(&clock::ParentQpcClock, original, caller, request_sha256)?;
    let bytes = serde_json::to_vec(&cutoff)
        .map_err(|error| ApiError::new(ErrorCode::Internal, error.to_string()))?;
    if bytes.len() > MAX_WORKER_CONTROL_BYTES {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Worker control sidecar exceeds bound",
        ));
    }
    let path = root.join("worker-control.v1.json");
    let mut file = read_shared(OpenOptions::new().write(true).create_new(true))
        .open(&path)
        .map_err(io)?;
    file.write_all(&bytes).map_err(io)?;
    file.sync_all().map_err(io)?;
    drop(file);
    let mut guard = no_follow(read_shared(OpenOptions::new().read(true)))
        .open(&path)
        .map_err(io)?;
    let metadata = guard.metadata().map_err(io)?;
    if !metadata.is_file()
        || kinekino_core::platform::os().is_reparse_point(&metadata)
        || metadata.len() != bytes.len() as u64
    {
        return Err(ApiError::new(
            ErrorCode::NeedsInput,
            "Original worker control sidecar changed",
        ));
    }
    let mut actual = Vec::new();
    Read::by_ref(&mut guard)
        .take(MAX_WORKER_CONTROL_BYTES as u64 + 1)
        .read_to_end(&mut actual)
        .map_err(io)?;
    if actual != bytes {
        return Err(ApiError::new(
            ErrorCode::NeedsInput,
            "Original worker control bytes changed",
        ));
    }
    Ok(PreparedWorkerControl {
        path,
        sha256: format!("{:x}", Sha256::digest(&bytes)),
        bytes: bytes.len(),
        _guard: guard,
    })
}

#[cfg(test)]
#[path = "worker_control/tests.rs"]
mod tests;

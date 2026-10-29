//! Concrete guarded child observations, never release or native admission.
mod clock;
mod signal;
pub use clock::VerifiedParentClock;
pub use signal::GuardedCancellation;

use crate::{entry::WorkerLaunch, staged_files::immutable_storage_paths};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_vad::domain::ArtifactPin;
use kinekino_vad_executor::{
    application::{MAX_WORKER_CONTROL_BYTES, decode_worker_control},
    domain::WorkerOperationCutoff,
};
use sha2::{Digest, Sha256};
use std::{fs::File, io::Read};

pub(crate) fn io(error: std::io::Error) -> ApiError {
    ApiError::new(
        ErrorCode::EngineFailed,
        format!("Worker control IO: {error}"),
    )
}
pub(crate) fn changed(message: &str) -> ApiError {
    ApiError::new(ErrorCode::SourceExpired, message)
}

pub struct ChildControlFiles {
    _directories: Vec<File>,
    _file: File,
    cutoff: WorkerOperationCutoff,
}
impl ChildControlFiles {
    pub fn open(launch: &WorkerLaunch, original_request: &ArtifactPin) -> Result<Self, ApiError> {
        let root = launch
            .request
            .parent()
            .ok_or_else(|| changed("Original worker root absent"))?;
        if launch.control_path != root.join("worker-control.v1.json") {
            return Err(changed("Control path differs from original root"));
        }
        let (_, directories) = immutable_storage_paths::root(root)?;
        if launch.control_pin.bytes() > MAX_WORKER_CONTROL_BYTES as u64 {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Worker control exceeds bound",
            ));
        }
        let mut file = immutable_storage_paths::file(&launch.control_path)?;
        if file.metadata().map_err(io)?.len() != launch.control_pin.bytes() {
            return Err(changed("Control declaration differs"));
        }
        let mut bytes = Vec::new();
        Read::by_ref(&mut file)
            .take(MAX_WORKER_CONTROL_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(io)?;
        if bytes.len() as u64 != launch.control_pin.bytes()
            || format!("{:x}", Sha256::digest(&bytes)) != launch.control_pin.sha256()
        {
            return Err(changed("Original guarded control pin differs"));
        }
        let cutoff = decode_worker_control(&bytes)?;
        if cutoff.request_sha256 != original_request.sha256() {
            return Err(changed("Control belongs to another original request"));
        }
        Ok(Self {
            _directories: directories,
            _file: file,
            cutoff,
        })
    }
    pub fn cutoff(&self) -> &WorkerOperationCutoff {
        &self.cutoff
    }
}

/// Same production/explicit test-fixture IO path. The caller's engine/admission
/// remains external; verified control does not authorize a profile or native DLL.
pub fn serve_controlled_request(
    reader: impl Read,
    files: &crate::staged_files::WorkerRequestFiles,
    launch: &WorkerLaunch,
    writer: impl std::io::Write,
    execute: impl FnOnce(
        &kinekino_vad::protocol::application::ValidatedWorkerRequest,
        &dyn kinekino_vad::application::AnalysisControl,
    ) -> Result<Vec<u8>, ApiError>,
) -> Result<(), ApiError> {
    use kinekino_vad_executor::application::{WorkerControlView, project_worker_deadline};
    crate::entry::serve_request(
        reader,
        files.declared_bytes(),
        files.descriptor_raw(),
        files.original_request(),
        writer,
        |request| {
            let original = ChildControlFiles::open(launch, request.expected().request())?;
            let clock = VerifiedParentClock::connect(&original.cutoff().clock)?;
            let root = launch
                .request
                .parent()
                .ok_or_else(|| changed("Worker root absent"))?;
            let cancellation = GuardedCancellation::open(root, &launch.cancel_signal)?;
            let view = WorkerControlView::new(
                original.cutoff(),
                request.expected().request().sha256(),
                &clock,
                &cancellation,
            )?;
            let projected = project_worker_deadline(&clock, original.cutoff())?;
            let control = crate::control_bridge::WorkerAnalysisControl::new(&view, projected)?;
            execute(request, &control)
        },
    )
}

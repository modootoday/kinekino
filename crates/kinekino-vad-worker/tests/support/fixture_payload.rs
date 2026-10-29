//! Exact fixed test data staged through the existing parent payload port.
use super::fixture_data::{FixtureData, io, ordinary_paths};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_vad_executor::{
    application::{RunControl, SnapshotLease},
    infrastructure::{StagedWorkerInput, WorkerPayloadPreparation},
};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
    sync::Arc,
    time::Instant,
};
pub struct FixturePayload {
    data: Arc<FixtureData>,
}
impl FixturePayload {
    pub fn new(data: Arc<FixtureData>) -> Self {
        Self { data }
    }
}
struct FixtureLease {
    _data: Arc<FixtureData>,
    _directories: Vec<File>,
    _files: Vec<File>,
}
impl SnapshotLease for FixtureLease {}
fn check(control: &dyn RunControl) -> Result<(), ApiError> {
    if control.cancellation_requested() {
        return Err(ApiError::new(
            ErrorCode::Cancelled,
            "Original fixture staging cancelled",
        ));
    }
    if Instant::now() >= control.deadline() {
        return Err(ApiError::new(
            ErrorCode::Timeout,
            "Original fixture staging cutoff",
        ));
    }
    Ok(())
}
impl WorkerPayloadPreparation for FixturePayload {
    fn stage(
        &self,
        request: &[u8],
        root: &Path,
        control: &dyn RunControl,
    ) -> Result<StagedWorkerInput, ApiError> {
        check(control)?;
        let decoded = kinekino_vad::protocol::application::decode_worker_request(
            request,
            request.len() as u64,
            &self.data.descriptor,
        )?;
        if decoded.expected().profile() != &self.data.profile {
            return Err(ApiError::new(
                ErrorCode::PermissionDenied,
                "Fixture profile changed",
            ));
        }
        let (_, directories) = ordinary_paths::root(root)?;
        let mut input = ordinary_paths::file(&root.join("stdin.raw"))?;
        let mut actual = Vec::new();
        Read::by_ref(&mut input)
            .take(kinekino_vad::protocol::application::MAX_DESCRIPTOR_BYTES + 1)
            .read_to_end(&mut actual)
            .map_err(io)?;
        if actual != request {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Fixture original stdin changed",
            ));
        }
        let mut files = vec![input];
        for (name, bytes) in &self.data.staged {
            check(control)?;
            let path = root.join(name);
            let mut output = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .map_err(io)?;
            output.write_all(bytes).map_err(io)?;
            output.sync_all().map_err(io)?;
            drop(output);
            let mut guard = ordinary_paths::file(&path)?;
            let mut actual = Vec::new();
            Read::by_ref(&mut guard)
                .take(bytes.len() as u64 + 1)
                .read_to_end(&mut actual)
                .map_err(io)?;
            if actual.as_slice() != bytes.as_slice() {
                return Err(ApiError::new(
                    ErrorCode::SourceExpired,
                    "Fixture staged bytes changed",
                ));
            }
            files.push(guard);
            check(control)?;
        }
        Ok(StagedWorkerInput::new(
            self.data.manifest_pin.clone(),
            Arc::new(FixtureLease {
                _data: self.data.clone(),
                _directories: directories,
                _files: files,
            }),
        ))
    }
}

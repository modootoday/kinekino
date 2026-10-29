//! Keeps actual prepared snapshot leases across the original session and output.
use crate::application::{
    AttemptResourcesFactory, OutputPublication, PreparedResources, PreparedVadAdmission,
    RunControl, WorkerRead, WorkerTransport,
};
use kinekino_jobs::{
    application::{JobAttemptIdentityV1, owned_attempt::OwnerObservation},
    domain::{PreparedJobInput, VerifiedJobOutput},
};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_vad::protocol::domain::WorkerOutcomeCandidate;
use std::{sync::Arc, time::Instant};

struct AdmittedResources {
    inner: Arc<dyn AttemptResourcesFactory>,
    admission: Arc<PreparedVadAdmission>,
    input_sha: String,
    source_sha: String,
}
/// Binds one original prepared input as data. It creates no scope, identity,
/// settlement or registry entry; actual scope checks remain in the SDK/executor.
pub fn admitted_attempt_resources(
    inner: Arc<dyn AttemptResourcesFactory>,
    input: &PreparedJobInput,
    admission: Arc<PreparedVadAdmission>,
) -> Result<Arc<dyn AttemptResourcesFactory>, ApiError> {
    validate_source(input, &admission)?;
    Ok(Arc::new(AdmittedResources {
        inner,
        admission,
        input_sha: input
            .sha256()
            .map_err(|m| ApiError::new(ErrorCode::InvalidArgument, m))?,
        source_sha: input.source.sha256.clone(),
    }))
}
fn validate_source(
    input: &PreparedJobInput,
    admission: &PreparedVadAdmission,
) -> Result<(), ApiError> {
    let expected = admission.descriptor().source();
    if input.source.sha256 != expected.sha256 || input.source.bytes != expected.bytes {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "VAD admission differs from original prepared source",
        ));
    };
    Ok(())
}
impl AttemptResourcesFactory for AdmittedResources {
    fn prepare(
        &self,
        identity: &JobAttemptIdentityV1,
        end: Instant,
    ) -> Result<PreparedResources, ApiError> {
        if identity.input_sha256() != self.input_sha
            || identity.source_blob_sha256() != self.source_sha
        {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "VAD admission does not match original attempt input",
            ));
        };
        if Instant::now() >= end {
            return Err(ApiError::new(
                ErrorCode::Timeout,
                "Original VAD admission cutoff expired",
            ));
        };
        Ok(attach(
            self.inner.prepare(identity, end)?,
            self.admission.clone(),
        ))
    }
}
fn attach(resources: PreparedResources, admission: Arc<PreparedVadAdmission>) -> PreparedResources {
    PreparedResources {
        encoder: resources.encoder,
        transport: Box::new(AdmittedTransport {
            inner: resources.transport,
            _admission: admission.clone(),
        }),
        publication: Arc::new(AdmittedPublication {
            target: admission.destination().publication.clone(),
            _attempt: resources.publication,
            _admission: admission,
        }),
    }
}
struct AdmittedTransport {
    inner: Box<dyn WorkerTransport>,
    _admission: Arc<PreparedVadAdmission>,
}
impl WorkerTransport for AdmittedTransport {
    fn cancellation_signal_sent(&self) -> bool {
        self.inner.cancellation_signal_sent()
    }
    fn launch_read(
        &mut self,
        request: &[u8],
        control: &dyn RunControl,
    ) -> Result<WorkerRead, ApiError> {
        self.inner.launch_read(request, control)
    }
    fn poll_close(&mut self, end: Instant) -> Result<OwnerObservation, ApiError> {
        self.inner.poll_close(end)
    }
}
struct AdmittedPublication {
    target: Arc<dyn OutputPublication>,
    _attempt: Arc<dyn OutputPublication>,
    _admission: Arc<PreparedVadAdmission>,
}
impl OutputPublication for AdmittedPublication {
    fn publish(&self, output: &WorkerOutcomeCandidate, end: Instant) -> Result<(), ApiError> {
        self.target.publish(output, end)
    }
    fn verify(
        &self,
        output: &WorkerOutcomeCandidate,
        end: Instant,
    ) -> Result<VerifiedJobOutput, ApiError> {
        self.target.verify(output, end)
    }
}
#[cfg(test)]
#[path = "admission_tests.rs"]
mod tests;

use super::{
    AttemptResourcesFactory, OutputPublication, PreparedResources, RequestEncoder,
    WorkerTransportFactory,
};
use crate::infrastructure::PreparedVadSession;
use kinekino_jobs::{
    application::{
        SourceBlobStore,
        owned_attempt::{
            AttemptScope, OwnedAttemptExecutor, OwnedAttemptSession, OwnedOutputCandidate,
        },
    },
    domain::PreparedJobInput,
};
use kinekino_media_contracts::{
    api_values::DecimalU64,
    errors::{ApiError, ErrorCode},
};
use kinekino_vad::domain::ArtifactPin;
use kinekino_vad::protocol::{
    application::{MAX_DESCRIPTOR_BYTES, decode_descriptor},
    domain::ExpectedWorkerContext,
};
use sha2::{Digest, Sha256};
use std::{sync::Arc, time::Instant};

pub struct VadExecutor {
    resources: Arc<dyn AttemptResourcesFactory>,
}
struct SingleAttemptResources {
    encoder: Arc<dyn RequestEncoder>,
    transport: Arc<dyn WorkerTransportFactory>,
    publication: Arc<dyn OutputPublication>,
}
impl AttemptResourcesFactory for SingleAttemptResources {
    fn prepare(
        &self,
        _identity: &kinekino_jobs::application::JobAttemptIdentityV1,
        _original_end: Instant,
    ) -> Result<PreparedResources, ApiError> {
        Ok(PreparedResources {
            encoder: self.encoder.clone(),
            transport: self.transport.prepare()?,
            publication: self.publication.clone(),
        })
    }
}
impl VadExecutor {
    /// Fixed single-attempt composition. Reusable hosts must use for_attempts
    /// so later original scopes receive fresh cutoffs, leases and destinations.
    pub fn new(
        encoder: Arc<dyn RequestEncoder>,
        transport: Arc<dyn WorkerTransportFactory>,
        publication: Arc<dyn OutputPublication>,
    ) -> Self {
        Self::for_attempts(Arc::new(SingleAttemptResources {
            encoder,
            transport,
            publication,
        }))
    }
    pub fn for_attempts(resources: Arc<dyn AttemptResourcesFactory>) -> Self {
        Self { resources }
    }
}
impl OwnedAttemptExecutor for VadExecutor {
    type Candidate = Box<dyn OwnedOutputCandidate>;
    fn prepare_owned(
        &self,
        input: &PreparedJobInput,
        blobs: &dyn SourceBlobStore,
        scope: &AttemptScope<'_>,
    ) -> Result<Box<dyn OwnedAttemptSession<Candidate = Self::Candidate>>, ApiError> {
        if scope.cancellation_requested() {
            return Err(ApiError::new(
                ErrorCode::Cancelled,
                "VAD preparation stopped; no cleanup ACK",
            ));
        }
        if Instant::now() >= scope.deadline() {
            return Err(ApiError::new(
                ErrorCode::Timeout,
                "Original VAD preparation deadline expired",
            ));
        }
        if input
            .sha256()
            .map_err(|m| ApiError::new(ErrorCode::InvalidArgument, m))?
            != scope.identity().input_sha256()
            || input.source.sha256 != scope.identity().source_blob_sha256()
            || input.source.bytes.0 > MAX_DESCRIPTOR_BYTES
        {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Original VAD input binding mismatch",
            ));
        }
        let pin = ArtifactPin::new(
            "vad-descriptor".into(),
            input.source.sha256.clone(),
            input.source.bytes,
        )?;
        let raw = blobs.read_utf8(&input.source)?;
        let descriptor = decode_descriptor(raw.as_bytes(), &pin)?;
        let resources = self.resources.prepare(scope.identity(), scope.deadline())?;
        let request = resources.encoder.encode(scope.identity(), &descriptor)?;
        if request.is_empty() || request.len() as u64 > MAX_DESCRIPTOR_BYTES {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "VAD request exceeds bounded internal envelope",
            ));
        }
        let request_pin = ArtifactPin::new(
            "vad-request".into(),
            format!("{:x}", Sha256::digest(&request)),
            DecimalU64(request.len() as u64),
        )?;
        let context = ExpectedWorkerContext::new(request_pin, descriptor);
        // Inert transport is moved into the session before core can call run.
        Ok(Box::new(PreparedVadSession::new(
            resources.transport,
            request,
            context,
            scope.identity().clone(),
            scope.deadline(),
            resources.publication,
        )))
    }
}

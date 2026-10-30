use super::{
    DOWNLOAD_OPERATION, DownloadPreparation, DownloadRequest, owned_session::DownloadOwnedExecutor,
};
use crate::application::{
    api::{ApiError, ErrorCode},
    jobs::{
        ExecutionControl, JobExecutor, SourceBlobStore, VerifiedJobOutput,
        owned_attempt::{
            AttemptScope, OwnedAttemptExecutor, OwnedAttemptSession, OwnedExecutorPort,
            OwnedOutputCandidate,
        },
    },
};
use crate::domain::api_values::DecimalU64;
use kinekino_jobs::domain::PreparedJobInput;
use serde_json::Value;
use std::sync::Arc;

/// Rust host injection only. Request data cannot construct source/tool authority.
pub(crate) struct DownloadJobExecutor {
    core: DownloadOwnedExecutor,
}
impl DownloadJobExecutor {
    pub(crate) fn new(preparation: Arc<dyn DownloadPreparation>) -> Self {
        Self {
            core: DownloadOwnedExecutor {
                preparation: Arc::new(RequestBoundPreparation(preparation)),
            },
        }
    }
}
struct RequestBoundPreparation(Arc<dyn DownloadPreparation>);
impl DownloadPreparation for RequestBoundPreparation {
    fn prepare_in_session(
        &self,
        input: &PreparedJobInput,
        blobs: &dyn SourceBlobStore,
        scope: &AttemptScope<'_>,
        custody: super::SourceCustody<'_>,
    ) -> Result<super::PreparedDownloadResources, ApiError> {
        let request = DownloadRequest::from_prepared(input, blobs)?;
        if input.sha256().map_err(|_| {
            ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid prepared download input",
            )
        })? != scope.identity().input_sha256()
        {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Download preparation differs from original attempt",
            ));
        }
        let mut prepared = self.0.prepare_in_session(input, blobs, scope, custody)?;
        prepared.maximum_bytes = prepared.maximum_bytes.min(request.maximum_bytes.0);
        Ok(prepared)
    }
}
impl JobExecutor for DownloadJobExecutor {
    fn operations(&self) -> Vec<String> {
        vec![DOWNLOAD_OPERATION.into()]
    }
    fn supports(&self, operation: &str) -> bool {
        operation == DOWNLOAD_OPERATION
    }
    fn owned_executor(&self, operation: &str) -> Option<&OwnedExecutorPort> {
        self.supports(operation)
            .then_some(self as &OwnedExecutorPort)
    }
    fn normalize_parameters(&self, operation: &str, parameters: Value) -> Result<Value, ApiError> {
        if !self.supports(operation) {
            return Err(unavailable());
        }
        serde_json::to_value(DownloadRequest::decode(parameters)?)
            .map_err(|_| ApiError::new(ErrorCode::Internal, "Download normalization failed"))
    }
    fn execution_timeout_limit_ms(&self, operation: &str) -> Option<u64> {
        self.supports(operation).then_some(120_000)
    }
    fn prepare(
        &self,
        operation: &str,
        parameters: Value,
        expected: Option<&str>,
        blobs: &dyn SourceBlobStore,
    ) -> Result<PreparedJobInput, ApiError> {
        if expected.is_some() {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Request descriptor SHA does not freeze live media content",
            ));
        }
        let parameters = self.normalize_parameters(operation, parameters)?;
        let descriptor = DownloadRequest::decode(parameters.clone())?.descriptor()?;
        Ok(PreparedJobInput {
            operation: DOWNLOAD_OPERATION.into(),
            parameters,
            source: blobs.put_utf8(&descriptor)?,
            execution_timeout_ms: DecimalU64(15_000),
        })
    }
    fn execute(
        &self,
        _: &PreparedJobInput,
        _: &dyn SourceBlobStore,
        _: &dyn ExecutionControl,
    ) -> Result<VerifiedJobOutput, ApiError> {
        Err(ApiError::new(
            ErrorCode::Unsupported,
            "Download requires the original owned jobs lifecycle",
        ))
    }
}
impl OwnedAttemptExecutor for DownloadJobExecutor {
    type Candidate = Box<dyn OwnedOutputCandidate>;
    fn prepare_owned(
        &self,
        input: &PreparedJobInput,
        blobs: &dyn SourceBlobStore,
        scope: &AttemptScope<'_>,
    ) -> Result<Box<dyn OwnedAttemptSession<Candidate = Self::Candidate>>, ApiError> {
        DownloadRequest::from_prepared(input, blobs)?;
        self.core.prepare_owned(input, blobs, scope)
    }
}
fn unavailable() -> ApiError {
    ApiError::new(ErrorCode::Unsupported, "Download operation is unavailable")
}

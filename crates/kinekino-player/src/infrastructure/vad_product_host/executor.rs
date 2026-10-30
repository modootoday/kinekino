//! Request routing only; original VAD admission/owned use cases retain policy.
use super::super::vad_job_executor::{ScopeControl, owned_execution_error, validate_owned_input};
use super::inputs::ProductInputFactory;
use crate::application::audio_vad::{VAD_JOB_OPERATION, VadSourceRequest, normalize_parameters};
use kinekino_jobs::{
    application::{
        ExecutionControl, JobExecutor, SourceBlobStore,
        owned_attempt::{
            AttemptScope, OwnedAttemptExecutor, OwnedAttemptSession, OwnedExecutorPort,
            OwnedOutputCandidate,
        },
    },
    domain::{PreparedJobInput, VerifiedJobOutput},
};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use serde_json::Value;
use std::sync::Arc;
pub(super) struct ProductVadExecutor {
    factory: Arc<ProductInputFactory>,
}
impl ProductVadExecutor {
    pub(super) fn new(factory: Arc<ProductInputFactory>) -> Self {
        Self { factory }
    }
}
impl JobExecutor for ProductVadExecutor {
    fn supports(&self, operation: &str) -> bool {
        operation == VAD_JOB_OPERATION
    }
    fn operations(&self) -> Vec<String> {
        vec![VAD_JOB_OPERATION.into()]
    }
    fn owned_executor(&self, operation: &str) -> Option<&OwnedExecutorPort> {
        self.supports(operation)
            .then_some(self as &OwnedExecutorPort)
    }
    fn normalize_parameters(&self, operation: &str, parameters: Value) -> Result<Value, ApiError> {
        if !self.supports(operation) {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Unknown VAD job operation",
            ));
        }
        normalize_parameters(parameters)
    }
    fn prepare(
        &self,
        operation: &str,
        parameters: Value,
        expected: Option<&str>,
        blobs: &dyn SourceBlobStore,
    ) -> Result<PreparedJobInput, ApiError> {
        let parameters = self.normalize_parameters(operation, parameters)?;
        let request = VadSourceRequest::decode(parameters.clone(), expected)?;
        let bound = self.factory.bind(&request, blobs, None, None)?;
        bound
            .executor
            .prepare(operation, parameters, expected, blobs)
    }
    fn execute(
        &self,
        _: &PreparedJobInput,
        _: &dyn SourceBlobStore,
        _: &dyn ExecutionControl,
    ) -> Result<VerifiedJobOutput, ApiError> {
        Err(owned_execution_error())
    }
}
impl OwnedAttemptExecutor for ProductVadExecutor {
    type Candidate = Box<dyn OwnedOutputCandidate>;
    fn prepare_owned(
        &self,
        input: &PreparedJobInput,
        blobs: &dyn SourceBlobStore,
        scope: &AttemptScope<'_>,
    ) -> Result<Box<dyn OwnedAttemptSession<Candidate = Self::Candidate>>, ApiError> {
        validate_owned_input(input, scope)?;
        let request = VadSourceRequest::from_prepared(input)?;
        let control = ScopeControl(scope);
        let bound = self.factory.bind(
            &request,
            blobs,
            Some(&control),
            Some(scope.identity().job_id()),
        )?;
        // The returned original session owns independently re-admitted guards;
        // dropping the per-request facade neither drops its owner nor renews time.
        bound.executor.prepare_owned(input, blobs, scope)
    }
}

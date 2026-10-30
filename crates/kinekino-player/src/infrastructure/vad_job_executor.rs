//! Concrete public request → per-attempt guarded VAD adapter. Construction does
//! not register an operation, create a scope, or launch a process.
use super::{
    vad_job_attempts::{
        VadAttemptConfiguration, assemble_parent_configured_attempt_executor,
        assemble_static_worker_attempt_executor,
    },
    vad_release_inputs::GuardedReleasedVadInputs,
};
use crate::application::audio_vad::product_release::ConfiguredVadRelease;
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
use kinekino_media_contracts::{
    api_values::DecimalU64,
    errors::{ApiError, ErrorCode},
};
use kinekino_vad::application::{AnalysisControl, ProfileAdmission};
use kinekino_vad_executor::{
    application::{
        ParentSelectedRelease, PrepareVadRequest, PreparedVadAdmission, VadAdmissionPorts,
        prepare_vad_admission,
    },
    domain::ExpectedDigest,
};
use serde_json::Value;
use std::{sync::Arc, time::Instant};

pub(super) struct VadJobExecutor {
    inputs: Arc<GuardedReleasedVadInputs>,
    admission: Arc<dyn ProfileAdmission + Send + Sync>,
    execution: VadAttemptConfiguration,
    selected: Option<Arc<ParentSelectedRelease>>,
}
impl VadJobExecutor {
    pub(super) fn new(
        inputs: Arc<GuardedReleasedVadInputs>,
        admission: Arc<dyn ProfileAdmission + Send + Sync>,
        execution: VadAttemptConfiguration,
    ) -> Self {
        Self {
            inputs,
            admission,
            execution,
            selected: None,
        }
    }

    fn mapped(&self, source: &VadSourceRequest) -> Result<PrepareVadRequest, ApiError> {
        let public = source.parameters();
        let profile_manifest =
            ExpectedDigest::new(public.expected_profile_manifest_sha256.clone())?;
        let audio_source = ExpectedDigest::new(source.original_audio_sha256().into())?;
        Ok(PrepareVadRequest {
            profile: self
                .inputs
                .resolve_profile(&public.profile_id, &profile_manifest)?,
            profile_manifest,
            audio: self
                .inputs
                .resolve_audio(&public.audio_artifact_id, &audio_source)?,
            audio_source,
            start: public.start_sample,
            end: public.end_sample,
            destination: self.inputs.resolve_destination(&public.destination_id)?,
        })
    }
    fn admit(
        &self,
        request: &VadSourceRequest,
        blobs: &dyn SourceBlobStore,
        scope: Option<&AttemptScope<'_>>,
    ) -> Result<PreparedVadAdmission, ApiError> {
        let mapped = self.mapped(request)?;
        if let Some(scope) = scope {
            check_scope(scope)?;
            let control = ScopeControl(scope);
            let inputs = self.inputs.controlled(&control);
            let admitted = prepare_vad_admission(
                &mapped,
                VadAdmissionPorts {
                    profiles: &inputs,
                    audio: &inputs,
                    destinations: &inputs,
                    admission: self.admission.as_ref(),
                    blobs,
                },
            )?;
            check_scope(scope)?;
            Ok(admitted)
        } else {
            // Submit has no Running identity/scope. This path is bounded file
            // admission, not a fabricated cancellable original attempt.
            prepare_vad_admission(
                &mapped,
                VadAdmissionPorts {
                    profiles: self.inputs.as_ref(),
                    audio: self.inputs.as_ref(),
                    destinations: self.inputs.as_ref(),
                    admission: self.admission.as_ref(),
                    blobs,
                },
            )
        }
    }
    fn configuration(&self) -> VadAttemptConfiguration {
        // Copy trusted configuration data, never a previous attempt's owner or cutoff.
        VadAttemptConfiguration {
            worker: self.execution.worker.clone(),
            worker_pin: self.execution.worker_pin.clone(),
            evidence_root: self.execution.evidence_root.clone(),
            cleanup_reserve: self.execution.cleanup_reserve,
            environment: self.execution.environment.clone(),
        }
    }
}
fn prepared(
    request: &VadSourceRequest,
    admission: &PreparedVadAdmission,
    execution_timeout_ms: DecimalU64,
) -> Result<PreparedJobInput, ApiError> {
    let input = PreparedJobInput {
        operation: VAD_JOB_OPERATION.into(),
        parameters: request.prepared_parameters()?,
        source: admission.descriptor().source().clone(),
        execution_timeout_ms,
    };
    input
        .validate()
        .map_err(|message| ApiError::new(ErrorCode::InvalidArgument, message))?;
    Ok(input)
}
impl JobExecutor for VadJobExecutor {
    fn supports(&self, operation: &str) -> bool {
        operation == VAD_JOB_OPERATION
    }
    fn operations(&self) -> Vec<String> {
        vec![VAD_JOB_OPERATION.into()]
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
    fn owned_executor(&self, operation: &str) -> Option<&OwnedExecutorPort> {
        self.supports(operation)
            .then_some(self as &OwnedExecutorPort)
    }
    fn prepare(
        &self,
        operation: &str,
        parameters: Value,
        expected_source: Option<&str>,
        blobs: &dyn SourceBlobStore,
    ) -> Result<PreparedJobInput, ApiError> {
        if !self.supports(operation) {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Unknown VAD job operation",
            ));
        }
        let request = VadSourceRequest::decode(parameters, expected_source)?;
        let admitted = self.admit(&request, blobs, None)?;
        // Same preparation seed convention as the existing subtitle adapter.
        // JobService replaces this with the actual submitted budget before storing.
        prepared(&request, &admitted, DecimalU64(30000))
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
pub(super) struct ScopeControl<'a, 'b>(pub(super) &'a AttemptScope<'b>);
impl AnalysisControl for ScopeControl<'_, '_> {
    fn cancellation_requested(&self) -> bool {
        self.0.cancellation_requested()
    }
    fn deadline(&self) -> Instant {
        self.0.deadline()
    }
}
fn check_scope(scope: &AttemptScope<'_>) -> Result<(), ApiError> {
    if Instant::now() >= scope.deadline() {
        return Err(ApiError::new(
            ErrorCode::Timeout,
            "Original VAD preparation deadline expired",
        ));
    }
    if scope.cancellation_requested() {
        return Err(ApiError::new(
            ErrorCode::Cancelled,
            "Original VAD preparation cancelled; no cleanup ACK",
        ));
    }
    Ok(())
}
impl OwnedAttemptExecutor for VadJobExecutor {
    type Candidate = Box<dyn OwnedOutputCandidate>;
    fn prepare_owned(
        &self,
        input: &PreparedJobInput,
        blobs: &dyn SourceBlobStore,
        scope: &AttemptScope<'_>,
    ) -> Result<Box<dyn OwnedAttemptSession<Candidate = Self::Candidate>>, ApiError> {
        validate_owned_input(input, scope)?;
        let expected = input
            .sha256()
            .map_err(|message| ApiError::new(ErrorCode::InvalidArgument, message))?;
        let request = VadSourceRequest::from_prepared(input)?;
        let admitted = self.admit(&request, blobs, Some(scope))?;
        let reconstructed = prepared(&request, &admitted, input.execution_timeout_ms)?;
        if reconstructed
            .sha256()
            .map_err(|message| ApiError::new(ErrorCode::InvalidArgument, message))?
            != expected
        {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Re-admitted VAD input differs from original prepared snapshot",
            ));
        }
        check_scope(scope)?;
        let executor = match &self.selected {
            Some(selected) => assemble_parent_configured_attempt_executor(
                self.configuration(),
                input,
                Arc::new(admitted),
                self.inputs.clone(),
                selected.clone(),
            )?,
            None => assemble_static_worker_attempt_executor(
                self.configuration(),
                input,
                Arc::new(admitted),
                self.inputs.clone(),
            )?,
        };
        check_scope(scope)?;
        // The returned original session owns its transport/publication/admission
        // leases. No per-job in-memory map or lifetime of this local executor is needed.
        executor.prepare_owned(input, blobs, scope)
    }
}
pub(super) fn validate_owned_input(
    input: &PreparedJobInput,
    scope: &AttemptScope<'_>,
) -> Result<(), ApiError> {
    check_scope(scope)?;
    let expected = input
        .sha256()
        .map_err(|message| ApiError::new(ErrorCode::InvalidArgument, message))?;
    if expected != scope.identity().input_sha256()
        || input.source.sha256 != scope.identity().source_blob_sha256()
    {
        return Err(ApiError::new(
            ErrorCode::RevisionConflict,
            "VAD input differs from original attempt",
        ));
    }
    Ok(())
}
pub(super) fn owned_execution_error() -> ApiError {
    ApiError::new(
        ErrorCode::Unsupported,
        "VAD requires original retaining owned execution",
    )
}
type ProductExecutionBinding = (
    Arc<GuardedReleasedVadInputs>,
    Arc<ConfiguredVadRelease>,
    VadAttemptConfiguration,
    Arc<ParentSelectedRelease>,
);
impl TryFrom<ProductExecutionBinding> for VadJobExecutor {
    type Error = ApiError;
    fn try_from(
        (inputs, release, execution, selected): ProductExecutionBinding,
    ) -> Result<Self, ApiError> {
        selected.check_before_spawn(
            release.profile(),
            release.manifest(),
            &execution.worker_pin,
        )?;
        inputs.resolve_profile(
            release.public_profile_id(),
            &ExpectedDigest::new(release.manifest().sha256().into())?,
        )?;
        let mut executor = Self::new(inputs, release, execution);
        executor.selected = Some(selected);
        Ok(executor)
    }
}

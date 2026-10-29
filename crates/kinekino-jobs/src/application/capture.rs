use super::{
    CapturedJobRequest, SubmitJob,
    service::{ExecutionResources, normalize_request},
};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};

/// Called only while the original service's admission gate is held.
pub(super) fn prepare(
    backend: &ExecutionResources,
    input: SubmitJob,
) -> Result<CapturedJobRequest, ApiError> {
    let mut prepared = backend.executor.prepare(
        &input.operation,
        input.parameters,
        input.expected_source_sha256.as_deref(),
        backend.blobs.as_ref(),
    )?;
    prepared.execution_timeout_ms = input.execution_timeout_ms;
    prepared
        .validate()
        .map_err(|message| ApiError::new(ErrorCode::InvalidArgument, message))?;
    if prepared.operation != input.operation {
        return Err(ApiError::new(
            ErrorCode::RevisionConflict,
            "Capture adapter changed its operation",
        ));
    }
    let captured = SubmitJob {
        operation: prepared.operation,
        parameters: prepared.parameters,
        expected_source_sha256: Some(prepared.source.sha256),
        execution_timeout_ms: input.execution_timeout_ms,
    };
    let repeated = normalize_request(backend, captured.clone())?;
    let bytes = serde_json::to_vec(&captured)
        .map_err(|_| ApiError::new(ErrorCode::Internal, "Invalid captured request"))?;
    let repeated_bytes = serde_json::to_vec(&repeated)
        .map_err(|_| ApiError::new(ErrorCode::Internal, "Invalid repeated capture request"))?;
    if bytes != repeated_bytes {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Captured request cannot reenter normalization without changing identity",
        ));
    }
    Ok(CapturedJobRequest::new(captured))
}

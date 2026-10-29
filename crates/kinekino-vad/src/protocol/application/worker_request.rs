use super::{MAX_DESCRIPTOR_BYTES, decode_descriptor, descriptor::DescriptorDto, payload};
use crate::domain::ArtifactPin;
use crate::protocol::domain::ExpectedWorkerContext;
use kinekino_media_contracts::{
    api_values::DecimalU64,
    errors::{ApiError, ErrorCode},
};
use serde::Deserialize;
use serde_json::Value;
use std::io::Read;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RequestDto {
    schema_version: String,
    job_id: String,
    attempt: DecimalU64,
    input_sha256: String,
    source_blob_sha256: String,
    submission_request_sha256: Option<String>,
    descriptor_sha256: String,
    context: DescriptorDto,
}

/// Validated wire data, not an admission credential or a new SDK attempt identity.
pub struct ValidatedWorkerRequest {
    pub(super) expected: ExpectedWorkerContext,
    pub(super) descriptor_json: Value,
}
impl ValidatedWorkerRequest {
    pub fn expected(&self) -> &ExpectedWorkerContext {
        &self.expected
    }
}

/// `descriptor_raw` is the parent's original immutable descriptor sidecar.
/// Never reconstruct its digest by serializing the nested request context.
pub fn decode_worker_request(
    reader: impl Read,
    declared_bytes: u64,
    descriptor_raw: &[u8],
) -> Result<ValidatedWorkerRequest, ApiError> {
    let raw = payload::read(reader, declared_bytes, MAX_DESCRIPTOR_BYTES)?;
    let request: RequestDto = payload::parse(&raw.bytes)?;
    if request.schema_version != "1"
        || request.attempt.0 == 0
        || request.job_id.is_empty()
        || request.job_id.len() > 128
        || request.job_id.chars().any(char::is_control)
    {
        return Err(payload::invalid("Invalid worker request labels/version"));
    }
    // Shape validation only: the worker does not authorize these parent labels.
    for digest in [&request.input_sha256, &request.source_blob_sha256]
        .into_iter()
        .chain(request.submission_request_sha256.as_ref())
    {
        ArtifactPin::new("parent-label".into(), digest.clone(), DecimalU64(1))?;
    }
    let descriptor_pin = ArtifactPin::new(
        "vad-descriptor".into(),
        request.descriptor_sha256,
        DecimalU64(descriptor_raw.len() as u64),
    )?;
    let descriptor = decode_descriptor(descriptor_raw, &descriptor_pin)?;
    let request_pin = ArtifactPin::new(
        "worker-request".into(),
        raw.sha256,
        DecimalU64(raw.bytes.len() as u64),
    )?;
    let expected = ExpectedWorkerContext::new(request_pin, descriptor);
    let (profile, input) = request.context.checked()?;
    if !expected.matches(&profile, &input) {
        return Err(ApiError::new(
            ErrorCode::RevisionConflict,
            "Request context differs from original descriptor",
        ));
    }
    Ok(ValidatedWorkerRequest {
        expected,
        descriptor_json: payload::parse(descriptor_raw)?,
    })
}

use crate::domain::identifiers::{valid_id, valid_sha256};
use kinekino_media_contracts::{
    api_values::DecimalU64,
    errors::{ApiError, ErrorCode},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const VAD_JOB_OPERATION: &str = "audio.vad";

/// Opaque product identities. A release manifest hash is not the private
/// executor's VadDescriptorV1 profile-snapshot hash.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VadJobParameters {
    pub profile_id: String,
    pub expected_profile_manifest_sha256: String,
    pub audio_artifact_id: String,
    pub start_sample: DecimalU64,
    pub end_sample: DecimalU64,
    pub destination_id: String,
}
impl VadJobParameters {
    fn decode(value: Value) -> Result<Self, ApiError> {
        let parameters: Self = serde_json::from_value(value)
            .map_err(|error| invalid(format!("Invalid VAD parameters: {error}")))?;
        if !valid_id(&parameters.profile_id)
            || !valid_id(&parameters.audio_artifact_id)
            || !valid_id(&parameters.destination_id)
            || !valid_sha256(&parameters.expected_profile_manifest_sha256)
        {
            return Err(invalid("Invalid VAD identity or release manifest SHA256"));
        }
        // Range bounds/sample format belong to the admitted VadInput use case.
        Ok(parameters)
    }
}

/// Validated wire syntax and original audio pin; not an admitted model, source,
/// immutable file guard, prepared descriptor or execution authority.
#[derive(Debug)]
pub struct VadSourceRequest {
    parameters: VadJobParameters,
    original_audio_sha256: String,
}
impl VadSourceRequest {
    pub fn decode(
        parameters: Value,
        expected_source_sha256: Option<&str>,
    ) -> Result<Self, ApiError> {
        let parameters = VadJobParameters::decode(parameters)?;
        let original_audio_sha256 = expected_source_sha256
            .filter(|sha| valid_sha256(sha))
            .ok_or_else(|| invalid("VAD requires the original audio source SHA256"))?
            .to_owned();
        Ok(Self {
            parameters,
            original_audio_sha256,
        })
    }
    pub fn parameters(&self) -> &VadJobParameters {
        &self.parameters
    }
    pub fn original_audio_sha256(&self) -> &str {
        &self.original_audio_sha256
    }
}

/// Canonical typed parameter serialization before JobService fingerprints the
/// full submission. This performs no lookup, file I/O or operation registration.
pub fn normalize_parameters(parameters: Value) -> Result<Value, ApiError> {
    serde_json::to_value(VadJobParameters::decode(parameters)?)
        .map_err(|_| ApiError::new(ErrorCode::Internal, "VAD parameter serialization failed"))
}
fn invalid(message: impl Into<String>) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}

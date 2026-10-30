use super::{VAD_JOB_OPERATION, VadJobParameters, VadSourceRequest};
use kinekino_jobs::domain::PreparedJobInput;
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const VERSION: &str = "1";

/// Internal persistence format, never accepted as public operation parameters.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    schema_version: String,
    request: VadJobParameters,
    original_source_sha256: String,
}

impl VadSourceRequest {
    /// Store the original audio pin alongside the normalized public request.
    /// Descriptor bytes/pin remain PreparedJobInput.source, not this audio pin.
    pub fn prepared_parameters(&self) -> Result<Value, ApiError> {
        serde_json::to_value(Envelope {
            schema_version: VERSION.into(),
            request: self.parameters().clone(),
            original_source_sha256: self.original_audio_sha256().into(),
        })
        .map_err(|_| ApiError::new(ErrorCode::Internal, "VAD prepared request encoding failed"))
    }

    /// Reconstruct the request after durable reopen/retry. This validates data
    /// only: callers must re-admit actual bytes/guards under the original scope.
    pub fn from_prepared(input: &PreparedJobInput) -> Result<Self, ApiError> {
        if input.operation != VAD_JOB_OPERATION {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Not a prepared VAD job",
            ));
        }
        input
            .validate()
            .map_err(|message| ApiError::new(ErrorCode::InvalidArgument, message))?;
        let envelope: Envelope =
            serde_json::from_value(input.parameters.clone()).map_err(|error| {
                ApiError::new(
                    ErrorCode::InvalidArgument,
                    format!("Invalid VAD prepared request: {error}"),
                )
            })?;
        if envelope.schema_version != VERSION {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Unsupported VAD prepared request version",
            ));
        }
        let public = serde_json::to_value(envelope.request).map_err(|_| {
            ApiError::new(ErrorCode::Internal, "VAD prepared request decoding failed")
        })?;
        Self::decode(public, Some(&envelope.original_source_sha256))
    }
}

#[cfg(test)]
mod tests;

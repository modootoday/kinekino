use super::{MAX_OUTCOME_BYTES, MAX_SCORES, ValidatedWorkerRequest, decode_worker_outcome};
use crate::protocol::domain::WorkerResult;
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use serde_json::json;
use std::io::{self, Write};

struct BoundedOutput(Vec<u8>);
impl Write for BoundedOutput {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > MAX_OUTCOME_BYTES as usize - self.0.len() {
            return Err(io::Error::other("VAD response exceeds wire bound"));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Uses the same existing outcome decoder as a final semantic validation.
/// Runtime observations stay absent until independently measured; configured
/// CPU/precision/thread settings must not be emitted as observed values.
pub fn encode_worker_response(
    request: &ValidatedWorkerRequest,
    result: &WorkerResult,
) -> Result<Vec<u8>, ApiError> {
    let (kind, processed, scores, failure) = match result {
        WorkerResult::Complete { scores } => (
            "complete",
            Some(request.expected.input().selected_samples()),
            scores.as_slice(),
            None,
        ),
        WorkerResult::Partial {
            processed_samples,
            scores,
        } => ("partial", Some(*processed_samples), scores.as_slice(), None),
        WorkerResult::Failure { code, message } => {
            if message.len() > 4096 {
                return Err(ApiError::new(
                    ErrorCode::ResourceExhausted,
                    "Failure text exceeds bound",
                ));
            }
            (
                "failure",
                None,
                &[][..],
                Some(json!({"code":code,"message":message})),
            )
        }
    };
    if scores.len() > MAX_SCORES {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Score wire bound exceeded",
        ));
    }
    let value = json!({
        "schema_version":"1", "request_sha256":request.expected.request().sha256(),
        "descriptor_sha256":request.expected.descriptor().raw().sha256(),
        "context":request.descriptor_json, "kind":kind,
        "processed_samples":processed.map(|n| n.to_string()),
        "scores":scores, "failure":failure, "actual_runtime":null,
    });
    let mut output = BoundedOutput(Vec::new());
    serde_json::to_writer(&mut output, &value).map_err(|_| {
        ApiError::new(
            ErrorCode::ResourceExhausted,
            "Response serialization/bound failed",
        )
    })?;
    decode_worker_outcome(
        output.0.as_slice(),
        output.0.len() as u64,
        &request.expected,
    )?;
    Ok(output.0)
}

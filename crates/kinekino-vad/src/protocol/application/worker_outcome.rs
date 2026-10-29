use super::{
    MAX_OUTCOME_BYTES, descriptor::DescriptorDto, modules::Modules, payload, scores::Scores,
};
use crate::domain::{ArtifactPin, CHUNK_SAMPLES};
use crate::protocol::domain::{
    ActualRuntimeObservation, ExpectedWorkerContext, WorkerOutcomeCandidate, WorkerResult,
};
use kinekino_media_contracts::{
    api_values::DecimalU64,
    errors::{ApiError, ErrorCode},
};
use serde::Deserialize;
use std::io::Read;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FailureDto {
    code: String,
    message: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RuntimeDto {
    provider: Option<String>,
    precision: Option<String>,
    intra_threads: Option<u32>,
    inter_threads: Option<u32>,
    modules: Modules,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OutcomeDto {
    schema_version: String,
    request_sha256: String,
    descriptor_sha256: String,
    context: DescriptorDto,
    kind: String,
    processed_samples: Option<DecimalU64>,
    scores: Scores,
    failure: Option<FailureDto>,
    actual_runtime: Option<RuntimeDto>,
}
pub fn decode_worker_outcome(
    reader: impl Read,
    declared_bytes: u64,
    expected: &ExpectedWorkerContext,
) -> Result<WorkerOutcomeCandidate, ApiError> {
    let raw = payload::read(reader, declared_bytes, MAX_OUTCOME_BYTES)?;
    let value: OutcomeDto = payload::parse(&raw.bytes)?;
    if value.schema_version != "1"
        || value.request_sha256 != expected.request().sha256()
        || value.descriptor_sha256 != expected.descriptor().raw().sha256()
    {
        return Err(ApiError::new(
            ErrorCode::RevisionConflict,
            "Worker request/descriptor binding differs",
        ));
    }
    let (profile, input) = value.context.checked()?;
    if !expected.matches(&profile, &input) {
        return Err(ApiError::new(
            ErrorCode::RevisionConflict,
            "Worker profile/input/range differs",
        ));
    }
    let actual_runtime = match value.actual_runtime {
        Some(runtime) => {
            if !payload::optional_text(&runtime.provider, 128)
                || !payload::optional_text(&runtime.precision, 64)
                || runtime.intra_threads == Some(0)
                || runtime.inter_threads == Some(0)
            {
                return Err(payload::invalid("Invalid actual runtime observation"));
            }
            Some(ActualRuntimeObservation {
                provider: runtime.provider,
                precision: runtime.precision,
                intra_threads: runtime.intra_threads,
                inter_threads: runtime.inter_threads,
                modules: runtime.modules.0,
            })
        }
        None => None,
    };
    let scores = value.scores.0;
    let result = match value.kind.as_str() {
        "complete" => {
            if value.failure.is_some()
                || value.processed_samples.map(|n| n.0) != Some(input.selected_samples())
                || scores.len() as u64 != input.expected_frames()
            {
                return Err(payload::invalid(
                    "Incomplete/contradictory complete outcome",
                ));
            }
            WorkerResult::Complete { scores }
        }
        "partial" => {
            let processed = value
                .processed_samples
                .ok_or_else(|| payload::invalid("Missing partial sample count"))?
                .0;
            if value.failure.is_some()
                || processed > input.selected_samples()
                || scores.len() as u64 != processed.div_ceil(CHUNK_SAMPLES)
            {
                return Err(payload::invalid("Invalid partial outcome"));
            }
            WorkerResult::Partial {
                processed_samples: processed,
                scores,
            }
        }
        "failure" => {
            let failure = value
                .failure
                .ok_or_else(|| payload::invalid("Missing engine failure"))?;
            if value.processed_samples.is_some()
                || !scores.is_empty()
                || failure.message.is_empty()
                || failure.message.len() > 4096
                || failure
                    .message
                    .chars()
                    .any(|c| c.is_control() && c != '\n' && c != '\t')
            {
                return Err(payload::invalid("Invalid failure outcome"));
            }
            WorkerResult::Failure {
                code: ErrorCode::deserialize(serde::de::value::StringDeserializer::<
                    serde::de::value::Error,
                >::new(failure.code))
                .map_err(|_| payload::invalid("Invalid engine error code"))?,
                message: failure.message,
            }
        }
        _ => return Err(payload::invalid("Unknown worker outcome kind")),
    };
    Ok(WorkerOutcomeCandidate {
        raw: ArtifactPin::new(
            "worker-outcome".into(),
            raw.sha256,
            DecimalU64(raw.bytes.len() as u64),
        )?,
        context: expected.clone(),
        result,
        actual_runtime,
    })
}

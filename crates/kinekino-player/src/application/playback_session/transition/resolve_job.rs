//! Transition preparation using the durable episodes.resolve executor.
use super::{
    adjacency::{self, EpisodeDirection, PreparedAdjacentEpisode},
    service::{EpisodeReplacementOwner, ReplacedEpisode},
};
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        catalog::{CatalogResults, RawArtifact},
        episode_resolve::EpisodeResolutionResults,
        jobs::{GetJob, JobId, JobRecord, JobState, JobsApi, SubmitJob, VerifiedJobOutput},
    },
    domain::api_values::DecimalU64,
};

#[cfg(test)]
#[path = "resolve_job/retirement_tests.rs"]
mod retirement_tests;
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BeginAdjacentResolution {
    pub expected_instance_id: DecimalU64,
    pub catalog_result_sha256: String,
    pub direction: EpisodeDirection,
    pub paused: bool,
    pub volume_percent: f64,
    pub execution_timeout_ms: DecimalU64,
}

/// In-memory ownership of one preparation. Durable resolve remains in JobsApi.
/// No Deserialize: restart recovery must re-prepare against the live owned engine.
pub struct PendingAdjacentResolution {
    prepared: PreparedAdjacentEpisode,
    expected_instance_id: DecimalU64,
    job_id: JobId,
    attempt: DecimalU64,
    paused: bool,
    volume_percent: f64,
}

pub fn begin(
    catalogs: &dyn CatalogResults,
    resolutions: &dyn EpisodeResolutionResults,
    jobs: &dyn JobsApi,
    owner: &dyn EpisodeReplacementOwner,
    submission_key: &str,
    input: BeginAdjacentResolution,
) -> Result<PendingAdjacentResolution, ApiError> {
    if input.expected_instance_id.0 == 0
        || !input.volume_percent.is_finite()
        || !(0.0..=100.0).contains(&input.volume_percent)
        || !(1..=120_000).contains(&input.execution_timeout_ms.0)
    {
        return Err(invalid());
    }
    let current = owner.observe()?;
    if current.engine_instance_id != input.expected_instance_id {
        return Err(changed());
    }
    let prepared = adjacency::prepare(
        catalogs,
        resolutions,
        &input.catalog_result_sha256,
        &current.resolution_artifact_id,
        input.direction,
    )?;
    let submitted = jobs.submit(
        SubmitJob {
            operation: "episodes.resolve".into(),
            parameters: serde_json::json!({"episode_url":prepared.target_episode_url()}),
            expected_source_sha256: None,
            execution_timeout_ms: input.execution_timeout_ms,
        },
        submission_key,
    )?;
    let pending = PendingAdjacentResolution {
        prepared,
        expected_instance_id: input.expected_instance_id,
        job_id: submitted.job.job_id.clone(),
        attempt: submitted.job.attempt,
        paused: input.paused,
        volume_percent: input.volume_percent,
    };
    pending.require_job(&submitted.job).map_err(|mut error| {
        // Submission may already be durable even if its returned record is inconsistent.
        // Keep the job identity available for explicit inspection/cancellation.
        error.details = serde_json::json!({
            "stage":"validate_admitted_adjacent_job",
            "resolution_job_id":submitted.job.job_id,"attempt":submitted.job.attempt,
            "job_created":submitted.created,"transition_handle_created":false,
            "current_engine_replaced":false,"job_cleanup_confirmed":false,
        });
        error
    })?;
    Ok(pending)
}

impl PendingAdjacentResolution {
    pub fn job_id(&self) -> &JobId {
        &self.job_id
    }

    /// No sleeps, retry, cancellation rewriting or automatic execution after host restart.
    pub fn observe(&self, jobs: &dyn JobsApi) -> Result<JobRecord, ApiError> {
        let job = jobs.get(GetJob {
            job_id: self.job_id.clone(),
        })?;
        self.require_job(&job)?;
        Ok(job)
    }

    pub fn commit(
        &self,
        jobs: &dyn JobsApi,
        resolutions: &dyn EpisodeResolutionResults,
        owner: &dyn EpisodeReplacementOwner,
    ) -> Result<ReplacedEpisode, ApiError> {
        let job = self.observe(jobs)?;
        if job.state != JobState::Completed {
            let mut error = ApiError::new(
                ErrorCode::NeedsInput,
                "Adjacent resolve job is not completed",
            );
            error.details = serde_json::json!({"job_id":job.job_id,"state":job.state,"current_engine_replaced":false});
            return Err(error);
        }
        let output = job.output.as_ref().ok_or_else(invalid)?;
        let artifact = resolved_artifact(output, &job.input.source.sha256)?;
        let source = self
            .prepared
            .verify_resolution(resolutions, &artifact.artifact_id)?;
        owner.replace_verified(
            self.expected_instance_id,
            self.prepared.current_resolution_artifact_id(),
            source,
            self.paused,
            self.volume_percent,
        )
    }

    pub(super) fn require_job(&self, job: &JobRecord) -> Result<(), ApiError> {
        job.validate().map_err(|_| invalid())?;
        if job.job_id != self.job_id
            || job.attempt != self.attempt
            || job.input.operation != "episodes.resolve"
            || job.input.parameters
                != serde_json::json!({"episode_url":self.prepared.target_episode_url()})
        {
            return Err(changed());
        }
        Ok(())
    }
}
fn resolved_artifact(
    output: &VerifiedJobOutput,
    descriptor: &str,
) -> Result<RawArtifact, ApiError> {
    let expired = || {
        ApiError::new(
            ErrorCode::SourceExpired,
            "Adjacent job output identity mismatch",
        )
    };
    output.validate().map_err(|_| expired())?;
    let artifact: RawArtifact =
        serde_json::from_value(output.receipt["resolution_artifact"].clone())
            .map_err(|_| expired())?;
    if output.receipt["kind"] != "episode_resolution"
        || output.receipt["source_descriptor_sha256"] != descriptor
        || output.sha256 != artifact.body_sha256
        || output.bytes != artifact.body_bytes
    {
        return Err(expired());
    }
    Ok(artifact)
}
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::InvalidArgument,
        "Invalid adjacent resolution contract",
    )
}
fn changed() -> ApiError {
    ApiError::new(
        ErrorCode::RevisionConflict,
        "Adjacent resolution ownership changed",
    )
}

#[cfg(test)]
mod output_tests {
    use super::*;
    fn fixture() -> (VerifiedJobOutput, String, RawArtifact) {
        let descriptor = "a".repeat(64);
        let artifact = RawArtifact {
            artifact_id: "b".repeat(64),
            body_sha256: "c".repeat(64),
            body_bytes: DecimalU64(123),
            body_path: "objects/result.body".into(),
            metadata_path: "captures/result.json".into(),
        };
        let output = VerifiedJobOutput {
            path: "private/result.body".into(),
            sha256: artifact.body_sha256.clone(),
            bytes: artifact.body_bytes,
            receipt: serde_json::json!({"kind":"episode_resolution","source_descriptor_sha256":descriptor,"resolution_artifact":artifact}),
        };
        (output, descriptor, artifact)
    }
    #[test]
    fn binds_output_to_the_resolve_operation_and_request_descriptor() {
        let (output, descriptor, artifact) = fixture();
        assert_eq!(resolved_artifact(&output, &descriptor).unwrap(), artifact);
        assert_eq!(
            resolved_artifact(&output, &"d".repeat(64))
                .unwrap_err()
                .code,
            ErrorCode::SourceExpired
        );
        let mut unrelated = output;
        unrelated.receipt["kind"] = serde_json::json!("catalog_result");
        assert_eq!(
            resolved_artifact(&unrelated, &descriptor).unwrap_err().code,
            ErrorCode::SourceExpired
        );
    }
    #[test]
    fn rejects_output_digest_and_length_mismatch_before_source_transfer() {
        let (output, descriptor, _) = fixture();
        let mut wrong_hash = output.clone();
        wrong_hash.sha256 = "e".repeat(64);
        let mut wrong_bytes = output;
        wrong_bytes.bytes = DecimalU64(124);
        for wrong in [wrong_hash, wrong_bytes] {
            assert_eq!(
                resolved_artifact(&wrong, &descriptor).unwrap_err().code,
                ErrorCode::SourceExpired
            );
        }
    }
    #[test]
    fn rejects_missing_or_malformed_reference_in_a_completed_output() {
        let (mut output, descriptor, _) = fixture();
        output.receipt["resolution_artifact"] = serde_json::Value::Null;
        assert_eq!(
            resolved_artifact(&output, &descriptor).unwrap_err().code,
            ErrorCode::SourceExpired
        );
        output.receipt["resolution_artifact"] = serde_json::json!({"artifact_id":"b".repeat(64)});
        assert_eq!(
            resolved_artifact(&output, &descriptor).unwrap_err().code,
            ErrorCode::SourceExpired
        );
    }
}

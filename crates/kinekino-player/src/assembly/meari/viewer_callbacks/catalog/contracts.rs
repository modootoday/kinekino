//! UI joins original typed replies; these values grant no SDK playback or browser authority.
use crate::application::{
    api::{ApiError, ErrorCode},
    catalog::{Entry, RawArtifact, ResultArtifactRef},
    episode_resolve::EpisodeResolutionInspection,
    jobs::{JobId, JobRecord, JobState},
    playback_session::OpenEpisodePlayback,
};
use serde::Deserialize;

#[derive(Clone)]
pub(super) struct SelectedSeries {
    pub key: String,
    pub list: ResultArtifactRef,
    pub row: Entry,
}
#[derive(Clone)]
pub(super) struct SelectedEpisode {
    pub key: String,
    pub series: SelectedSeries,
    pub list: ResultArtifactRef,
    pub row: Entry,
}
#[derive(Deserialize)]
pub(super) struct CatalogReceipt {
    pub kind: String,
    pub source_kind: String,
    pub result_artifact: ResultArtifactRef,
}
#[derive(Deserialize)]
pub(super) struct ResolutionReceipt {
    pub kind: String,
    pub source_kind: String,
    pub source_descriptor_sha256: String,
    pub resolution_artifact: RawArtifact,
}
pub(super) struct ResolutionChoice {
    pub selected: SelectedEpisode,
    pub job_id: JobId,
    pub attempt: u64,
    pub inspection: EpisodeResolutionInspection,
}
impl ResolutionChoice {
    /// The inspector still reopens the original SDK evidence. UI never reads output.path.
    pub(super) fn inspector_id(job: &JobRecord) -> Result<String, ApiError> {
        if job.input.operation != "episodes.resolve" || job.state != JobState::Completed {
            return Err(mismatch("Original resolution job has not completed"));
        }
        let output = job
            .output
            .as_ref()
            .ok_or_else(|| mismatch("Original resolution output missing"))?;
        let receipt: ResolutionReceipt = serde_json::from_value(output.receipt.clone())
            .map_err(|_| mismatch("Original resolution receipt type mismatch"))?;
        if receipt.kind != "episode_resolution"
            || receipt.source_kind != "request_descriptor"
            || receipt.source_descriptor_sha256 != job.input.source.sha256
        {
            return Err(mismatch("Original resolution receipt source mismatch"));
        }
        // body SHA and artifact identity are different contracts; never use output.sha256 here.
        Ok(receipt.resolution_artifact.artifact_id)
    }
    pub(super) fn open_input(&self, volume_percent: f64, paused: bool) -> OpenEpisodePlayback {
        OpenEpisodePlayback {
            resolution_artifact_id: self.inspection.resolution_artifact_id.clone(),
            volume_percent,
            paused,
        }
    }
}
pub(super) fn catalog_read_id(job: &JobRecord, operation: &str) -> Result<String, ApiError> {
    if !matches!(operation, "catalog.search" | "episodes.list")
        || job.input.operation != operation
        || job.state != JobState::Completed
    {
        return Err(mismatch("Original catalog job has not completed"));
    }
    let output = job
        .output
        .as_ref()
        .ok_or_else(|| mismatch("Original catalog output missing"))?;
    let receipt: CatalogReceipt = serde_json::from_value(output.receipt.clone())
        .map_err(|_| mismatch("Original catalog receipt type mismatch"))?;
    if receipt.kind != "catalog_result"
        || receipt.source_kind != "request_descriptor"
        || receipt.result_artifact.operation != operation
        || receipt.result_artifact.source_descriptor_sha256 != job.input.source.sha256
        || receipt.result_artifact.sha256 != output.sha256
    {
        return Err(mismatch("Original catalog receipt identity mismatch"));
    }
    Ok(receipt.result_artifact.sha256)
}
fn mismatch(message: &'static str) -> ApiError {
    ApiError::new(ErrorCode::RevisionConflict, message)
}

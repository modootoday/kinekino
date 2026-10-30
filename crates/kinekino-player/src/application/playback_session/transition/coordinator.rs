//! Shared application facade for API, GUI and CLI consumers.
use super::{
    resolve_job::{self, BeginAdjacentResolution},
    service::{EpisodeReplacementOwner, ReplacedEpisode},
    slot::TransitionSlot,
};
use crate::{
    application::{
        api::ApiError,
        catalog::CatalogResults,
        episode_resolve::EpisodeResolutionResults,
        jobs::{JobId, JobState, JobsApi},
    },
    domain::api_values::DecimalU64,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct EpisodeTransitions {
    pub(super) catalogs: Arc<dyn CatalogResults>,
    pub(super) resolutions: Arc<dyn EpisodeResolutionResults>,
    pub(super) jobs: Arc<dyn JobsApi>,
    pub(super) owner: Arc<dyn EpisodeReplacementOwner>,
    pub(super) slot: Arc<TransitionSlot>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubmittedTransition {
    pub transition_id: DecimalU64,
    pub resolution_job_id: JobId,
    pub engine_replaced: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransitionObservation {
    pub transition_id: DecimalU64,
    pub resolution_job_id: JobId,
    pub attempt: DecimalU64,
    pub job_state: JobState,
    pub engine_replaced: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleasedTransition {
    pub resolution_job_id: JobId,
    pub cancellation_confirmed: bool,
}
impl EpisodeTransitions {
    pub fn new(
        catalogs: Arc<dyn CatalogResults>,
        resolutions: Arc<dyn EpisodeResolutionResults>,
        jobs: Arc<dyn JobsApi>,
        owner: Arc<dyn EpisodeReplacementOwner>,
        slot: Arc<TransitionSlot>,
    ) -> Self {
        Self {
            catalogs,
            resolutions,
            jobs,
            owner,
            slot,
        }
    }
    pub fn begin(
        &self,
        input: BeginAdjacentResolution,
        submission_key: &str,
    ) -> Result<SubmittedTransition, ApiError> {
        let (handle, job_id) = self.slot.begin(|| {
            resolve_job::begin(
                self.catalogs.as_ref(),
                self.resolutions.as_ref(),
                self.jobs.as_ref(),
                self.owner.as_ref(),
                submission_key,
                input,
            )
        })?;
        Ok(SubmittedTransition {
            transition_id: DecimalU64(handle),
            resolution_job_id: job_id,
            engine_replaced: false,
        })
    }
    pub fn observe(&self, handle: DecimalU64) -> Result<TransitionObservation, ApiError> {
        let job = self.slot.observe(handle.0, self.jobs.as_ref())?;
        Ok(TransitionObservation {
            transition_id: handle,
            resolution_job_id: job.job_id,
            attempt: job.attempt,
            job_state: job.state,
            engine_replaced: false,
        })
    }
    pub fn commit(&self, handle: DecimalU64) -> Result<ReplacedEpisode, ApiError> {
        self.slot.commit(
            handle.0,
            self.jobs.as_ref(),
            self.resolutions.as_ref(),
            self.owner.as_ref(),
        )
    }
    pub fn release(&self, handle: DecimalU64) -> Result<ReleasedTransition, ApiError> {
        Ok(ReleasedTransition {
            resolution_job_id: self.slot.release(handle.0)?,
            cancellation_confirmed: false,
        })
    }
}

use super::{
    contracts::{ResolutionChoice, catalog_read_id},
    model::{Active, CatalogModel, Intent, Listing, mismatch},
};
use crate::application::{
    api::ApiError,
    catalog::{CatalogResult, ReadCatalogResult},
    episode_resolve::{EpisodeResolutionInspection, InspectEpisodeResolution},
    jobs::{CancelJob, CancelledJob, JobRecord, JobState, SubmittedJob},
};
use meari::application::plugins::PluginTicket;

impl CatalogModel {
    pub(super) fn requested(&mut self, ticket: &PluginTicket) -> Result<(), ApiError> {
        if self.rpc.is_some() {
            return Err(mismatch("Original catalog RPC still requires acceptance"));
        }
        if ticket.capability() == "jobs.submit" {
            self.staged
                .as_mut()
                .ok_or_else(|| mismatch("Original staged browser input missing"))?
                .ticket = Some(ticket.clone());
        }
        self.rpc = Some(ticket.clone());
        Ok(())
    }
    pub(super) fn accept_submission(
        &mut self,
        ticket: &PluginTicket,
        submitted: SubmittedJob,
    ) -> Result<(), ApiError> {
        self.require_rpc(ticket)?;
        let staged = self
            .staged
            .as_ref()
            .ok_or_else(|| mismatch("Original staged browser input missing"))?;
        if staged.ticket.as_ref() != Some(ticket)
            || submitted.job.input.operation != staged.input.operation
            || submitted.job.input.parameters != staged.input.parameters
        {
            return Err(mismatch(
                "Submitted browser job does not match the original UI request",
            ));
        }
        let staged = self
            .staged
            .take()
            .ok_or_else(|| mismatch("Original staged input lost"))?;
        self.active = Some(Active {
            intent: staged.intent,
            job: submitted.job,
        });
        self.rpc = None;
        self.diagnostic = None;
        self.cancel_requested = false;
        Ok(())
    }
    pub(super) fn accept_job(
        &mut self,
        ticket: &PluginTicket,
        job: JobRecord,
    ) -> Result<(), ApiError> {
        self.require_rpc(ticket)?;
        let active = self
            .active
            .as_mut()
            .ok_or_else(|| mismatch("Original browser job missing"))?;
        if active.job.job_id != job.job_id
            || active.job.attempt != job.attempt
            || active.job.input_sha256 != job.input_sha256
            || active.job.revision > job.revision
        {
            return Err(mismatch(
                "Browser observation belongs to another job, attempt or source",
            ));
        }
        active.job = job;
        self.rpc = None;
        self.diagnostic = None;
        Ok(())
    }
    pub(super) fn cancel_input(&self) -> Result<CancelJob, ApiError> {
        let active = self
            .active
            .as_ref()
            .ok_or_else(|| mismatch("Original browser job missing"))?;
        Ok(CancelJob {
            job_id: active.job.job_id.clone(),
            expected_attempt: active.job.attempt,
        })
    }
    pub(super) fn accept_cancel(
        &mut self,
        ticket: &PluginTicket,
        cancelled: CancelledJob,
    ) -> Result<(), ApiError> {
        self.accept_job(ticket, cancelled.job)?;
        self.cancel_requested = true;
        Ok(())
    }
    pub(super) fn read_input(&self, operation: &str) -> Result<ReadCatalogResult, ApiError> {
        let active = self
            .active
            .as_ref()
            .ok_or_else(|| mismatch("Original browser job missing"))?;
        Ok(ReadCatalogResult {
            result_artifact_sha256: catalog_read_id(&active.job, operation)?,
        })
    }
    pub(super) fn inspect_input(&self) -> Result<InspectEpisodeResolution, ApiError> {
        let active = self
            .active
            .as_ref()
            .ok_or_else(|| mismatch("Original browser job missing"))?;
        Ok(InspectEpisodeResolution {
            resolution_artifact_id: ResolutionChoice::inspector_id(&active.job)?,
        })
    }
    pub(super) fn accept_listing(
        &mut self,
        ticket: &PluginTicket,
        result: CatalogResult,
    ) -> Result<(), ApiError> {
        self.require_rpc(ticket)?;
        let active = self
            .active
            .as_ref()
            .ok_or_else(|| mismatch("Original browser job missing"))?;
        let expected = catalog_read_id(&active.job, &active.job.input.operation)?;
        let artifact = result
            .result_artifact
            .as_ref()
            .ok_or_else(|| mismatch("Read catalog result has no original artifact"))?;
        if artifact.sha256 != expected
            || artifact.operation != active.job.input.operation
            || artifact.source_descriptor_sha256 != active.job.input.source.sha256
        {
            return Err(mismatch(
                "Read catalog artifact differs from the original completed job",
            ));
        }
        match &active.intent {
            Intent::Category { .. } => {
                self.series = Some(Listing {
                    artifact: artifact.clone(),
                    result,
                });
                self.episodes = None;
                self.selected_series = None;
                self.resolution = None;
            }
            Intent::Series(selected) => {
                self.selected_series = Some(selected.clone());
                self.episodes = Some(Listing {
                    artifact: artifact.clone(),
                    result,
                });
                self.resolution = None;
            }
            Intent::Episode(_) => {
                return Err(mismatch("Resolution job cannot supply a catalog listing"));
            }
        }
        self.active = None;
        self.rpc = None;
        self.diagnostic = None;
        Ok(())
    }
    pub(super) fn accept_inspection(
        &mut self,
        ticket: &PluginTicket,
        inspection: EpisodeResolutionInspection,
    ) -> Result<(), ApiError> {
        self.require_rpc(ticket)?;
        let active = self
            .active
            .as_ref()
            .ok_or_else(|| mismatch("Original browser job missing"))?;
        let expected = ResolutionChoice::inspector_id(&active.job)?;
        let Intent::Episode(selected) = &active.intent else {
            return Err(mismatch("Original episode selection missing"));
        };
        if self
            .episodes
            .as_ref()
            .is_none_or(|current| current.artifact.sha256 != selected.list.sha256)
        {
            return Err(mismatch(
                "Selected episode belongs to another original episode result",
            ));
        }
        if inspection.resolution_artifact_id != expected
            || inspection.source_descriptor_sha256 != active.job.input.source.sha256
        {
            return Err(mismatch(
                "Resolution inspection differs from the original selected job",
            ));
        }
        self.resolution = Some(ResolutionChoice {
            selected: selected.clone(),
            job_id: active.job.job_id.clone(),
            attempt: active.job.attempt.0,
            inspection,
        });
        self.previous = self.active.take();
        self.rpc = None;
        self.diagnostic = None;
        Ok(())
    }
    fn require_rpc(&self, ticket: &PluginTicket) -> Result<(), ApiError> {
        if self.rpc.as_ref() == Some(ticket) {
            Ok(())
        } else {
            Err(mismatch("Foreign or stale catalog RPC ticket"))
        }
    }
    pub(super) fn job_terminal(&self) -> bool {
        self.active.as_ref().is_some_and(|active| {
            matches!(
                active.job.state,
                JobState::Completed
                    | JobState::Failed
                    | JobState::Cancelled
                    | JobState::NeedsInput
                    | JobState::Interrupted
            )
        })
    }
    pub(super) fn accepted_failure(
        &mut self,
        ticket: &PluginTicket,
        sdk: Option<ApiError>,
        adapter: meari::application::plugins::PluginError,
    ) -> Result<(), ApiError> {
        self.require_rpc(ticket)?;
        // A failed submission reply does not prove that no durable job was created. Keep
        // the staged original ticket/input and block a new submission rather than replay it.
        self.rpc = None;
        self.diagnostic = sdk;
        self.adapter_error = Some(adapter);
        Ok(())
    }
}

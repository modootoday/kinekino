use crate::{
    application::{
        api::{ApiError, ErrorCode},
        jobs::{CancelJob, CancelledJob, GetJob, JobRecord, JobState},
        playback_session::{
            EpisodePlaybackObservation,
            transition::{
                BeginAdjacentResolution, EpisodeDirection, ReleasedTransition, ReplacedEpisode,
                SubmittedTransition, TransitionObservation,
            },
        },
    },
    domain::api_values::DecimalU64,
};
use meari::application::plugins::PluginTicket;

pub(crate) struct PlayingContext {
    pub engine: DecimalU64,
    pub artifact: String,
    pub catalog_result_sha256: String,
}
#[derive(Clone, Copy)]
pub(crate) enum TransitionRequest {
    Begin(EpisodeDirection),
    Observe,
    Commit,
    CancelJob,
    ReadJob,
    Release,
}
impl TransitionRequest {
    fn command(self) -> &'static str {
        match self {
            Self::Begin(EpisodeDirection::Previous) => "playback.previous",
            Self::Begin(EpisodeDirection::Next) => "playback.next",
            Self::Observe => "playback.transition.observe",
            Self::Commit => "playback.transition.commit",
            Self::CancelJob => "jobs.cancel",
            Self::ReadJob => "jobs.get",
            Self::Release => "playback.transition.release",
        }
    }
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HandleInput {
    pub transition_id: DecimalU64,
}
pub(super) struct Active {
    pub(super) origin: PlayingContext,
    pub(super) submitted: SubmittedTransition,
    pub(super) observation: Option<TransitionObservation>,
    pub(super) job: Option<JobRecord>,
}
#[derive(Default)]
pub(crate) struct TransitionUi {
    pub(super) staged: Option<(TransitionRequest, Option<PlayingContext>)>,
    pub(super) rpc: Option<PluginTicket>,
    pub(super) active: Option<Active>,
    previous: Option<Active>,
    pub replacement: Option<(ReplacedEpisode, String)>,
    pub released: Option<ReleasedTransition>,
    pub diagnostic: Option<ApiError>,
    pub commit_unconfirmed: bool,
    pub(super) cancel_requested: bool,
}
fn mismatch(message: &'static str) -> ApiError {
    ApiError::new(ErrorCode::RevisionConflict, message)
}
impl TransitionUi {
    pub(crate) fn retained_resolution_url(&self) -> Option<&str> {
        let job = self.previous.as_ref()?.job.as_ref()?;
        if job.input.operation != "episodes.resolve" {
            return None;
        }
        job.input.parameters.get("episode_url")?.as_str()
    }
    pub(crate) fn staged_for(&self, command: &str) -> bool {
        self.rpc.is_none()
            && self
                .staged
                .as_ref()
                .is_some_and(|(request, _)| request.command() == command)
    }
    pub(crate) fn owns_rpc(&self, ticket: &PluginTicket) -> bool {
        self.rpc.as_ref() == Some(ticket)
    }
    pub(crate) fn blocks_catalog(&self) -> bool {
        self.active.is_some()
            || self.staged.is_some()
            || self.rpc.is_some()
            || self.commit_unconfirmed
    }
    pub(crate) fn begin_input(
        &mut self,
        origin: PlayingContext,
        actual: &EpisodePlaybackObservation,
        direction: EpisodeDirection,
    ) -> Result<BeginAdjacentResolution, ApiError> {
        if self.blocks_catalog() {
            return Err(mismatch(
                "Original transition still requires observation or release",
            ));
        }
        if actual.engine_instance_id != origin.engine
            || actual.resolution_artifact_id != origin.artifact
        {
            return Err(mismatch("Transition UI belongs to another observed engine"));
        }
        let paused = actual
            .playback
            .paused
            .ok_or_else(|| mismatch("Original pause state has not been observed"))?;
        let volume_percent = actual
            .playback
            .volume_percent
            .ok_or_else(|| mismatch("Original volume has not been observed"))?;
        let input = BeginAdjacentResolution {
            expected_instance_id: origin.engine,
            catalog_result_sha256: origin.catalog_result_sha256.clone(),
            direction,
            paused,
            volume_percent,
            execution_timeout_ms: DecimalU64(60_000),
        };
        self.staged = Some((TransitionRequest::Begin(direction), Some(origin)));
        self.cancel_requested = false;
        Ok(input)
    }
    pub(crate) fn requested(&mut self, ticket: &PluginTicket) -> Result<(), ApiError> {
        if self.rpc.is_some() || self.staged.is_none() {
            return Err(mismatch(
                "Original staged transition request missing or busy",
            ));
        }
        if self.staged.as_ref().map(|(request, _)| request.command()) != Some(ticket.capability()) {
            return Err(mismatch("Original transition request command changed"));
        }
        if ticket.capability() == "playback.transition.commit" {
            self.commit_unconfirmed = true;
        }
        if ticket.capability() == "jobs.cancel" {
            self.cancel_requested = true;
        }
        self.rpc = Some(ticket.clone());
        Ok(())
    }
    fn accepted(&self, ticket: &PluginTicket) -> Result<(), ApiError> {
        if self.rpc.as_ref() != Some(ticket) {
            return Err(mismatch("Foreign transition RPC ticket"));
        }
        Ok(())
    }
    pub(crate) fn submitted(
        &mut self,
        ticket: &PluginTicket,
        submitted: SubmittedTransition,
    ) -> Result<(), ApiError> {
        self.accepted(ticket)?;
        if submitted.engine_replaced || submitted.transition_id.0 == 0 {
            return Err(mismatch("Unexpected transition begin observation"));
        }
        let Some((TransitionRequest::Begin(_), Some(origin))) = self.staged.take() else {
            return Err(mismatch("Original transition begin context missing"));
        };
        self.active = Some(Active {
            origin,
            submitted,
            observation: None,
            job: None,
        });
        self.rpc = None;
        self.diagnostic = None;
        Ok(())
    }
    pub(crate) fn observe(
        &mut self,
        ticket: &PluginTicket,
        value: TransitionObservation,
    ) -> Result<(), ApiError> {
        self.accepted(ticket)?;
        let active = self
            .active
            .as_mut()
            .ok_or_else(|| mismatch("Original transition handle missing"))?;
        if value.transition_id != active.submitted.transition_id
            || value.resolution_job_id != active.submitted.resolution_job_id
            || value.engine_replaced
            || active
                .observation
                .as_ref()
                .is_some_and(|old| old.attempt != value.attempt)
        {
            return Err(mismatch(
                "Transition observation belongs to another original handle/job/attempt",
            ));
        }
        active.observation = Some(value);
        self.staged = None;
        self.rpc = None;
        self.diagnostic = None;
        Ok(())
    }
    pub(crate) fn job_input(
        &mut self,
        cancel: bool,
    ) -> Result<(GetJob, Option<CancelJob>), ApiError> {
        if self.rpc.is_some() || self.staged.is_some() || self.commit_unconfirmed {
            return Err(mismatch("Original transition RPC pending"));
        }
        let active = self
            .active
            .as_ref()
            .ok_or_else(|| mismatch("Original transition job missing"))?;
        let observation = active
            .observation
            .as_ref()
            .ok_or_else(|| mismatch("Observe the original transition attempt first"))?;
        let get = GetJob {
            job_id: active.submitted.resolution_job_id.clone(),
        };
        let cancel_input = cancel.then(|| CancelJob {
            job_id: get.job_id.clone(),
            expected_attempt: observation.attempt,
        });
        self.staged = Some((
            if cancel {
                TransitionRequest::CancelJob
            } else {
                TransitionRequest::ReadJob
            },
            None,
        ));
        Ok((get, cancel_input))
    }
    pub(crate) fn accept_job(
        &mut self,
        ticket: &PluginTicket,
        job: JobRecord,
    ) -> Result<(), ApiError> {
        self.accepted(ticket)?;
        let active = self
            .active
            .as_mut()
            .ok_or_else(|| mismatch("Original transition job missing"))?;
        let observation = active
            .observation
            .as_ref()
            .ok_or_else(|| mismatch("Original transition attempt missing"))?;
        if job.job_id != active.submitted.resolution_job_id
            || job.attempt != observation.attempt
            || active.job.as_ref().is_some_and(|old| {
                old.input_sha256 != job.input_sha256 || old.revision > job.revision
            })
        {
            return Err(mismatch("Foreign or stale transition job observation"));
        }
        active.job = Some(job);
        self.staged = None;
        self.rpc = None;
        Ok(())
    }
    pub(crate) fn accept_cancel(
        &mut self,
        ticket: &PluginTicket,
        reply: CancelledJob,
    ) -> Result<(), ApiError> {
        self.accept_job(ticket, reply.job)
    }
    pub(crate) fn handle_input(
        &mut self,
        request: TransitionRequest,
    ) -> Result<HandleInput, ApiError> {
        if self.rpc.is_some() || self.staged.is_some() || self.commit_unconfirmed {
            return Err(mismatch(
                "Original transition RPC or commit still requires observation",
            ));
        }
        let active = self
            .active
            .as_ref()
            .ok_or_else(|| mismatch("Original transition handle missing"))?;
        match request {
            TransitionRequest::Observe => (),
            TransitionRequest::Commit
                if !self.cancel_requested
                    && active
                        .job
                        .as_ref()
                        .is_some_and(|job| job.state == JobState::Completed)
                    && active
                        .observation
                        .as_ref()
                        .is_some_and(|value| value.job_state == JobState::Completed) => {}
            TransitionRequest::Release
                if active.job.as_ref().is_some_and(|job| job.state.terminal()) => {}
            _ => {
                return Err(mismatch(
                    "Explicit commit requires completed resolve; release requires the original terminal job observation",
                ));
            }
        }
        let input = HandleInput {
            transition_id: active.submitted.transition_id,
        };
        self.staged = Some((request, None));
        Ok(input)
    }
    pub(crate) fn committed(
        &mut self,
        ticket: &PluginTicket,
        reply: ReplacedEpisode,
    ) -> Result<(), ApiError> {
        self.accepted(ticket)?;
        let active = self
            .active
            .as_ref()
            .ok_or_else(|| mismatch("Original transition owner missing"))?;
        if !matches!(self.staged, Some((TransitionRequest::Commit, _)))
            || reply.previous.engine_instance_id != active.origin.engine
            || !reply.previous.engine_exited
            || !reply.previous.ipc_worker_exited
            || !reply.previous.stdout_worker_exited
            || !reply.previous.stderr_worker_exited
        {
            return Err(mismatch(
                "Replacement receipt differs from the original engine or its explicit worker exits",
            ));
        }
        self.replacement = Some((reply, active.origin.catalog_result_sha256.clone()));
        self.previous = self.active.take();
        self.staged = None;
        self.rpc = None;
        self.commit_unconfirmed = false;
        Ok(()) // current is process creation only; UI must query actual engine/media separately
    }
    pub(crate) fn release(
        &mut self,
        ticket: &PluginTicket,
        reply: ReleasedTransition,
    ) -> Result<(), ApiError> {
        self.accepted(ticket)?;
        let active = self
            .active
            .as_ref()
            .ok_or_else(|| mismatch("Original transition owner missing"))?;
        if !matches!(self.staged, Some((TransitionRequest::Release, _)))
            || reply.resolution_job_id != active.submitted.resolution_job_id
            || reply.cancellation_confirmed
        {
            return Err(mismatch(
                "Transition release reply differs from the original job or contract",
            ));
        }
        self.released = Some(reply);
        self.previous = self.active.take();
        self.staged = None;
        self.rpc = None;
        Ok(()) // release is not a cancellation receipt; the original job record remains separate
    }
    pub(crate) fn failed(
        &mut self,
        ticket: &PluginTicket,
        original: ApiError,
    ) -> Result<(), ApiError> {
        self.accepted(ticket)?;
        self.diagnostic = Some(original);
        if !matches!(
            self.staged,
            Some((TransitionRequest::Begin(_) | TransitionRequest::Commit, _))
        ) {
            self.staged = None;
        }
        self.rpc = None;
        Ok(()) // uncertain begin/commit is never automatically retried
    }
}

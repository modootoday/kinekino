use super::super::episode_transition::{HandleInput, PlayingContext, TransitionRequest};
use super::CatalogConsumer;
use crate::application::{
    api::{ApiError, ErrorCode},
    jobs::{CancelJob, GetJob},
    playback_session::{
        EpisodePlaybackObservation,
        transition::{BeginAdjacentResolution, EpisodeDirection},
    },
};
fn unavailable() -> ApiError {
    ApiError::new(
        ErrorCode::NeedsInput,
        "Original playing source or idle transition UI is not observed",
    )
}
impl CatalogConsumer {
    pub(crate) fn transition_props(
        &self,
        actual: Option<&EpisodePlaybackObservation>,
        enabled: bool,
        admitted: bool,
    ) -> std::collections::BTreeMap<String, meari::domain::Value> {
        let ready = enabled
            && admitted
            && self.transition_available
            && self.idle_catalog_rpc().is_ok()
            && actual.is_some_and(|actual| self.playing_context(actual).is_ok());
        let view = self.transition.view(ready);
        let mut props = view.props();
        props.insert(
            "transitionNavigationDisabled".into(),
            meari::domain::Value::Bool(!ready || self.transition.blocks_catalog()),
        );
        if !enabled {
            props.insert(
                "transitionCancelDisabled".into(),
                meari::domain::Value::Bool(true),
            );
        }
        props
    }
    pub(crate) fn automatic_transition_step(
        &self,
        actual: &EpisodePlaybackObservation,
        enabled: bool,
    ) -> Option<super::super::progression::Continuation> {
        if self.model.rpc.is_some() || self.model.adapter_error.is_some() {
            return None;
        }
        self.transition.automatic_step(actual, enabled)
    }
    pub(crate) fn transition_blocks_controls(&self) -> bool {
        self.transition.blocks_catalog()
    }
    pub(super) fn retain_transition(
        &mut self,
        event: &meari::application::plugins::PluginEvent,
    ) -> Result<(), ApiError> {
        use crate::application::{
            jobs::{CancelledJob, JobRecord},
            playback_session::transition::{
                ReleasedTransition, ReplacedEpisode, SubmittedTransition, TransitionObservation,
            },
        };
        use meari::application::plugins::PluginEventKind;
        let command = event.ticket.capability();
        let manifest = super::super::codec::manifest();
        let descriptor = manifest.capabilities.get(command).ok_or_else(unavailable)?;
        let payload = match &event.event {
            PluginEventKind::Completed(payload) => payload,
            PluginEventKind::Failed(error) => {
                let original =
                    super::super::codec::sdk_error(command, error).ok_or_else(unavailable)?;
                return self.transition.failed(&event.ticket, original);
            }
            _ => return Err(unavailable()),
        };
        let decode_error = |error: meari::application::plugins::PluginError| {
            ApiError::new(ErrorCode::InvalidArgument, error.message)
        };
        match command {
            "playback.previous" | "playback.next" => self.transition.submitted(
                &event.ticket,
                payload
                    .decode::<SubmittedTransition>(&descriptor.reply_schema)
                    .map_err(decode_error)?,
            ),
            "playback.transition.observe" => self.transition.observe(
                &event.ticket,
                payload
                    .decode::<TransitionObservation>(&descriptor.reply_schema)
                    .map_err(decode_error)?,
            ),
            "playback.transition.commit" => {
                self.transition.committed(
                    &event.ticket,
                    payload
                        .decode::<ReplacedEpisode>(&descriptor.reply_schema)
                        .map_err(decode_error)?,
                )?;
                self.engine_queries = 0;
                Ok(())
            }
            "playback.transition.release" => self.transition.release(
                &event.ticket,
                payload
                    .decode::<ReleasedTransition>(&descriptor.reply_schema)
                    .map_err(decode_error)?,
            ),
            "jobs.get" => self.transition.accept_job(
                &event.ticket,
                payload
                    .decode::<JobRecord>(&descriptor.reply_schema)
                    .map_err(decode_error)?,
            ),
            "jobs.cancel" => self.transition.accept_cancel(
                &event.ticket,
                payload
                    .decode::<CancelledJob>(&descriptor.reply_schema)
                    .map_err(decode_error)?,
            ),
            _ => Err(unavailable()),
        }
    }
    pub(crate) fn require_catalog_idle(&self) -> Result<(), ApiError> {
        if self.transition.blocks_catalog() {
            Err(unavailable())
        } else {
            Ok(())
        }
    }
    fn idle_catalog_rpc(&self) -> Result<(), ApiError> {
        if self.model.rpc.is_some()
            || self.model.active.is_some()
            || self
                .model
                .staged
                .as_ref()
                .is_some_and(|staged| staged.ticket.is_some())
        {
            Err(unavailable())
        } else {
            Ok(())
        }
    }
    fn playing_context(
        &self,
        actual: &EpisodePlaybackObservation,
    ) -> Result<PlayingContext, ApiError> {
        if let Some((replacement, catalog)) = &self.transition.replacement
            && replacement.current.engine_instance_id == actual.engine_instance_id
            && replacement.current.resolution_artifact_id == actual.resolution_artifact_id
        {
            return Ok(PlayingContext {
                engine: actual.engine_instance_id,
                artifact: actual.resolution_artifact_id.clone(),
                catalog_result_sha256: catalog.clone(),
            });
        }
        let original = self
            .opened
            .as_ref()
            .filter(|choice| {
                choice.opened.engine_instance_id == actual.engine_instance_id
                    && choice.opened.resolution_artifact_id == actual.resolution_artifact_id
            })
            .ok_or_else(unavailable)?;
        Ok(PlayingContext {
            engine: actual.engine_instance_id,
            artifact: actual.resolution_artifact_id.clone(),
            catalog_result_sha256: original.selected.list.sha256.clone(),
        })
    }
    pub(crate) fn stage_adjacent(
        &mut self,
        actual: &EpisodePlaybackObservation,
        direction: EpisodeDirection,
    ) -> Result<BeginAdjacentResolution, ApiError> {
        self.idle_catalog_rpc()?;
        if !self.transition_available || !self.automatic_enabled {
            return Err(unavailable());
        }
        let origin = self.playing_context(actual)?;
        self.transition.begin_input(origin, actual, direction)
    }
    pub(crate) fn stage_transition_handle(
        &mut self,
        request: TransitionRequest,
    ) -> Result<HandleInput, ApiError> {
        self.idle_catalog_rpc()?;
        self.transition.handle_input(request)
    }
    pub(crate) fn stage_transition_cancel(&mut self) -> Result<CancelJob, ApiError> {
        self.idle_catalog_rpc()?;
        self.transition.job_input(true)?.1.ok_or_else(unavailable)
    }
    pub(crate) fn stage_transition_job(&mut self) -> Result<GetJob, ApiError> {
        self.idle_catalog_rpc()?;
        Ok(self.transition.job_input(false)?.0)
    }
}

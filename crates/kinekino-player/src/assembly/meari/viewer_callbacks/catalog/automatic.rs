use super::{CatalogConsumer, model::Intent};
use crate::{
    application::jobs::JobState,
    assembly::meari::viewer_callbacks::progression::{Action, Continuation},
};

impl CatalogConsumer {
    pub(crate) fn expected_engine_matches(
        &self,
        actual: &crate::application::playback_session::EpisodePlaybackObservation,
    ) -> bool {
        let expected = self
            .transition
            .replacement
            .as_ref()
            .map(|(replacement, _)| &replacement.current)
            .or_else(|| self.opened.as_ref().map(|choice| &choice.opened));
        expected.is_some_and(|expected| {
            actual.engine_instance_id == expected.engine_instance_id
                && actual.resolution_artifact_id == expected.resolution_artifact_id
                && actual.playback.connected
        })
    }
    pub(crate) fn engine_observation_step(
        &self,
        actual: Option<&crate::application::playback_session::EpisodePlaybackObservation>,
    ) -> Option<Continuation> {
        if self.engine_queries >= 120 {
            return None;
        }
        let (engine, artifact) = if let Some((replacement, _)) = &self.transition.replacement {
            (
                replacement.current.engine_instance_id,
                &replacement.current.resolution_artifact_id,
            )
        } else {
            let opened = &self.opened.as_ref()?.opened;
            (opened.engine_instance_id, &opened.resolution_artifact_id)
        };
        if actual.is_some_and(|actual| {
            actual.engine_instance_id == engine
                && actual.resolution_artifact_id == *artifact
                && actual.playback.connected
        }) {
            return None;
        }
        Some(Continuation {
            action: Action::ReadEngine,
            operation: format!("{}:{artifact}", engine.0),
        })
    }
    /// Only a host with its original document timer may enable the final automatic flow.
    pub(crate) fn automatic_catalog_step(&self, enabled: bool) -> Option<Continuation> {
        if !enabled
            || !self.available
            || self.transition.blocks_catalog()
            || self.open_unconfirmed
            || self.opened.is_some()
            || self.model.rpc.is_some()
            || self.model.diagnostic.is_some()
            || self.model.adapter_error.is_some()
            || self
                .model
                .staged
                .as_ref()
                .is_some_and(|staged| staged.ticket.is_some())
        {
            return None;
        }
        if let Some(active) = &self.model.active {
            let action = if !active.job.state.terminal() {
                Action::PollCatalog
            } else if active.job.state == JobState::Completed && !self.model.cancel_requested {
                match &active.intent {
                    Intent::Category { .. } => Action::ReadSeries,
                    Intent::Series(_) => Action::ReadEpisodes,
                    Intent::Episode(_) => Action::InspectEpisode,
                }
            } else {
                return None; // Original terminal failure/cancellation remains visible for user action.
            };
            return Some(Continuation {
                action,
                operation: format!(
                    "{}:{}:{}",
                    active.job.job_id.as_str(),
                    active.job.attempt.0,
                    active.job.revision.0
                ),
            });
        }
        // Original inspection already verifies selected episode, job, attempt and source joins.
        let choice = self.resolution().ok()?;
        Some(Continuation {
            action: Action::PlayEpisode,
            operation: format!(
                "{}:{}:{}",
                choice.job_id.as_str(),
                choice.attempt,
                choice.inspection.resolution_artifact_id
            ),
        })
    }
}

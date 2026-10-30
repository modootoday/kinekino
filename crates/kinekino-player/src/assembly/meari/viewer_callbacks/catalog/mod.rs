//! UI-owned source/row/job joins; browser, cancellation and playback policy stay in the SDK.
mod automatic;
mod callbacks;
pub(super) mod codec;
mod contracts;
mod episode_identity;
mod model;
mod progress;
#[cfg(test)]
mod tests;
mod transition;
mod view;
use crate::application::{
    api::{ApiError, ErrorCode},
    catalog::CatalogResult,
    episode_resolve::EpisodeResolutionInspection,
    jobs::{CancelledJob, JobRecord, SubmittedJob},
    playback_session::{EpisodePlaybackObservation, OpenedEpisodePlayback},
};
pub(super) use callbacks::bind;
use contracts::ResolutionChoice;
use meari::application::plugins::*;
use model::CatalogModel;

struct OpenedChoice {
    opened: OpenedEpisodePlayback,
    selected: contracts::SelectedEpisode,
}
#[derive(Default)]
pub(crate) struct CatalogConsumer {
    pub(crate) automatic_enabled: bool,
    pub(crate) engine_queries: u16,
    transition: super::episode_transition::TransitionUi,
    transition_available: bool,
    model: CatalogModel,
    opened: Option<OpenedChoice>,
    open_unconfirmed: bool,
    available: bool,
}
pub(crate) fn is_command(command: &str) -> bool {
    codec::COMMANDS.contains(&command)
        || super::episode_transition::codec::COMMANDS.contains(&command)
}
impl CatalogConsumer {
    pub(crate) fn capabilities(&mut self, event: &PluginEvent, ack: &Result<(), PluginError>) {
        self.available = false;
        self.transition_available = false;
        if ack.is_err() {
            return;
        }
        let PluginEventKind::Completed(payload) = &event.event else {
            return;
        };
        let manifest = super::codec::manifest();
        let Some(descriptor) = manifest.capabilities.get("capabilities") else {
            return;
        };
        let Ok(reply) = payload.decode::<super::codec::CapabilitiesReply>(&descriptor.reply_schema)
        else {
            return;
        };
        self.transition_available = reply
            .sections
            .get("episode_transitions")
            .and_then(|value| value.get("available"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        #[derive(serde::Deserialize)]
        struct JobsView {
            execution_host_available: bool,
            operations: Vec<String>,
        }
        let Some(value) = reply.sections.get("jobs") else {
            return;
        };
        let Ok(jobs) = serde_json::from_value::<JobsView>(value.clone()) else {
            return;
        };
        self.available = jobs.execution_host_available
            && ["catalog.search", "episodes.list", "episodes.resolve"]
                .iter()
                .all(|op| jobs.operations.iter().any(|item| item == op));
    }
    pub(crate) fn requested(&mut self, ticket: &PluginTicket) -> Result<(), ApiError> {
        if self.transition.staged_for(ticket.capability()) {
            return self.transition.requested(ticket);
        }
        if super::episode_transition::codec::COMMANDS.contains(&ticket.capability()) {
            return Err(model::mismatch(
                "Original transition stage was not retained",
            ));
        }
        self.require_catalog_idle()?;
        if ticket.capability() == "playback.open" {
            self.open_unconfirmed = true;
        }
        self.model.requested(ticket)
    }
    pub(crate) fn retain(&mut self, event: &PluginEvent, ack: &Result<(), PluginError>) {
        if let Err(error) = ack {
            self.model.adapter_error = Some(error.clone());
            return;
        }
        if self.transition.owns_rpc(&event.ticket) {
            if let Err(error) = self.retain_transition(event) {
                self.transition.diagnostic = Some(error);
            }
            return;
        }
        let manifest = super::codec::manifest();
        let result = (|| {
            let command = event.ticket.capability();
            let descriptor = manifest
                .capabilities
                .get(command)
                .ok_or_else(|| model::mismatch("Original browser capability missing"))?;
            let payload = match &event.event {
                PluginEventKind::Completed(payload) => payload,
                PluginEventKind::Failed(error) => {
                    return self.model.accepted_failure(
                        &event.ticket,
                        super::codec::sdk_error(command, error),
                        error.clone(),
                    );
                }
                _ => return Err(model::mismatch("Unexpected browser nonterminal event")),
            };
            let invalid =
                |error: PluginError| ApiError::new(ErrorCode::InvalidArgument, error.message);
            match command {
                "jobs.submit" => self.model.accept_submission(
                    &event.ticket,
                    payload
                        .decode::<SubmittedJob>(&descriptor.reply_schema)
                        .map_err(invalid)?,
                ),
                "jobs.get" => self.model.accept_job(
                    &event.ticket,
                    payload
                        .decode::<JobRecord>(&descriptor.reply_schema)
                        .map_err(invalid)?,
                ),
                "jobs.cancel" => self.model.accept_cancel(
                    &event.ticket,
                    payload
                        .decode::<CancelledJob>(&descriptor.reply_schema)
                        .map_err(invalid)?,
                ),
                "catalog.search" | "episodes.list" => self.model.accept_listing(
                    &event.ticket,
                    payload
                        .decode::<CatalogResult>(&descriptor.reply_schema)
                        .map_err(invalid)?,
                ),
                "episodes.inspect_resolution" => self.model.accept_inspection(
                    &event.ticket,
                    payload
                        .decode::<EpisodeResolutionInspection>(&descriptor.reply_schema)
                        .map_err(invalid)?,
                ),
                "playback.open" => {
                    let opened: OpenedEpisodePlayback =
                        payload.decode(&descriptor.reply_schema).map_err(invalid)?;
                    let choice =
                        self.model.resolution.as_ref().ok_or_else(|| {
                            model::mismatch("Original selected resolution missing")
                        })?;
                    if self.model.rpc.as_ref() != Some(&event.ticket)
                        || opened.resolution_artifact_id != choice.inspection.resolution_artifact_id
                    {
                        return Err(model::mismatch(
                            "Opened engine differs from the original selected resolution",
                        ));
                    }
                    self.opened = Some(OpenedChoice {
                        opened,
                        selected: choice.selected.clone(),
                    });
                    self.engine_queries = 0;
                    self.open_unconfirmed = false;
                    self.model.rpc = None;
                    self.model.diagnostic = None;
                    Ok(())
                }
                _ => Err(model::mismatch("Unexpected browser command")),
            }
        })();
        if let Err(error) = result {
            self.model.diagnostic = Some(error);
        }
    }
    pub(crate) fn fault(&mut self, error: &PluginError) {
        self.model.adapter_error = Some(error.clone());
    }
    pub(crate) fn release_terminal(&mut self) -> Result<(), ApiError> {
        self.model.release_terminal()
    }
    pub(crate) fn current_titles(
        &self,
        observation: &EpisodePlaybackObservation,
    ) -> Option<(&str, &str)> {
        if let Some((replacement, original_catalog)) = &self.transition.replacement
            && replacement.current.engine_instance_id == observation.engine_instance_id
            && replacement.current.resolution_artifact_id == observation.resolution_artifact_id
        {
            let original = self.opened.as_ref()?;
            let listing = self
                .model
                .episodes
                .as_ref()
                .filter(|listing| &listing.artifact.sha256 == original_catalog)?;
            let target = self.transition.retained_resolution_url()?;
            let row = listing
                .result
                .entries
                .iter()
                .find(|row| row.url == target)?;
            return Some((
                original.selected.series.row.title.as_str(),
                row.title.as_str(),
            ));
        }
        let choice = self.opened.as_ref()?;
        (choice.opened.engine_instance_id == observation.engine_instance_id
            && choice.opened.resolution_artifact_id == observation.resolution_artifact_id)
            .then_some((
                choice.selected.series.row.title.as_str(),
                choice.selected.row.title.as_str(),
            ))
    }
    fn resolution(&self) -> Result<&ResolutionChoice, ApiError> {
        self.require_catalog_idle()?;
        if self.open_unconfirmed || self.opened.is_some() {
            return Err(model::mismatch(
                "Original open is pending or an episode already exists; use the shared transition API",
            ));
        }
        if self.model.rpc.is_some()
            || self.model.active.is_some()
            || self
                .model
                .staged
                .as_ref()
                .is_some_and(|s| s.ticket.is_some())
        {
            return Err(model::mismatch(
                "Original browser RPC still requires acceptance",
            ));
        }
        let choice = self.model.resolution.as_ref().ok_or_else(|| {
            model::mismatch("Select and inspect an original episode resolution first")
        })?;
        if self.model.previous.as_ref().is_none_or(|original| {
            original.job.job_id != choice.job_id || original.job.attempt.0 != choice.attempt
        }) {
            return Err(model::mismatch(
                "Original resolution job/attempt was not retained",
            ));
        }
        Ok(choice)
    }
}

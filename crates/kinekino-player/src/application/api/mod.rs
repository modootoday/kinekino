mod asset_artifacts;
mod asset_projects;
mod asset_sources;
mod audio_handoff;
pub mod capabilities;
mod capture_mutation;
mod card_projects;
mod catalog;
mod catalog_raw_schema;
mod catalog_response_schema;
pub mod client;
mod clip_projects;
mod clip_sources;
pub mod commands;
mod episode_resolution_schema;
mod episode_transition;
mod episode_transition_schema;
pub mod errors;
pub mod events;
mod job_batches;
mod job_schema;
mod jobs;
mod native_response_schema;
mod native_session;
mod project_schema;
mod projects;
pub mod queries;
pub(crate) mod schema;
mod snapshot_export;
#[cfg(test)]
mod tests;

use crate::application::frame_capture::{FrameCapturePort, UnavailableFrameCapture};
use crate::application::model_review::{ReviewStore, UnavailableReviewStore};
use crate::application::playback_control::{PlaybackControlPort, UnavailablePlaybackControl};
use crate::application::playback_status::{PlaybackStatusPort, UnavailablePlaybackStatus};
use crate::application::subtitles::storage::{SubtitleStorage, UnavailableStorage};
pub use client::{
    ApplicationClient, ApplicationClientHandle, AssetClient, CardClient, ClipClient, EpisodeClient,
    EpisodeTransitionClient, JobBatchClient, JobClient, PlaybackClient, ProjectClient,
    ProposalClient, ReviewClient, SkipSegmentsClient,
};
pub use commands::{Request, RequestContext};
pub use errors::{ApiError, ErrorCode};
use serde::Serialize;
use serde_json::Value;

pub const API_VERSION: &str = "1";

#[derive(Debug, Serialize)]
pub struct Response {
    pub api_version: &'static str,
    pub request_id: String,
    pub ok: bool,
    pub data: Option<Value>,
    pub error: Option<ApiError>,
}

impl Response {
    pub fn failure(request_id: String, mut error: ApiError) -> Self {
        let request_id = if commands::valid_request_id(&request_id) {
            request_id
        } else {
            "cli".into()
        };
        error.request_id = request_id.clone();
        Self {
            api_version: API_VERSION,
            request_id,
            ok: false,
            data: None,
            error: Some(error),
        }
    }
}

pub struct PlayerSdk<
    S = UnavailableStorage,
    D = UnavailableFrameCapture,
    P = UnavailablePlaybackStatus,
    C = UnavailablePlaybackControl,
    R = UnavailableReviewStore,
> {
    storage: S,
    frame_capture: D,
    snapshot_export: Option<std::sync::Arc<snapshot_export::SnapshotPort>>,
    card_export:
        Option<std::sync::Arc<dyn crate::application::proposals::card_export::CardImagePort>>,
    skip_host: std::sync::Arc<dyn crate::application::skip_segments::host::SkipHostApi>,
    playback_status: P,
    playback_control: C,
    review_store: R,
    review_source:
        Option<std::sync::Arc<dyn crate::application::review_source::api::ReviewSourceApi>>,
    review_playback: Option<
        std::sync::Arc<crate::application::review_playback::sessions::ReviewPlaybackSessions>,
    >,
    proposal_store: std::sync::Arc<dyn crate::application::proposals::ProposalStore>,
    project_store: std::sync::Arc<dyn crate::application::projects::ProjectStore>,
    card_project_store:
        Option<std::sync::Arc<dyn crate::application::card_projects::CardProjectStore>>,
    asset_artifact_inspector:
        Option<std::sync::Arc<dyn crate::application::asset_artifacts::AssetArtifactInspector>>,
    asset_source_archives: Option<
        std::sync::Arc<dyn crate::application::asset_source_archives::AssetSourceArchivePort>,
    >,
    asset_project_store:
        Option<std::sync::Arc<dyn crate::application::asset_projects::AssetProjectStore>>,
    clip_project_store:
        Option<std::sync::Arc<dyn crate::application::clip_projects::ClipProjectStore>>,
    clip_source_inspector:
        Option<std::sync::Arc<dyn crate::application::clip_render::ClipSourceInspector>>,
    card_templates:
        Option<std::sync::Arc<dyn crate::application::card_bundle::CardTemplateCatalogPort>>,
    clip_presets:
        Option<std::sync::Arc<dyn crate::application::clip_render::presets::ClipPresetCatalogPort>>,
    jobs: std::sync::Arc<dyn crate::application::jobs::JobsApi>,
    job_batches: Option<std::sync::Arc<dyn crate::application::job_batches::service::BatchApi>>,
    catalog: std::sync::Arc<dyn crate::application::catalog::CatalogApi>,
    model_catalog: Option<std::sync::Arc<dyn crate::application::model_catalog::ModelCatalogPort>>,
    speaker_annotations:
        Option<std::sync::Arc<dyn crate::application::speaker_annotations::SpeakerDocumentFiles>>,
    episode_transitions: Option<
        std::sync::Arc<crate::application::playback_session::transition::EpisodeTransitions>,
    >,
    episode_playback:
        Option<std::sync::Arc<crate::application::playback_session::EpisodePlaybackSessions>>,
}

impl Default for PlayerSdk<UnavailableStorage> {
    fn default() -> Self {
        Self::new(UnavailableStorage)
    }
}

impl<S: SubtitleStorage> PlayerSdk<S> {
    pub fn new(storage: S) -> Self {
        Self {
            storage,
            frame_capture: UnavailableFrameCapture,
            snapshot_export: None,
            card_export: None,
            card_project_store: None,
            asset_project_store: None,
            asset_source_archives: None,
            asset_artifact_inspector: None,
            clip_project_store: None,
            clip_source_inspector: None,
            card_templates: None,
            clip_presets: None,
            skip_host: std::sync::Arc::new(
                crate::application::skip_segments::host::UnavailableSkipHost,
            ),
            playback_status: UnavailablePlaybackStatus,
            playback_control: UnavailablePlaybackControl,
            review_store: UnavailableReviewStore,
            review_source: None,
            review_playback: None,
            jobs: std::sync::Arc::new(crate::application::jobs::UnavailableJobs),
            job_batches: None,
            catalog: std::sync::Arc::new(crate::application::catalog::UnavailableCatalog),
            model_catalog: None,
            speaker_annotations: None,
            episode_playback: None,
            episode_transitions: None,
            project_store: std::sync::Arc::new(
                crate::application::projects::UnavailableProjectStore,
            ),
            proposal_store: std::sync::Arc::new(
                crate::application::proposals::UnavailableProposalStore,
            ),
        }
    }
}

impl<
    S: SubtitleStorage,
    D: FrameCapturePort,
    P: PlaybackStatusPort,
    C: PlaybackControlPort,
    R: ReviewStore,
> PlayerSdk<S, D, P, C, R>
{
    pub fn with_frame_capture<F: FrameCapturePort>(
        self,
        frame_capture: F,
    ) -> PlayerSdk<S, F, P, C, R> {
        PlayerSdk {
            storage: self.storage,
            frame_capture,
            snapshot_export: self.snapshot_export,
            card_export: self.card_export,
            skip_host: self.skip_host,
            playback_status: self.playback_status,
            playback_control: self.playback_control,
            review_store: self.review_store,
            review_source: self.review_source,
            proposal_store: self.proposal_store,
            review_playback: self.review_playback,
            project_store: self.project_store,
            card_project_store: self.card_project_store,
            asset_project_store: self.asset_project_store,
            asset_source_archives: self.asset_source_archives,
            asset_artifact_inspector: self.asset_artifact_inspector,
            clip_project_store: self.clip_project_store,
            clip_source_inspector: self.clip_source_inspector,
            card_templates: self.card_templates,
            clip_presets: self.clip_presets,
            jobs: self.jobs,
            job_batches: self.job_batches,
            catalog: self.catalog,
            model_catalog: self.model_catalog,
            speaker_annotations: self.speaker_annotations,
            episode_playback: self.episode_playback,
            episode_transitions: self.episode_transitions,
        }
    }
    pub fn with_playback_status<F: PlaybackStatusPort>(
        self,
        playback_status: F,
    ) -> PlayerSdk<S, D, F, C, R> {
        PlayerSdk {
            storage: self.storage,
            frame_capture: self.frame_capture,
            snapshot_export: self.snapshot_export,
            card_export: self.card_export,
            skip_host: self.skip_host,
            playback_status,
            playback_control: self.playback_control,
            review_store: self.review_store,
            review_source: self.review_source,
            proposal_store: self.proposal_store,
            review_playback: self.review_playback,
            project_store: self.project_store,
            card_project_store: self.card_project_store,
            asset_project_store: self.asset_project_store,
            asset_source_archives: self.asset_source_archives,
            asset_artifact_inspector: self.asset_artifact_inspector,
            clip_project_store: self.clip_project_store,
            clip_source_inspector: self.clip_source_inspector,
            card_templates: self.card_templates,
            clip_presets: self.clip_presets,
            jobs: self.jobs,
            job_batches: self.job_batches,
            catalog: self.catalog,
            model_catalog: self.model_catalog,
            speaker_annotations: self.speaker_annotations,
            episode_playback: self.episode_playback,
            episode_transitions: self.episode_transitions,
        }
    }
    pub fn with_playback_control<F: PlaybackControlPort>(
        self,
        playback_control: F,
    ) -> PlayerSdk<S, D, P, F, R> {
        PlayerSdk {
            storage: self.storage,
            frame_capture: self.frame_capture,
            snapshot_export: self.snapshot_export,
            card_export: self.card_export,
            skip_host: self.skip_host,
            playback_status: self.playback_status,
            playback_control,
            review_store: self.review_store,
            review_source: self.review_source,
            proposal_store: self.proposal_store,
            review_playback: self.review_playback,
            project_store: self.project_store,
            card_project_store: self.card_project_store,
            asset_project_store: self.asset_project_store,
            asset_source_archives: self.asset_source_archives,
            asset_artifact_inspector: self.asset_artifact_inspector,
            clip_project_store: self.clip_project_store,
            clip_source_inspector: self.clip_source_inspector,
            card_templates: self.card_templates,
            clip_presets: self.clip_presets,
            jobs: self.jobs,
            job_batches: self.job_batches,
            catalog: self.catalog,
            model_catalog: self.model_catalog,
            speaker_annotations: self.speaker_annotations,
            episode_playback: self.episode_playback,
            episode_transitions: self.episode_transitions,
        }
    }
    pub fn with_review_store<F: ReviewStore>(self, review_store: F) -> PlayerSdk<S, D, P, C, F> {
        PlayerSdk {
            storage: self.storage,
            frame_capture: self.frame_capture,
            snapshot_export: self.snapshot_export,
            card_export: self.card_export,
            skip_host: self.skip_host,
            playback_status: self.playback_status,
            playback_control: self.playback_control,
            review_store,
            review_source: self.review_source,
            proposal_store: self.proposal_store,
            review_playback: self.review_playback,
            project_store: self.project_store,
            card_project_store: self.card_project_store,
            asset_project_store: self.asset_project_store,
            asset_source_archives: self.asset_source_archives,
            asset_artifact_inspector: self.asset_artifact_inspector,
            clip_project_store: self.clip_project_store,
            clip_source_inspector: self.clip_source_inspector,
            card_templates: self.card_templates,
            clip_presets: self.clip_presets,
            jobs: self.jobs,
            job_batches: self.job_batches,
            catalog: self.catalog,
            model_catalog: self.model_catalog,
            speaker_annotations: self.speaker_annotations,
            episode_playback: self.episode_playback,
            episode_transitions: self.episode_transitions,
        }
    }
    pub fn with_review_source(
        mut self,
        api: impl crate::application::review_source::api::ReviewSourceApi + 'static,
    ) -> Self {
        self.review_source = Some(std::sync::Arc::new(api));
        self
    }
    pub fn with_review_playback_sessions(
        mut self,
        sessions: std::sync::Arc<
            crate::application::review_playback::sessions::ReviewPlaybackSessions,
        >,
    ) -> Self {
        self.review_playback = Some(sessions);
        self
    }
    pub fn with_proposal_store(
        mut self,
        store: impl crate::application::proposals::ProposalStore + 'static,
    ) -> Self {
        self.proposal_store = std::sync::Arc::new(store);
        self
    }
    pub fn with_card_image_export(
        mut self,
        port: impl crate::application::proposals::card_export::CardImagePort + 'static,
    ) -> Self {
        self.card_export = Some(std::sync::Arc::new(port));
        self
    }
    pub fn with_skip_host(
        mut self,
        host: std::sync::Arc<dyn crate::application::skip_segments::host::SkipHostApi>,
    ) -> Self {
        self.skip_host = host;
        self
    }
    pub fn with_shared_episode_playback(
        mut self,
        sessions: std::sync::Arc<crate::application::playback_session::EpisodePlaybackSessions>,
    ) -> Self {
        self.episode_playback = Some(sessions);
        self
    }
    pub fn with_snapshot_export(
        mut self,
        port: impl crate::application::snapshot_export::SnapshotExportPort + Send + Sync + 'static,
    ) -> Self {
        self.snapshot_export = Some(std::sync::Arc::new(port));
        self
    }
    pub fn with_project_store(
        mut self,
        store: impl crate::application::projects::ProjectStore + 'static,
    ) -> Self {
        self.project_store = std::sync::Arc::new(store);
        self
    }
    pub fn with_card_project_store(
        mut self,
        store: impl crate::application::card_projects::CardProjectStore + 'static,
    ) -> Self {
        self.card_project_store = Some(std::sync::Arc::new(store));
        self
    }
    pub fn with_asset_source_archives(
        mut self,
        port: std::sync::Arc<dyn crate::application::asset_source_archives::AssetSourceArchivePort>,
    ) -> Self {
        self.asset_source_archives = Some(port);
        self
    }
    pub fn with_asset_artifact_inspector(
        mut self,
        port: std::sync::Arc<dyn crate::application::asset_artifacts::AssetArtifactInspector>,
    ) -> Self {
        self.asset_artifact_inspector = Some(port);
        self
    }
    pub fn with_asset_project_store(
        mut self,
        store: impl crate::application::asset_projects::AssetProjectStore + 'static,
    ) -> Self {
        self.asset_project_store = Some(std::sync::Arc::new(store));
        self
    }
    pub fn with_shared_asset_project_store(
        mut self,
        store: std::sync::Arc<dyn crate::application::asset_projects::AssetProjectStore>,
    ) -> Self {
        self.asset_project_store = Some(store);
        self
    }
    pub fn with_clip_project_store(
        mut self,
        store: impl crate::application::clip_projects::ClipProjectStore + 'static,
    ) -> Self {
        self.clip_project_store = Some(std::sync::Arc::new(store));
        self
    }
    pub fn with_jobs(
        mut self,
        jobs: std::sync::Arc<dyn crate::application::jobs::JobsApi>,
    ) -> Self {
        self.jobs = jobs;
        self.job_batches = None;
        self
    }
    pub fn with_shared_clip_project_store(
        mut self,
        store: std::sync::Arc<dyn crate::application::clip_projects::ClipProjectStore>,
    ) -> Self {
        self.clip_project_store = Some(store);
        self
    }
    pub fn with_clip_source_inspector(
        mut self,
        inspector: std::sync::Arc<dyn crate::application::clip_render::ClipSourceInspector>,
    ) -> Self {
        self.clip_source_inspector = Some(inspector);
        self
    }
    pub fn with_job_batches(
        mut self,
        batches: std::sync::Arc<dyn crate::application::job_batches::service::BatchApi>,
    ) -> Self {
        self.job_batches = Some(batches);
        self
    }
    pub fn with_shared_card_project_store(
        mut self,
        store: std::sync::Arc<dyn crate::application::card_projects::CardProjectStore>,
    ) -> Self {
        self.card_project_store = Some(store);
        self
    }
    pub fn with_card_template_catalog(
        mut self,
        port: std::sync::Arc<dyn crate::application::card_bundle::CardTemplateCatalogPort>,
    ) -> Self {
        self.card_templates = Some(port);
        self
    }
    pub fn with_clip_preset_catalog(
        mut self,
        port: std::sync::Arc<dyn crate::application::clip_render::presets::ClipPresetCatalogPort>,
    ) -> Self {
        self.clip_presets = Some(port);
        self
    }
    pub fn with_catalog(
        mut self,
        catalog: impl crate::application::catalog::CatalogApi + 'static,
    ) -> Self {
        self.catalog = std::sync::Arc::new(catalog);
        self
    }
    pub fn with_model_catalog(
        mut self,
        catalog: impl crate::application::model_catalog::ModelCatalogPort + 'static,
    ) -> Self {
        self.model_catalog = Some(std::sync::Arc::new(catalog));
        self
    }
    pub fn with_speaker_documents(
        mut self,
        files: impl crate::application::speaker_annotations::SpeakerDocumentFiles + 'static,
    ) -> Self {
        self.speaker_annotations = Some(std::sync::Arc::new(files));
        self
    }
    pub fn with_episode_transitions(
        mut self,
        transitions: crate::application::playback_session::transition::EpisodeTransitions,
    ) -> Self {
        self.episode_transitions = Some(std::sync::Arc::new(transitions));
        self
    }
    pub fn with_episode_playback(
        mut self,
        sessions: crate::application::playback_session::EpisodePlaybackSessions,
    ) -> Self {
        self.episode_playback = Some(std::sync::Arc::new(sessions));
        self
    }
    pub fn execute(&self, request: Request) -> Response {
        self.execute_inner(request, None, None)
    }
    pub fn execute_controlled(
        &self,
        request: Request,
        control: &dyn crate::application::review_source::MutationControl,
    ) -> Response {
        self.execute_inner(request, Some(control), None)
    }
    fn execute_inner(
        &self,
        request: Request,
        control: Option<&dyn crate::application::review_source::MutationControl>,
        recovery: Option<&dyn crate::application::sdk_execution::ReceiptRecoveryControl>,
    ) -> Response {
        let result = request.validate().and_then(|()| {
            if crate::application::review_source::media_api::is_command(&request.command) {
                return crate::application::review_source::media_api::execute(self.review_source.as_ref().and_then(|api| api.media_preparation()), &request, control);
            }
            if request.command == "schema" && request.payload["command"].as_str().is_some_and(crate::application::review_source::media_api::is_command) {
                let port = self.review_source.as_ref().and_then(|api| api.media_preparation());
                return crate::application::review_source::media_api::schema(&request.payload, port.is_some(), port.is_some_and(|port| port.writable()));
            }
            if crate::application::review_playback::api::is_command(&request.command) {
                return crate::application::review_playback::api::execute(self.review_playback.as_deref().ok_or_else(|| ApiError::new(ErrorCode::Unsupported,"Review playback runtime is not configured"))?, &request);
            }
            if request.command == "schema" && request.payload["command"].as_str().is_some_and(crate::application::review_playback::api::is_command) {
                return crate::application::review_playback::schema::describe(&request.payload,self.review_playback.is_some(),self.review_playback.as_ref().is_some_and(|sessions| sessions.controls_available()));
            }
            if crate::application::review_source::api::is_command(&request.command) {
                return self.review_source.as_ref().ok_or_else(|| ApiError::new(ErrorCode::Unsupported, "Review source adapter unavailable"))?.execute(&request);
            }
            if request.command == "schema" && request.payload["command"].as_str().is_some_and(crate::application::review_source::api::is_command) {
                return crate::application::review_source::schema::describe(&request.payload,self.review_source.is_some(),self.review_source.as_ref().is_some_and(|api| api.writable()),self.review_source.as_ref().is_some_and(|api| api.timeline_verification_available()));
            }
            if request.command==crate::application::proposals::card_export::COMMAND{
                return crate::application::proposals::card_export::execute_request(&request,self.proposal_store.as_ref(),self.card_export.as_deref());
            }
            if request.command=="schema" && request.payload["command"]==crate::application::proposals::card_export::COMMAND{
                return crate::application::proposals::card_export::schema::describe(&request,self.card_export.is_some()&&self.proposal_store.available());
            }
            if crate::application::skip_segments::host::is_command(&request.command) {
                return self.skip_host.execute(&request);
            }
            if request.command == "schema" && request.payload["command"].as_str().is_some_and(crate::application::skip_segments::host::is_command) {
                return crate::application::skip_segments::host::schema::describe_with_export(&request.payload,self.skip_host.capability(),self.skip_host.export_capability());
            }
            if request.command == snapshot_export::COMMAND {
                return snapshot_export::execute(&request, self.snapshot_export.as_deref());
            }
            if request.command == "schema" && request.payload["command"] == snapshot_export::COMMAND {
                return snapshot_export::schema(&request, self.snapshot_export.is_some());
            }
            if episode_transition::is_command(&request.command) {
                let transitions = self.episode_transitions.as_ref().ok_or_else(|| {
                    ApiError::new(ErrorCode::Unsupported, "Explicit combined playback host unavailable")
                })?;
                return episode_transition::execute(&request, transitions);
            }
            if request.command == "schema" && request.payload["command"] == crate::application::proposals::source_context::COMMAND {
                return crate::application::proposals::source_context::describe(&request.payload, self.proposal_store.available() && self.proposal_store.source_context_available());
            }
            if request.command == "schema"
                && request.payload["command"].as_str().is_some_and(episode_transition::is_command)
            {
                if request.payload.as_object().is_none_or(|payload| payload.len() != 1) {
                    return Err(ApiError::new(ErrorCode::InvalidArgument, "Invalid transition schema payload"));
                }
                return episode_transition_schema::schema(
                    request.payload["command"].as_str().unwrap(),
                    self.episode_transitions.is_some(),
                );
            }
            if let Some(sessions) = self.episode_playback.as_ref() {
                if native_session::is_command(&request.command)
                    && (request.command != "production.capture_frame" || request.payload.get("engine_instance_id").is_some()) {
                    return native_session::execute(&request, sessions);
                }
                if request.command == "schema"
                    && request.payload["command"]
                        .as_str()
                        .is_some_and(|name| native_session::available(name, sessions))
                {
                    if request.payload.as_object().is_none_or(|p| p.len() != 1) {
                        return Err(ApiError::new(
                            ErrorCode::InvalidArgument,
                            "Invalid native schema payload",
                        ));
                    }
                    let command = request.payload["command"].as_str().unwrap();
                    let mut native = native_session::schema(command);
                    if command == "production.capture_frame" && self.frame_capture.capability().available {
                        let local = schema::schema(&request.payload, self.storage.available(), true, self.playback_status.available(), self.playback_control.available(), self.review_store.available())?;
                        native["request"] = serde_json::json!({"oneOf":[native["request"],local["request"]]});
                        native["response"]["properties"]["data"] = serde_json::json!({"anyOf":[native["response"]["properties"]["data"],local["response"]["properties"]["data"]]});
                    }
                    return Ok(native);
                }
            }
            if request.command == crate::application::proposals::cue_edit::COMMAND {
                return crate::application::proposals::cue_edit::execute(&request, self.proposal_store.as_ref());
            }
            if request.command == "schema" && request.payload["command"] == crate::application::proposals::cue_edit::COMMAND {
                return crate::application::proposals::cue_edit::describe(&request.payload, self.proposal_store.available());
            }
            if crate::application::speaker_annotations::is_command(&request.command) {
                return crate::application::speaker_annotations::execute_controlled(self.speaker_annotations.as_deref(), &request, control);
            }
            if request.command == "schema" && request.payload["command"].as_str().is_some_and(crate::application::speaker_annotations::is_command) {
                return crate::application::speaker_annotations::schema(&request.payload,self.speaker_annotations.is_some());
            }
            if crate::application::model_catalog::is_command(&request.command) {
                return crate::application::model_catalog::execute_controlled(self.model_catalog.as_deref(), &request, control);
            }
            if request.command == "schema" && request.payload["command"].as_str().is_some_and(crate::application::model_catalog::is_command) {
                return crate::application::model_catalog::schema(&request.payload, self.model_catalog.is_some(), self.model_catalog.as_ref().is_some_and(|port| port.verification_available()));
            }
            if catalog::is_command(&request.command) {
                return catalog::execute(&request, self.catalog.as_ref());
            }
            if request.command == "schema"
                && request.payload["command"]
                    .as_str()
                    .is_some_and(catalog::is_command)
            {
                return catalog::schema(&request.payload, self.catalog.as_ref());
            }
            if jobs::is_command(&request.command) {
                // Original per-job API remains independent of batch orchestration.
                return jobs::execute(&request, self.jobs.as_ref());
            }
            if job_batches::is_command(&request.command) {
                return job_batches::execute(&request, self.job_batches.as_deref());
            }
            if request.command == "schema" && request.payload["command"].as_str().is_some_and(job_batches::is_command) {
                return job_batches::describe(&request.payload, self.job_batches.is_some(), self.jobs.capability());
            }
            if request.command == "schema"
                && request.payload["command"]
                    .as_str()
                    .is_some_and(jobs::is_command)
            {
                return job_schema::describe(&request.payload, self.jobs.capability());
            }
            if capture_mutation::is_command(&request.command) {
                return capture_mutation::execute(&request, self.project_store.as_ref(), self.proposal_store.as_ref());
            }
            if request.command == "schema" && request.payload["command"].as_str().is_some_and(capture_mutation::is_command) {
                return capture_mutation::describe(&request.payload, capture_mutation::available(self.project_store.as_ref(), self.proposal_store.as_ref()));
            }
            if projects::is_command(&request.command) {
                return projects::execute(
                    &request,
                    self.project_store.as_ref(),
                    self.proposal_store.as_ref(),
                );
            }
            if card_projects::is_command(&request.command) {
                return card_projects::execute(&request, self.card_project_store.as_deref());
            }
            if request.command=="schema" && request.payload["command"].as_str().is_some_and(asset_sources::is_command){return asset_sources::describe(&request.payload,self.asset_source_archives.is_some());}
            if asset_sources::is_command(&request.command) {return asset_sources::execute(&request,self.asset_source_archives.as_deref());}
            if request.command == "asset_artifacts.inspect" { return asset_artifacts::execute(&request,self.asset_artifact_inspector.as_deref()); }
            if request.command == "schema" && request.payload["command"] == "asset_artifacts.inspect" { if request.payload.as_object().is_none_or(|o|o.len()!=1) {return Err(ApiError::new(ErrorCode::InvalidArgument,"Schema accepts only command"));} return Ok(asset_artifacts::describe(self.asset_artifact_inspector.is_some())); }
            if asset_projects::is_command(&request.command) {
                return asset_projects::execute(&request, self.asset_project_store.as_deref());
            }
            if clip_projects::is_command(&request.command) {
                return clip_projects::execute(&request, self.clip_project_store.as_deref());
            }
            if clip_sources::is_command(&request.command) {
                return clip_sources::execute(&request, self.clip_source_inspector.as_deref());
            }
            if request.command == "schema" && request.payload["command"].as_str().is_some_and(clip_sources::is_command) {
                return clip_sources::describe(&request.payload, self.clip_source_inspector.is_some());
            }
            if request.command == "schema" && request.payload["command"].as_str().is_some_and(asset_projects::is_command) {
                return asset_projects::describe(&request.payload, self.asset_project_store.is_some());
            }
            if request.command == "schema" && request.payload["command"].as_str().is_some_and(clip_projects::is_command) {
                return clip_projects::describe(&request.payload, self.clip_project_store.is_some());
            }
            if request.command == "schema" && request.payload["command"].as_str().is_some_and(card_projects::is_command) {
                return card_projects::describe(&request.payload, self.card_project_store.is_some());
            }
            if request.command == "schema"
                && request.payload["command"]
                    .as_str()
                    .is_some_and(projects::is_command)
            {
                return project_schema::describe(&request.payload, self.project_store.available());
            }
            if request.command == crate::application::proposals::subtitle_export::COMMAND {
                return crate::application::proposals::subtitle_export::execute_request(&request, self.proposal_store.as_ref(), &self.storage);
            }
            if request.command == "schema" && request.payload["command"] == crate::application::proposals::subtitle_export::COMMAND {
                return crate::application::proposals::subtitle_export::schema::describe(&request.payload, self.proposal_store.available() && self.storage.available());
            }
            if crate::application::proposals::output_review::is_command(&request.command) {
                return crate::application::proposals::output_review::execute(&request, self.proposal_store.as_ref());
            }
            if request.command == "schema" && request.payload["command"].as_str().is_some_and(crate::application::proposals::output_review::is_command) {
                return crate::application::proposals::output_review::schema::describe(&request.payload, self.proposal_store.available() && self.proposal_store.output_review_available());
            }
            if crate::application::proposals::is_command(&request.command) {
                return crate::application::proposals::execute(
                    &request,
                    self.proposal_store.as_ref(),
                );
            }
            if request.command == "schema"
                && request.payload["command"]
                    .as_str()
                    .is_some_and(crate::application::proposals::is_command)
            {
                return crate::application::proposals::schema::describe_with_list(
                    &request.payload,
                    self.proposal_store.available() && (request.payload["command"] != "proposals.import_historical" || self.proposal_store.historical_reader().is_some()),
                    self.proposal_store.available() && self.proposal_store.list_available(),
                );
            }
            queries::execute_with_reviews_recoverable(
                &request,
                &self.storage,
                &self.frame_capture,
                &self.playback_status,
                &self.playback_control,
                &self.review_store,
                crate::application::frame_capture::FrameExecutionContext { control, recovery },
            )
            .and_then(|mut value| {
                if request.command == "capabilities" {
                    crate::application::review_playback::schema::capabilities(&mut value,self.review_playback.as_deref());
                    crate::application::review_source::schema::capabilities(&mut value,self.review_source.is_some(),self.review_source.as_ref().is_some_and(|api| api.writable()),self.review_source.as_ref().and_then(|api| api.storage_limits()),self.review_source.as_ref().is_some_and(|api| api.timeline_verification_available()));
                    let port = self.review_source.as_ref().and_then(|api| api.media_preparation());
                    crate::application::review_source::media_api::capabilities(&mut value, port.is_some(), port.is_some_and(|port| port.writable()));
                    crate::application::proposals::card_export::schema::capabilities(&mut value,self.card_export.is_some()&&self.proposal_store.available());
                    crate::application::skip_segments::host::schema::capabilities_with_export(&mut value,self.skip_host.capability(),self.skip_host.export_capability());
                    snapshot_export::capabilities(&mut value, self.snapshot_export.is_some());
                    episode_transition::capabilities(&mut value, self.episode_transitions.is_some());
                    if let Some(sessions) = &self.episode_playback {
                        native_session::capabilities(&mut value, sessions.capture_frame_available(), sessions.options_available());
                    }
                    catalog::capabilities(&mut value, self.catalog.as_ref());
                    crate::application::model_catalog::capabilities(&mut value, self.model_catalog.is_some(), self.model_catalog.as_ref().is_some_and(|port| port.verification_available()));
                    crate::application::speaker_annotations::capabilities(&mut value,self.speaker_annotations.is_some());
                    crate::application::proposals::cue_edit::capabilities(&mut value,self.proposal_store.available());
                    job_schema::capabilities(&mut value, self.jobs.capability());
                    job_batches::capabilities(&mut value, self.job_batches.is_some());
                    project_schema::capabilities(&mut value, self.project_store.available());
                    card_projects::capabilities(&mut value, self.card_project_store.is_some());
                    asset_sources::capabilities(&mut value,self.asset_source_archives.is_some());
                    asset_projects::capabilities(&mut value, self.asset_project_store.is_some());
                    asset_artifacts::capabilities(&mut value, self.asset_artifact_inspector.is_some());
                    clip_projects::capabilities(&mut value, self.clip_project_store.is_some());
                    clip_sources::capabilities(&mut value, self.clip_source_inspector.is_some());
                    let presets=match self.clip_presets.as_deref(){Some(port)=>port.inspect_presets()?,None=>crate::application::clip_render::presets::ClipPresetCatalogPort::inspect_presets(&crate::application::clip_render::presets::BuiltinClipPresets)?};
                    value["clip_presets"]=serde_json::to_value(presets).map_err(|_|ApiError::new(ErrorCode::Internal,"Invalid clip preset catalog"))?;
                    let templates = match self.card_templates.as_deref() {
                        Some(port) => port.inspect_templates()?,
                        None => crate::application::card_bundle::CardTemplateCatalog::unavailable(),
                    };
                    value["card_templates"] = serde_json::to_value(templates).map_err(|_| ApiError::new(ErrorCode::Internal,"Invalid card template catalog"))?;
                    capture_mutation::capabilities(&mut value, capture_mutation::available(self.project_store.as_ref(), self.proposal_store.as_ref()));
                    crate::application::proposals::schema::capabilities_with_list(
                        &mut value,
                        self.proposal_store.available(),
                        self.proposal_store.historical_reader().is_some(),
                        self.proposal_store.list_available(),
                    );
                    crate::application::proposals::source_context::capabilities(&mut value, self.proposal_store.available() && self.proposal_store.source_context_available());
                    crate::application::proposals::output_review::schema::capabilities(&mut value,self.proposal_store.available() && self.proposal_store.output_review_available());
                    crate::application::proposals::subtitle_export::schema::capabilities(&mut value, self.proposal_store.available() && self.storage.available());
                }
                Ok(value)
            })
        });
        match result {
            Ok(data) => Response {
                api_version: API_VERSION,
                request_id: request.context.request_id,
                ok: true,
                data: Some(data),
                error: None,
            },
            Err(error) => Response::failure(request.context.request_id, error),
        }
    }
}

impl<
    S: SubtitleStorage + Send + Sync,
    D: FrameCapturePort + Send + Sync,
    P: PlaybackStatusPort,
    C: PlaybackControlPort,
    R: ReviewStore,
> ApplicationClient for PlayerSdk<S, D, P, C, R>
{
    fn execute_recoverable(
        &self,
        request: Request,
        control: &(dyn crate::application::review_source::MutationControl + Sync),
        recovery: &dyn crate::application::sdk_execution::ReceiptRecoveryControl,
    ) -> Response {
        self.execute_inner(request, Some(control), Some(recovery))
    }
    fn execute(&self, request: Request) -> Response {
        PlayerSdk::execute(self, request)
    }
    fn execute_controlled(
        &self,
        request: Request,
        control: &(dyn crate::application::review_source::MutationControl + Sync),
    ) -> Response {
        PlayerSdk::execute_controlled(self, request, control)
    }
}

#[cfg(test)]
mod export_registration_tests {
    use super::*;
    use crate::{
        application::{playback_session::v1, skip_segments::host},
        domain::api_values::DecimalU64,
    };
    #[test]
    fn global_commands_schema_capabilities_and_typed_client_share_unsupported_export_names() {
        let sdk = PlayerSdk::default();
        let capabilities = sdk.execute(Request::new("capabilities")).data.unwrap();
        assert_eq!(capabilities["skip_exports"]["available"], false);
        let mut describe = Request::new("schema");
        describe.payload = serde_json::json!({"command":"schema"});
        let global = sdk.execute(describe).data.unwrap();
        let names = &global["request"]["properties"]["payload"]["properties"]["command"]["enum"];
        for command in host::EXPORT_COMMANDS {
            assert_eq!(
                capabilities::COMMANDS
                    .iter()
                    .filter(|name| *name == command)
                    .count(),
                1
            );
            assert!(
                capabilities["commands"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|row| row["command"] == *command && row["status"] == "unsupported")
            );
            assert!(names.as_array().unwrap().iter().any(|name| name == command));
            let mut request = Request::new("schema");
            request.payload = serde_json::json!({"command":command});
            assert_eq!(sdk.execute(request).data.unwrap()["status"], "unsupported");
        }
        let input = host::ListSkipExports {
            issuer_scope: "nsi1.abcdefabcdefabcdefabcdefabcdefab".parse().unwrap(),
            origin: host::SkipExportOrigin {
                session: host::SessionRef {
                    skip_session_id: "skip-original-1".into(),
                    binding: crate::application::skip_segments::SkipBinding {
                        engine_instance_id: DecimalU64(7),
                        resolution_artifact_id: "f".repeat(64),
                    },
                },
                pass_id: DecimalU64(1),
                request_revision: DecimalU64(2),
                segment_id: "op".into(),
            },
        };
        assert_eq!(
            sdk.list_skip_exports(RequestContext::default(), input)
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
        assert_eq!(
            capabilities["skip_exports"]["json_bytes"],
            v1::NATIVE_EXPORT_JSON_BYTES
        );
    }
    #[test]
    fn default_unavailable_reuses_shape_then_context_then_availability_without_fallback() {
        let sdk = PlayerSdk::default();
        let mut request = Request::new(host::EXPORT_LIST);
        request.context.expected_revision = Some("1".into());
        request.payload = serde_json::json!({"issuer_scope":"noncanonical"});
        assert_eq!(
            sdk.execute(request.clone()).error.unwrap().code,
            ErrorCode::InvalidArgument
        );
        request.payload = serde_json::json!({"issuer_scope":"nsi1.abcdefabcdefabcdefabcdefabcdefab","origin":{"session":{"skip_session_id":"skip-original-1","binding":{"engine_instance_id":"7","resolution_artifact_id":"f".repeat(64)}},"pass_id":"1","request_revision":"2","segment_id":"op"}});
        assert!(
            sdk.execute(request.clone())
                .error
                .unwrap()
                .message
                .contains("expected_revision")
        );
        request.context.expected_revision = None;
        assert!(
            sdk.execute(request)
                .error
                .unwrap()
                .message
                .contains("unavailable")
        );
    }
}

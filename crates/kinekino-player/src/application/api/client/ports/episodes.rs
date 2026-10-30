//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

/// Episode playback, resolution and catalog reads.
pub trait EpisodeClient: ApplicationClient {
    fn set_episode_rate(
        &self,
        context: RequestContext,
        input: crate::application::playback_session::SetEpisodeRate,
    ) -> Result<crate::application::playback_control::ControlReceipt, ApiError> {
        review_call(self, context, "playback.set_rate", input)
    }
    fn select_episode_track(
        &self,
        context: RequestContext,
        input: crate::application::playback_session::SelectEpisodeTrack,
    ) -> Result<crate::application::playback_session::TrackControlReceipt, ApiError> {
        review_call(self, context, "playback.select_track", input)
    }
    fn capture_episode_frame(
        &self,
        context: RequestContext,
        input: crate::application::playback_session::CaptureEpisodeFrame,
    ) -> Result<crate::application::playback_session::CapturedEpisodeFrame, ApiError> {
        review_call(self, context, "production.capture_frame", input)
    }
    fn open_episode_playback(
        &self,
        context: RequestContext,
        input: crate::application::playback_session::OpenEpisodePlayback,
    ) -> Result<crate::application::playback_session::OpenedEpisodePlayback, ApiError> {
        review_call(self, context, "playback.open", input)
    }
    fn close_episode_playback(
        &self,
        context: RequestContext,
        input: crate::application::playback_session::CloseEpisodePlayback,
    ) -> Result<crate::application::playback_session::ClosedEpisodePlayback, ApiError> {
        review_call(self, context, "playback.close", input)
    }
    fn observe_episode_playback(
        &self,
        context: RequestContext,
    ) -> Result<crate::application::playback_session::EpisodePlaybackObservation, ApiError> {
        review_call(self, context, "playback.engine", serde_json::json!({}))
    }
    fn control_episode_playback(
        &self,
        context: RequestContext,
        input: crate::application::playback_session::ControlEpisodePlayback,
    ) -> Result<crate::application::playback_control::ControlReceipt, ApiError> {
        let mut payload = input.control.payload();
        payload["engine_instance_id"] = serde_json::json!(input.engine_instance_id);
        review_call(self, context, input.control.command(), payload)
    }
    fn inspect_episode_resolution(
        &self,
        context: RequestContext,
        payload: crate::application::episode_resolve::InspectEpisodeResolution,
    ) -> Result<crate::application::episode_resolve::EpisodeResolutionInspection, ApiError> {
        review_call(self, context, "episodes.inspect_resolution", payload)
    }
    fn read_catalog_raw(
        &self,
        context: RequestContext,
        payload: crate::application::catalog::ReadCatalogRaw,
    ) -> Result<crate::application::catalog::CatalogRawChunk, ApiError> {
        review_call(self, context, "catalog.read_raw", payload)
    }
    fn read_catalog_result(
        &self,
        context: RequestContext,
        payload: crate::application::catalog::ReadCatalogResult,
    ) -> Result<crate::application::catalog::CatalogResult, ApiError> {
        review_call(self, context, "catalog.search", payload)
    }
    fn read_episode_list_result(
        &self,
        context: RequestContext,
        payload: crate::application::catalog::ReadCatalogResult,
    ) -> Result<crate::application::catalog::CatalogResult, ApiError> {
        review_call(self, context, "episodes.list", payload)
    }
    fn search_catalog(
        &self,
        context: RequestContext,
        payload: crate::application::catalog::SearchCatalog,
    ) -> Result<crate::application::catalog::CatalogResult, ApiError> {
        review_call(self, context, "catalog.search", payload)
    }
    fn list_episodes(
        &self,
        context: RequestContext,
        payload: crate::application::catalog::ListEpisodes,
    ) -> Result<crate::application::catalog::CatalogResult, ApiError> {
        review_call(self, context, "episodes.list", payload)
    }
    fn submit_episode_resolution(
        &self,
        context: RequestContext,
        input: crate::application::episode_resolve::ResolveEpisode,
        execution_timeout_ms: Option<crate::domain::api_values::DecimalU64>,
    ) -> Result<crate::application::jobs::SubmittedJob, ApiError> {
        self.submit_job(
            context,
            crate::application::jobs::SubmitJob {
                operation: "episodes.resolve".into(),
                parameters: serde_json::to_value(input).map_err(|_| {
                    ApiError::new(
                        ErrorCode::InvalidArgument,
                        "Episode request serialization failed",
                    )
                })?,
                expected_source_sha256: None,
                execution_timeout_ms: execution_timeout_ms
                    .unwrap_or(crate::domain::api_values::DecimalU64(60_000)),
            },
        )
    }
}

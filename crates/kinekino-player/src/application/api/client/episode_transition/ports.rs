//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

pub trait EpisodeTransitionClient: ApplicationClient {
    fn begin_episode_transition(
        &self,
        context: RequestContext,
        input: BeginAdjacentResolution,
        resolution_submission_key: &str,
    ) -> Result<SubmittedTransition, ApiError> {
        let command = match input.direction {
            EpisodeDirection::Previous => "playback.previous",
            EpisodeDirection::Next => "playback.next",
        };
        review_call(
            self,
            context,
            command,
            json!({
                "expected_engine_instance_id":input.expected_instance_id,
                "catalog_result_sha256":input.catalog_result_sha256,
                "resolution_submission_key":resolution_submission_key,
                "paused":input.paused,"volume_percent":input.volume_percent,
                "execution_timeout_ms":input.execution_timeout_ms,
            }),
        )
    }
    fn observe_episode_transition(
        &self,
        context: RequestContext,
        transition_id: DecimalU64,
    ) -> Result<TransitionObservation, ApiError> {
        review_call(
            self,
            context,
            "playback.transition.observe",
            json!({"transition_id":transition_id}),
        )
    }
    fn commit_episode_transition(
        &self,
        context: RequestContext,
        transition_id: DecimalU64,
    ) -> Result<ReplacedEpisode, ApiError> {
        review_call(
            self,
            context,
            "playback.transition.commit",
            json!({"transition_id":transition_id}),
        )
    }
    fn release_episode_transition(
        &self,
        context: RequestContext,
        transition_id: DecimalU64,
    ) -> Result<ReleasedTransition, ApiError> {
        review_call(
            self,
            context,
            "playback.transition.release",
            json!({"transition_id":transition_id}),
        )
    }
}

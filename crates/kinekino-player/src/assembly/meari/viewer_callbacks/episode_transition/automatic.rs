use super::state::TransitionUi;
use crate::{
    application::{jobs::JobState, playback_session::EpisodePlaybackObservation},
    assembly::meari::viewer_callbacks::progression::{Action, Continuation},
};

impl TransitionUi {
    pub(crate) fn automatic_step(
        &self,
        actual: &EpisodePlaybackObservation,
        enabled: bool,
    ) -> Option<Continuation> {
        if !enabled
            || self.rpc.is_some()
            || self.staged.is_some()
            || self.commit_unconfirmed
            || self.diagnostic.is_some()
        {
            return None;
        }
        let active = self.active.as_ref()?;
        if active.origin.engine != actual.engine_instance_id
            || active.origin.artifact != actual.resolution_artifact_id
        {
            return None;
        }
        let action = if let Some(observation) = &active.observation {
            if self.cancel_requested {
                if active.job.as_ref().is_some_and(|job| job.state.terminal()) {
                    Action::ReleaseTransition
                } else {
                    Action::ReadTransitionJob
                }
            } else if observation.job_state == JobState::Completed {
                match &active.job {
                    Some(job) if job.state == JobState::Completed => Action::CommitTransition,
                    Some(job) if job.state.terminal() => return None,
                    _ => Action::ReadTransitionJob,
                }
            } else if observation.job_state.terminal() {
                return None;
            } else {
                Action::ObserveTransition
            }
        } else {
            Action::ObserveTransition
        };
        Some(Continuation {
            action,
            operation: format!(
                "{}:{}",
                active.submitted.transition_id.0,
                active.submitted.resolution_job_id.as_str()
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        application::{
            jobs::JobRecord,
            playback_session::transition::{SubmittedTransition, TransitionObservation},
        },
        assembly::meari::viewer_callbacks::episode_transition::{
            PlayingContext, TransitionRequest, state::Active,
        },
        domain::api_values::DecimalU64,
    };
    #[test]
    fn automatic_transition_cannot_commit_after_cancel_or_replay_a_staged_commit() {
        // Display-state fixture only: no native Ready, engine, route or replacement receipt.
        let job = JobRecord::queued(
            "job-00000000000000000000000000000001"
                .to_owned()
                .try_into()
                .unwrap(),
            kinekino_jobs::domain::PreparedJobInput {
                operation: "episodes.resolve".into(),
                parameters: serde_json::json!({"episode_url":"https://catalog.example.test/fixture"}),
                source: kinekino_jobs::domain::SourceBlobRef {
                    sha256: "a".repeat(64),
                    bytes: DecimalU64(1),
                },
                execution_timeout_ms: DecimalU64(60000),
            },
        )
        .unwrap();
        let mut job = job;
        job.state = JobState::Completed;
        let actual: EpisodePlaybackObservation = serde_json::from_value(serde_json::json!({
            "engine_instance_id":"7", "resolution_artifact_id":"a".repeat(64),
            "playback":{"origin":"native_observation", "published_age_ms":"0", "connected":true,
                "paused":true, "idle":false, "position_us":"10", "duration_us":"100", "volume_percent":50.0,
                "muted":false, "video_size":null, "audio_channels":null, "has_error":false},
            "diagnostics":null, "options":null,
        })).unwrap();
        let mut owner = TransitionUi::default();
        owner.active = Some(Active {
            origin: PlayingContext {
                engine: DecimalU64(7),
                artifact: "a".repeat(64),
                catalog_result_sha256: "b".repeat(64),
            },
            submitted: SubmittedTransition {
                transition_id: DecimalU64(1),
                resolution_job_id: job.job_id.clone(),
                engine_replaced: false,
            },
            observation: Some(TransitionObservation {
                transition_id: DecimalU64(1),
                resolution_job_id: job.job_id.clone(),
                attempt: job.attempt,
                job_state: job.state,
                engine_replaced: false,
            }),
            job: Some(job),
        });
        assert!(owner.automatic_step(&actual, false).is_none());
        assert_eq!(
            owner.automatic_step(&actual, true).unwrap().action,
            Action::CommitTransition
        );
        owner.cancel_requested = true;
        assert_eq!(
            owner.automatic_step(&actual, true).unwrap().action,
            Action::ReleaseTransition
        );
        assert!(owner.handle_input(TransitionRequest::Commit).is_err());
        owner.cancel_requested = false;
        let mut foreign = actual;
        foreign.engine_instance_id = DecimalU64(8);
        assert!(owner.automatic_step(&foreign, true).is_none());
        owner.handle_input(TransitionRequest::Commit).unwrap();
        foreign.engine_instance_id = DecimalU64(7);
        assert!(owner.automatic_step(&foreign, true).is_none());
    }
}

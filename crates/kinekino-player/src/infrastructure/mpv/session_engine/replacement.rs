//! Atomic replacement within one native engine owner.
use super::{Active, NativeEpisodeEngine, missing, unavailable};
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        playback_control::ControlCompletion,
        playback_session::{
            EpisodePlaybackSource, OpenedEpisodePlayback,
            transition::{EpisodeReplacementOwner, ReplacedEpisode},
        },
    },
    domain::api_values::DecimalU64,
    infrastructure::mpv::Mpv,
};
use std::sync::atomic::Ordering;

impl EpisodeReplacementOwner for NativeEpisodeEngine {
    fn replace_verified(
        &self,
        expected_instance: DecimalU64,
        expected_artifact: &str,
        source: EpisodePlaybackSource,
        paused: bool,
        volume_percent: f64,
    ) -> Result<ReplacedEpisode, ApiError> {
        self.replace_using(
            ReplacementInput {
                expected_instance,
                expected_artifact,
                source,
                paused,
                volume_percent,
            },
            &mut NativeReplacementRuntime,
        )
    }
}

struct ReplacementInput<'a> {
    expected_instance: DecimalU64,
    expected_artifact: &'a str,
    source: EpisodePlaybackSource,
    paused: bool,
    volume_percent: f64,
}

trait ReplacementRuntime {
    fn close(
        &mut self,
        active: &mut Active,
    ) -> Result<crate::application::playback_session::ClosedEpisodePlayback, ApiError> {
        active.close()
    }
    fn launch(
        &mut self,
        owner: &NativeEpisodeEngine,
        source: &EpisodePlaybackSource,
        paused: bool,
        volume_percent: f64,
    ) -> Result<Mpv, Box<super::super::owned_startup::OwnedStartupFailure>>;
}

struct NativeReplacementRuntime;
impl ReplacementRuntime for NativeReplacementRuntime {
    fn launch(
        &mut self,
        owner: &NativeEpisodeEngine,
        source: &EpisodePlaybackSource,
        paused: bool,
        volume_percent: f64,
    ) -> Result<Mpv, Box<super::super::owned_startup::OwnedStartupFailure>> {
        Mpv::launch_episode_owned_scoped(
            &owner.executable,
            source,
            owner.parent,
            paused,
            volume_percent,
            owner.scope.clone(),
        )
    }
}

impl NativeEpisodeEngine {
    fn replace_using(
        &self,
        input: ReplacementInput<'_>,
        runtime: &mut impl ReplacementRuntime,
    ) -> Result<ReplacedEpisode, ApiError> {
        let ReplacementInput {
            expected_instance,
            expected_artifact,
            source,
            paused,
            volume_percent,
        } = input;
        if expected_instance.0 == 0
            || !volume_percent.is_finite()
            || !(0.0..=100.0).contains(&volume_percent)
        {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid native replacement",
            ));
        }
        // Reject unsupported transfer policy (for example explicit empty Referer)
        // before closing the current engine. Reuse the actual launch argument validator.
        if self.closed.load(Ordering::Acquire) {
            return Err(ended());
        }
        let _ = super::super::network_plan::arguments(&source, paused, volume_percent)?;
        let mut active = self.active.lock().map_err(|_| unavailable())?;
        if self.closed.load(Ordering::Acquire) {
            return Err(ended());
        }
        let previous = active.as_mut().ok_or_else(missing)?;
        previous.require_id(expected_instance.0)?;
        if previous.artifact_id != expected_artifact {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Native source changed",
            ));
        }
        // Shutdown/diagnostic failure retains the old owned Active for explicit recovery.
        let closed = runtime.close(previous)?;
        *active = None;
        let after_close = |mut error: ApiError, process_created: Option<bool>| {
            error.details = serde_json::json!({
                "stage":"native_episode_replacement", "previous":closed,
                "new_process_created":process_created, "replacement_accepted":false,
                "new_engine_owned":false,
                "automatic_rollback":false,
            });
            error
        };
        if self.closed.load(Ordering::Acquire) {
            return Err(after_close(ended(), Some(false)));
        }
        let mpv = match runtime.launch(self, &source, paused, volume_percent) {
            Ok(mpv) => mpv,
            Err(failure) => {
                let mut error = self.retain_startup_failure(&mut active, &source, *failure);
                let startup = error.details.clone();
                let retained = active.is_some();
                error.details = serde_json::json!({
                    "stage":"native_episode_replacement","previous":closed,
                    "new_process_created":startup["process_created"],
                    "new_cleanup_confirmed":startup["cleanup_confirmed"],
                    "new_engine_instance_id":startup["engine_instance_id"],
                    "replacement_accepted":false,"new_engine_owned":retained,
                    "startup":startup,"automatic_rollback":false,
                });
                return Err(error);
            }
        };
        let mut next = Active::from_mpv(mpv, &source, self.diagnostics_store.clone());
        // EOF may set the permanent flag while launch holds the ownership lock.
        if self.closed.load(Ordering::Acquire) {
            let cleanup = runtime.close(&mut next);
            let next_id = DecimalU64(next.id());
            let mut error = ended();
            match cleanup {
                Ok(receipt) => {
                    error.details = serde_json::json!({"stage":"native_episode_replacement", "previous":closed,"new_process_created":true,"replacement_accepted":false,"new_engine_owned":false,"cancelled_new":receipt,"automatic_rollback":false});
                }
                Err(cleanup_error) => {
                    // Keep the new child reachable for the waiting host shutdown path.
                    *active = Some(next);
                    error.details = serde_json::json!({"stage":"native_episode_replacement", "previous":closed,"new_process_created":true,"replacement_accepted":false,"new_engine_owned":true,"new_engine_instance_id":next_id,"new_cleanup_confirmed":false,"new_cleanup_error":cleanup_error,"automatic_rollback":false});
                }
            }
            return Err(error);
        }
        let current = OpenedEpisodePlayback {
            engine_instance_id: DecimalU64(next.id()),
            resolution_artifact_id: next.artifact_id.clone(),
            completion: ControlCompletion::Unobserved,
        };
        *active = Some(next);
        Ok(ReplacedEpisode {
            previous: closed,
            current,
        })
    }
}

fn ended() -> ApiError {
    ApiError::new(ErrorCode::Cancelled, "Native host lifetime has ended")
}

#[cfg(test)]
mod tests;

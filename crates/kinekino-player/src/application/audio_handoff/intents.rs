//! Shared user intent admission for SDK and presentation consumers of one native lifetime.
use super::{AudioHandoffAction, AudioHandoffPhase, NativeAudioHandoff, WebSilenceObservation};
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        native_playback::NativePlayback,
        playback_control::{ControlReceipt, PlaybackControlPort},
    },
    domain::{
        api_values::DecimalU64, native_audio::AudioIntent, playback_control::PlaybackControl,
    },
};
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub struct RequestedStartupState {
    pub audio: AudioIntent,
    pub paused: bool,
}
struct Scope {
    engine: DecimalU64,
    pending: bool,
    ending: bool,
    requested: Option<RequestedStartupState>,
    planned_revision: Option<u64>,
}
pub struct ScopedHandoffAction {
    pub request_revision: DecimalU64,
    pub action: AudioHandoffAction,
}
#[derive(Default)]
struct State {
    scope: Option<Scope>,
    last_bound_engine: Option<DecimalU64>,
    user_volume: Option<f64>,
    user_muted: Option<bool>,
    revision: u64,
    mute_revision: u64,
    reported: Option<(Instant, serde_json::Value)>,
}
#[derive(Clone, Default)]
pub struct AudioHandoffIntents(Arc<Mutex<State>>);

impl AudioHandoffIntents {
    pub fn bind_ready(&self, engine: DecimalU64) -> Result<(), ApiError> {
        if engine.0 == 0 {
            return Err(invalid());
        }
        let mut state = self.0.lock().map_err(|_| unavailable())?;
        state.reported = None;
        state.last_bound_engine = Some(engine);
        state.scope = Some(Scope {
            engine,
            pending: false,
            ending: false,
            requested: None,
            planned_revision: None,
        });
        Ok(())
    }
    pub fn bind_for_handoff(
        &self,
        engine: DecimalU64,
        mut requested: RequestedStartupState,
    ) -> Result<(), ApiError> {
        if engine.0 == 0 {
            return Err(invalid());
        }
        let mut state = self.0.lock().map_err(|_| unavailable())?;
        state.reported = None;
        state.last_bound_engine = Some(engine);
        requested.audio = AudioIntent::new(
            state
                .user_volume
                .unwrap_or(requested.audio.volume_percent()),
            state.user_muted.unwrap_or(requested.audio.muted()),
        )
        .map_err(|_| invalid())?;
        state.scope = Some(Scope {
            engine,
            pending: true,
            ending: false,
            requested: Some(requested),
            planned_revision: None,
        });
        Ok(())
    }
    pub fn clear(&self) -> Result<(), ApiError> {
        self.0.lock().map_err(|_| unavailable())?.scope = None;
        Ok(())
    }
    pub fn clear_expected(&self, engine: DecimalU64) -> Result<(), ApiError> {
        let mut state = self.0.lock().map_err(|_| unavailable())?;
        if state.last_bound_engine != Some(engine) {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "A newer native audio intent scope is bound",
            ));
        }
        state.scope = None;
        Ok(())
    }
    pub fn end_expected(&self, engine: DecimalU64) -> Result<(), ApiError> {
        let mut state = self.0.lock().map_err(|_| unavailable())?;
        if state.last_bound_engine != Some(engine) {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Native audio intent lifetime changed",
            ));
        }
        if let Some(current) = state.scope.as_mut() {
            if current.engine != engine {
                return Err(ApiError::new(
                    ErrorCode::RevisionConflict,
                    "A newer native audio intent scope is bound",
                ));
            }
            current.ending = true;
        }
        Ok(())
    }
    pub fn pending_requested(
        &self,
        engine: DecimalU64,
    ) -> Result<Option<RequestedStartupState>, ApiError> {
        let state = self.0.lock().map_err(|_| unavailable())?;
        let current = scope(&state, engine)?;
        Ok(if current.pending && !current.ending {
            current.requested
        } else {
            None
        })
    }
    pub fn publish_handoff(
        &self,
        engine: DecimalU64,
        observation: serde_json::Value,
        now: Instant,
    ) -> Result<(), ApiError> {
        let mut state = self.0.lock().map_err(|_| unavailable())?;
        if state.last_bound_engine != Some(engine) {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "A newer native handoff observation is bound",
            ));
        }
        state.reported = Some((now, observation));
        Ok(())
    }
    pub fn reported_handoff(&self) -> Result<serde_json::Value, ApiError> {
        let state = self.0.lock().map_err(|_| unavailable())?;
        let (time, reported) = state.reported.as_ref().ok_or_else(|| {
            ApiError::new(
                ErrorCode::NotFound,
                "No native audio handoff observation has been published",
            )
        })?;
        let mut reported = reported.clone();
        reported["published_age_ms"] = serde_json::json!(DecimalU64(
            time.elapsed().as_millis().try_into().unwrap_or(u64::MAX)
        ));
        Ok(reported)
    }
    pub fn requested(&self, engine: DecimalU64) -> Result<RequestedStartupState, ApiError> {
        let state = self.0.lock().map_err(|_| unavailable())?;
        scope(&state, engine)?
            .requested
            .ok_or_else(|| ApiError::new(ErrorCode::NotFound, "No native startup intent is bound"))
    }
    pub fn explicit_user_mute(&self) -> Result<Option<(DecimalU64, bool)>, ApiError> {
        let state = self.0.lock().map_err(|_| unavailable())?;
        Ok(state
            .user_muted
            .map(|muted| (DecimalU64(state.mute_revision), muted)))
    }
    /// Only nonblocking native queue admission is called under this mutex. No COM,
    /// UI callbacks, engine completion waits or handoff-decorated port re-entry.
    pub fn submit(
        &self,
        port: &impl PlaybackControlPort,
        engine: DecimalU64,
        control: PlaybackControl,
    ) -> Result<ControlReceipt, ApiError> {
        control
            .validate()
            .map_err(|message| ApiError::new(ErrorCode::InvalidArgument, message))?;
        let mut state = self.0.lock().map_err(|_| unavailable())?;
        let pending = scope(&state, engine)?.pending;
        if scope(&state, engine)?.ending {
            return Err(ApiError::new(
                ErrorCode::Cancelled,
                "Native audio lifetime is ending; cleanup is required",
            ));
        }
        let changes_intent = !matches!(control, PlaybackControl::Seek { .. });
        let revision = if changes_intent {
            state.revision.checked_add(1).ok_or_else(exhausted)?
        } else {
            state.revision
        };
        let mute_revision = if matches!(control, PlaybackControl::SetMuted { .. }) {
            state.mute_revision.checked_add(1).ok_or_else(exhausted)?
        } else {
            state.mute_revision
        };
        let receipt = if pending && changes_intent {
            ControlReceipt::deferred_until_handoff(engine.0)
        } else {
            let receipt = port.submit(control.clone())?;
            if receipt.engine_instance_id != engine {
                let mut error = ApiError::new(
                    ErrorCode::RevisionConflict,
                    "Native queue receipt belongs to another instance",
                );
                error.details = serde_json::json!({"native_submission_may_have_been_accepted":true,"receipt":receipt});
                return Err(error);
            }
            receipt
        };
        state.revision = revision;
        state.mute_revision = mute_revision;
        match control {
            PlaybackControl::SetVolume { volume_percent } => {
                state.user_volume = Some(volume_percent)
            }
            PlaybackControl::SetMuted { muted } => state.user_muted = Some(muted),
            _ => (),
        }
        if let Some(requested) = state
            .scope
            .as_mut()
            .and_then(|scope| scope.requested.as_mut())
        {
            match control {
                PlaybackControl::Play => requested.paused = false,
                PlaybackControl::Pause => requested.paused = true,
                PlaybackControl::SetVolume { volume_percent } => {
                    requested.audio = AudioIntent::new(volume_percent, requested.audio.muted())
                        .map_err(|_| invalid())?
                }
                PlaybackControl::SetMuted { muted } => {
                    requested.audio = AudioIntent::new(requested.audio.volume_percent(), muted)
                        .map_err(|_| invalid())?
                }
                PlaybackControl::Seek { .. } => (),
            }
        }
        Ok(receipt)
    }
    /// Current intent and publication of readiness change atomically. The observations
    /// were collected before taking this lock; pure policy here makes no external call.
    pub fn observe_and_plan(
        &self,
        core: &mut NativeAudioHandoff,
        engine: DecimalU64,
        native: &NativePlayback,
        web: Option<&WebSilenceObservation>,
        now: Instant,
    ) -> Result<Option<ScopedHandoffAction>, ApiError> {
        let mut state = self.0.lock().map_err(|_| unavailable())?;
        let current = scope(&state, engine)?;
        if current.ending {
            return Err(ApiError::new(
                ErrorCode::Cancelled,
                "Native audio lifetime is ending",
            ));
        }
        let revision = state.revision;
        let revision_changed = current
            .planned_revision
            .is_some_and(|prior| prior != revision);
        let requested = current.requested.ok_or_else(|| {
            ApiError::new(ErrorCode::NotFound, "No native startup intent is bound")
        })?;
        if core.phase() == AudioHandoffPhase::Ready {
            return Ok(None);
        }
        let action = if core.intent() != requested.audio
            || core.requested_pause() != Some(requested.paused)
        {
            core.replace_startup_intent(requested.audio, Some(requested.paused), now)?
        } else {
            None
        };
        let action = match action {
            Some(action) => Some(action),
            None if revision_changed
                && matches!(
                    core.phase(),
                    AudioHandoffPhase::AwaitingNativeSubmission
                        | AudioHandoffPhase::AwaitingNativeAudio
                ) =>
            {
                core.refresh_submission(now)?
            }
            None => core.observe(engine, native, web, now)?,
        };
        state
            .scope
            .as_mut()
            .ok_or_else(unavailable)?
            .planned_revision = Some(revision);
        if core.phase() == AudioHandoffPhase::Ready {
            state.scope.as_mut().ok_or_else(unavailable)?.pending = false;
        }
        Ok(action.map(|action| ScopedHandoffAction {
            request_revision: DecimalU64(revision),
            action,
        }))
    }
    /// A superseded plan performs no native queue operation. The provided action
    /// contains only fixed controls queued through the captured instance's raw port.
    pub fn perform_plan<T>(
        &self,
        engine: DecimalU64,
        request_revision: DecimalU64,
        audio: AudioIntent,
        paused: Option<bool>,
        submit: impl FnOnce() -> Result<T, ApiError>,
    ) -> Result<Option<T>, ApiError> {
        let state = self.0.lock().map_err(|_| unavailable())?;
        let current = scope(&state, engine)?;
        let requested = current.requested.ok_or_else(unavailable)?;
        if current.ending
            || state.revision != request_revision.0
            || !current.pending
            || requested.audio != audio
            || paused != Some(requested.paused)
        {
            return Ok(None);
        }
        submit().map(Some)
    }
}
fn scope(state: &State, engine: DecimalU64) -> Result<&Scope, ApiError> {
    state
        .scope
        .as_ref()
        .filter(|scope| scope.engine == engine)
        .ok_or_else(|| {
            ApiError::new(
                ErrorCode::RevisionConflict,
                "Native audio intent instance changed",
            )
        })
}
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::InvalidArgument,
        "Invalid native audio startup intent",
    )
}
fn unavailable() -> ApiError {
    ApiError::new(
        ErrorCode::Internal,
        "Native audio intent owner is unavailable",
    )
}
fn exhausted() -> ApiError {
    ApiError::new(
        ErrorCode::ResourceExhausted,
        "Native user intent identities exhausted",
    )
}

#[cfg(test)]
mod tests;

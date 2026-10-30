//! Intended application/audio_handoff/mod.rs. No UI, browser or OS dependency.
pub mod coordinator;
pub mod intents;
pub mod web_silence;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        native_playback::{NativePlayback, matches_video},
        playback_control::{ControlAcceptance, ControlReceipt},
    },
    domain::{api_values::DecimalU64, native_audio::AudioIntent},
};
use std::time::{Duration, Instant};

const HANDOFF_BUDGET: Duration = Duration::from_secs(30);
const VOLUME_OBSERVATION_TOLERANCE: f64 = 0.01;

#[derive(Debug, Clone, Copy)]
pub enum AudioOrigin {
    Web {
        document_generation: DecimalU64,
        observed_sequence: DecimalU64,
        expected_duration_seconds: f64,
    },
    Local,
    LocalWithWebView {
        document_generation: DecimalU64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioHandoffPhase {
    AwaitingMedia,
    AwaitingWebSilence,
    AwaitingNativeSubmission,
    AwaitingNativeAudio,
    Ready,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioHandoffAction {
    SilenceWeb {
        expected_document_generation: DecimalU64,
        after_observation_sequence: DecimalU64,
        document_bound: bool,
    },
    ApplyNativeAudio {
        expected_engine_instance_id: DecimalU64,
        plan_id: DecimalU64,
        intent: AudioIntent,
        paused: Option<bool>,
    },
}

pub struct WebSilenceObservation {
    pub document_generation: DecimalU64,
    pub observation_sequence: DecimalU64,
    pub paused: Option<bool>,
    pub muted: Option<bool>,
    /// Native WebView output property read after this handoff's silence request.
    /// Periodic DOM observations leave both fields None and cannot release native audio.
    pub view_muted: Option<bool>,
    pub confirmed_for_engine_instance_id: Option<DecimalU64>,
}

/// The caller keeps the native engine initially muted until this use case completes.
/// A failed handoff must retain the first error and close the native attempt explicitly;
/// it must not mark native_started or hide the web player as if the handoff succeeded.
pub struct NativeAudioHandoff {
    engine_instance_id: DecimalU64,
    origin: AudioOrigin,
    intent: AudioIntent,
    started: Instant,
    phase: AudioHandoffPhase,
    web_sequence_before_pause: DecimalU64,
    plan_id: DecimalU64,
    requested_pause: Option<bool>,
}
impl NativeAudioHandoff {
    pub fn begin(
        engine_instance_id: DecimalU64,
        origin: AudioOrigin,
        intent: AudioIntent,
        now: Instant,
    ) -> Result<Self, ApiError> {
        validate_origin(origin)?;
        if engine_instance_id.0 == 0 {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid native audio handoff identity",
            ));
        }
        let web_sequence_before_pause = match origin {
            AudioOrigin::Web {
                observed_sequence, ..
            } => observed_sequence,
            AudioOrigin::Local => DecimalU64(0),
            AudioOrigin::LocalWithWebView { .. } => DecimalU64(0),
        };
        Ok(Self {
            engine_instance_id,
            origin,
            intent,
            started: now,
            phase: AudioHandoffPhase::AwaitingMedia,
            web_sequence_before_pause,
            plan_id: DecimalU64(0),
            requested_pause: None,
        })
    }
    pub fn phase(&self) -> AudioHandoffPhase {
        self.phase
    }
    pub fn intent(&self) -> AudioIntent {
        self.intent
    }
    pub fn requested_pause(&self) -> Option<bool> {
        self.requested_pause
    }
    pub fn require_pause_observation(&mut self, paused: bool) -> Result<(), ApiError> {
        if self.phase != AudioHandoffPhase::AwaitingMedia {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Audio handoff pause intent must be bound before preparation",
            ));
        }
        self.requested_pause = Some(paused);
        Ok(())
    }
    pub fn cancel(&mut self) {
        self.phase = AudioHandoffPhase::Failed;
    }
    pub fn submission_failed(&mut self, error: ApiError) -> ApiError {
        self.phase = AudioHandoffPhase::Failed;
        error
    }
    fn check_lifetime(&mut self, now: Instant) -> Result<(), ApiError> {
        if self.phase == AudioHandoffPhase::Failed {
            return Err(ApiError::new(
                ErrorCode::Cancelled,
                "Native audio handoff has ended; no automatic retry",
            ));
        }
        let elapsed = now.checked_duration_since(self.started).ok_or_else(|| {
            ApiError::new(
                ErrorCode::InvalidArgument,
                "Audio handoff clock moved backwards",
            )
        })?;
        if self.phase != AudioHandoffPhase::Ready && elapsed >= HANDOFF_BUDGET {
            let previous_phase = self.phase;
            self.phase = AudioHandoffPhase::Failed;
            let mut error = ApiError::new(
                ErrorCode::Timeout,
                "Native audio handoff was not observed within its budget",
            );
            error.details = serde_json::json!({"engine_instance_id":self.engine_instance_id,"phase_before_timeout":previous_phase,"completion":"unobserved","native_audio_released":null,"cleanup_required":true,"automatic_retry":false});
            return Err(error);
        }
        Ok(())
    }
    fn native_action(&mut self) -> Result<Option<AudioHandoffAction>, ApiError> {
        let Some(plan_id) = self.plan_id.0.checked_add(1) else {
            self.phase = AudioHandoffPhase::Failed;
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Audio handoff plan identities exhausted",
            ));
        };
        self.plan_id = DecimalU64(plan_id);
        self.phase = AudioHandoffPhase::AwaitingNativeSubmission;
        Ok(Some(AudioHandoffAction::ApplyNativeAudio {
            expected_engine_instance_id: self.engine_instance_id,
            plan_id: self.plan_id,
            intent: self.intent,
            paused: self.requested_pause,
        }))
    }
    pub fn refresh_submission(
        &mut self,
        now: Instant,
    ) -> Result<Option<AudioHandoffAction>, ApiError> {
        self.check_lifetime(now)?;
        if !matches!(
            self.phase,
            AudioHandoffPhase::AwaitingNativeSubmission | AudioHandoffPhase::AwaitingNativeAudio
        ) {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Native audio plan is not awaiting submission or observation",
            ));
        }
        self.native_action()
    }
    /// User intent changes do not extend the original handoff budget. Superseded
    /// submissions cannot acknowledge the new plan. Ready lifetimes use ordinary controls.
    pub fn replace_intent(
        &mut self,
        intent: AudioIntent,
        now: Instant,
    ) -> Result<Option<AudioHandoffAction>, ApiError> {
        self.replace_startup_intent(intent, self.requested_pause, now)
    }
    pub fn replace_startup_intent(
        &mut self,
        intent: AudioIntent,
        paused: Option<bool>,
        now: Instant,
    ) -> Result<Option<AudioHandoffAction>, ApiError> {
        self.check_lifetime(now)?;
        if self.phase == AudioHandoffPhase::Ready {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Completed audio handoff cannot reapply user intent",
            ));
        }
        if intent == self.intent && paused == self.requested_pause {
            return Ok(None);
        }
        self.intent = intent;
        self.requested_pause = paused;
        if matches!(
            self.phase,
            AudioHandoffPhase::AwaitingNativeSubmission | AudioHandoffPhase::AwaitingNativeAudio
        ) {
            self.native_action()
        } else {
            Ok(None)
        }
    }
    /// Called with independent current native/web observations, never echoed requests.
    pub fn observe(
        &mut self,
        engine_instance_id: DecimalU64,
        native: &NativePlayback,
        web: Option<&WebSilenceObservation>,
        now: Instant,
    ) -> Result<Option<AudioHandoffAction>, ApiError> {
        self.check_lifetime(now)?;
        if engine_instance_id != self.engine_instance_id {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Audio handoff native instance changed",
            ));
        }
        if self.phase == AudioHandoffPhase::Ready {
            return Ok(None);
        }
        if native.error.is_some()
            || (self.phase != AudioHandoffPhase::AwaitingMedia && !native.connected)
        {
            self.phase = AudioHandoffPhase::Failed;
            return Err(ApiError::new(
                ErrorCode::EngineFailed,
                "Native audio handoff lost its playback owner",
            ));
        }
        match self.phase {
            AudioHandoffPhase::AwaitingMedia => {
                let expected = match self.origin {
                    AudioOrigin::Web {
                        expected_duration_seconds,
                        ..
                    } => expected_duration_seconds,
                    AudioOrigin::Local | AudioOrigin::LocalWithWebView { .. } => {
                        native.duration.unwrap_or_default()
                    }
                };
                if !matches_video(native, expected) {
                    return Ok(None);
                }
                match self.origin {
                    AudioOrigin::Web {
                        document_generation,
                        ..
                    }
                    | AudioOrigin::LocalWithWebView {
                        document_generation,
                    } => {
                        let document_bound = matches!(self.origin, AudioOrigin::Web { .. });
                        if native.muted == Some(false) {
                            self.phase = AudioHandoffPhase::Failed;
                            return Err(ApiError::new(
                                ErrorCode::EngineFailed,
                                "Native audio became unmuted before web silence was observed",
                            ));
                        }
                        if native.muted != Some(true) {
                            return Ok(None);
                        }
                        if let Some(web) = web {
                            if document_bound && web.document_generation != document_generation {
                                return Err(ApiError::new(
                                    ErrorCode::RevisionConflict,
                                    "Audio handoff web document changed",
                                ));
                            }
                            self.web_sequence_before_pause.0 = self
                                .web_sequence_before_pause
                                .0
                                .max(web.observation_sequence.0);
                        }
                        self.phase = AudioHandoffPhase::AwaitingWebSilence;
                        Ok(Some(AudioHandoffAction::SilenceWeb {
                            expected_document_generation: document_generation,
                            after_observation_sequence: self.web_sequence_before_pause,
                            document_bound,
                        }))
                    }
                    AudioOrigin::Local => self.native_action(),
                }
            }
            AudioHandoffPhase::AwaitingWebSilence => {
                let Some(web) = web else {
                    return Ok(None);
                };
                let document_generation = match self.origin {
                    AudioOrigin::Web {
                        document_generation,
                        ..
                    } => Some(document_generation),
                    AudioOrigin::LocalWithWebView { .. } => None,
                    AudioOrigin::Local => {
                        return Err(ApiError::new(
                            ErrorCode::Internal,
                            "Invalid audio handoff phase",
                        ));
                    }
                };
                if document_generation
                    .is_some_and(|generation| web.document_generation != generation)
                {
                    return Err(ApiError::new(
                        ErrorCode::RevisionConflict,
                        "Audio handoff web document changed",
                    ));
                }
                if web.observation_sequence.0 <= self.web_sequence_before_pause.0 {
                    return Ok(None);
                }
                if web
                    .confirmed_for_engine_instance_id
                    .is_some_and(|id| id != self.engine_instance_id)
                {
                    return Err(ApiError::new(
                        ErrorCode::RevisionConflict,
                        "Web silence observation belongs to another native handoff",
                    ));
                }
                if web.view_muted != Some(true)
                    || web.confirmed_for_engine_instance_id != Some(self.engine_instance_id)
                {
                    return Ok(None);
                }
                self.native_action()
            }
            AudioHandoffPhase::AwaitingNativeSubmission => Ok(None),
            AudioHandoffPhase::AwaitingNativeAudio => {
                if self
                    .requested_pause
                    .is_none_or(|paused| native.pause_observed && native.paused == paused)
                    && native.muted == Some(self.intent.muted())
                    && native.volume.is_some_and(|volume| {
                        volume.is_finite()
                            && (volume - self.intent.volume_percent()).abs()
                                < VOLUME_OBSERVATION_TOLERANCE
                    })
                {
                    self.phase = AudioHandoffPhase::Ready;
                }
                Ok(None)
            }
            AudioHandoffPhase::Ready | AudioHandoffPhase::Failed => Ok(None),
        }
    }
    /// Both queue receipts are required, but neither receipt proves application.
    pub fn native_audio_submitted(
        &mut self,
        plan_id: DecimalU64,
        volume: &ControlReceipt,
        mute: &ControlReceipt,
        now: Instant,
    ) -> Result<(), ApiError> {
        self.check_lifetime(now)?;
        if self.phase != AudioHandoffPhase::AwaitingNativeSubmission || plan_id != self.plan_id {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Native audio submission is not pending",
            ));
        }
        if volume.engine_instance_id != self.engine_instance_id
            || mute.engine_instance_id != self.engine_instance_id
        {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Native audio receipt belongs to another engine",
            ));
        }
        if volume.acceptance != ControlAcceptance::Accepted
            || mute.acceptance != ControlAcceptance::Accepted
        {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Deferred user intent is not a native audio queue receipt",
            ));
        }
        self.phase = AudioHandoffPhase::AwaitingNativeAudio;
        Ok(())
    }
}

fn validate_origin(origin: AudioOrigin) -> Result<(), ApiError> {
    if matches!(origin, AudioOrigin::LocalWithWebView { document_generation } if document_generation.0 == 0)
    {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Invalid local WebView audio scope",
        ));
    }
    if matches!(origin, AudioOrigin::Web { document_generation, observed_sequence, expected_duration_seconds }
        if document_generation.0 == 0 || observed_sequence.0 == 0 || !expected_duration_seconds.is_finite() || expected_duration_seconds <= 0.)
    {
        Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Invalid native audio handoff origin",
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests;

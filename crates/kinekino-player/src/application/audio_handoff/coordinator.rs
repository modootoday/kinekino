//! Shared handoff driver. Ports collect actual facts and execute bounded fixed operations.
use super::{
    AudioHandoffAction, AudioHandoffPhase, AudioOrigin, NativeAudioHandoff, WebSilenceObservation,
    intents::{AudioHandoffIntents, RequestedStartupState},
    web_silence::WebAudioSilenceReceipt,
};
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        native_playback::NativePlayback,
        playback_control::{ControlAcceptance, ControlReceipt, StoppedNativeEngine},
    },
    domain::{
        api_values::DecimalU64, native_audio::AudioIntent, playback_control::PlaybackControl,
    },
};
use std::time::Instant;

pub trait AudioHandoffPorts {
    fn native_snapshot(&mut self) -> Result<(DecimalU64, NativePlayback), ApiError>;
    /// Captured native instance's nonblocking raw queue; never re-enter the intent decorator.
    fn submit_native(
        &mut self,
        engine: DecimalU64,
        control: PlaybackControl,
    ) -> Result<ControlReceipt, ApiError>;
    fn silence_web(
        &mut self,
        generation: DecimalU64,
        engine: DecimalU64,
        after: DecimalU64,
        document_bound: bool,
    ) -> Result<WebAudioSilenceReceipt, ApiError>;
    fn pause_web(&mut self, generation: DecimalU64) -> Result<(), ApiError>;
    fn active_web_lease(&self) -> Result<Option<DecimalU64>, ApiError>;
    fn observe_web(
        &mut self,
        lease: DecimalU64,
        after: DecimalU64,
    ) -> Result<WebSilenceObservation, ApiError>;
    fn record_web_user_mute(&mut self, muted: bool) -> Result<(), ApiError>;
    fn stop_native(&mut self, engine: DecimalU64) -> Result<StoppedNativeEngine, ApiError>;
    fn restore_web(
        &mut self,
        lease: DecimalU64,
        stopped: &StoppedNativeEngine,
    ) -> Result<(), ApiError>;
}

pub struct PreparedAudioHandoff {
    origin: AudioOrigin,
    requested: RequestedStartupState,
    started: Instant,
}
impl PreparedAudioHandoff {
    pub fn web(
        origin: Option<AudioOrigin>,
        current_generation: DecimalU64,
        backend_available: bool,
        audio: AudioIntent,
        paused: bool,
        now: Instant,
    ) -> Result<Self, ApiError> {
        if !backend_available {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Whole-WebView audio silence is unavailable",
            ));
        }
        let origin = origin.ok_or_else(|| {
            ApiError::new(
                ErrorCode::NeedsInput,
                "A bound web playback observation is required before native handoff",
            )
        })?;
        super::validate_origin(origin)?;
        if !matches!(origin, AudioOrigin::Web { document_generation, .. } if document_generation == current_generation && current_generation.0 != 0)
        {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Web audio source changed before native launch",
            ));
        }
        Ok(Self {
            origin,
            requested: RequestedStartupState { audio, paused },
            started: now,
        })
    }
    pub fn local(audio: AudioIntent, paused: bool, now: Instant) -> Self {
        Self {
            origin: AudioOrigin::Local,
            requested: RequestedStartupState { audio, paused },
            started: now,
        }
    }
    pub fn local_with_web_view(
        generation: DecimalU64,
        backend_available: bool,
        audio: AudioIntent,
        paused: bool,
        now: Instant,
    ) -> Result<Self, ApiError> {
        if !backend_available {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Whole-WebView audio silence is unavailable",
            ));
        }
        let origin = AudioOrigin::LocalWithWebView {
            document_generation: generation,
        };
        super::validate_origin(origin)?;
        Ok(Self {
            origin,
            requested: RequestedStartupState { audio, paused },
            started: now,
        })
    }
    pub fn requested(&self) -> RequestedStartupState {
        self.requested
    }
    pub fn bind(
        self,
        engine: DecimalU64,
        intents: AudioHandoffIntents,
    ) -> Result<AudioHandoffCoordinator, ApiError> {
        let requested = intents.requested(engine)?;
        let mut core =
            NativeAudioHandoff::begin(engine, self.origin, requested.audio, self.started)?;
        core.require_pause_observation(requested.paused)?;
        Ok(AudioHandoffCoordinator {
            core,
            intents,
            engine,
            lease: None,
            web: None,
            stopped: None,
            first_error: None,
            cleanup_errors: Vec::new(),
            closing: false,
            cleanup_complete: false,
            web_mute_revision: 0,
        })
    }
}

pub struct AudioHandoffCoordinator {
    core: NativeAudioHandoff,
    intents: AudioHandoffIntents,
    engine: DecimalU64,
    lease: Option<DecimalU64>,
    web: Option<WebSilenceObservation>,
    stopped: Option<StoppedNativeEngine>,
    first_error: Option<ApiError>,
    cleanup_errors: Vec<ApiError>,
    closing: bool,
    cleanup_complete: bool,
    web_mute_revision: u64,
}
#[derive(serde::Serialize)]
pub struct AudioHandoffObservation<'a> {
    pub engine_instance_id: DecimalU64,
    pub phase: AudioHandoffPhase,
    pub lifetime_status: &'static str,
    pub ready: bool,
    pub closing: bool,
    pub native_stop_confirmed: bool,
    pub web_silence_lease_id: Option<DecimalU64>,
    pub cleanup_complete: bool,
    pub error: Option<&'a ApiError>,
    pub cleanup_errors: &'a [ApiError],
}
impl AudioHandoffCoordinator {
    pub fn is_ready(&self) -> bool {
        !self.closing && self.first_error.is_none() && self.core.phase() == AudioHandoffPhase::Ready
    }
    pub fn cleanup_complete(&self) -> bool {
        self.cleanup_complete
    }
    pub fn requested_pause(&self) -> Option<bool> {
        self.core.requested_pause()
    }
    pub fn failure(&self) -> Option<&ApiError> {
        self.first_error
            .as_ref()
            .or_else(|| self.cleanup_errors.first())
    }
    pub fn observation(&self) -> AudioHandoffObservation<'_> {
        let lifetime_status = if self.first_error.is_some() {
            "failed"
        } else if self.cleanup_complete {
            "closed"
        } else if self.closing {
            "closing"
        } else if self.is_ready() {
            "ready"
        } else {
            "preparing"
        };
        AudioHandoffObservation {
            engine_instance_id: self.engine,
            phase: self.core.phase(),
            lifetime_status,
            ready: self.is_ready(),
            closing: self.closing,
            native_stop_confirmed: self.stopped.is_some(),
            web_silence_lease_id: self.lease,
            cleanup_complete: self.cleanup_complete,
            error: self.first_error.as_ref(),
            cleanup_errors: &self.cleanup_errors,
        }
    }
    pub fn drive(&mut self, ports: &mut impl AudioHandoffPorts, now: Instant) {
        if self.closing || self.first_error.is_some() {
            return;
        }
        if let Err(error) = self.try_drive(ports, now) {
            self.first_error = Some(error);
            self.core.cancel();
            self.closing = true;
            if let Err(cleanup) = self.intents.end_expected(self.engine) {
                self.remember_cleanup_error(cleanup);
            }
            self.cleanup(ports);
        }
        self.publish(now);
    }
    pub fn close(&mut self, ports: &mut impl AudioHandoffPorts) {
        self.closing = true;
        self.core.cancel();
        if let Err(error) = self.intents.end_expected(self.engine) {
            self.remember_cleanup_error(error);
        }
        self.cleanup(ports);
        self.publish(Instant::now());
    }
    fn publish(&mut self, now: Instant) {
        let result = serde_json::to_value(self.observation())
            .map_err(|_| {
                ApiError::new(
                    ErrorCode::Internal,
                    "Audio handoff observation encoding failed",
                )
            })
            .and_then(|value| self.intents.publish_handoff(self.engine, value, now));
        if let Err(error) = result {
            self.remember_cleanup_error(error);
        }
    }
    fn remember_cleanup_error(&mut self, error: ApiError) {
        if self.cleanup_errors.len() == 8 {
            self.cleanup_errors.remove(1);
        }
        self.cleanup_errors.push(error);
    }
    fn sync_user_web_mute(&mut self, ports: &mut impl AudioHandoffPorts) -> Result<(), ApiError> {
        if self.lease.is_none() {
            return Ok(());
        }
        if let Some((revision, muted)) = self.intents.explicit_user_mute()?
            && revision.0 != self.web_mute_revision
        {
            ports.record_web_user_mute(muted)?;
            self.web_mute_revision = revision.0;
        }
        Ok(())
    }
    fn cleanup(&mut self, ports: &mut impl AudioHandoffPorts) {
        if self.cleanup_complete {
            return;
        }
        let outcome = (|| -> Result<(), ApiError> {
            if self.stopped.is_none() {
                self.stopped = Some(ports.stop_native(self.engine)?);
            }
            if matches!(
                self.core.origin,
                AudioOrigin::Web { .. } | AudioOrigin::LocalWithWebView { .. }
            ) && self.lease.is_none()
            {
                self.lease = ports.active_web_lease()?;
            }
            // Queue admission is closed by the stop proof. Read the final accepted user
            // intent after that fence, so an in-flight mute request cannot be lost.
            self.sync_user_web_mute(ports)?;
            if let Some(lease) = self.lease {
                ports.restore_web(
                    lease,
                    self.stopped.as_ref().ok_or_else(|| {
                        ApiError::new(ErrorCode::Internal, "Native stop proof unavailable")
                    })?,
                )?;
                self.lease = None;
            }
            self.intents.clear_expected(self.engine)?;
            self.cleanup_complete = true;
            Ok(())
        })();
        if let Err(error) = outcome {
            self.remember_cleanup_error(error);
        }
    }
    fn try_drive(
        &mut self,
        ports: &mut impl AudioHandoffPorts,
        now: Instant,
    ) -> Result<(), ApiError> {
        let (engine, native) = ports.native_snapshot()?;
        if engine != self.engine {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Native handoff observation belongs to another instance",
            ));
        }
        if native.error.is_some()
            || (self.core.phase() != AudioHandoffPhase::AwaitingMedia && !native.connected)
        {
            return Err(ApiError::new(
                ErrorCode::EngineFailed,
                "Native playback failed during audio handoff",
            ));
        }
        if let Some(lease) = self.lease {
            let observation = ports.observe_web(lease, self.core.web_sequence_before_pause)?;
            if observation.view_muted != Some(true) {
                return Err(ApiError::new(
                    ErrorCode::EngineFailed,
                    "Owned WebView audio silence was lost",
                ));
            }
            self.web = Some(observation);
            self.sync_user_web_mute(ports)?;
        }
        let Some(scoped) = self.intents.observe_and_plan(
            &mut self.core,
            engine,
            &native,
            self.web.as_ref(),
            now,
        )?
        else {
            return Ok(());
        };
        match scoped.action {
            AudioHandoffAction::SilenceWeb {
                expected_document_generation,
                after_observation_sequence,
                document_bound,
            } => {
                let silence = ports.silence_web(
                    expected_document_generation,
                    self.engine,
                    after_observation_sequence,
                    document_bound,
                );
                match silence {
                    Ok(receipt) => {
                        self.lease = Some(receipt.lease_id);
                        self.web = Some(receipt.observation);
                    }
                    Err(error) => {
                        match ports.active_web_lease() {
                            Ok(lease) => self.lease = lease,
                            Err(cleanup) => self.remember_cleanup_error(cleanup),
                        }
                        return Err(error);
                    }
                }
                let pause_generation = if document_bound {
                    expected_document_generation
                } else {
                    self.web
                        .as_ref()
                        .map_or(expected_document_generation, |observation| {
                            observation.document_generation
                        })
                };
                ports.pause_web(pause_generation)?;
            }
            AudioHandoffAction::ApplyNativeAudio {
                expected_engine_instance_id,
                plan_id,
                intent,
                paused,
            } => {
                let submitted = self.intents.perform_plan(
                    expected_engine_instance_id,
                    scoped.request_revision,
                    intent,
                    paused,
                    || submit_plan(ports, expected_engine_instance_id, plan_id, intent, paused),
                )?;
                if let Some((volume, mute)) = submitted {
                    self.core
                        .native_audio_submitted(plan_id, &volume, &mute, now)?;
                }
            }
        }
        Ok(())
    }
}

fn queue_receipt(receipt: ControlReceipt, engine: DecimalU64) -> Result<ControlReceipt, ApiError> {
    if receipt.engine_instance_id != engine || receipt.acceptance != ControlAcceptance::Accepted {
        return Err(ApiError::new(
            ErrorCode::RevisionConflict,
            "Audio plan did not reach the captured native queue",
        ));
    }
    Ok(receipt)
}
fn submit_plan(
    ports: &mut impl AudioHandoffPorts,
    engine: DecimalU64,
    plan: DecimalU64,
    intent: AudioIntent,
    paused: Option<bool>,
) -> Result<(ControlReceipt, ControlReceipt), ApiError> {
    let mut pause_accepted = None;
    let mut volume_accepted = false;
    let mut mute_accepted = false;
    let result = (|| {
        if let Some(paused) = paused {
            queue_receipt(
                ports.submit_native(
                    engine,
                    if paused {
                        PlaybackControl::Pause
                    } else {
                        PlaybackControl::Play
                    },
                )?,
                engine,
            )?;
            pause_accepted = Some(true);
        }
        let volume = queue_receipt(
            ports.submit_native(
                engine,
                PlaybackControl::SetVolume {
                    volume_percent: intent.volume_percent(),
                },
            )?,
            engine,
        )?;
        volume_accepted = true;
        let mute = queue_receipt(
            ports.submit_native(
                engine,
                PlaybackControl::SetMuted {
                    muted: intent.muted(),
                },
            )?,
            engine,
        )?;
        mute_accepted = true;
        Ok((volume, mute))
    })();
    result.map_err(|mut error: ApiError| {
        error.details = serde_json::json!({"cause":error.details,"stage":"native_audio_plan","engine_instance_id":engine,"plan_id":plan,
            "pause_requested":paused,"pause_accepted":pause_accepted,"volume_accepted":volume_accepted,"mute_accepted":mute_accepted,"completion":"unobserved"});
        error
    })
}

#[cfg(test)]
mod tests;

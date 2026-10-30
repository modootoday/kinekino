//! Legacy process/browser adapter for the shared application handoff driver.
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        audio_handoff::{
            WebSilenceObservation, coordinator::AudioHandoffPorts,
            web_silence::WebAudioSilenceReceipt,
        },
        native_playback::NativePlayback,
        playback_control::{ControlReceipt, StoppedNativeEngine},
    },
    domain::{api_values::DecimalU64, playback_control::PlaybackControl},
    infrastructure::{browser::Browser, mpv::Mpv},
};

pub struct LegacyAudioHandoffPorts<'a> {
    native: &'a mut Option<Mpv>,
    browser: &'a Browser,
}
impl<'a> LegacyAudioHandoffPorts<'a> {
    pub fn new(native: &'a mut Option<Mpv>, browser: &'a Browser) -> Self {
        Self { native, browser }
    }
    fn player(&mut self, expected: DecimalU64) -> Result<&mut Mpv, ApiError> {
        let player = self
            .native
            .as_mut()
            .ok_or_else(|| ApiError::new(ErrorCode::NotFound, "No native process is owned"))?;
        if player.control_handle().instance_id() != expected.0 {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Native audio handoff process changed",
            ));
        }
        Ok(player)
    }
}
impl AudioHandoffPorts for LegacyAudioHandoffPorts<'_> {
    fn native_snapshot(&mut self) -> Result<(DecimalU64, NativePlayback), ApiError> {
        let player = self
            .native
            .as_mut()
            .ok_or_else(|| ApiError::new(ErrorCode::NotFound, "No native process is owned"))?;
        let id = DecimalU64(player.control_handle().instance_id());
        Ok((id, player.snapshot().clone()))
    }
    fn submit_native(
        &mut self,
        engine: DecimalU64,
        control: PlaybackControl,
    ) -> Result<ControlReceipt, ApiError> {
        // The captured handle validates instance lifetime and performs bounded try_send.
        // It deliberately does not re-enter the SDK's user-intent decorator.
        self.player(engine)?.control_handle().submit(control)
    }
    fn silence_web(
        &mut self,
        generation: DecimalU64,
        engine: DecimalU64,
        after: DecimalU64,
        document_bound: bool,
    ) -> Result<WebAudioSilenceReceipt, ApiError> {
        if document_bound {
            self.browser
                .silence_for_native_handoff(generation, engine, after)
        } else {
            self.browser
                .silence_whole_view_for_local_native(engine, after)
        }
    }
    fn pause_web(&mut self, generation: DecimalU64) -> Result<(), ApiError> {
        self.browser.pause_media_for_handoff(generation)
    }
    fn active_web_lease(&self) -> Result<Option<DecimalU64>, ApiError> {
        self.browser.active_audio_silence_lease()
    }
    fn observe_web(
        &mut self,
        lease: DecimalU64,
        after: DecimalU64,
    ) -> Result<WebSilenceObservation, ApiError> {
        self.browser.observe_audio_silence(lease, after)
    }
    fn record_web_user_mute(&mut self, muted: bool) -> Result<(), ApiError> {
        self.browser
            .set_web_user_muted(DecimalU64(self.browser.generation()), muted)?;
        Ok(())
    }
    fn stop_native(&mut self, engine: DecimalU64) -> Result<StoppedNativeEngine, ApiError> {
        let stopped = self.player(engine)?.shutdown_observed()?;
        self.native.take();
        Ok(stopped)
    }
    fn restore_web(
        &mut self,
        lease: DecimalU64,
        stopped: &StoppedNativeEngine,
    ) -> Result<(), ApiError> {
        self.browser.restore_web_audio(lease, stopped)?;
        Ok(())
    }
}

//! Intended infrastructure/browser/audio_handoff.rs. Fixed existing pause operation.
use super::{Browser, audio_backend::WebViewAudioBackend};
use crate::application::api::{ApiError, ErrorCode};
use crate::application::{
    audio_handoff::{
        WebSilenceObservation,
        web_silence::{
            RestoredWebAudio, WebAudioSilenceOwner, WebAudioSilenceReceipt, WebUserMuteReceipt,
        },
    },
    playback_control::StoppedNativeEngine,
};
use crate::domain::api_values::DecimalU64;
use std::{cell::RefCell, rc::Rc};

pub(super) type BrowserAudioSilence = Rc<RefCell<WebAudioSilenceOwner<WebViewAudioBackend>>>;

impl Browser {
    pub fn audio_handoff_available(&self) -> bool {
        self.audio_silence.is_some()
    }
    fn audio_owner(&self) -> Result<&BrowserAudioSilence, ApiError> {
        self.audio_silence.as_ref().ok_or_else(|| {
            ApiError::new(
                ErrorCode::Unsupported,
                "Native WebView audio silence is unavailable",
            )
        })
    }
    pub fn silence_for_native_handoff(
        &self,
        generation: DecimalU64,
        engine: DecimalU64,
        after: DecimalU64,
    ) -> Result<WebAudioSilenceReceipt, ApiError> {
        self.audio_owner()?
            .try_borrow_mut()
            .map_err(|_| busy())?
            .silence(generation, engine, after)
    }
    pub fn active_audio_silence_lease(&self) -> Result<Option<DecimalU64>, ApiError> {
        Ok(self
            .audio_owner()?
            .try_borrow()
            .map_err(|_| busy())?
            .active_lease_id())
    }
    pub fn silence_whole_view_for_local_native(
        &self,
        engine: DecimalU64,
        after: DecimalU64,
    ) -> Result<WebAudioSilenceReceipt, ApiError> {
        self.audio_owner()?
            .try_borrow_mut()
            .map_err(|_| busy())?
            .silence_view(engine, after)
    }
    pub fn observe_audio_silence(
        &self,
        lease: DecimalU64,
        after: DecimalU64,
    ) -> Result<WebSilenceObservation, ApiError> {
        self.audio_owner()?
            .try_borrow_mut()
            .map_err(|_| busy())?
            .observe(lease, after)
    }
    pub fn restore_web_audio(
        &self,
        lease: DecimalU64,
        stopped: &StoppedNativeEngine,
    ) -> Result<RestoredWebAudio, ApiError> {
        self.audio_owner()?
            .try_borrow_mut()
            .map_err(|_| busy())?
            .restore(lease, stopped)
    }
    pub fn set_web_user_muted(
        &self,
        generation: DecimalU64,
        muted: bool,
    ) -> Result<WebUserMuteReceipt, ApiError> {
        self.audio_owner()?
            .try_borrow_mut()
            .map_err(|_| busy())?
            .set_user_muted(generation, muted)
    }
    /// Submission only. This never supplies silence evidence; the native WebView
    /// IsMuted property must independently confirm whole-view output silence.
    pub fn pause_media_for_handoff(
        &self,
        expected_document_generation: DecimalU64,
    ) -> Result<(), ApiError> {
        if expected_document_generation.0 == 0 {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid web audio handoff generation",
            ));
        }
        if self.generation() != expected_document_generation.0 {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Web document changed before audio pause submission",
            ));
        }
        self.pause_media().map_err(|_| {
            ApiError::new(
                ErrorCode::EngineFailed,
                "Web pause submission failed during audio handoff",
            )
        })
    }
}

fn busy() -> ApiError {
    ApiError::new(
        ErrorCode::NeedsInput,
        "WebView audio owner is currently borrowed",
    )
}

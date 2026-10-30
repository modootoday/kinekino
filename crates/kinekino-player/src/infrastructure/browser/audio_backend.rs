//! Intended infrastructure/browser/audio_backend.rs. All calls stay on the owning STA.
use crate::application::{api::ApiError, audio_handoff::web_silence::WebAudioBackend};
use crate::infrastructure::webview_audio::NativeWebViewAudio;
use std::{cell::Cell, rc::Rc};
use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2;

#[cfg(test)]
mod actual;

pub(super) struct WebViewAudioBackend {
    core: NativeWebViewAudio,
    generation: Rc<Cell<u64>>,
}
impl WebViewAudioBackend {
    pub(super) fn new(core: &ICoreWebView2, generation: Rc<Cell<u64>>) -> Result<Self, ApiError> {
        let core = NativeWebViewAudio::new(core)?;
        Ok(Self { core, generation })
    }
}
impl WebAudioBackend for WebViewAudioBackend {
    fn document_generation(&self) -> u64 {
        self.generation.get()
    }
    fn is_muted(&self) -> Result<bool, ApiError> {
        self.core.is_muted()
    }
    fn set_muted(&self, muted: bool) -> Result<(), ApiError> {
        self.core.set_muted(muted)
    }
}

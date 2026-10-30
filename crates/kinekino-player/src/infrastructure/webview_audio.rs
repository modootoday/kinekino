//! Fixed whole-WebView audio properties. Keep this handle on its creating STA.
use crate::application::api::{ApiError, ErrorCode};
use std::{marker::PhantomData, rc::Rc};
use webview2_com::Microsoft::Web::WebView2::Win32::{ICoreWebView2, ICoreWebView2_8};
use windows::core::{BOOL, Interface};

pub(crate) struct NativeWebViewAudio {
    core: ICoreWebView2_8,
    _sta: PhantomData<Rc<()>>,
}
impl NativeWebViewAudio {
    pub(crate) fn new(core: &ICoreWebView2) -> Result<Self, ApiError> {
        core.cast::<ICoreWebView2_8>()
            .map(|core| Self {
                core,
                _sta: PhantomData,
            })
            .map_err(|_| {
                ApiError::new(
                    ErrorCode::Unsupported,
                    "Native WebView audio properties are unavailable",
                )
            })
    }
    pub(crate) fn is_muted(&self) -> Result<bool, ApiError> {
        let mut value = BOOL(0);
        // SAFETY: this fixed property adapter is used only on the owning WebView STA.
        unsafe { self.core.IsMuted(&mut value) }.map_err(|_| {
            ApiError::new(
                ErrorCode::EngineFailed,
                "WebView audio mute observation failed",
            )
        })?;
        Ok(value.as_bool())
    }
    pub(crate) fn set_muted(&self, muted: bool) -> Result<(), ApiError> {
        // SAFETY: fixed native property; no page script or caller-supplied command.
        unsafe { self.core.SetIsMuted(muted) }.map_err(|_| {
            ApiError::new(
                ErrorCode::EngineFailed,
                "WebView audio mute submission failed",
            )
        })
    }
}

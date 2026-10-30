//! Hidden-provider script-dialog and download suppression on the owning STA.
use crate::application::api::{ApiError, ErrorCode};
use std::rc::Rc;
use webview2_com::{
    DownloadStartingEventHandler, Microsoft::Web::WebView2::Win32::ICoreWebView2_4,
};
use windows::core::Interface;
use wry::{WebView, WebViewExtWindows};

/// Caller supplies only a bounded nonblocking failure enqueue operation, with no IO.
pub(super) type FailureSink = Rc<dyn Fn(ApiError)>;

/// Create on the owning STA before any provider URL is loaded.
/// Keep this guard alive and drop it before the owning WebView/controller is destroyed.
pub(super) struct HiddenUiPolicy {
    core: ICoreWebView2_4,
    download_token: i64,
    failure: FailureSink,
}

impl HiddenUiPolicy {
    pub fn install(view: &WebView, failure: FailureSink) -> Result<Self, ApiError> {
        // SAFETY: all settings, handlers and event args stay on this WebView's STA.
        unsafe {
            let core = view.webview();
            let audio = crate::infrastructure::webview_audio::NativeWebViewAudio::new(&core)?;
            audio.set_muted(true)?;
            if !audio.is_muted()? {
                return Err(failed("Hidden provider audio silence was not observed"));
            }
            core.Settings()
                .and_then(|settings| settings.SetAreDefaultScriptDialogsEnabled(false))
                .map_err(|_| failed("Hidden provider script dialogs could not be disabled"))?;
            let core = core
                .cast::<ICoreWebView2_4>()
                .map_err(|_| failed("Hidden provider download cancellation is unavailable"))?;
            let callback_failure = failure.clone();
            let handler = DownloadStartingEventHandler::create(Box::new(move |_, args| {
                let Some(args) = args else {
                    callback_failure(failed("Hidden provider download callback had no arguments"));
                    return Ok(());
                };
                // Explicitly cancel the transfer and suppress the default download UI.
                // No destination lookup, file IO, engine wait or HTML processing here.
                let cancelled = args.SetCancel(true);
                let handled = args.SetHandled(true);
                if cancelled.is_err() || handled.is_err() {
                    callback_failure(failed(
                        "Hidden provider download could not be cancelled without UI",
                    ));
                }
                Ok(())
            }));
            let mut download_token = 0;
            core.add_DownloadStarting(&handler, &mut download_token)
                .map_err(|_| failed("Hidden provider download handler could not be installed"))?;
            Ok(Self {
                core,
                download_token,
                failure,
            })
        }
    }
}

fn failed(message: &str) -> ApiError {
    ApiError::new(ErrorCode::EngineFailed, message)
}

impl Drop for HiddenUiPolicy {
    fn drop(&mut self) {
        // SAFETY: guard is owned and destroyed by the same STA before its WebView.
        if unsafe { self.core.remove_DownloadStarting(self.download_token) }.is_err() {
            (self.failure)(failed(
                "Hidden provider download handler could not be removed",
            ));
        }
    }
}

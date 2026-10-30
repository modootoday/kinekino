//! Native navigation identities stay on the owning STA; DOM status remains separate.
use super::{
    capture::{Callback, Page, now_ms},
    error,
};
use crate::application::{api::ErrorCode, browser_provider::*};
use crate::domain::api_values::DecimalU64;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use webview2_com::{
    CoTaskMemPWSTR, Microsoft::Web::WebView2::Win32::*, NavigationCompletedEventHandler,
    NavigationStartingEventHandler,
};
use windows::core::{BOOL, Interface, PWSTR, w};
use wry::{WebView, WebViewExtWindows};

pub(super) type EventSink = Rc<dyn Fn(Callback)>;
pub(super) struct NavigationEvents {
    core: ICoreWebView2,
    starting: Option<i64>,
    completed: Option<i64>,
    frame: Option<i64>,
    closed: Rc<Cell<bool>>,
}
impl NavigationEvents {
    pub fn install(
        view: &WebView,
        page: Rc<RefCell<Page>>,
        policy: ValidatedBrowserRequest,
        sink: EventSink,
    ) -> Result<Self, crate::application::api::ApiError> {
        let closed = Rc::new(Cell::new(false));
        // SAFETY: registration, event handling and removal are confined to this WebView's STA.
        unsafe {
            let core = view.webview();
            let mut guard = Self {
                core: core.clone(),
                starting: None,
                completed: None,
                frame: None,
                closed: closed.clone(),
            };
            let start_page = page.clone();
            let start_sink = sink.clone();
            let start_closed = closed.clone();
            let handler = NavigationStartingEventHandler::create(Box::new(move |_, args| {
                if start_closed.get() {
                    return Ok(());
                }
                let Some(args) = args else {
                    return Ok(());
                };
                let mut uri = PWSTR::null();
                args.Uri(&mut uri)?;
                let uri = CoTaskMemPWSTR::from(uri).to_string();
                let mut native_id = 0;
                args.NavigationId(&mut native_id)?;
                if uri == "about:blank" && start_page.borrow().generation == 0 {
                    args.SetCancel(true)?;
                    return Ok(());
                }
                if !policy.navigation_allowed(&uri) {
                    args.SetCancel(true)?;
                    start_sink(Callback::Failed(error(
                        ErrorCode::PermissionDenied,
                        "Browser navigation left its allowed scope",
                    )));
                    return Ok(());
                }
                let mut current = start_page.borrow_mut();
                let Some(generation) = current.generation.checked_add(1) else {
                    drop(current);
                    args.SetCancel(true)?;
                    start_sink(Callback::Failed(error(
                        ErrorCode::ResourceExhausted,
                        "Browser navigation identity is unavailable",
                    )));
                    return Ok(());
                };
                current.url = uri.clone();
                current.generation = generation;
                current.native_navigation_id = Some(native_id);
                current.activation = None;
                drop(current);
                start_sink(Callback::Navigation(uri, generation, native_id));
                Ok(())
            }));
            let mut token = 0;
            core.add_NavigationStarting(&handler, &mut token)
                .map_err(|_| {
                    error(
                        ErrorCode::EngineFailed,
                        "Native navigation handler registration failed",
                    )
                })?;
            guard.starting = Some(token);

            let complete_page = page.clone();
            let complete_sink = sink.clone();
            let complete_closed = closed.clone();
            let handler = NavigationCompletedEventHandler::create(Box::new(move |_, args| {
                if complete_closed.get() {
                    return Ok(());
                }
                let Some(args) = args else {
                    return Ok(());
                };
                let mut native_id = 0;
                args.NavigationId(&mut native_id)?;
                let mut success = BOOL::default();
                args.IsSuccess(&mut success)?;
                let mut web_error = COREWEBVIEW2_WEB_ERROR_STATUS(0);
                args.WebErrorStatus(&mut web_error)?;
                let status = args
                    .cast::<ICoreWebView2NavigationCompletedEventArgs2>()
                    .ok()
                    .and_then(|args| {
                        let mut status = 0;
                        args.HttpStatusCode(&mut status).ok().map(|_| status)
                    });
                let current = complete_page.borrow();
                if current.native_navigation_id != Some(native_id) || current.generation == 0 {
                    return Ok(());
                }
                let generation = current.generation;
                let uri = current.url.clone();
                drop(current);
                complete_sink(Callback::NavigationCompleted(MainNavigationDiagnostic {
                    observed_unix_ms: now_ms(),
                    document_generation: DecimalU64(generation),
                    native_navigation_id: DecimalU64(native_id),
                    http_status_code: status,
                    is_success: success.as_bool(),
                    web_error_status: web_error.0,
                }));
                complete_sink(Callback::Loaded(uri));
                Ok(())
            }));
            let mut token = 0;
            core.add_NavigationCompleted(&handler, &mut token)
                .map_err(|_| {
                    error(
                        ErrorCode::EngineFailed,
                        "Native navigation completion registration failed",
                    )
                })?;
            guard.completed = Some(token);

            let handler = NavigationStartingEventHandler::create(Box::new(move |_, args| {
                if closed.get() {
                    return Ok(());
                }
                let Some(args) = args else {
                    return Ok(());
                };
                let mut uri = PWSTR::null();
                args.Uri(&mut uri)?;
                let uri = CoTaskMemPWSTR::from(uri).to_string();
                if uri.len() > 8192 || !crate::domain::navigation::player_page(&uri) {
                    return Ok(());
                }
                let mut frame_navigation_id = 0;
                args.NavigationId(&mut frame_navigation_id)?;
                let referer = args
                    .RequestHeaders()
                    .ok()
                    .and_then(|headers| {
                        let mut value = PWSTR::null();
                        headers
                            .GetHeader(w!("Referer"), &mut value)
                            .ok()
                            .map(|_| CoTaskMemPWSTR::from(value).to_string())
                    })
                    .filter(|value| !value.is_empty() && value.len() <= 8192);
                let current = page.borrow();
                let Some(native_id) = current.native_navigation_id else {
                    return Ok(());
                };
                if current.activation != Some((current.generation, native_id)) {
                    return Ok(());
                }
                let generation = current.generation;
                drop(current);
                sink(Callback::PlayerFrame(PlayerFrameObservation {
                    kind: PlayerDocumentObservationKind::FrameNavigation,
                    owner_document_generation_at_observation: DecimalU64(generation),
                    owner_navigation_id_at_observation: DecimalU64(native_id),
                    frame_navigation_id: Some(DecimalU64(frame_navigation_id)),
                    frame_id: None,
                    parent_frame_id: None,
                    request_source_kind: None,
                    lineage_verified: false,
                    url: uri,
                    observed_unix_ms: now_ms(),
                    request_method: None,
                    referer,
                }));
                Ok(())
            }));
            let mut token = 0;
            core.add_FrameNavigationStarting(&handler, &mut token)
                .map_err(|_| {
                    error(
                        ErrorCode::EngineFailed,
                        "Native frame navigation registration failed",
                    )
                })?;
            guard.frame = Some(token);
            Ok(guard)
        }
    }
}
impl Drop for NavigationEvents {
    fn drop(&mut self) {
        self.closed.set(true);
        // SAFETY: the guard is dropped on its creating STA before the owning WebView.
        unsafe {
            if let Some(token) = self.frame {
                let _ = self.core.remove_FrameNavigationStarting(token);
            }
            if let Some(token) = self.completed {
                let _ = self.core.remove_NavigationCompleted(token);
            }
            if let Some(token) = self.starting {
                let _ = self.core.remove_NavigationStarting(token);
            }
        }
    }
}

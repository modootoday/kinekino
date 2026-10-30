//! Direct child frame handles and callbacks stay on the WebView's owning STA.
use super::{
    capture::{Callback, Page, now_ms},
    error,
    frame_completion::RepeatingFrameCompletion,
    frame_runtime::{self, CurrentFrame},
    navigation_events::EventSink,
};
use crate::application::{
    api::{ApiError, ErrorCode},
    browser_provider::*,
};
use crate::domain::api_values::DecimalU64;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use webview2_com::{
    CoTaskMemPWSTR, ExecuteScriptCompletedHandler, FrameCreatedEventHandler,
    FrameDestroyedEventHandler, FrameNavigationStartingEventHandler,
    Microsoft::Web::WebView2::Win32::*,
};
use windows::core::{BOOL, HSTRING, Interface, PWSTR};
use wry::{WebView, WebViewExtWindows};

pub(super) struct FrameEvents {
    sink: EventSink,
    core: ICoreWebView2_4,
    token: i64,
    frames: Rc<RefCell<Vec<FrameGuard>>>,
    closed: Rc<Cell<bool>>,
}
struct FrameGuard {
    runtime_current: CurrentFrame,
    frame: ICoreWebView2Frame2,
    starting: Option<i64>,
    completed: Option<i64>,
    destroyed: Option<i64>,
    closed: Rc<Cell<bool>>,
}
#[derive(Default)]
struct FramePage {
    generation: u64,
    identity: Option<FrameDocumentIdentity>,
}
impl FrameEvents {
    pub fn install(
        view: &WebView,
        page: Rc<RefCell<Page>>,
        sink: EventSink,
    ) -> Result<Self, ApiError> {
        let frames = Rc::new(RefCell::new(Vec::new()));
        let closed = Rc::new(Cell::new(false));
        let owned_frames = frames.clone();
        let is_closed = closed.clone();
        let runtime_sink = sink.clone();
        // SAFETY: registration, all frame handles and teardown use this WebView's STA.
        unsafe {
            let core = view
                .webview()
                .cast::<ICoreWebView2_4>()
                .map_err(|_| failed("Direct child frame observation is unavailable"))?;
            let handler = FrameCreatedEventHandler::create(Box::new(move |_, args| {
                if is_closed.get() {
                    return Ok(());
                }
                let Some(args) = args else {
                    return Ok(());
                };
                let owner = page.borrow();
                let Some(owner_id) = owner.native_navigation_id else {
                    return Ok(());
                };
                if owner.generation == 0 || !crate::domain::navigation::player_page(&owner.url) {
                    return Ok(());
                }
                let owner_generation = owner.generation;
                drop(owner);
                let mut guards = owned_frames.borrow_mut();
                guards.retain(|guard: &FrameGuard| !guard.closed.get());
                if guards.len() >= 8 {
                    sink(Callback::Failed(error(
                        ErrorCode::ResourceExhausted,
                        "Direct child frame limit exceeded",
                    )));
                    return Ok(());
                }
                match FrameGuard::install(
                    args.Frame()?,
                    page.clone(),
                    owner_generation,
                    owner_id,
                    sink.clone(),
                ) {
                    Ok(guard) => guards.push(guard),
                    Err(failure) => sink(Callback::Failed(failure)),
                }
                Ok(())
            }));
            let mut token = 0;
            core.add_FrameCreated(&handler, &mut token)
                .map_err(|_| failed("Direct child frame registration failed"))?;
            Ok(Self {
                sink: runtime_sink,
                core,
                token,
                frames,
                closed,
            })
        }
    }
    pub fn observe_runtime(&self, request: FrameRuntimeRequest) -> Result<(), ApiError> {
        let observed = self
            .frames
            .borrow()
            .iter()
            .find(|frame| (frame.runtime_current)(request.identity()))
            .map(|frame| (frame.frame.clone(), frame.runtime_current.clone()));
        let Some((frame, current)) = observed else {
            return Err(error(
                ErrorCode::RevisionConflict,
                "Child runtime frame is no longer current",
            ));
        };
        // Release the frame-list borrow before COM can deliver a reentrant callback.
        frame_runtime::observe(frame, request, current, self.sink.clone())
    }
}
impl FrameGuard {
    fn install(
        native: ICoreWebView2Frame,
        page: Rc<RefCell<Page>>,
        owner_generation: u64,
        owner_id: u64,
        sink: EventSink,
    ) -> Result<Self, ApiError> {
        // SAFETY: native was obtained by FrameCreated on the current STA.
        unsafe {
            let frame = native
                .cast::<ICoreWebView2Frame2>()
                .map_err(|_| failed("Direct child navigation API is unavailable"))?;
            let mut frame_id = 0;
            native
                .cast::<ICoreWebView2Frame5>()
                .and_then(|frame| frame.FrameId(&mut frame_id))
                .map_err(|_| failed("Native child frame identity is unavailable"))?;
            let state = Rc::new(RefCell::new(FramePage::default()));
            let closed = Rc::new(Cell::new(false));
            let runtime_state = state.clone();
            let runtime_page = page.clone();
            let runtime_closed = closed.clone();
            let runtime_current: CurrentFrame = Rc::new(move |identity| {
                !runtime_closed.get()
                    && owner_current(&runtime_page, owner_generation, owner_id)
                    && runtime_state.borrow().identity.as_ref() == Some(identity)
            });
            let mut guard = Self {
                runtime_current,
                frame: frame.clone(),
                starting: None,
                completed: None,
                destroyed: None,
                closed: closed.clone(),
            };
            let start_state = state.clone();
            let start_page = page.clone();
            let start_closed = closed.clone();
            let start_sink = sink.clone();
            let handler = FrameNavigationStartingEventHandler::create(Box::new(move |_, args| {
                if start_closed.get() || !owner_current(&start_page, owner_generation, owner_id) {
                    return Ok(());
                }
                let Some(args) = args else {
                    return Ok(());
                };
                let mut uri = PWSTR::null();
                args.Uri(&mut uri)?;
                let url = CoTaskMemPWSTR::from(uri).to_string();
                let mut native_id = 0;
                args.NavigationId(&mut native_id)?;
                let mut current = start_state.borrow_mut();
                let Some(generation) = current.generation.checked_add(1) else {
                    drop(current);
                    start_sink(Callback::Failed(failed(
                        "Child document generation exhausted",
                    )));
                    return Ok(());
                };
                current.generation = generation;
                let previous = current.identity.take();
                if url.len() > 8192 || !crate::domain::navigation::player_page(&url) {
                    drop(current);
                    if let Some(identity) = previous {
                        start_sink(Callback::FrameDocument(FrameDocumentEvent::Superseded(
                            identity,
                        )));
                    }
                    return Ok(());
                }
                let identity = FrameDocumentIdentity {
                    owner_document_generation: DecimalU64(owner_generation),
                    owner_native_navigation_id: DecimalU64(owner_id),
                    frame_id: DecimalU64(u64::from(frame_id)),
                    document_generation: DecimalU64(generation),
                    native_navigation_id: DecimalU64(native_id),
                    url,
                };
                current.identity = Some(identity.clone());
                drop(current);
                if let Some(previous) = previous {
                    start_sink(Callback::FrameDocument(FrameDocumentEvent::Superseded(
                        previous,
                    )));
                }
                start_sink(Callback::FrameDocument(FrameDocumentEvent::Started(
                    identity,
                )));
                Ok(())
            }));
            let mut token = 0;
            frame
                .add_NavigationStarting(&handler, &mut token)
                .map_err(|_| failed("Child navigation registration failed"))?;
            guard.starting = Some(token);
            let complete_state = state.clone();
            let complete_page = page.clone();
            let complete_closed = closed.clone();
            let complete_sink = sink.clone();
            let handler = RepeatingFrameCompletion::create(Box::new(move |sender, args| {
                if complete_closed.get()
                    || !owner_current(&complete_page, owner_generation, owner_id)
                {
                    return Ok(());
                }
                let (Some(sender), Some(args)) = (sender, args) else {
                    return Ok(());
                };
                let mut native_id = 0;
                args.NavigationId(&mut native_id)?;
                let Some(identity) = complete_state
                    .borrow()
                    .identity
                    .clone()
                    .filter(|identity| identity.native_navigation_id.0 == native_id)
                else {
                    return Ok(());
                };
                let mut success = BOOL::default();
                args.IsSuccess(&mut success)?;
                let mut web_error = COREWEBVIEW2_WEB_ERROR_STATUS(0);
                args.WebErrorStatus(&mut web_error)?;
                let status = args
                    .cast::<ICoreWebView2NavigationCompletedEventArgs2>()
                    .ok()
                    .and_then(|args| {
                        let mut value = 0;
                        args.HttpStatusCode(&mut value).ok().map(|_| value)
                    });
                complete_sink(Callback::FrameDocument(FrameDocumentEvent::Completed(
                    identity.clone(),
                    FrameNavigationDiagnostic {
                        observed_unix_ms: now_ms(),
                        http_status_code: status,
                        is_success: success.as_bool(),
                        web_error_status: web_error.0,
                    },
                )));
                let dom_state = complete_state.clone();
                let dom_page = complete_page.clone();
                let dom_closed = complete_closed.clone();
                let dom_sink = complete_sink.clone();
                let callback_identity = identity.clone();
                let handler =
                    ExecuteScriptCompletedHandler::create(Box::new(move |result, text| {
                        if dom_closed.get()
                            || !owner_current(&dom_page, owner_generation, owner_id)
                            || dom_state.borrow().identity.as_ref() != Some(&callback_identity)
                        {
                            return Ok(());
                        }
                        if result.is_err() {
                            dom_sink(Callback::Failed(failed("Child DOM serialization failed")));
                            return Ok(());
                        }
                        let json = text;
                        if json.len() > 6 * (MAX_CAPTURE_BYTES + 1 + 8192) + 64 {
                            dom_sink(Callback::Failed(error(
                                ErrorCode::ResourceExhausted,
                                "Child DOM envelope exceeds capture limit",
                            )));
                        } else {
                            dom_sink(Callback::FrameDom(callback_identity.clone(), json));
                        }
                        Ok(())
                    }));
                let script = HSTRING::from(format!(
                    "(()=>{{const root=document.documentElement;if(!root)return null;const limit={MAX_CAPTURE_BYTES};const html=root.outerHTML;return [location.href,html.slice(0,limit+1),html.length>limit];}})()"
                ));
                if sender
                    .cast::<ICoreWebView2Frame2>()
                    .and_then(|frame| frame.ExecuteScript(&script, &handler))
                    .is_err()
                {
                    complete_sink(Callback::Failed(failed(
                        "Child DOM capture could not start",
                    )));
                }
                Ok(())
            }));
            frame
                .add_NavigationCompleted(&handler, &mut token)
                .map_err(|_| failed("Child completion registration failed"))?;
            guard.completed = Some(token);
            let handler = FrameDestroyedEventHandler::create(Box::new(move |_, _| {
                if !closed.replace(true)
                    && owner_current(&page, owner_generation, owner_id)
                    && let Some(identity) = state.borrow().identity.clone()
                {
                    sink(Callback::FrameDocument(FrameDocumentEvent::Destroyed(
                        identity,
                    )));
                }
                Ok(())
            }));
            frame
                .add_Destroyed(&handler, &mut token)
                .map_err(|_| failed("Child lifetime registration failed"))?;
            guard.destroyed = Some(token);
            Ok(guard)
        }
    }
}
fn owner_current(page: &RefCell<Page>, generation: u64, navigation_id: u64) -> bool {
    let page = page.borrow();
    page.generation == generation && page.native_navigation_id == Some(navigation_id)
}
fn failed(message: &'static str) -> ApiError {
    error(ErrorCode::EngineFailed, message)
}
impl Drop for FrameGuard {
    fn drop(&mut self) {
        self.closed.set(true);
        // SAFETY: the guard never leaves its creating STA.
        unsafe {
            if let Some(token) = self.destroyed {
                let _ = self.frame.remove_Destroyed(token);
            }
            if let Some(token) = self.completed {
                let _ = self.frame.remove_NavigationCompleted(token);
            }
            if let Some(token) = self.starting {
                let _ = self.frame.remove_NavigationStarting(token);
            }
        }
    }
}
impl Drop for FrameEvents {
    fn drop(&mut self) {
        self.closed.set(true);
        // SAFETY: unregister before releasing the owned child frames and WebView.
        unsafe {
            let _ = self.core.remove_FrameCreated(self.token);
        }
        self.frames.borrow_mut().clear();
    }
}

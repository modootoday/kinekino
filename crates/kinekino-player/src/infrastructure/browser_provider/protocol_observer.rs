//! Owned CDP adapter: all receivers and calls remain on the owning WebView STA.
use super::{capture::now_ms, error};
use crate::application::{
    api::{ApiError, ErrorCode},
    browser_network::{MAX_RESPONSE_BYTES, ProtocolSession},
    browser_provider::OwnedResponseCapture,
};
use crate::domain::api_values::DecimalU64;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::mpsc::{self, Receiver, SyncSender},
};
use webview2_com::{
    CallDevToolsProtocolMethodCompletedHandler, CoTaskMemPWSTR,
    DevToolsProtocolEventReceivedEventHandler, Microsoft::Web::WebView2::Win32::*,
};
use windows::core::{HSTRING, Interface, PWSTR};
use wry::{WebView, WebViewExtWindows};
#[path = "protocol_targets.rs"]
mod targets;

const EVENTS: &[&str] = &[
    "Target.attachedToTarget",
    "Target.detachedFromTarget",
    "Page.frameAttached",
    "Page.frameNavigated",
    "Page.frameDetached",
    "Network.requestWillBeSent",
    "Network.requestWillBeSentExtraInfo",
    "Network.responseReceived",
    "Network.loadingFinished",
    "Network.loadingFailed",
];

pub(super) enum ProtocolMessage {
    Event {
        received_unix_ms: DecimalU64,
        name: &'static str,
        session: ProtocolSession,
        json: String,
    },
    Body {
        read: Box<OwnedResponseCapture>,
        json: String,
        observed_unix_ms: DecimalU64,
    },
    BodyUnavailable {
        read: Box<OwnedResponseCapture>,
    },
}
struct ReceiverGuard {
    receiver: ICoreWebView2DevToolsProtocolEventReceiver,
    token: i64,
}
struct Shared {
    closed: Cell<bool>,
    failed: Cell<bool>,
    enabled: RefCell<Vec<(ProtocolSession, u8)>>,
    pending_bodies: Cell<u8>,
    overflow: Cell<bool>,
    failure: RefCell<Option<ApiError>>,
    sender: SyncSender<ProtocolMessage>,
}
impl Shared {
    fn send(&self, message: ProtocolMessage) {
        if !self.closed.get() && self.sender.try_send(message).is_err() {
            self.overflow.set(true);
        }
    }
    fn fail(&self, message: &'static str) {
        self.failed.set(true);
        if !self.closed.get() && self.failure.borrow().is_none() {
            *self.failure.borrow_mut() = Some(error(ErrorCode::EngineFailed, message));
        }
    }
}
pub(super) struct ProtocolObserver {
    targets: RefCell<Vec<targets::TargetInitialization>>,
    receivers: Vec<ReceiverGuard>,
    core: ICoreWebView2,
    state: Rc<Shared>,
    queue: Receiver<ProtocolMessage>,
}
impl ProtocolObserver {
    pub fn install(view: &WebView) -> Result<Self, ApiError> {
        let (sender, queue) = mpsc::sync_channel(256);
        let state = Rc::new(Shared {
            closed: Cell::new(false),
            failed: Cell::new(false),
            enabled: RefCell::new(Vec::new()),
            pending_bodies: Cell::new(0),
            overflow: Cell::new(false),
            failure: RefCell::new(None),
            sender,
        });
        // SAFETY: this guard, callbacks, receivers and the WebView share their creating STA.
        unsafe {
            let core = view.webview();
            let mut observer = Self {
                targets: RefCell::new(Vec::new()),
                receivers: Vec::new(),
                core,
                state,
                queue,
            };
            for &name in EVENTS {
                let receiver = observer
                    .core
                    .GetDevToolsProtocolEventReceiver(&HSTRING::from(name))
                    .map_err(|_| {
                        error(
                            ErrorCode::EngineFailed,
                            "Owned protocol event receiver unavailable",
                        )
                    })?;
                let shared = observer.state.clone();
                let handler =
                    DevToolsProtocolEventReceivedEventHandler::create(Box::new(move |_, args| {
                        if shared.closed.get() {
                            return Ok(());
                        }
                        let Some(args) = args else {
                            return Ok(());
                        };
                        let session = args
                            .cast::<ICoreWebView2DevToolsProtocolEventReceivedEventArgs2>()
                            .ok()
                            .and_then(|args| {
                                let mut value = PWSTR::null();
                                args.SessionId(&mut value)
                                    .ok()
                                    .map(|_| CoTaskMemPWSTR::from(value).to_string())
                            });
                        let session = match ProtocolSession::from_observed(session.as_deref()) {
                            Ok(session) => session,
                            Err(_) => {
                                shared.fail("Invalid owned protocol target identity");
                                return Ok(());
                            }
                        };
                        let mut json = PWSTR::null();
                        args.ParameterObjectAsJson(&mut json)?;
                        let json = CoTaskMemPWSTR::from(json).to_string();
                        if json.len() > 128 * 1024 {
                            shared.fail("Owned protocol event exceeds limit");
                        } else {
                            shared.send(ProtocolMessage::Event {
                                received_unix_ms: now_ms(),
                                name,
                                session,
                                json,
                            });
                        }
                        Ok(())
                    }));
                let mut token = 0;
                receiver
                    .add_DevToolsProtocolEventReceived(&handler, &mut token)
                    .map_err(|_| {
                        error(
                            ErrorCode::EngineFailed,
                            "Owned protocol registration failed",
                        )
                    })?;
                observer.receivers.push(ReceiverGuard { receiver, token });
            }
            observer.enable_session(ProtocolSession::Default)?;
            observer.enable(
                ProtocolSession::Default,
                "Target.setAutoAttach",
                r#"{"autoAttach":true,"waitForDebuggerOnStart":true,"flatten":true,"filter":[{"type":"iframe"},{"exclude":true}]}"#,
            )?;
            Ok(observer)
        }
    }
    pub fn enable_session(&self, session: ProtocolSession) -> Result<(), ApiError> {
        if matches!(session, ProtocolSession::Unavailable) {
            return Err(error(
                ErrorCode::Unsupported,
                "Protocol target identity was not observed",
            ));
        }
        {
            let mut sessions = self.state.enabled.borrow_mut();
            if sessions.iter().any(|(known, _)| *known == session) {
                return Ok(());
            }
            if sessions.len() >= 9 {
                return Err(error(
                    ErrorCode::ResourceExhausted,
                    "Owned protocol target limit exceeded",
                ));
            }
            sessions.push((session.clone(), 0));
        }
        self.enable(session.clone(), "Page.enable", "{}")?;
        self.enable(
            session,
            "Network.enable",
            &format!(
                "{{\"maxTotalBufferSize\":{},\"maxResourceBufferSize\":{}}}",
                MAX_RESPONSE_BYTES * 2,
                MAX_RESPONSE_BYTES
            ),
        )
    }
    fn enable(
        &self,
        session: ProtocolSession,
        method: &str,
        parameters: &str,
    ) -> Result<(), ApiError> {
        let state = self.state.clone();
        let completed_session = session.clone();
        let completed =
            CallDevToolsProtocolMethodCompletedHandler::create(Box::new(move |result, _| {
                if !state.closed.get()
                    && state
                        .enabled
                        .borrow()
                        .iter()
                        .any(|(known, _)| known == &completed_session)
                {
                    if result.is_err() {
                        state.fail("Owned protocol initialization failed");
                    } else if let Some((_, enabled)) = state
                        .enabled
                        .borrow_mut()
                        .iter_mut()
                        .find(|(known, _)| *known == completed_session)
                    {
                        *enabled = enabled.saturating_add(1);
                    }
                }
                Ok(())
            }));
        // SAFETY: only fixed enable commands are called on the owning STA.
        unsafe {
            match session {
                ProtocolSession::Default => self.core.CallDevToolsProtocolMethod(
                    &HSTRING::from(method),
                    &HSTRING::from(parameters),
                    &completed,
                ),
                ProtocolSession::Target(session) => {
                    self.core.cast::<ICoreWebView2_11>().and_then(|core| {
                        core.CallDevToolsProtocolMethodForSession(
                            &HSTRING::from(session),
                            &HSTRING::from(method),
                            &HSTRING::from(parameters),
                            &completed,
                        )
                    })
                }
                ProtocolSession::Unavailable => unreachable!("Rejected before initialization"),
            }
        }
        .map_err(|_| {
            error(
                ErrorCode::EngineFailed,
                "Owned protocol initialization could not start",
            )
        })
    }
    pub fn ready(&self) -> bool {
        self.session_ready(&ProtocolSession::Default)
    }
    fn session_ready(&self, session: &ProtocolSession) -> bool {
        self.state.enabled.borrow().iter().any(|(known, enabled)| {
            known == session
                && *enabled
                    == if *session == ProtocolSession::Default {
                        3
                    } else {
                        2
                    }
        }) && !self.state.failed.get()
            && self.state.failure.borrow().is_none()
            && !self.state.closed.get()
    }
    pub fn take_failure(&self) -> Option<ApiError> {
        if self.state.overflow.get() {
            return Some(error(
                ErrorCode::ResourceExhausted,
                "Owned protocol observation queue overflow",
            ));
        }
        self.state.failure.borrow_mut().take()
    }
    pub fn next(&self) -> Option<ProtocolMessage> {
        self.queue.try_recv().ok()
    }
    pub fn body_capacity(&self) -> bool {
        self.ready() && self.state.pending_bodies.get() < 4
    }
    pub fn capture_body(&self, read: OwnedResponseCapture) -> Result<(), ApiError> {
        let session = read.session.clone();
        if !self.ready() || !self.session_ready(&session) {
            return Err(error(
                ErrorCode::RevisionConflict,
                "Owned protocol observer is not ready",
            ));
        }
        if self.state.pending_bodies.get() >= 4 {
            return Err(error(
                ErrorCode::ResourceExhausted,
                "Owned protocol body queue limit exceeded",
            ));
        }
        if matches!(session, ProtocolSession::Unavailable) {
            return Err(error(
                ErrorCode::Unsupported,
                "Protocol target identity was not observed",
            ));
        }
        let parameters = serde_json::json!({"requestId":read.capture.request_id()}).to_string();
        let state = self.state.clone();
        let called = Rc::new(Cell::new(false));
        let callback_called = called.clone();
        let completed =
            CallDevToolsProtocolMethodCompletedHandler::create(Box::new(move |result, json| {
                if callback_called.replace(true) {
                    return Ok(());
                }
                state
                    .pending_bodies
                    .set(state.pending_bodies.get().saturating_sub(1));
                if state.closed.get() {
                    return Ok(());
                }
                if result.is_err() {
                    state.send(ProtocolMessage::BodyUnavailable {
                        read: Box::new(read),
                    });
                } else if json.len() > MAX_RESPONSE_BYTES * 2 + 1024 {
                    state.fail("Owned protocol body envelope exceeds limit");
                } else {
                    state.send(ProtocolMessage::Body {
                        read: Box::new(read),
                        json,
                        observed_unix_ms: now_ms(),
                    });
                }
                Ok(())
            }));
        self.state
            .pending_bodies
            .set(self.state.pending_bodies.get() + 1);
        // SAFETY: fixed body-read command and observed opaque session/request IDs only; no navigation or eval.
        let result = unsafe {
            match session {
                ProtocolSession::Default => self.core.CallDevToolsProtocolMethod(
                    &HSTRING::from("Network.getResponseBody"),
                    &HSTRING::from(parameters),
                    &completed,
                ),
                ProtocolSession::Target(session) => {
                    self.core.cast::<ICoreWebView2_11>().and_then(|core| {
                        core.CallDevToolsProtocolMethodForSession(
                            &HSTRING::from(session),
                            &HSTRING::from("Network.getResponseBody"),
                            &HSTRING::from(parameters),
                            &completed,
                        )
                    })
                }
                ProtocolSession::Unavailable => {
                    unreachable!("Rejected before calling native protocol")
                }
            }
        };
        if result.is_err() {
            if !called.replace(true) {
                self.state
                    .pending_bodies
                    .set(self.state.pending_bodies.get().saturating_sub(1));
            }
            return Err(error(
                ErrorCode::EngineFailed,
                "Owned protocol body read could not start",
            ));
        }
        Ok(())
    }
}
impl Drop for ReceiverGuard {
    fn drop(&mut self) {
        // SAFETY: unregister on the receiver's creating STA.
        unsafe {
            let _ = self
                .receiver
                .remove_DevToolsProtocolEventReceived(self.token);
        }
    }
}
impl Drop for ProtocolObserver {
    fn drop(&mut self) {
        self.state.closed.set(true);
        self.receivers.clear();
    }
}

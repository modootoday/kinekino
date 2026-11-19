//! DevTools protocol pass-through over WebView2 (ADR 20261109071720 decision 2).
//!
//! WebView2 completes protocol calls through callbacks on the WebView's STA thread. The port's
//! `call` is synchronous, so it waits for the completion while pumping that thread's messages.
//! Events come from one receiver per name in [`EVENTS`] and are buffered until drained.
use crate::engine::port::{EngineError, EngineResult, ProtocolEvent};
use std::{cell::RefCell, rc::Rc, sync::mpsc};
use webview2_com::{
    CallDevToolsProtocolMethodCompletedHandler, CoTaskMemPWSTR,
    DevToolsProtocolEventReceivedEventHandler, Microsoft::Web::WebView2::Win32::*,
};
use windows::core::{HSTRING, Interface, PWSTR};

/// Events forwarded to callers: the set the browser provider reads, plus runtime contexts so
/// a caller can tell when an auto-attached child is ready.
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
    "Runtime.executionContextCreated",
];

/// Upper bound for one event's parameters, as in the provider's own observer.
const MAX_EVENT_JSON: usize = 128 * 1024;

struct Registration {
    receiver: ICoreWebView2DevToolsProtocolEventReceiver,
    token: i64,
}

pub(super) struct WebView2Protocol {
    core: ICoreWebView2,
    registrations: Vec<Registration>,
    events: Rc<RefCell<Vec<ProtocolEvent>>>,
}

impl WebView2Protocol {
    /// Registers the event receivers. Must run on the WebView's STA thread.
    pub(super) fn install(core: ICoreWebView2) -> EngineResult<Self> {
        let mut protocol = Self {
            core,
            registrations: Vec::new(),
            events: Rc::new(RefCell::new(Vec::new())),
        };
        for &name in EVENTS {
            protocol.register(name)?;
        }
        Ok(protocol)
    }

    fn register(&mut self, name: &'static str) -> EngineResult<()> {
        let events = self.events.clone();
        let handler =
            DevToolsProtocolEventReceivedEventHandler::create(Box::new(move |_, args| {
                let Some(args) = args else {
                    return Ok(());
                };
                let mut json = PWSTR::null();
                // SAFETY: the args object is valid for this callback on the owning STA.
                unsafe { args.ParameterObjectAsJson(&mut json)? };
                let json = CoTaskMemPWSTR::from(json).to_string();
                if json.len() > MAX_EVENT_JSON {
                    return Ok(());
                }
                let Ok(params) = serde_json::from_str(&json) else {
                    return Ok(());
                };
                let session = args
                    .cast::<ICoreWebView2DevToolsProtocolEventReceivedEventArgs2>()
                    .ok()
                    .and_then(|args| {
                        let mut value = PWSTR::null();
                        // SAFETY: as above.
                        unsafe { args.SessionId(&mut value) }
                            .ok()
                            .map(|_| CoTaskMemPWSTR::from(value).to_string())
                    })
                    .filter(|id| !id.is_empty());
                events.borrow_mut().push(ProtocolEvent {
                    session,
                    method: name.to_owned(),
                    params,
                });
                Ok(())
            }));
        // SAFETY: receivers and handlers are created and registered on the owning STA.
        unsafe {
            let receiver = self
                .core
                .GetDevToolsProtocolEventReceiver(&HSTRING::from(name))
                .map_err(windows_error)?;
            let mut token = 0;
            receiver
                .add_DevToolsProtocolEventReceived(&handler, &mut token)
                .map_err(windows_error)?;
            self.registrations.push(Registration { receiver, token });
        }
        Ok(())
    }

    pub(super) fn call(
        &self,
        session: Option<&str>,
        method: &str,
        params: serde_json::Value,
    ) -> EngineResult<serde_json::Value> {
        let params =
            serde_json::to_string(&params).map_err(|e| EngineError::Call(e.to_string()))?;
        let (tx, rx) = mpsc::channel();
        let completed =
            CallDevToolsProtocolMethodCompletedHandler::create(Box::new(move |result, json| {
                let _ = tx.send(result.map(|_| json));
                Ok(())
            }));
        let method_name = HSTRING::from(method);
        let params = HSTRING::from(params);
        // SAFETY: called on the owning STA; the completion runs there while we pump below.
        unsafe {
            match session {
                None => self
                    .core
                    .CallDevToolsProtocolMethod(&method_name, &params, &completed),
                Some(id) => self.core.cast::<ICoreWebView2_11>().and_then(|core| {
                    core.CallDevToolsProtocolMethodForSession(
                        &HSTRING::from(id),
                        &method_name,
                        &params,
                        &completed,
                    )
                }),
            }
        }
        .map_err(windows_error)?;
        let json = webview2_com::wait_with_pump(rx)
            .map_err(|e| EngineError::Call(format!("{method}: {e:?}")))?
            .map_err(|e| EngineError::Call(format!("{method}: {e}")))?;
        serde_json::from_str(&json).map_err(|e| EngineError::Call(format!("{method}: {e}")))
    }

    pub(super) fn drain(&self) -> Vec<ProtocolEvent> {
        std::mem::take(&mut *self.events.borrow_mut())
    }
}

impl Drop for WebView2Protocol {
    fn drop(&mut self) {
        for registration in self.registrations.drain(..) {
            // SAFETY: removal runs on the owning STA with the token registration returned.
            unsafe {
                let _ = registration
                    .receiver
                    .remove_DevToolsProtocolEventReceived(registration.token);
            }
        }
    }
}

fn windows_error(e: windows::core::Error) -> EngineError {
    EngineError::Call(e.to_string())
}

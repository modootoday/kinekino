//! The OS window: Meari's native driver (winit + softbuffer) hosting the settings
//! document. The same code runs on every OS; only winit/blitz pick the backend.
use crate::session::Session;
use kinekino_core::platform::settings_window::{SettingsWindowPort, WindowReply};
use meari::application::host_callbacks::PendingCallback;
use meari::domain::Value as MeariValue;
use meari_native::application::{Renderer, WindowOptions};
use meari_native::infrastructure::native_host::{
    CloseDecision, CloseRequest, HostEffects, HostLifecycle, HostStamp, HostTurn, IntentAdmission,
    NativeDocumentHandle, NativeHostRuntime, NativeWindowHost,
};
use meari_native::infrastructure::window::NativeWindowDriver;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::io;
use std::path::PathBuf;
use std::rc::Rc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

/// Settings window adapter used by `kinekino settings gui` on all OSes.
#[derive(Debug, Default, Clone, Copy)]
pub struct MeariSettingsWindow;

#[derive(Debug)]
pub struct HostError(String);
impl std::fmt::Display for HostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for HostError {}

/// Work the UI loop hands to the host's next document turn.
#[derive(Default)]
struct Pending {
    props: Option<BTreeMap<String, MeariValue>>,
    reset: bool,
    capture: Option<PathBuf>,
}

struct Shared {
    session: Session,
    handle: NativeDocumentHandle,
    outbox: VecDeque<String>,
    pending: Pending,
    failure: Option<String>,
    captured: bool,
}

struct Host(Rc<RefCell<Shared>>);

impl Shared {
    /// Collects clicks since the last turn; reset is local, the rest go to the controller.
    fn collect(&mut self) {
        let handle = self.handle.clone();
        match handle.try_with(|doc| self.session.messages(doc)) {
            Ok(Ok(messages)) => {
                for message in messages {
                    if message.contains(r#""op":"reset""#) {
                        self.pending.reset = true;
                    } else {
                        self.outbox.push_back(message);
                    }
                }
            }
            Ok(Err(e)) => self.failure = Some(e),
            Err(e) => self.failure = Some(e.to_string()),
        }
    }
    /// Applies queued props/reset/capture in one synchronous document commit.
    fn commit(&mut self, stamp: &HostStamp) -> Result<HostEffects, HostError> {
        if self.pending.props.is_none() && !self.pending.reset && self.pending.capture.is_none() {
            return Ok(HostEffects::default());
        }
        let pending = std::mem::take(&mut self.pending);
        let session = &self.session;
        let (captured, receipt) = self
            .handle
            .try_reproject(stamp, |doc| -> Result<bool, String> {
                if let Some(props) = pending.props {
                    doc.update_module_props(props)?;
                }
                if pending.reset {
                    session.reset(doc)?;
                }
                match pending.capture {
                    Some(path) => {
                        let viewport = doc.viewport.clone();
                        let rgba = doc.renderer.rasterize(&viewport)?;
                        crate::write_png(&path, viewport.width, viewport.height, &rgba)?;
                        Ok(true)
                    }
                    None => Ok(false),
                }
            })
            .map_err(|e| HostError(format!("{e:?}")))?;
        self.captured |= captured;
        Ok(HostEffects {
            committed: Some(receipt),
            redraw: true,
        })
    }
}

impl NativeWindowHost for Host {
    type Error = HostError;
    type CloseCheckpoint = ();
    fn admit_callback(&mut self, _: &HostStamp, _: &PendingCallback) -> IntentAdmission<HostError> {
        IntentAdmission::Rejected(HostError("settings window has no host callbacks".into()))
    }
    fn after_dispatch(&mut self, turn: HostTurn) -> Result<HostEffects, HostError> {
        let mut shared = self.0.borrow_mut();
        shared.collect();
        shared.commit(&turn.stamp)
    }
    fn close(&mut self, _: CloseRequest<()>) -> CloseDecision<(), HostError> {
        CloseDecision::HostCleanupComplete { checkpoint: () }
    }
}

/// Smoke steps: wait for the first frame, apply the invalid batch, report and close.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Smoke {
    Off,
    FirstFrame,
    Applied(u64),
    Reported,
}

struct App<'a> {
    driver: NativeWindowDriver,
    shared: Rc<RefCell<Shared>>,
    on_message: &'a mut dyn FnMut(&str) -> WindowReply,
    smoke: Smoke,
    closing: Option<PathBuf>,
    dismissed: bool,
}

impl App<'_> {
    fn send(&mut self, message: &str) -> Result<(), String> {
        let reply = (self.on_message)(message);
        let mut shared = self.shared.borrow_mut();
        if let Some(json) = reply.json.as_deref() {
            let body: Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
            let (page, errors): (Option<Value>, Vec<String>) = match body["op"].as_str() {
                Some("dialog") => {
                    self.dismissed = true;
                    (None, Vec::new())
                }
                _ if body["ok"] == false => (
                    None,
                    body["errors"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|e| e.as_str().map(str::to_owned))
                        .collect(),
                ),
                _ => (body.get("page").cloned(), Vec::new()),
            };
            let mut props = shared.session.update_props(page, &errors);
            if self.dismissed {
                props.insert("dialogOpen".into(), MeariValue::Bool(false));
            }
            shared.pending.props = Some(props);
        }
        if reply.close {
            if let Some(path) = reply.capture_png {
                shared.pending.capture = Some(path.clone());
                self.closing = Some(path);
            } else {
                self.closing = Some(PathBuf::new());
            }
        }
        Ok(())
    }

    fn tick(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.shared.borrow().handle.lifecycle() == HostLifecycle::Closed {
            event_loop.exit();
            return;
        }
        if let Err(e) = self.advance() {
            self.shared.borrow_mut().failure = Some(e);
        }
        let shared = self.shared.borrow();
        if shared.failure.is_some() || shared.handle.lifecycle() == HostLifecycle::Closed {
            event_loop.exit();
        }
    }

    fn advance(&mut self) -> Result<(), String> {
        let wake = self.driver.wake_handle();
        let frame = self
            .driver
            .last_submission()
            .map(|f| f.ticket.scene_generation);
        match self.smoke {
            Smoke::FirstFrame if frame.is_some() => {
                self.send(&format!(
                    r#"{{"op":"apply","edits":{}}}"#,
                    crate::SMOKE_EDITS
                ))?;
                self.smoke = Smoke::Applied(frame.unwrap_or_default());
            }
            Smoke::Applied(first) if frame.is_some_and(|f| f > first) => {
                let report = {
                    let shared = self.shared.borrow();
                    shared
                        .handle
                        .try_with(|doc| shared.session.rendered(doc))
                        .map_err(|e| e.to_string())?
                };
                let mut ready = report;
                ready["op"] = "ready".into();
                self.send(&ready.to_string())?;
                self.smoke = Smoke::Reported;
            }
            _ => (),
        }
        loop {
            let next = self.shared.borrow_mut().outbox.pop_front();
            let Some(message) = next else { break };
            self.send(&message)?;
        }
        let shared = self.shared.borrow();
        let queued = shared.pending.props.is_some()
            || shared.pending.reset
            || shared.pending.capture.is_some();
        drop(shared);
        if queued {
            wake.wake().map_err(|e| e.to_string())?;
        } else if let Some(path) = &self.closing
            && (path.as_os_str().is_empty() || self.shared.borrow().captured)
        {
            wake.request_close().map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

impl ApplicationHandler for App<'_> {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.driver.can_create_surfaces(event_loop);
        self.tick(event_loop);
    }
    fn destroy_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.driver.destroy_surfaces(event_loop);
    }
    fn window_event(&mut self, event_loop: &dyn ActiveEventLoop, id: WindowId, event: WindowEvent) {
        self.driver.window_event(event_loop, id, event);
        self.tick(event_loop);
    }
    fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.driver.proxy_wake_up(event_loop);
        self.tick(event_loop);
    }
    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.tick(event_loop);
    }
}

fn other(e: impl std::fmt::Display) -> io::Error {
    io::Error::other(e.to_string())
}

impl SettingsWindowPort for MeariSettingsWindow {
    fn run(
        &self,
        title: &str,
        smoke: bool,
        on_message: &mut dyn FnMut(&str) -> WindowReply,
    ) -> io::Result<()> {
        let reply = on_message(r#"{"op":"page"}"#);
        let body: Value =
            serde_json::from_str(reply.json.as_deref().unwrap_or("null")).map_err(other)?;
        let Some(page) = body.get("page").cloned() else {
            // No page model (e.g. unreadable config): report like the page would and stop.
            let errors = body["errors"].clone();
            if smoke {
                on_message(&serde_json::json!({"op":"ready","rendered":0,"dialog":false,"errors_shown":errors}).to_string());
            }
            return Err(other(format!("settings page unavailable: {errors}")));
        };
        let (session, document) = Session::mount(page).map_err(other)?;
        let handle = NativeDocumentHandle::new(document, 1).map_err(other)?;
        let shared = Rc::new(RefCell::new(Shared {
            session,
            handle: handle.clone(),
            outbox: VecDeque::new(),
            pending: Pending::default(),
            failure: None,
            captured: false,
        }));
        let runtime = Rc::new(RefCell::new(NativeHostRuntime::new(
            handle,
            Host(shared.clone()),
        )));
        let mut event_loop = blitz_shell::create_default_event_loop();
        let driver = NativeWindowDriver::attach(
            &event_loop,
            runtime,
            WindowOptions {
                title: title.into(),
            },
        )
        .map_err(other)?;
        let mut app = App {
            driver,
            shared: shared.clone(),
            on_message,
            smoke: if smoke { Smoke::FirstFrame } else { Smoke::Off },
            closing: None,
            dismissed: false,
        };
        // `run_app` needs a 'static handler; the borrowed `on_message` callback needs on-demand.
        winit::event_loop::run_on_demand::EventLoopExtRunOnDemand::run_app_on_demand(
            &mut event_loop,
            &mut app,
        )
        .map_err(other)?;
        let failure = shared.borrow_mut().failure.take();
        failure.map_or(Ok(()), |e| Err(other(e)))
    }
}

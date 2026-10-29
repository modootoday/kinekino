//! CEF adapter (feature `cef`): official CEF via cef-rs, created as a native child window of
//! a host-owned window (X11 XID on Linux, HWND on Windows; Windows path unverified).
//!
//! Single-threaded: the owner thread pumps CEF (`external_message_pump`), so every call here
//! blocks while pumping until its result arrives. Build needs `CEF_PATH`; run needs
//! `KINEKINO_CEF_DIR` (libcef + resources). The Widevine CDM is never bundled: CEF's
//! component updater downloads it into the cache at runtime.
mod config;
mod handlers;
mod runtime;

pub use config::CefLaunch;
pub use runtime::subprocess_entry;

use crate::engine::port::{
    AudioControl, BrowserEngine, BrowserEvent, EngineError, EngineKind, EngineResult,
    NetworkInterceptor,
};
use cef::{Browser, CefString, ImplBrowser, ImplBrowserHost, ImplFrame};
use handlers::Shared;
use std::time::Duration;

const LOAD_TIMEOUT: Duration = Duration::from_secs(30);
const EVAL_TIMEOUT: Duration = Duration::from_secs(10);

pub struct CefEngine {
    browser: Browser,
    shared: Shared,
    next_eval: u64,
}

impl CefEngine {
    /// Initializes CEF in this (browser) process and creates one child browser.
    /// Call [`subprocess_entry`] first thing in `main`.
    pub fn launch(spec: &CefLaunch) -> EngineResult<Self> {
        let shared = Shared::default();
        runtime::initialize(spec, &shared)?;
        let browser = runtime::create_browser(spec, &shared)?;
        let mut engine = Self {
            browser,
            shared,
            next_eval: 0,
        };
        engine.wait_loaded(0)?;
        Ok(engine)
    }

    /// Notifies CEF that the parent window was resized.
    pub fn resized(&self) {
        if let Some(host) = self.browser.host() {
            host.was_resized();
        }
    }

    /// Keeps CEF responsive for `d` (e.g. while a demo window stays on screen).
    pub fn pump_for(&self, d: Duration) {
        runtime::pump_until(d, || false);
    }

    /// Closes the browser and shuts CEF down (once per process).
    pub fn close(self) {
        if let Some(host) = self.browser.host() {
            host.close_browser(1);
        }
        runtime::pump_until(Duration::from_secs(3), || self.shared.closed());
        drop(self.browser);
        cef::shutdown();
    }

    fn wait_loaded(&mut self, after: u64) -> EngineResult<()> {
        let ok = runtime::pump_until(LOAD_TIMEOUT, || self.shared.load_seq() > after);
        if ok {
            Ok(())
        } else {
            Err(EngineError::Call("cef: page load timed out".into()))
        }
    }
}

impl BrowserEngine for CefEngine {
    fn kind(&self) -> EngineKind {
        EngineKind::Cef
    }

    fn navigate(&mut self, url: &str) -> EngineResult<()> {
        self.shared
            .push(BrowserEvent::NavigationStarted { url: url.into() });
        let frame = self
            .browser
            .main_frame()
            .ok_or_else(|| EngineError::Call("cef: no main frame".into()))?;
        let seq = self.shared.load_seq();
        frame.load_url(Some(&CefString::from(url)));
        self.wait_loaded(seq)
    }

    /// Runs the expression in an async wrapper and reads the JSON result back from a
    /// prefixed console message (CEF has no browser-side eval-with-result).
    fn eval(&mut self, script: &str) -> EngineResult<serde_json::Value> {
        self.next_eval += 1;
        let id = self.next_eval;
        let frame = self
            .browser
            .main_frame()
            .ok_or_else(|| EngineError::Call("cef: no main frame".into()))?;
        let code = handlers::eval_wrapper(id, script);
        frame.execute_java_script(Some(&CefString::from(code.as_str())), None, 0);
        let mut out = None;
        runtime::pump_until(EVAL_TIMEOUT, || {
            out = self.shared.take_eval(id);
            out.is_some()
        });
        out.ok_or_else(|| EngineError::Call("cef: eval timed out".into()))?
    }

    fn inject_script(&mut self, _source: &str) -> EngineResult<()> {
        Err(EngineError::Unsupported(
            "cef adapter: script injection not implemented",
        ))
    }

    fn set_interceptor(&mut self, _i: Box<dyn NetworkInterceptor>) -> EngineResult<()> {
        Err(EngineError::Unsupported(
            "cef adapter: request interception not implemented",
        ))
    }

    fn drain_events(&mut self) -> Vec<BrowserEvent> {
        self.shared.drain()
    }

    fn audio(&mut self) -> Option<&mut dyn AudioControl> {
        Some(self)
    }
}

impl AudioControl for CefEngine {
    fn set_muted(&mut self, muted: bool) -> EngineResult<()> {
        let host = self
            .browser
            .host()
            .ok_or_else(|| EngineError::Call("cef: no host".into()))?;
        host.set_audio_muted(i32::from(muted));
        Ok(())
    }

    fn is_muted(&mut self) -> EngineResult<bool> {
        let host = self
            .browser
            .host()
            .ok_or_else(|| EngineError::Call("cef: no host".into()))?;
        Ok(host.is_audio_muted() == 1)
    }
}

//! WebView2 adapter (Windows default engine, ADR 0002) over wry.
//! Declares media, WindowEmbed and the DevTools protocol session. eval returns Null, so `Eval`
//! stays undeclared. Built only for Windows; the contract run on a real WebView2 is pending.
use crate::engine::port::{
    BrowserEngine, BrowserEvent, EngineError, EngineKind, EngineResult, NetworkInterceptor,
    ProtocolEvent, ProtocolSession,
};
use wry::{WebView, WebViewExtWindows};

mod protocol;

pub struct WebView2Engine {
    // Declared before `view` so the receivers are removed while the WebView is alive.
    protocol: protocol::WebView2Protocol,
    view: WebView,
    events: Vec<BrowserEvent>,
}

impl WebView2Engine {
    /// Wraps a WebView built on the calling thread, which must be its STA UI thread.
    pub fn from_view(view: WebView) -> EngineResult<Self> {
        let protocol = protocol::WebView2Protocol::install(view.webview())?;
        Ok(Self {
            protocol,
            view,
            events: Vec::new(),
        })
    }
}

impl BrowserEngine for WebView2Engine {
    fn kind(&self) -> EngineKind {
        EngineKind::WebView2
    }

    fn navigate(&mut self, url: &str) -> EngineResult<()> {
        self.events
            .push(BrowserEvent::NavigationStarted { url: url.into() });
        self.view
            .load_url(url)
            .map_err(|e| EngineError::Call(e.to_string()))
    }

    fn eval(&mut self, script: &str) -> EngineResult<serde_json::Value> {
        // Result-returning eval stays in kinekino-player's callback path until migrated.
        self.view
            .evaluate_script(script)
            .map_err(|e| EngineError::Call(e.to_string()))?;
        Ok(serde_json::Value::Null)
    }

    fn inject_script(&mut self, _source: &str) -> EngineResult<()> {
        Err(EngineError::Unsupported(
            "webview2 init scripts are set at build time",
        ))
    }

    fn set_interceptor(&mut self, _i: Box<dyn NetworkInterceptor>) -> EngineResult<()> {
        Err(EngineError::Unsupported(
            "webview2 interception still lives in kinekino-player",
        ))
    }

    fn drain_events(&mut self) -> Vec<BrowserEvent> {
        std::mem::take(&mut self.events)
    }

    fn protocol(&mut self) -> Option<&mut dyn ProtocolSession> {
        Some(self)
    }
}

impl ProtocolSession for WebView2Engine {
    fn call(
        &mut self,
        session: Option<&str>,
        method: &str,
        params: serde_json::Value,
    ) -> EngineResult<serde_json::Value> {
        self.protocol.call(session, method, params)
    }

    fn drain_protocol_events(&mut self) -> Vec<ProtocolEvent> {
        self.protocol.drain()
    }

    fn set_auto_attach(&mut self, on: bool) -> EngineResult<()> {
        let params = serde_json::json!({
            "autoAttach": on,
            "waitForDebuggerOnStart": on,
            "flatten": true,
        });
        self.call(None, "Target.setAutoAttach", params).map(|_| ())
    }
}

//! WebView2 adapter (Windows default engine, ADR 0002) over wry.
//! Declares media + WindowEmbed only; eval returns Null, so `Eval` stays undeclared.
//! NOT compiled or tested on Linux; Windows verification pending.
use crate::engine::port::{
    BrowserEngine, BrowserEvent, EngineError, EngineKind, EngineResult, NetworkInterceptor,
};
use wry::WebView;

pub struct WebView2Engine {
    view: WebView,
    events: Vec<BrowserEvent>,
}

impl WebView2Engine {
    pub fn from_view(view: WebView) -> Self {
        Self {
            view,
            events: Vec::new(),
        }
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
}

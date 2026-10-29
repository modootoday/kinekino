//! CEF callbacks that record into shared state read by the pumping owner thread.
use crate::engine::port::{BrowserEvent, EngineError, EngineResult};
use cef::*;
use std::collections::HashMap;
use std::os::raw::c_int;
use std::sync::{Arc, Mutex};

const EVAL_PREFIX: &str = "__kinekino_eval:";

#[derive(Default)]
struct State {
    context_ready: bool,
    load_seq: u64,
    closed: bool,
    events: Vec<BrowserEvent>,
    evals: HashMap<u64, EngineResult<serde_json::Value>>,
}

/// Cheap shared handle; CEF wrapper fields must be `Clone`.
#[derive(Clone, Default)]
pub(super) struct Shared(Arc<Mutex<State>>);

impl Shared {
    fn with<T>(&self, f: impl FnOnce(&mut State) -> T) -> T {
        f(&mut self.0.lock().unwrap_or_else(|p| p.into_inner()))
    }
    pub fn context_ready(&self) -> bool {
        self.with(|s| s.context_ready)
    }
    pub fn load_seq(&self) -> u64 {
        self.with(|s| s.load_seq)
    }
    pub fn closed(&self) -> bool {
        self.with(|s| s.closed)
    }
    pub fn push(&self, e: BrowserEvent) {
        self.with(|s| s.events.push(e));
    }
    pub fn drain(&self) -> Vec<BrowserEvent> {
        self.with(|s| std::mem::take(&mut s.events))
    }
    pub fn take_eval(&self, id: u64) -> Option<EngineResult<serde_json::Value>> {
        self.with(|s| s.evals.remove(&id))
    }

    fn on_console(&self, message: &str) -> bool {
        let Some(rest) = message.strip_prefix(EVAL_PREFIX) else {
            return false;
        };
        let Some((id, json)) = rest.split_once(':') else {
            return false;
        };
        let Ok(id) = id.parse::<u64>() else {
            return false;
        };
        let result = match serde_json::from_str::<serde_json::Value>(json) {
            Ok(mut v) if v.get("ok").is_some() => Ok(v["ok"].take()),
            Ok(v) => Err(EngineError::Call(format!("js: {}", v["err"]))),
            Err(e) => Err(EngineError::Call(e.to_string())),
        };
        self.with(|s| s.evals.insert(id, result));
        true
    }
}

/// Wraps an expression so its awaited JSON value comes back through the console channel.
pub(super) fn eval_wrapper(id: u64, script: &str) -> String {
    format!(
        "(async()=>{{let o;try{{const v=await ({script});o={{ok:v===undefined?null:v}}}}catch(e){{o={{err:String(e)}}}}console.log('{EVAL_PREFIX}{id}:'+JSON.stringify(o))}})()"
    )
}

pub(super) fn app(shared: Shared) -> App {
    KkApp::new(shared)
}

pub(super) fn client(shared: Shared) -> Client {
    KkClient::new(shared)
}

wrap_app! {
    struct KkApp { shared: Shared, }
    impl App {
        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(KkBrowserProcess::new(self.shared.clone()))
        }
    }
}

wrap_browser_process_handler! {
    struct KkBrowserProcess { shared: Shared, }
    impl BrowserProcessHandler {
        fn on_context_initialized(&self) {
            self.shared.with(|s| s.context_ready = true);
        }
        // External pump: we poll do_message_loop_work ourselves, so scheduling hints are ignored.
        fn on_schedule_message_pump_work(&self, _delay_ms: i64) {}
    }
}

wrap_client! {
    struct KkClient { shared: Shared, }
    impl Client {
        fn load_handler(&self) -> Option<LoadHandler> { Some(KkLoad::new(self.shared.clone())) }
        fn display_handler(&self) -> Option<DisplayHandler> { Some(KkDisplay::new(self.shared.clone())) }
        fn life_span_handler(&self) -> Option<LifeSpanHandler> { Some(KkLifeSpan::new(self.shared.clone())) }
    }
}

wrap_load_handler! {
    struct KkLoad { shared: Shared, }
    impl LoadHandler {
        fn on_load_end(&self, _browser: Option<&mut Browser>, frame: Option<&mut Frame>, _status: c_int) {
            if let Some(f) = frame && f.is_main() == 1 {
                let url = CefString::from(&f.url()).to_string();
                self.shared.with(|s| {
                    s.load_seq += 1;
                    s.events.push(BrowserEvent::Loaded { url });
                });
            }
        }
    }
}

wrap_display_handler! {
    struct KkDisplay { shared: Shared, }
    impl DisplayHandler {
        fn on_console_message(&self, _b: Option<&mut Browser>, _l: LogSeverity, message: Option<&CefString>,
                              _s: Option<&CefString>, _line: c_int) -> c_int {
            // Swallow eval replies; let page logs through to the default logger.
            message.is_some_and(|m| self.shared.on_console(&m.to_string())).into()
        }
    }
}

wrap_life_span_handler! {
    struct KkLifeSpan { shared: Shared, }
    impl LifeSpanHandler {
        fn on_before_close(&self, _browser: Option<&mut Browser>) {
            self.shared.with(|s| s.closed = true);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn console_channel_parses_eval_replies() {
        let s = Shared::default();
        assert!(s.on_console("__kinekino_eval:3:{\"ok\":2}"));
        assert!(!s.on_console("page log"));
        assert_eq!(s.take_eval(3).unwrap().unwrap(), serde_json::json!(2));
        assert!(s.on_console("__kinekino_eval:4:{\"err\":\"boom\"}"));
        assert!(s.take_eval(4).unwrap().is_err());
        assert!(eval_wrapper(1, "1 + 1").contains("await (1 + 1)"));
    }
}

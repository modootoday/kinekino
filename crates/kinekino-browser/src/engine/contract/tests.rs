//! The suite itself, exercised with in-memory engines (no browser).
use super::{Outcome, run};
use crate::engine::port::{
    AudioControl, BrowserEngine, BrowserEvent, CapabilitySet, EngineError, EngineKind,
    EngineResult, NetworkInterceptor,
};

/// Pretends to be CEF: eval and mute work, injection/interception refused.
struct FakeCef {
    events: Vec<BrowserEvent>,
    muted: bool,
    lie_about_table: bool,
}

impl BrowserEngine for FakeCef {
    fn kind(&self) -> EngineKind {
        EngineKind::Cef
    }
    fn capabilities(&self) -> CapabilitySet {
        if self.lie_about_table {
            CapabilitySet::EMPTY
        } else {
            self.kind().declared()
        }
    }
    fn navigate(&mut self, url: &str) -> EngineResult<()> {
        self.events
            .push(BrowserEvent::NavigationStarted { url: url.into() });
        Ok(())
    }
    fn eval(&mut self, script: &str) -> EngineResult<serde_json::Value> {
        Ok(if script == "1 + 1" {
            2.into()
        } else {
            serde_json::Value::Null
        })
    }
    fn inject_script(&mut self, _: &str) -> EngineResult<()> {
        Err(EngineError::Unsupported("fake"))
    }
    fn set_interceptor(&mut self, _: Box<dyn NetworkInterceptor>) -> EngineResult<()> {
        Ok(()) // bug: undeclared but accepted
    }
    fn drain_events(&mut self) -> Vec<BrowserEvent> {
        std::mem::take(&mut self.events)
    }
    fn audio(&mut self) -> Option<&mut dyn AudioControl> {
        Some(self)
    }
}

impl AudioControl for FakeCef {
    fn set_muted(&mut self, muted: bool) -> EngineResult<()> {
        self.muted = muted;
        Ok(())
    }
    fn is_muted(&mut self) -> EngineResult<bool> {
        Ok(self.muted)
    }
}

fn fake() -> FakeCef {
    FakeCef {
        events: Vec::new(),
        muted: false,
        lie_about_table: false,
    }
}

#[test]
fn passes_declared_and_flags_undeclared_success() {
    let r = run(&mut fake(), "http://fixture");
    let get = |n| {
        r.checks
            .iter()
            .find(|(k, _)| *k == n)
            .map(|(_, o)| o.clone())
    };
    assert_eq!(get("eval"), Some(Outcome::Pass));
    assert_eq!(get("audio-mute"), Some(Outcome::Pass));
    assert_eq!(get("inject-script"), Some(Outcome::NotDeclared));
    assert_eq!(get("frame-capture"), Some(Outcome::NotDeclared));
    assert_eq!(
        r.failures(),
        vec!["set-interceptor: undeclared but succeeded".to_string()]
    );
}

#[test]
fn flags_table_drift() {
    let mut e = fake();
    e.lie_about_table = true;
    let r = run(&mut e, "http://fixture");
    assert!(r.failures().iter().any(|f| f.starts_with("declared-table")));
}

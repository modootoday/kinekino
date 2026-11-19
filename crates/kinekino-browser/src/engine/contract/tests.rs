//! The suite itself, exercised with in-memory engines (no browser).
use super::{Outcome, run};
use crate::engine::port::{
    AudioControl, BrowserEngine, BrowserEvent, Capability, CapabilitySet, EngineError, EngineKind,
    EngineResult, NetworkInterceptor, ProtocolEvent, ProtocolSession,
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

/// CEF stand-in that also claims the DevTools protocol session (ADR 20261109071720).
struct FakeProtocol {
    inner: FakeCef,
    serve_protocol: bool,
    attached: bool,
}

impl BrowserEngine for FakeProtocol {
    fn kind(&self) -> EngineKind {
        EngineKind::Cef
    }
    fn capabilities(&self) -> CapabilitySet {
        self.inner
            .capabilities()
            .union(CapabilitySet::of(&[Capability::ProtocolSession]))
    }
    fn navigate(&mut self, url: &str) -> EngineResult<()> {
        self.inner.navigate(url)
    }
    fn eval(&mut self, script: &str) -> EngineResult<serde_json::Value> {
        self.inner.eval(script)
    }
    fn inject_script(&mut self, s: &str) -> EngineResult<()> {
        self.inner.inject_script(s)
    }
    fn set_interceptor(&mut self, _: Box<dyn NetworkInterceptor>) -> EngineResult<()> {
        Err(EngineError::Unsupported("fake"))
    }
    fn drain_events(&mut self) -> Vec<BrowserEvent> {
        self.inner.drain_events()
    }
    fn protocol(&mut self) -> Option<&mut dyn ProtocolSession> {
        if self.serve_protocol {
            Some(self)
        } else {
            None
        }
    }
}

impl ProtocolSession for FakeProtocol {
    fn call(
        &mut self,
        session: Option<&str>,
        method: &str,
        params: serde_json::Value,
    ) -> EngineResult<serde_json::Value> {
        match (session, method) {
            (None, "Runtime.evaluate") if params["expression"] == "1 + 1" => {
                Ok(serde_json::json!({ "result": { "type": "number", "value": 2 } }))
            }
            _ => Err(EngineError::Unsupported("fake method")),
        }
    }
    fn drain_protocol_events(&mut self) -> Vec<ProtocolEvent> {
        vec![ProtocolEvent {
            session: None,
            method: "Page.frameNavigated".into(),
            params: serde_json::json!({}),
        }]
    }
    fn set_auto_attach(&mut self, on: bool) -> EngineResult<()> {
        self.attached = on;
        Ok(())
    }
}

fn outcome(r: &super::ContractReport, name: &str) -> Option<Outcome> {
    r.checks
        .iter()
        .find(|(k, _)| *k == name)
        .map(|(_, o)| o.clone())
}

#[test]
fn protocol_session_undeclared_is_not_declared() {
    let r = run(&mut fake(), "http://fixture");
    assert_eq!(outcome(&r, "protocol-session"), Some(Outcome::NotDeclared));
}

#[test]
fn declared_protocol_session_must_answer() {
    let mut e = FakeProtocol {
        inner: fake(),
        serve_protocol: true,
        attached: false,
    };
    let r = run(&mut e, "http://fixture");
    assert_eq!(outcome(&r, "protocol-session"), Some(Outcome::Pass));
    assert!(!e.attached, "the suite switches auto-attach back off");
    e.serve_protocol = false;
    let r = run(&mut e, "http://fixture");
    assert_eq!(
        outcome(&r, "protocol-session"),
        Some(Outcome::Fail("declared but protocol() is None".into()))
    );
}

#[test]
fn protocol_session_is_declared_by_the_adapters_that_pass_calls_through() {
    assert_eq!(Capability::ProtocolSession.name(), "protocol-session");
    for kind in EngineKind::ALL {
        let passes = matches!(
            kind,
            EngineKind::ChromeCdp | EngineKind::SystemChrome | EngineKind::WebView2
        );
        assert_eq!(
            kind.declared().contains(Capability::ProtocolSession),
            passes,
            "{}: only adapters that pass calls through declare protocol-session",
            kind.name()
        );
    }
}

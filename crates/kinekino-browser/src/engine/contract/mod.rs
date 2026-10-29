//! Common contract suite every `BrowserEngine` adapter must pass (ADR 0002).
//!
//! Rule: declared capabilities work; undeclared API capabilities answer `Unsupported`/`None`.
//! Runs against the local fixture only.
pub mod fixture;

use crate::engine::port::{
    BrowserEngine, BrowserEvent, Capability, EngineError, InterceptedRequest, NetworkInterceptor,
    RequestDecision,
};

/// Outcome of one contract check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Pass,
    /// Capability not declared and the engine correctly refused it.
    NotDeclared,
    Fail(String),
}

#[derive(Debug, Default)]
pub struct ContractReport {
    pub checks: Vec<(&'static str, Outcome)>,
}

impl ContractReport {
    pub fn failures(&self) -> Vec<String> {
        self.checks
            .iter()
            .filter_map(|(n, o)| match o {
                Outcome::Fail(why) => Some(format!("{n}: {why}")),
                _ => None,
            })
            .collect()
    }

    pub fn passed(&self) -> bool {
        self.failures().is_empty()
    }

    fn record(&mut self, name: &'static str, outcome: Outcome) {
        self.checks.push((name, outcome));
    }
}

impl std::fmt::Display for ContractReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (name, outcome) in &self.checks {
            writeln!(f, "{name:<22} {outcome:?}")?;
        }
        Ok(())
    }
}

struct BlockMarker;
impl NetworkInterceptor for BlockMarker {
    fn decide(&self, r: &InterceptedRequest) -> RequestDecision {
        if r.url.ends_with("/blocked.js") {
            RequestDecision::Block
        } else {
            RequestDecision::Allow
        }
    }
}

fn refused<T>(r: Result<T, EngineError>) -> Outcome {
    match r {
        Err(EngineError::Unsupported(_)) => Outcome::NotDeclared,
        Err(e) => Outcome::Fail(format!("undeclared but failed with {e}")),
        Ok(_) => Outcome::Fail("undeclared but succeeded".into()),
    }
}

fn ok<T>(r: Result<T, EngineError>) -> Outcome {
    r.map_or_else(|e| Outcome::Fail(e.to_string()), |_| Outcome::Pass)
}

fn expect(cond: bool, why: impl FnOnce() -> String) -> Outcome {
    if cond {
        Outcome::Pass
    } else {
        Outcome::Fail(why())
    }
}

/// Runs the suite against `engine`, navigating to `base_url` (a [`fixture::serve`] root).
pub fn run(engine: &mut dyn BrowserEngine, base_url: &str) -> ContractReport {
    let caps = engine.capabilities();
    let mut report = ContractReport::default();
    report.record(
        "declared-table",
        expect(caps == engine.kind().declared(), || {
            format!("{caps} differs from table {}", engine.kind().declared())
        }),
    );

    let injected = caps.contains(Capability::ScriptInjection);
    let r = engine.inject_script("window.__kk = 1;");
    report.record("inject-script", if injected { ok(r) } else { refused(r) });

    let intercepts = caps.contains(Capability::RequestInterception);
    let r = engine.set_interceptor(Box::new(BlockMarker));
    report.record(
        "set-interceptor",
        if intercepts { ok(r) } else { refused(r) },
    );

    let url = format!("{base_url}/");
    let nav = engine.navigate(&url);
    let events = engine.drain_events();
    report.record(
        "navigate",
        match nav {
            Err(e) => Outcome::Fail(e.to_string()),
            Ok(()) => expect(
                events
                    .iter()
                    .any(|e| matches!(e, BrowserEvent::NavigationStarted { url: u } if *u == url)),
                || format!("no NavigationStarted for {url}: {events:?}"),
            ),
        },
    );

    if caps.contains(Capability::Eval) {
        let two = engine.eval("1 + 1");
        report.record(
            "eval",
            match two {
                Ok(v) => expect(v == serde_json::json!(2), || format!("1+1 gave {v}")),
                Err(e) => Outcome::Fail(e.to_string()),
            },
        );
        if injected {
            let v = engine.eval("window.__kk === 1");
            report.record(
                "inject-visible",
                expect(matches!(v, Ok(serde_json::Value::Bool(true))), || {
                    format!("{v:?}")
                }),
            );
        }
        if intercepts {
            let v = engine.eval("!!window.__blockedRan");
            let blocked_event = events
                .iter()
                .any(|e| matches!(e, BrowserEvent::RequestBlocked { url } if url.ends_with("/blocked.js")));
            report.record(
                "intercept-blocks",
                expect(
                    matches!(v, Ok(serde_json::Value::Bool(false))) && blocked_event,
                    || format!("blockedRan={v:?} event={blocked_event}"),
                ),
            );
        }
    } else {
        report.record("eval", Outcome::NotDeclared);
    }

    let declared = caps.contains(Capability::AudioMute);
    report.record(
        "audio-mute",
        match (declared, engine.audio()) {
            (true, Some(a)) => {
                let round = a.set_muted(true).and_then(|()| a.is_muted()).and_then(|m| {
                    a.set_muted(false)?;
                    Ok(m && !a.is_muted()?)
                });
                match round {
                    Ok(true) => Outcome::Pass,
                    Ok(false) => Outcome::Fail("mute state did not round-trip".into()),
                    Err(e) => Outcome::Fail(e.to_string()),
                }
            }
            (true, None) => Outcome::Fail("declared but audio() is None".into()),
            (false, Some(_)) => Outcome::Fail("undeclared but audio() is Some".into()),
            (false, None) => Outcome::NotDeclared,
        },
    );

    let declared = caps.contains(Capability::FrameCapture);
    report.record(
        "frame-capture",
        match (declared, engine.frame_capture()) {
            (true, Some(c)) => match c.capture_png() {
                Ok(png) => expect(png.get(1..4) == Some(b"PNG".as_slice()), || {
                    "not a PNG".into()
                }),
                Err(e) => Outcome::Fail(e.to_string()),
            },
            (true, None) => Outcome::Fail("declared but frame_capture() is None".into()),
            (false, Some(_)) => Outcome::Fail("undeclared but frame_capture() is Some".into()),
            (false, None) => Outcome::NotDeclared,
        },
    );
    report
}

#[cfg(test)]
mod tests;

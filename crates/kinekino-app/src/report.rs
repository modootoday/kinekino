//! JSON reports for `kinekino platform …`.
use crate::{App, AppConfig};
use kinekino_browser::engine::port::{CapabilitySet, EngineKind};
use kinekino_core::port::registry::Policy;
use serde_json::{Value, json};

fn config_value(c: &AppConfig) -> Value {
    let rows: serde_json::Map<String, Value> = c
        .rows()
        .into_iter()
        .map(|(k, v, s)| (k.to_owned(), json!({ "value": v, "source": s.name() })))
        .collect();
    json!({ "file": c.file.as_ref().map(|p| p.display().to_string()), "settings": rows })
}

pub(crate) fn config(c: &AppConfig) -> String {
    config_value(c).to_string()
}

/// Platform, config and every port's selection with per-adapter rejection reasons.
pub(crate) fn capabilities(app: &App) -> String {
    let wanted = &app.config.browser_engine.value;
    let policy = EngineKind::ALL
        .into_iter()
        .find(|k| k.name() == wanted)
        .map_or(Policy::PlatformOrder, Policy::Only);
    let decision = app
        .browser
        .explain(app.target, CapabilitySet::EMPTY, policy);
    let browser: Value = serde_json::from_str(&decision.to_json()).unwrap_or(Value::Null);
    json!({
        "platform": {
            "os": app.target.os.name(),
            "display": app.target.display.map(|d| d.name()),
        },
        "config": config_value(&app.config),
        "ports": [
            browser,
            { "port": "credential", "selected": { "kind": "session", "role": "default" },
              "note": "process-lifetime store; OS keyring adapters not registered yet" },
        ],
    })
    .to_string()
}

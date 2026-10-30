//! `kinekino settings …` (plan P1b): one JSON settings file shared by CLI, GUI and MCP.
//!
//! File: `KINEKINO_SETTINGS_FILE`, else `<config>/kinekino/settings.json` where `<config>` is
//! `XDG_CONFIG_HOME`, `HOME/.config` or `APPDATA`. Sections: `mcp` (`crate::mcp::McpSettings`), `inference` (validated by
//! [`InferenceSettings`]). Tokens never go here.
use crate::Env;
pub mod gui;
pub mod page;
use kinekino_core::port::credential::CredentialStore;
use kinekino_core::port::{ErrorCode, PortError, PortResult};
use kinekino_inference::{CostGuard, InferenceSettings};
use serde_json::{Value, json};
use std::path::PathBuf;

pub fn settings_file(env: &Env) -> Option<PathBuf> {
    if let Some(p) = env.get("KINEKINO_SETTINGS_FILE") {
        return Some(PathBuf::from(p));
    }
    let base = env
        .get("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env.get("HOME").map(|h| PathBuf::from(h).join(".config")))
        .or_else(|| env.get("APPDATA").map(PathBuf::from))?;
    Some(base.join("kinekino").join("settings.json"))
}

fn invalid(msg: impl Into<String>) -> PortError {
    PortError::new(ErrorCode::InvalidInput, msg.into())
}

/// Whole settings document; a missing file is an empty object.
pub fn load(env: &Env) -> PortResult<Value> {
    let Some(path) = settings_file(env) else {
        return Ok(json!({}));
    };
    match std::fs::read_to_string(&path) {
        Ok(text) => {
            serde_json::from_str(&text).map_err(|e| invalid(format!("{}: {e}", path.display())))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(json!({})),
        Err(e) => Err(PortError::new(
            ErrorCode::Io,
            format!("{}: {e}", path.display()),
        )),
    }
}

fn save(env: &Env, doc: &Value) -> PortResult<()> {
    let path = settings_file(env).ok_or_else(|| invalid("no settings path (set HOME)"))?;
    let io = |e: std::io::Error| PortError::new(ErrorCode::Io, format!("{}: {e}", path.display()));
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(io)?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, format!("{doc:#}\n")).map_err(io)?;
    std::fs::rename(&tmp, &path).map_err(io)
}

pub fn inference(doc: &Value) -> PortResult<InferenceSettings> {
    serde_json::from_value(doc.get("inference").cloned().unwrap_or_else(|| json!({})))
        .map_err(|e| invalid(format!("inference: {e}")))
}

pub fn mcp(doc: &Value) -> PortResult<crate::mcp::McpSettings> {
    serde_json::from_value(doc.get("mcp").cloned().unwrap_or_else(|| json!({})))
        .map_err(|e| invalid(format!("mcp: {e}")))
}

fn section_get(doc: &Value, key: &str) -> PortResult<Value> {
    let (section, rest) = key
        .split_once('.')
        .map_or((key, None), |(s, r)| (s, Some(r)));
    let full = match section {
        "inference" => serde_json::to_value(inference(doc)?),
        "mcp" => serde_json::to_value(mcp(doc)?),
        other => {
            return Err(invalid(format!(
                "{other}: unknown section (inference, mcp)"
            )));
        }
    }
    .map_err(|e| invalid(e.to_string()))?;
    let Some(rest) = rest else { return Ok(full) };
    rest.split('.')
        .try_fold(full, |v, part| v.get(part).cloned())
        .ok_or_else(|| invalid(format!("{key}: unknown key")))
}

/// One dotted-key write into the document, validated by its section.
pub fn set_value(doc: &mut Value, key: &str, raw: &str) -> PortResult<()> {
    let (section, rest) = key
        .split_once('.')
        .ok_or_else(|| invalid(format!("{key}: expected <section>.<key>")))?;
    let next = match section {
        "inference" => serde_json::to_value(inference(doc)?.set(rest, raw).map_err(invalid)?),
        "mcp" => serde_json::to_value(mcp(doc)?.set(rest, raw).map_err(invalid)?),
        other => {
            return Err(invalid(format!(
                "{other}: unknown section (inference, mcp)"
            )));
        }
    }
    .map_err(|e| invalid(e.to_string()))?;
    if !doc.is_object() {
        *doc = json!({});
    }
    doc[section] = next;
    Ok(())
}

fn ledger_path(env: &Env) -> Option<PathBuf> {
    settings_file(env).map(|p| p.with_file_name("ledger.jsonl"))
}

fn ledger_text(env: &Env) -> String {
    ledger_path(env)
        .and_then(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_default()
}

/// Ledger path: `ledger.jsonl` beside the settings file.
fn ledger(env: &Env, doc: &Value) -> PortResult<Value> {
    let budget = inference(doc)?.budget;
    let path = ledger_path(env);
    let text = ledger_text(env);
    let guard = CostGuard::from_jsonl(budget.clone(), &text).map_err(invalid)?;
    Ok(json!({
        "file": path,
        "total_usd": guard.total(),
        "budget": serde_json::to_value(&budget).unwrap_or(Value::Null),
    }))
}

/// [`run_with`] on the process session credential store.
pub fn run(args: &crate::Args, env: &Env) -> (u8, String) {
    run_with(
        args,
        env,
        &kinekino_platform::session::SessionCredentialStore::new(),
    )
}

fn current_month() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    page::month_of(secs)
}

/// `settings path|get <key>|set <key> <value>|validate|ledger|page [YYYY-MM]|apply <json>|gui`.
pub fn run_with(args: &crate::Args, env: &Env, credentials: &dyn CredentialStore) -> (u8, String) {
    if args.get(1) == Some("gui") {
        return gui::run(
            args,
            env,
            credentials,
            &kinekino_settings_ui::MeariSettingsWindow,
        );
    }
    let result = load(env).and_then(|mut doc| match (args.get(1), args.get(2), args.get(3)) {
        (Some("path"), None, None) => Ok(json!({ "file": settings_file(env) })),
        (Some("get"), Some(key), None) => section_get(&doc, key),
        (Some("set"), Some(key), Some(raw)) => {
            set_value(&mut doc, key, raw)?;
            save(env, &doc)?;
            section_get(&doc, key)
        }
        (Some("validate"), None, None) => {
            mcp(&doc)?;
            let errors = inference(&doc)?.validate();
            Ok(json!({ "ok": errors.is_empty(), "errors": errors }))
        }
        (Some("ledger"), None, None) => ledger(env, &doc),
        (Some("page"), month, None) => {
            let month = month.map_or_else(current_month, str::to_owned);
            page::page(&doc, &ledger_text(env), &month, credentials).map_err(invalid)
        }
        (Some("apply"), Some(edits), None) => {
            let edits: serde_json::Map<String, Value> = serde_json::from_str(edits)
                .map_err(|e| invalid(format!("apply: expected a JSON object: {e}")))?;
            match page::apply(&doc, &edits) {
                Ok(next) => {
                    save(env, &next)?;
                    Ok(json!({ "ok": true, "applied": edits.keys().collect::<Vec<_>>() }))
                }
                Err(errors) => Ok(json!({ "ok": false, "errors": errors })),
            }
        }
        other => Err(invalid(format!(
            "usage: kinekino settings <path|get KEY|set KEY VALUE|validate|ledger|page [YYYY-MM]|apply JSON|gui [YYYY-MM] [--smoke] [--capture BMP]>, got {other:?}"
        ))),
    });
    match result {
        Ok(v) => (0, v.to_string()),
        Err(e) => (2, e.to_json()),
    }
}

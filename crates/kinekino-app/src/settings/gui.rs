//! `kinekino settings gui` (plan P6): the settings page in a native window.
//! OS-neutral: the message protocol and the handler. The window is a `SettingsWindowPort`;
//! the shipped one renders with Meari on every OS (`kinekino-settings-ui`). Messages are JSON:
//! `{"op":"page"}`, `{"op":"apply","edits":{key: value}}`, `{"op":"dialog","action":a}`,
//! `{"op":"ready","rendered":n}` (smoke runs) and `{"op":"close"}`.
use super::{current_month, invalid, ledger_text, load, page, save};
use crate::{Args, Env};
use kinekino_core::platform::settings_window::{SettingsWindowPort, WindowReply};
use kinekino_core::port::credential::CredentialStore;
use serde_json::{Value, json};
use std::path::PathBuf;

/// Handles page messages against the settings file. Each `apply` is atomic (see
/// [`page::apply`]): all edits save, or none and every error returns to the page.
pub struct Controller<'a> {
    env: &'a Env,
    credentials: &'a dyn CredentialStore,
    month: String,
    capture: Option<PathBuf>,
    /// Log of handled ops, returned as the command result.
    pub log: Vec<Value>,
}

impl<'a> Controller<'a> {
    pub fn new(env: &'a Env, credentials: &'a dyn CredentialStore, month: String) -> Self {
        Self {
            env,
            credentials,
            month,
            capture: None,
            log: Vec::new(),
        }
    }

    fn page(&self) -> Result<Value, String> {
        let doc = load(self.env).map_err(|e| e.message)?;
        page::page(&doc, &ledger_text(self.env), &self.month, self.credentials)
    }

    /// One page message → reply. Unknown or malformed messages answer with an error.
    pub fn handle(&mut self, text: &str) -> WindowReply {
        let msg: Value = serde_json::from_str(text).unwrap_or(Value::Null);
        let op = msg["op"].as_str().unwrap_or("").to_owned();
        let mut reply = WindowReply::default();
        let body = match op.as_str() {
            "page" => self.page().map(|p| json!({"op": "page", "page": p})),
            "apply" => match msg["edits"].as_object() {
                None => Ok(json!({"op": "apply", "ok": false, "errors": ["edits must be an object"]})),
                Some(edits) => load(self.env)
                    .map_err(|e| e.message)
                    .and_then(|doc| match page::apply(&doc, edits) {
                        Ok(next) => save(self.env, &next)
                            .map_err(|e| e.message)
                            .and_then(|()| self.page())
                            .map(|p| json!({"op": "apply", "ok": true, "applied": edits.len(), "page": p})),
                        Err(errors) => Ok(json!({"op": "apply", "ok": false, "errors": errors})),
                    }),
            },
            "dialog" => Ok(json!({"op": "dialog", "action": msg["action"]})),
            "ready" => {
                reply.capture_png = self.capture.clone();
                reply.close = true;
                Ok(json!({"op": "ready"}))
            }
            "close" => {
                reply.close = true;
                Ok(json!({"op": "close"}))
            }
            other => Err(format!("{other:?}: unknown settings window message")),
        };
        let body = body.unwrap_or_else(|e| json!({"op": op, "ok": false, "errors": [e]}));
        let mut entry = json!({"op": op});
        for key in ["ok", "errors", "applied", "action"] {
            if let Some(v) = body.get(key) {
                entry[key] = v.clone();
            }
        }
        if op == "ready" {
            entry["rendered"] = msg["rendered"].clone();
            entry["dialog"] = msg["dialog"].clone();
            entry["errors_shown"] = msg["errors_shown"].clone();
        }
        self.log.push(entry);
        if op != "ready" && op != "close" {
            reply.json = Some(body.to_string());
        }
        reply
    }
}

/// `settings gui [YYYY-MM] [--smoke] [--capture <file.png>]`.
pub fn run(
    args: &Args,
    env: &Env,
    credentials: &dyn CredentialStore,
    window: &dyn SettingsWindowPort,
) -> (u8, String) {
    let mut smoke = false;
    let mut month = None;
    let mut capture = None;
    let mut i = 2;
    while let Some(arg) = args.get(i) {
        match arg {
            "--smoke" => smoke = true,
            "--capture" => {
                i += 1;
                capture = args.get(i).map(PathBuf::from);
                if capture.is_none() {
                    return (2, invalid("--capture needs a .png path").to_json());
                }
            }
            m if month.is_none() && !m.starts_with('-') => month = Some(m.to_owned()),
            other => {
                return (
                    2,
                    invalid(format!("settings gui: unexpected {other:?}")).to_json(),
                );
            }
        }
        i += 1;
    }
    if capture.is_some() && !smoke {
        return (2, invalid("--capture needs --smoke").to_json());
    }
    let mut controller = Controller::new(env, credentials, month.unwrap_or_else(current_month));
    controller.capture = capture;
    let outcome = window.run("kinekino 설정", smoke, &mut |m| controller.handle(m));
    match outcome {
        Ok(()) => (
            0,
            json!({"ok": true, "messages": controller.log}).to_string(),
        ),
        Err(e) => {
            let code = if e.kind() == std::io::ErrorKind::Unsupported {
                kinekino_core::port::ErrorCode::Unsupported
            } else {
                kinekino_core::port::ErrorCode::Internal
            };
            (
                2,
                kinekino_core::port::PortError::new(code, e.to_string()).to_json(),
            )
        }
    }
}

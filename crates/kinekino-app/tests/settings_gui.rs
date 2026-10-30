//! P6 settings window: the OS-neutral handler driven through a fake window port, plus the
//! HTML contract the WebView2 adapter loads. The real window runs only on Windows.
use kinekino_app::settings::gui::{Controller, run};
use kinekino_app::{Args, Env};
use kinekino_core::platform::settings_window::{SettingsWindowPort, WindowReply};
use kinekino_platform::session::SessionCredentialStore;
use serde_json::{Value, json};
use std::{cell::RefCell, io};

fn fixture(name: &str) -> (std::path::PathBuf, Env) {
    let dir = std::env::temp_dir().join(format!("kk-settings-gui-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let env = Env::from_pairs([(
        "KINEKINO_SETTINGS_FILE",
        dir.join("settings.json").display().to_string(),
    )]);
    (dir, env)
}

/// Plays the page side: sends scripted messages and keeps every reply.
struct FakeWindow {
    script: Vec<Value>,
    seen_smoke: RefCell<Option<bool>>,
    replies: RefCell<Vec<WindowReply>>,
}

impl SettingsWindowPort for FakeWindow {
    fn run(&self, _: &str, smoke: bool, on: &mut dyn FnMut(&str) -> WindowReply) -> io::Result<()> {
        *self.seen_smoke.borrow_mut() = Some(smoke);
        for m in &self.script {
            let r = on(&m.to_string());
            let close = r.close;
            self.replies.borrow_mut().push(r);
            if close {
                return Ok(());
            }
        }
        Err(io::Error::other("page never closed"))
    }
}

fn reply(r: &WindowReply) -> Value {
    serde_json::from_str(r.json.as_deref().unwrap()).unwrap()
}

#[test]
fn window_flow_page_atomic_apply_dialog_and_close() {
    let (dir, env) = fixture("flow");
    std::fs::write(
        dir.join("ledger.jsonl"),
        "{\"month\":\"2026-10\",\"backend\":\"mock\",\"model\":\"m\",\"cost_usd\":4.1}\n",
    )
    .unwrap();
    let window = FakeWindow {
        script: vec![
            json!({"op":"page"}),
            json!({"op":"apply","edits":{"inference.budget.monthly_usd":3,"inference.budget.confirm_ratio":2}}),
            json!({"op":"apply","edits":{"inference.budget.monthly_usd":3,"mcp.allowed_tools":"all"}}),
            json!({"op":"dialog","action":"cancel"}),
            json!({"op":"bogus"}),
            json!({"op":"close"}),
        ],
        seen_smoke: RefCell::default(),
        replies: RefCell::default(),
    };
    let store = SessionCredentialStore::new();
    let (code, out) = run(
        &Args::new(["settings", "gui", "2026-10"]),
        &env,
        &store,
        &window,
    );
    assert_eq!(code, 0, "{out}");
    let replies = window.replies.borrow();
    let first = reply(&replies[0]);
    assert_eq!(first["page"]["fields"].as_array().unwrap().len(), 20);
    assert_eq!(first["page"]["limit_dialog"]["state"], "confirm", "{first}");
    let rejected = reply(&replies[1]);
    assert_eq!(rejected["ok"], false);
    assert_eq!(
        rejected["errors"].as_array().unwrap().len(),
        1,
        "{rejected}"
    );
    let applied = reply(&replies[2]);
    assert_eq!(applied["ok"], true, "{applied}");
    let saved: Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join("settings.json")).unwrap()).unwrap();
    assert_eq!(saved["inference"]["budget"]["monthly_usd"], 3.0);
    assert!(
        saved["inference"]["budget"]["confirm_ratio"]
            .as_f64()
            .unwrap()
            < 1.0
    );
    assert_eq!(saved["mcp"]["allowed_tools"], "all");
    assert_eq!(reply(&replies[3])["action"], "cancel");
    assert_eq!(reply(&replies[4])["ok"], false);
    assert!(replies[5].close && replies[5].json.is_none());
    let log: Value = serde_json::from_str(&out).unwrap();
    let ops: Vec<&str> = log["messages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["op"].as_str().unwrap())
        .collect();
    assert_eq!(ops, ["page", "apply", "apply", "dialog", "bogus", "close"]);
    assert_eq!(*window.seen_smoke.borrow(), Some(false));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn smoke_ready_closes_and_requests_capture() {
    let (dir, env) = fixture("smoke");
    let store = SessionCredentialStore::new();
    let mut c = Controller::new(&env, &store, "2026-10".into());
    assert!(!c.handle(r#"{"op":"page"}"#).close);
    let window = FakeWindow {
        script: vec![json!({"op":"ready","rendered":20,"dialog":false,"errors_shown":""})],
        seen_smoke: RefCell::default(),
        replies: RefCell::default(),
    };
    let png = dir.join("shot.png");
    let png_arg = png.display().to_string();
    let (code, out) = run(
        &Args::new(["settings", "gui", "--smoke", "--capture", png_arg.as_str()]),
        &env,
        &store,
        &window,
    );
    assert_eq!(code, 0, "{out}");
    let r = &window.replies.borrow()[0];
    assert!(r.close);
    assert_eq!(r.capture_png.as_deref(), Some(png.as_path()));
    assert_eq!(*window.seen_smoke.borrow(), Some(true));
    let log: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(log["messages"][0]["rendered"], 20);
    let (code, _) = run(
        &Args::new(["settings", "gui", "--capture", "x.png"]),
        &env,
        &store,
        &window,
    );
    assert_eq!(code, 2, "--capture without --smoke is rejected");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn unsupported_window_port_maps_to_unsupported_error() {
    let (dir, env) = fixture("unsupported");
    let store = SessionCredentialStore::new();
    let port = kinekino_core::platform::settings_window::UnsupportedSettingsWindow;
    let (code, out) = run(
        &Args::new(["settings", "gui", "--smoke"]),
        &env,
        &store,
        &port,
    );
    assert_eq!(code, 2);
    assert!(out.contains("settings page|apply"), "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}

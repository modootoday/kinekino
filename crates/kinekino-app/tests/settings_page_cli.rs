//! P6 settings page contract: page model, atomic apply, token status, limit dialog.
use kinekino_app::{Args, Env, settings};
use kinekino_core::port::PortResult;
use kinekino_core::port::credential::{CredentialStore, Secret};
use kinekino_inference::settings::{TOKEN_ACCOUNT, TOKEN_SERVICE};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Default)]
struct Store(BTreeMap<(String, String), Secret>);

impl CredentialStore for Store {
    fn get(&self, service: &str, account: &str) -> PortResult<Option<Secret>> {
        Ok(self.0.get(&(service.into(), account.into())).cloned())
    }
    fn set(&mut self, service: &str, account: &str, secret: Secret) -> PortResult<()> {
        self.0.insert((service.into(), account.into()), secret);
        Ok(())
    }
    fn delete(&mut self, service: &str, account: &str) -> PortResult<bool> {
        Ok(self.0.remove(&(service.into(), account.into())).is_some())
    }
}

struct Fixture {
    dir: std::path::PathBuf,
    env: Env,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let dir =
            std::env::temp_dir().join(format!("kk-settings-page-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let env = Env::from_pairs([(
            "KINEKINO_SETTINGS_FILE",
            dir.join("settings.json").display().to_string(),
        )]);
        Self { dir, env }
    }

    fn run(&self, store: &dyn CredentialStore, argv: &[&str]) -> (u8, Value) {
        let (code, out) = settings::run_with(&Args::new(argv.iter().copied()), &self.env, store);
        (code, serde_json::from_str(&out).unwrap())
    }

    fn ledger(&self, costs: &[f64]) {
        let lines: String = costs
            .iter()
            .map(|c| format!("{{\"month\":\"2026-10\",\"backend\":\"mock\",\"model\":\"m\",\"cost_usd\":{c}}}\n"))
            .collect();
        std::fs::write(self.dir.join("ledger.jsonl"), lines).unwrap();
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn field<'a>(page: &'a Value, key: &str) -> &'a Value {
    page["fields"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["key"] == key)
        .unwrap_or_else(|| panic!("{key} missing"))
}

#[test]
fn default_page_lists_every_plan_field_with_defaults_and_no_dialog() {
    let fx = Fixture::new("defaults");
    let (code, page) = fx.run(&Store::default(), &["settings", "page", "2026-10"]);
    assert_eq!(code, 0, "{page}");
    assert_eq!(page["fields"].as_array().unwrap().len(), 20);
    let mode = field(&page, "inference.mode");
    assert_eq!(mode["kind"], "choice");
    assert_eq!(mode["value"], "cloud");
    assert_eq!(mode["default"], "cloud");
    assert_eq!(
        mode["options"],
        json!(["cloud", "local_first", "local_only"])
    );
    assert_eq!(field(&page, "inference.budget.total_usd")["value"], 5.0);
    assert_eq!(field(&page, "mcp.enabled")["value"], false);
    assert_eq!(page["errors"], json!([]));
    assert_eq!(page["ledger"], json!({"total_usd": 0.0, "month_usd": 0.0}));
    assert_eq!(page["limit_dialog"], Value::Null);
}

#[test]
fn token_field_reports_registration_only() {
    let fx = Fixture::new("token");
    let mut store = Store::default();
    let (_, page) = fx.run(&store, &["settings", "page", "2026-10"]);
    let token = field(&page, "inference.token");
    assert_eq!(token["kind"], "secret");
    assert_eq!(token["registered"], false);
    assert!(token.get("value").is_none());
    store
        .set(TOKEN_SERVICE, TOKEN_ACCOUNT, Secret::new("hf_secret_value"))
        .unwrap();
    let (_, page) = fx.run(&store, &["settings", "page", "2026-10"]);
    assert_eq!(field(&page, "inference.token")["registered"], true);
    assert!(!page.to_string().contains("hf_secret_value"));
    let (_, out) = fx.run(&store, &["settings", "apply", r#"{"inference.token":"x"}"#]);
    assert_eq!(out["ok"], false);
}

#[test]
fn apply_is_atomic_and_persists_only_valid_batches() {
    let fx = Fixture::new("apply");
    let store = Store::default();
    let (_, out) = fx.run(
        &store,
        &[
            "settings",
            "apply",
            r#"{"inference.budget.monthly_usd":3,"inference.budget.confirm_ratio":2}"#,
        ],
    );
    assert_eq!(out["ok"], false);
    assert_eq!(out["errors"].as_array().unwrap().len(), 1, "{out}");
    assert!(
        !fx.dir.join("settings.json").exists(),
        "rejected batch must not save"
    );

    let (code, out) = fx.run(
        &store,
        &[
            "settings",
            "apply",
            r#"{"inference.budget.monthly_usd":3,"mcp.allowed_tools":"all"}"#,
        ],
    );
    assert_eq!((code, &out["ok"]), (0, &json!(true)), "{out}");
    let (_, page) = fx.run(&store, &["settings", "page", "2026-10"]);
    assert_eq!(field(&page, "inference.budget.monthly_usd")["value"], 3.0);
    assert_eq!(
        field(&page, "inference.budget.monthly_usd")["default"],
        10.0
    );
    assert_eq!(field(&page, "mcp.allowed_tools")["value"], "all");

    let (_, out) = fx.run(&store, &["settings", "apply", r#"{"other.key":1}"#]);
    assert_eq!(out["ok"], false);
    assert_eq!(fx.run(&store, &["settings", "apply", "[1]"]).0, 2);
}

#[test]
fn limit_dialog_confirms_near_cap_and_blocks_at_cap() {
    let fx = Fixture::new("limit");
    let store = Store::default();
    fx.ledger(&[1.0]);
    let (_, page) = fx.run(&store, &["settings", "page", "2026-10"]);
    assert_eq!(page["limit_dialog"], Value::Null);

    fx.ledger(&[2.0, 2.1]);
    let (_, page) = fx.run(&store, &["settings", "page", "2026-10"]);
    assert_eq!(page["ledger"]["total_usd"], 4.1);
    assert_eq!(page["limit_dialog"]["state"], "confirm", "{page}");
    assert_eq!(
        page["limit_dialog"]["actions"],
        json!(["continue", "cancel"])
    );

    fx.ledger(&[5.5]);
    let (_, page) = fx.run(&store, &["settings", "page", "2026-10"]);
    assert_eq!(page["limit_dialog"]["state"], "blocked", "{page}");
    assert_eq!(page["limit_dialog"]["actions"], json!(["cancel"]));
    let (_, page) = fx.run(&store, &["settings", "page", "2026-11"]);
    assert_eq!(page["ledger"]["month_usd"], 0.0);
}

/// A credential store without an OS adapter still yields the page; only the secret row degrades.
struct BrokenStore;

impl CredentialStore for BrokenStore {
    fn get(&self, _: &str, _: &str) -> PortResult<Option<Secret>> {
        Err(kinekino_core::port::PortError::new(
            kinekino_core::port::ErrorCode::Unsupported,
            "no credential adapter",
        ))
    }
    fn set(&mut self, _: &str, _: &str, _: Secret) -> PortResult<()> {
        unreachable!("page never writes secrets")
    }
    fn delete(&mut self, _: &str, _: &str) -> PortResult<bool> {
        unreachable!("page never deletes secrets")
    }
}

#[test]
fn page_survives_credential_store_errors() {
    let fx = Fixture::new("broken-store");
    let (code, page) = fx.run(&BrokenStore, &["settings", "page", "2026-10"]);
    assert_eq!(code, 0, "{page}");
    let token = field(&page, "inference.token");
    assert!(token["registered"].is_null());
    assert_eq!(token["store_error"], "no credential adapter");
    assert!(page["fields"].as_array().unwrap().len() > 1);
}

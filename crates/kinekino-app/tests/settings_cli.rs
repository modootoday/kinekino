//! `kinekino settings` round trip through a temp settings file (P1b).
use kinekino_app::{Args, Env, settings};

fn env(file: &std::path::Path) -> Env {
    Env::from_pairs([("KINEKINO_SETTINGS_FILE", file.display().to_string())])
}

fn run(env: &Env, argv: &[&str]) -> (u8, serde_json::Value) {
    let (code, out) = settings::run(&Args::new(argv.iter().copied()), env);
    (code, serde_json::from_str(&out).unwrap())
}

#[test]
fn get_set_validate_and_ledger() {
    let dir = std::env::temp_dir().join(format!("kk-settings-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let file = dir.join("settings.json");
    let env = env(&file);

    assert_eq!(
        run(&env, &["settings", "get", "inference.mode"]),
        (0, "cloud".into())
    );
    assert_eq!(
        run(
            &env,
            &["settings", "set", "inference.budget.monthly_usd", "3"]
        )
        .0,
        0
    );
    assert_eq!(
        run(&env, &["settings", "get", "inference.budget.monthly_usd"]).1,
        3.0
    );
    let saved: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
    assert_eq!(saved["inference"]["budget"]["monthly_usd"], 3.0);

    assert_eq!(
        run(
            &env,
            &["settings", "set", "inference.budget.confirm_ratio", "2"]
        )
        .0,
        2
    );
    assert_eq!(
        run(&env, &["settings", "set", "inference.token", "hf_x"]).0,
        2
    );
    assert_eq!(run(&env, &["settings", "get", "other.x"]).0, 2);
    assert_eq!(run(&env, &["settings", "validate"]).1["ok"], true);

    std::fs::write(
        dir.join("ledger.jsonl"),
        "{\"month\":\"2026-10\",\"backend\":\"mock\",\"model\":\"m\",\"cost_usd\":0.25}\n",
    )
    .unwrap();
    assert_eq!(run(&env, &["settings", "ledger"]).1["total_usd"], 0.25);
    let _ = std::fs::remove_dir_all(&dir);
}

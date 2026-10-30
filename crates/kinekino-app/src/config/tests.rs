use super::*;

fn tmp(name: &str, body: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("kinekino-app-cfg-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join("config.json");
    std::fs::write(&p, body).unwrap();
    p
}

#[test]
fn layers_apply_in_order_with_sources() {
    let p = tmp(
        "order",
        r#"{"browser":{"engine":"cef","headless":false},"inference":{"mode":"remote"}}"#,
    );
    let env = Env::from_pairs([
        ("KINEKINO_CONFIG", p.to_str().unwrap()),
        ("KINEKINO_INFERENCE_MODE", "off"),
        ("KINEKINO_OUTPUT", "text"),
        ("KINEKINO_UNKNOWN_THING", "ignored"),
    ]);
    let args = Args::new(["platform", "config", "--output", "json"]);
    let c = AppConfig::load(&args, &env).unwrap();
    assert_eq!(
        c.browser_engine,
        Setting {
            value: "cef".into(),
            source: Source::File
        }
    );
    assert_eq!(
        c.browser_headless,
        Setting {
            value: false,
            source: Source::File
        }
    );
    assert_eq!(
        c.inference_mode,
        Setting {
            value: "off".into(),
            source: Source::Env
        }
    );
    assert_eq!(
        c.output_format,
        Setting {
            value: "json".into(),
            source: Source::Cli
        }
    );
}

#[test]
fn defaults_when_no_file() {
    let env = Env::from_pairs([("KINEKINO_CONFIG", "/nonexistent/kinekino.json")]);
    let c = AppConfig::load(&Args::default(), &env).unwrap();
    assert_eq!(
        c.rows().iter().filter(|r| r.2 == Source::Default).count(),
        4
    );
}

#[test]
fn secrets_and_bad_values_are_rejected() {
    let p = tmp("secret", r#"{"inference":{"api_key":"sk-x"}}"#);
    let env = Env::from_pairs([("KINEKINO_CONFIG", p.to_str().unwrap())]);
    let e = AppConfig::load(&Args::default(), &env).unwrap_err();
    assert_eq!(e.code, ErrorCode::InvalidInput);
    assert!(!e.message.contains("sk-x"), "{e}");
    let env = Env::from_pairs([
        ("KINEKINO_CONFIG", "/nonexistent"),
        ("KINEKINO_BROWSER_ENGINE", "gecko"),
    ]);
    assert_eq!(
        AppConfig::load(&Args::default(), &env).unwrap_err().code,
        ErrorCode::InvalidInput
    );
}

#[test]
fn config_path_precedence() {
    let p = |pairs: &[(&str, &str)]| config_path(&Env::from_pairs(pairs.iter().copied()));
    assert_eq!(
        p(&[("XDG_CONFIG_HOME", "/x"), ("HOME", "/h")]),
        Some("/x/kinekino/config.json".into())
    );
    assert_eq!(
        p(&[("HOME", "/h")]),
        Some("/h/.config/kinekino/config.json".into())
    );
    assert_eq!(
        p(&[("KINEKINO_CONFIG", "/c.json"), ("HOME", "/h")]),
        Some("/c.json".into())
    );
    assert_eq!(p(&[]), None);
}

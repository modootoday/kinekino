//! Linux bootstrap smoke: real process env plus overrides, end to end through `run_platform`.
use kinekino_app::{Args, Env, bootstrap, run_platform};
use serde_json::Value;

fn env(extra: &[(&str, &str)]) -> Env {
    let mut pairs = vec![("KINEKINO_CONFIG", "/nonexistent/kinekino-smoke.json")];
    pairs.extend_from_slice(extra);
    Env::from_pairs(pairs)
}

#[test]
fn bootstrap_detects_this_host() {
    let app = bootstrap(&Args::default(), &env(&[])).unwrap();
    assert_eq!(app.target.os.name(), std::env::consts::OS);
    assert_eq!(app.browser.kinds().len(), 4);
}

#[test]
fn capabilities_report_on_linux_x11() {
    let (code, out) = run_platform(
        &Args::new(["platform", "capabilities"]),
        &env(&[("KINEKINO_PLATFORM", "linux-x11")]),
    );
    assert_eq!(code, 0, "{out}");
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["platform"]["display"], "x11");
    let browser = &v["ports"][0];
    assert_eq!(browser["port"], "browser");
    let kinds: Vec<&str> = browser["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["kind"].as_str().unwrap())
        .collect();
    assert_eq!(kinds[..2], ["cef", "system-chrome"]);
    for c in browser["candidates"].as_array().unwrap() {
        assert!(c["status"].is_string(), "{c}");
    }
}

#[test]
fn wayland_only_has_no_default_browser() {
    let (code, out) = run_platform(
        &Args::new(["platform", "capabilities"]),
        &env(&[("KINEKINO_PLATFORM", "linux-wayland")]),
    );
    assert_eq!(code, 0);
    let v: Value = serde_json::from_str(&out).unwrap();
    assert!(v["ports"][0]["selected"].is_null(), "{out}");
}

#[test]
fn errors_are_port_error_json() {
    let (code, out) = run_platform(&Args::new(["platform", "nope"]), &env(&[]));
    assert_eq!(code, 2);
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["code"], "invalid_input");
    assert_eq!(v["retryable"], false);
    let (code, out) = run_platform(
        &Args::new(["platform", "config"]),
        &env(&[("KINEKINO_PLATFORM", "beos")]),
    );
    assert_eq!(
        (
            code,
            serde_json::from_str::<Value>(&out).unwrap()["code"].as_str()
        ),
        (2, Some("invalid_input"))
    );
}

#[test]
fn config_report_shows_sources() {
    let (_, out) = run_platform(
        &Args::new(["platform", "config", "--output=text"]),
        &env(&[("KINEKINO_BROWSER_ENGINE", "chrome-cdp")]),
    );
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["settings"]["browser.engine"]["source"], "env");
    assert_eq!(v["settings"]["output.format"]["source"], "cli");
    assert_eq!(v["settings"]["inference.mode"]["source"], "default");
}

//! `kinekino adblock` offline contract (P3, inventory #6).
use serde_json::Value;
use std::process::Command;

fn run(args: &[&str]) -> (Option<i32>, Value) {
    let out = Command::new(env!("CARGO_BIN_EXE_kinekino"))
        .args(args)
        .output()
        .unwrap();
    let text = String::from_utf8(out.stdout).unwrap();
    (
        out.status.code(),
        serde_json::from_str(text.trim()).unwrap(),
    )
}

#[test]
fn rules_and_check() {
    let (code, rules) = run(&["adblock", "rules"]);
    assert_eq!(code, Some(0));
    assert!(rules["bundled_rules"].as_u64().unwrap() > 0);

    let ad = [
        "adblock",
        "check",
        "https://googleads.g.doubleclick.net/pagead/ads",
        "--source",
        "https://frame.example.test/",
        "--kind",
        "script",
    ];
    let (code, v) = run(&ad);
    assert_eq!((code, v["blocked"].as_bool()), (Some(0), Some(true)));

    let (_, v) = run(&[
        "adblock",
        "check",
        "https://cdn.example/video.m3u8",
        "--source",
        "https://media.example.test/",
        "--kind",
        "xmlhttprequest",
    ]);
    assert_eq!(v["blocked"], false);

    let dir = std::env::temp_dir().join(format!("kk-adblock-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let extra = dir.join("extra.txt");
    std::fs::write(&extra, "@@||googleads.g.doubleclick.net^$script\n").unwrap();
    let mut with_extra = ad.to_vec();
    let extra_path = extra.display().to_string();
    with_extra.extend(["--extra", &extra_path]);
    assert_eq!(run(&with_extra).1["blocked"], false);
    let _ = std::fs::remove_dir_all(&dir);

    assert_eq!(
        run(&["adblock", "check", "https://a.test/", "--kind", "bogus"]).0,
        Some(2)
    );
    assert_eq!(run(&["adblock"]).0, Some(2));
}

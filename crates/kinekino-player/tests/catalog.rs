#![cfg(windows)]
#[path = "support/cli_exe.rs"]
mod cli_exe;
#[path = "support/fixture_plugin.rs"]
mod fixture_plugin;
use serde_json::{Value, json};
use std::os::windows::process::CommandExt;
use std::{
    io::Write,
    process::{Command, Stdio},
};

fn invoke(root: &std::path::Path, request: Value) -> std::process::Output {
    let mut child = Command::new(crate::cli_exe::kinekino_cli_exe())
        .creation_flags(0x08000000)
        .env("KINEKINO_PLUGIN_DIR", root.join("plugins"))
        .args(["--catalog-root", root.to_str().unwrap(), "--input", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(serde_json::to_string(&request).unwrap().as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}
#[test]
fn explicit_cli_discovery_is_experimental_and_rejects_unsupported_search_fields() {
    kinekino_platform::install();
    let root = std::env::temp_dir().join(format!("catalog-cli-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    // The catalog backend is a `document/1` source plugin (M4); isolate from user plugins.
    crate::fixture_plugin::install_fixture_plugin(
        &root,
        std::path::Path::new(&crate::cli_exe::kinekino_cli_exe()),
    );
    let request = |command: &str, payload: Value| json!({"context":{"api_version":"1","request_id":"catalog-test"},"command":command,"payload":payload});
    let output = invoke(&root, request("capabilities", json!({})));
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["data"]["catalog"]["site_verified"], false);
    assert_eq!(response["data"]["catalog"]["backend_available"], true);
    assert_eq!(response["data"]["catalog"]["status"], "experimental");
    let default = Command::new(crate::cli_exe::kinekino_cli_exe())
        .creation_flags(0x08000000)
        .env("KINEKINO_PLUGIN_DIR", root.join("no-plugins"))
        .arg("capabilities")
        .output()
        .unwrap();
    assert!(default.status.success());
    let default: Value = serde_json::from_slice(&default.stdout).unwrap();
    assert_eq!(default["data"]["catalog"]["backend_available"], false);
    assert_eq!(default["data"]["catalog"]["status"], "unsupported");
    assert_eq!(default["data"]["catalog"]["maturity"], "experimental");
    for command in ["catalog.search", "episodes.list"] {
        let configured = invoke(&root, request("schema", json!({"command":command})));
        assert!(configured.status.success());
        let configured: Value = serde_json::from_slice(&configured.stdout).unwrap();
        assert_eq!(configured["data"]["status"], "experimental");
        assert!(configured["data"]["response"]["properties"]["data"].is_object());
        let default = Command::new(crate::cli_exe::kinekino_cli_exe())
            .creation_flags(0x08000000)
            .env("KINEKINO_PLUGIN_DIR", root.join("no-plugins"))
            .args(["schema", command])
            .output()
            .unwrap();
        assert!(default.status.success());
        assert_eq!(
            serde_json::from_slice::<Value>(&default.stdout).unwrap()["data"]["status"],
            "unsupported"
        );
    }
    let output = invoke(
        &root,
        request(
            "catalog.search",
            json!({"category":"airing","query":"unsupported"}),
        ),
    );
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap()["error"]["code"],
        "invalid_argument"
    );
    assert_eq!(std::fs::read_dir(root.join("objects")).unwrap().count(), 0);
    std::fs::remove_dir_all(root).unwrap();
}

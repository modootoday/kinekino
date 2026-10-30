//! Contract: `kinekino mcp serve` over real stdio (P9).
use serde_json::{Value, json};
use std::io::Write;
use std::process::{Command, Stdio};

fn serve(settings: &std::path::Path, input: &str) -> (Option<i32>, Vec<Value>) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kinekino"))
        .args(["mcp", "serve"])
        .env("KINEKINO_SETTINGS_FILE", settings)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let lines = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    (out.status.code(), lines)
}

#[test]
fn disabled_by_default_then_lists_and_calls_read_only_tools() {
    let dir = std::env::temp_dir().join(format!("kk-mcp-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("settings.json");

    let (code, lines) = serve(&file, "");
    assert_eq!(code, Some(2));
    assert!(
        lines[0]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("disabled")
            || lines[0].to_string().contains("disabled")
    );

    std::fs::write(&file, r#"{"mcp":{"enabled":true}}"#).unwrap();
    let input = [
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"t","version":"0"}}}),
        json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
        json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
        json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"version","arguments":{}}}),
        json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"subtitles.convert","arguments":{}}}),
    ]
    .map(|m| m.to_string() + "\n")
    .concat();
    let (code, lines) = serve(&file, &input);
    assert_eq!(code, Some(0));
    assert_eq!(lines.len(), 4);
    assert_eq!(lines[0]["result"]["serverInfo"]["name"], "kinekino");
    let tools: Vec<&str> = lines[1]["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    // Schema snapshot of the default (read-only) tool surface.
    assert_eq!(tools, ["version", "capabilities", "schema"]);
    for tool in lines[1]["result"]["tools"].as_array().unwrap() {
        assert_eq!(tool["inputSchema"]["type"], "object", "{tool}");
        assert_eq!(tool["annotations"]["readOnlyHint"], true);
    }
    assert_eq!(lines[2]["result"]["isError"], false);
    assert_eq!(lines[2]["result"]["structuredContent"]["ok"], true);
    assert_eq!(lines[3]["error"]["code"], -32602);
    let _ = std::fs::remove_dir_all(&dir);
}

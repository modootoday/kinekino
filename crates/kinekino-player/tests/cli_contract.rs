#[path = "support/cli_exe.rs"]
mod cli_exe;
use std::io::Write;
use std::process::{Command, Stdio};

fn invoke(args: &[&str], input: Option<&str>) -> (i32, serde_json::Value) {
    let mut child = Command::new(crate::cli_exe::kinekino_cli_exe())
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(input) = input {
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
    }
    let output = child.wait_with_output().unwrap();
    assert!(output.stderr.is_empty());
    let text = String::from_utf8(output.stdout).unwrap();
    assert_eq!(text.lines().count(), 1);
    (
        output.status.code().unwrap(),
        serde_json::from_str(&text).unwrap(),
    )
}

#[test]
fn executable_discovery_and_failures_match_sdk_contract() {
    kinekino_platform::install();
    let (code, version) = invoke(&["version"], None);
    assert_eq!(code, 0);
    assert_eq!(version["data"]["product"]["cli_executable"], "kinekino.exe");
    let (code, response) = invoke(&["playback", "status"], None);
    assert_eq!(code, 3);
    assert_eq!(response["error"]["code"], "unsupported");
    let (code, response) = invoke(&["version", "--unexpected"], None);
    assert_eq!(code, 2);
    assert_eq!(response["error"]["code"], "invalid_argument");
}

#[test]
fn help_discloses_conditional_frame_support() {
    kinekino_platform::install();
    let output = Command::new(crate::cli_exe::kinekino_cli_exe())
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    assert!(help.contains("production.capture_frame"));
    assert!(help.contains("when FFmpeg/FFprobe are available"));
    assert!(help.contains("Query capabilities"));
    assert!(help.contains("kinekino --input FILE|- [--json|--human]"));
    assert!(help.contains("full Request JSON"));
    assert!(help.contains("Do not combine it with a positional command or context flags"));
    assert!(!help.contains("media commands are unsupported"));
}

#[test]
fn utf8_stdin_and_unknown_fields_are_handled_without_echoing_secrets() {
    kinekino_platform::install();
    let input = r#"{"context":{"request_id":"요청-한글","api_version":"1"},"command":"version"}"#;
    let (code, response) = invoke(&["--input", "-"], Some(input));
    assert_eq!(code, 0);
    assert_eq!(response["request_id"], "요청-한글");
    let input = r#"{"context":{"request_id":"a","api_version":"1"},"command":"version","token":"do-not-echo"}"#;
    let (code, response) = invoke(&["--input", "-"], Some(input));
    assert_eq!(code, 2);
    assert!(!response.to_string().contains("do-not-echo"));
}

#[test]
fn invalid_endpoint_errors_preserve_valid_request_correlation_without_echoing_endpoint() {
    kinekino_platform::install();
    let id = "endpoint-요청";
    let secret_endpoint = "TEST_SECRET_ENDPOINT";
    let (code, response) = invoke(
        &["version", "--request-id", id, "--endpoint", secret_endpoint],
        None,
    );
    assert_eq!(code, 2);
    assert_eq!(response["request_id"], id);
    assert_eq!(response["error"]["request_id"], id);
    assert_eq!(response["error"]["code"], "invalid_argument");
    assert!(!response.to_string().contains(secret_endpoint));

    let input =
        r#"{"context":{"request_id":"json-endpoint-요청","api_version":"1"},"command":"version"}"#;
    let (code, response) = invoke(
        &["--input", "-", "--endpoint", secret_endpoint],
        Some(input),
    );
    assert_eq!(code, 2);
    assert_eq!(response["request_id"], "json-endpoint-요청");
    assert_eq!(response["error"]["request_id"], "json-endpoint-요청");
    assert_eq!(response["error"]["code"], "invalid_argument");
    assert!(!response.to_string().contains(secret_endpoint));
}

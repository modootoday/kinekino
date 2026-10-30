#[path = "support/cli_exe.rs"]
mod cli_exe;
use kinekino_player::application::api::{PlayerSdk, Request};
use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

struct Observation {
    code: i32,
    response: Value,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}
fn invoke(bytes: &[u8]) -> Observation {
    let mut child = Command::new(crate::cli_exe::kinekino_cli_exe())
        .args(["--input", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(bytes).unwrap();
    let output = child.wait_with_output().unwrap();
    let text = std::str::from_utf8(&output.stdout).unwrap();
    assert_eq!(text.lines().count(), 1);
    assert!(output.stderr.is_empty());
    Observation {
        code: output.status.code().unwrap(),
        response: serde_json::from_slice(&output.stdout).unwrap(),
        stdout: output.stdout,
        stderr: output.stderr,
    }
}
fn save(root: &Path, name: &str, bytes: &[u8], files: &mut Vec<Value>) {
    let path = root.join(name);
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    file.write_all(bytes).unwrap();
    file.sync_all().unwrap();
    files.push(json!({"path":name,"bytes":bytes.len()}));
}

#[test]
fn actual_cli_preserves_valid_unicode_context_errors_and_never_reflects_invalid_input() {
    kinekino_platform::install();
    let sdk = PlayerSdk::default();
    let mut cases = Vec::new();
    for (label, id, version, command, exit) in [
        ("unicode-128", "가".repeat(128), "1", "version", 0),
        ("invalid-version", "요청-상관".into(), "2", "version", 2),
        (
            "unsupported",
            "요청-미지원".into(),
            "1",
            "playback.status",
            3,
        ),
        ("unicode-129", "가".repeat(129), "1", "version", 2),
        ("empty-id", String::new(), "1", "version", 2),
        ("control-id", "\nTEST_SECRET_ID".into(), "1", "version", 2),
    ] {
        let mut request = Request::new(command);
        request.context.request_id = id;
        request.context.api_version = version.into();
        let input = serde_json::to_vec(&request).unwrap();
        let actual = invoke(&input);
        let expected = serde_json::to_value(sdk.execute(request)).unwrap();
        assert_eq!(actual.code, exit, "{label}");
        assert_eq!(actual.response, expected, "{label}");
        cases.push((label, input, actual));
    }
    let mut request = Request::new("version");
    request.context.request_id = "public-correlation".into();
    request.context.api_version = "2".into();
    request.payload = json!({"secret":"TEST_SECRET_PAYLOAD"});
    let input = serde_json::to_vec(&request).unwrap();
    let actual = invoke(&input);
    assert_eq!(actual.code, 2);
    assert_eq!(actual.response["request_id"], "public-correlation");
    assert!(
        !std::str::from_utf8(&actual.stdout)
            .unwrap()
            .contains("TEST_SECRET_PAYLOAD")
    );
    cases.push(("secret-payload", input, actual));
    for (label,input) in [
        ("invalid-json",br#"{"context":{"request_id":"do-not-recover","api_version":"2"},"secret":"TEST_SECRET_RAW""#.as_slice()),
        ("unknown-field",br#"{"context":{"request_id":"do-not-recover","api_version":"1"},"command":"version","secret":"TEST_SECRET_RAW"}"#.as_slice()),
    ] {
        let actual=invoke(input);assert_eq!(actual.code,2);assert_eq!(actual.response["request_id"],"cli");assert!(!std::str::from_utf8(&actual.stdout).unwrap().contains("TEST_SECRET_RAW"));assert!(!std::str::from_utf8(&actual.stdout).unwrap().contains("do-not-recover"));
        cases.push((label,input.to_vec(),actual));
    }
    assert!(!cases.iter().any(|(_, _, actual)| {
        std::str::from_utf8(&actual.stdout)
            .unwrap()
            .contains("TEST_SECRET_ID")
    }));
    if let Some(path) = std::env::var_os("SDK_REQUEST_ID_EVIDENCE_DIR") {
        let root = std::path::PathBuf::from(path);
        assert!(root.is_absolute());
        fs::create_dir(&root).unwrap();
        let mut files = Vec::new();
        let mut results = Vec::new();
        for (label, input, actual) in cases {
            save(&root, &format!("{label}.request.json"), &input, &mut files);
            save(
                &root,
                &format!("{label}.stdout"),
                &actual.stdout,
                &mut files,
            );
            save(
                &root,
                &format!("{label}.stderr"),
                &actual.stderr,
                &mut files,
            );
            results.push(json!({"case":label,"exit_code":actual.code,"request_id":actual.response["request_id"],"error_code":actual.response["error"]["code"]}));
        }
        let exe_bytes = fs::metadata(crate::cli_exe::kinekino_cli_exe())
            .unwrap()
            .len();
        let manifest = json!({"executable_path":crate::cli_exe::kinekino_cli_exe(),"executable_bytes":exe_bytes,"cases":results,"files":files,"hash_provenance":"Source, executable and observation hashes recorded by separate native file-hash inspection in the followup report; no new product dependency","scope":"actual read-only CLI requests; deliberately synthetic secret markers, no model/GUI invocation; no claim of later source identity"});
        let mut ignored = Vec::new();
        save(
            &root,
            "manifest.json",
            &serde_json::to_vec_pretty(&manifest).unwrap(),
            &mut ignored,
        );
    }
}

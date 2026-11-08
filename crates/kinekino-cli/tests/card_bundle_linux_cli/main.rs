//! Card bundle export on Linux (roadmap M4-5): the bundle renderer, staging publication and
//! Jobs executor are not available in this build, and the CLI says so instead of accepting
//! card work it cannot finish. Partial-failure, cancellation and retry semantics of the
//! shared bundle use case are covered by application fixture tests in kinekino-player.
use serde_json::{Value, json};
use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

fn call(request: &Value) -> (Option<i32>, Value) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kinekino"))
        .args(["--input", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(request.to_string().as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    (
        output.status.code(),
        serde_json::from_slice(&output.stdout).unwrap(),
    )
}

fn request(command: &str, request_id: &str, payload: Value) -> Value {
    json!({"command":command,"context":{"request_id":request_id,"api_version":"1"},"payload":payload})
}

fn scratch(tag: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!("kk-card-cli-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(base.join("jobs")).unwrap();
    std::fs::create_dir_all(base.join("cards")).unwrap();
    base
}

fn empty(path: &std::path::Path) -> bool {
    std::fs::read_dir(path).unwrap().next().is_none()
}

#[test]
fn job_host_refuses_card_project_root_without_starting_or_writing() {
    let base = scratch("host");
    let output = Command::new(env!("CARGO_BIN_EXE_kinekino"))
        .args(["--serve-jobs", "--job-root"])
        .arg(base.join("jobs"))
        .arg("--card-project-root")
        .arg(base.join("cards"))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.contains("job_host_ready"), "{stdout}");
    let refusal: Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(refusal["ok"], false);
    assert_eq!(refusal["error"]["code"], "unsupported");
    assert_eq!(refusal["error"]["retryable"], false);
    // No job store, staging or card project files are created for a refused host.
    assert!(empty(&base.join("jobs")));
    assert!(empty(&base.join("cards")));
    std::fs::remove_dir_all(&base).unwrap();
}

#[test]
fn direct_card_export_is_unsupported_and_publishes_nothing() {
    let base = scratch("direct");
    let output = base.join("bundle");
    let (code, refused) = call(&request(
        "production.export_cards",
        "cards",
        json!({"project":{"document_id":"bundle-1","publication_revision":"1","publication_sha256":"a".repeat(64)},"frames":[],"output_directory":output,"review_artifact_only":true}),
    ));
    assert_eq!(code, Some(3), "{refused}");
    assert_eq!(refused["error"]["code"], "unsupported");
    assert!(!output.exists());
    std::fs::remove_dir_all(&base).unwrap();
}

#[test]
fn capabilities_report_card_bundle_unavailable_without_quality_claims() {
    let (code, capabilities) = call(&request("capabilities", "caps", json!({})));
    assert_eq!(code, Some(0), "{capabilities}");
    let data = &capabilities["data"];
    for command in [
        "production.export_cards",
        "production.export_card_review_png",
        "card_projects.create",
    ] {
        let entry = data["commands"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["command"] == command)
            .unwrap_or_else(|| panic!("{command} listed"));
        assert_eq!(entry["status"], "unsupported", "{command}");
    }
    assert_eq!(data["card_projects"]["backend_available"], false);
    assert_eq!(data["card_projects"]["export_cards_supported"], false);
    assert_eq!(data["card_projects"]["quality_approved"], false);
    assert_eq!(data["card_templates"]["backend_available"], false);
    assert_eq!(data["card_templates"]["templates"], json!([]));
    assert_eq!(data["card_review_png"]["review_artifact_available"], false);
    assert_eq!(data["card_review_png"]["overwrite"], false);
}

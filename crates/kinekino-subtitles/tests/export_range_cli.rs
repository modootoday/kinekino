use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

fn cli(payload: Value) -> (i32, Value) {
    let executable = std::env::var_os("SUBTITLE_RANGE_CLI_EXE")
        .expect("Central gate must supply the freshly built, pinned player-cli path");
    let mut child = Command::new(executable)
        .args(["--input", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let request = json!({"context":{"request_id":"range-export-gate","api_version":"1","expected_revision":null,"idempotency_key":null,"timeout_ms":null},"command":"subtitles.export_range","payload":payload});
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(&request).unwrap())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    (
        output.status.code().unwrap(),
        serde_json::from_slice(&output.stdout).unwrap(),
    )
}

struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
#[ignore = "Requires central SDK command integration and a freshly built pinned SUBTITLE_RANGE_CLI_EXE"]
fn fresh_cli_exports_and_rejects_loss_empty_and_overwrite_without_mutating_source() {
    let fixture =
        Fixture(std::env::temp_dir().join(format!("subtitle-range-cli-{}", std::process::id())));
    fs::create_dir(&fixture.0).unwrap();
    let source = "1\n00:00:01,000 --> 00:00:03,000\n原文 한글\n\n";
    fs::write(fixture.0.join("source.srt"), source).unwrap();
    let mut request = json!({"input_path":fixture.0.join("source.srt"),"output_path":fixture.0.join("out.vtt"),"input_format":"srt","output_format":"vtt","range":{"in_us":"2000000","out_us":"3000000"}});
    let (code, result) = cli(request.clone());
    assert_eq!(code, 0, "{result}");
    assert_eq!(result["data"]["cue_count"], 1);
    let published = fs::read_to_string(fixture.0.join("out.vtt")).unwrap();
    assert_eq!(
        published,
        "WEBVTT\n\n00:00:00.000 --> 00:00:01.000\n原文 한글\n\n"
    );
    let (code, result) = cli(request.clone());
    assert_eq!(code, 5, "{result}");
    assert_eq!(result["error"]["code"], "revision_conflict");
    assert_eq!(
        fs::read_to_string(fixture.0.join("out.vtt")).unwrap(),
        published
    );
    request["output_path"] = json!(fixture.0.join("loss.vtt"));
    request["range"]["in_us"] = json!("1000001");
    let (code, result) = cli(request.clone());
    assert_eq!(code, 3, "{result}");
    assert_eq!(result["error"]["code"], "needs_input");
    assert!(!fixture.0.join("loss.vtt").exists());
    request["allow_loss"] = json!(true);
    let (code, result) = cli(request.clone());
    assert_eq!(code, 0, "{result}");
    request["output_path"] = json!(fixture.0.join("empty.vtt"));
    request["range"] = json!({"in_us":"3000000","out_us":"4000000"});
    let (code, result) = cli(request);
    assert_eq!(code, 2, "{result}");
    assert_eq!(result["error"]["code"], "invalid_argument");
    assert!(!fixture.0.join("empty.vtt").exists());
    assert_eq!(
        fs::read_to_string(fixture.0.join("source.srt")).unwrap(),
        source
    );
}

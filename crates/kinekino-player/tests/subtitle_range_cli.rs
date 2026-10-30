use kinekino_player::application::api::ProposalClient;
#[path = "support/cli_exe.rs"]
mod cli_exe;
use kinekino_player::{
    application::{
        api::{ApplicationClientHandle, PlayerSdk, Request},
        subtitles::{export_range::ExportSubtitleRange, storage::UnavailableStorage},
    },
    infrastructure::subtitle_files::SubtitleFiles,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

static NEXT: AtomicU64 = AtomicU64::new(0);
const SOURCE: &str = "1\n00:00:01,000 --> 00:00:03,000\n첫째 原文\n\n2\n00:00:04,000 --> 00:00:06,000\n둘째 한글\n\n";
struct Fixture {
    path: PathBuf,
    retain: bool,
}
impl Fixture {
    fn new() -> Self {
        let evidence = std::env::var_os("SUBTITLE_RANGE_EVIDENCE_DIR");
        let root = evidence
            .clone()
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let path = root.join(format!(
            "subtitle-range-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        fs::write(path.join("source.srt"), SOURCE).unwrap();
        Self {
            path,
            retain: evidence.is_some(),
        }
    }
    fn request(&self, output: &str, start: &str, end: &str) -> Request {
        let mut request = Request::new("subtitles.export_range");
        request.payload = json!({"input_path":self.path.join("source.srt"),"output_path":self.path.join(output),"input_format":"srt","output_format":"vtt","range":{"in_us":start,"out_us":end}});
        request
    }
    fn cli(&self, request: &Request) -> (i32, Value) {
        let mut child = Command::new(crate::cli_exe::kinekino_cli_exe())
            .args(["--input", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(&serde_json::to_vec(request).unwrap())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(
            output.stderr.is_empty(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        let code = output.status.code().unwrap();
        if self.retain {
            fs::write(
                self.path.join(format!(
                    "call-{}.json",
                    NEXT.fetch_add(1, Ordering::Relaxed)
                )),
                serde_json::to_vec_pretty(
                    &json!({"request":request,"exit_code":code,"response":value}),
                )
                .unwrap(),
            )
            .unwrap();
        }
        (code, value)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if self.retain {
            let files: Vec<Value> = fs::read_dir(&self.path).unwrap().map(|entry| {
                let entry = entry.unwrap();
                let bytes = fs::read(entry.path()).unwrap();
                json!({"name":entry.file_name().to_string_lossy(),"bytes":bytes.len(),"sha256":format!("{:x}",Sha256::digest(&bytes))})
            }).collect();
            fs::write(
                self.path.join("files.json"),
                serde_json::to_vec_pretty(&json!({"files":files,"cli_path":crate::cli_exe::kinekino_cli_exe(),"cli_sha256":format!("{:x}",Sha256::digest(fs::read(crate::cli_exe::kinekino_cli_exe()).unwrap()))})).unwrap(),
            )
            .unwrap();
        } else {
            fs::remove_dir_all(&self.path).unwrap();
        }
    }
}

#[test]
fn fresh_cli_and_typed_client_clip_rebase_and_refuse_overwrite() {
    kinekino_platform::install();
    let fixture = Fixture::new();
    let request = fixture.request("cli.vtt", "2000000", "5000000");
    let (code, result) = fixture.cli(&request);
    assert_eq!(code, 0, "{result}");
    assert_eq!(result["data"]["cue_count"], 2);
    let expected = "WEBVTT\n\n00:00:00.000 --> 00:00:01.000\n첫째 原文\n\n00:00:02.000 --> 00:00:03.000\n둘째 한글\n\n";
    assert_eq!(
        fs::read_to_string(fixture.path.join("cli.vtt")).unwrap(),
        expected
    );
    let typed_request = fixture.request("typed.vtt", "2000000", "5000000");
    let payload: ExportSubtitleRange = serde_json::from_value(typed_request.payload).unwrap();
    let client: ApplicationClientHandle = Arc::new(PlayerSdk::new(SubtitleFiles));
    let typed =
        std::thread::spawn(move || client.export_subtitle_range(typed_request.context, payload))
            .join()
            .unwrap()
            .unwrap();
    assert_eq!(typed.cue_count, 2);
    assert_eq!(
        fs::read(fixture.path.join("typed.vtt")).unwrap(),
        expected.as_bytes()
    );
    let (code, result) = fixture.cli(&request);
    assert_eq!(code, 5, "{result}");
    assert_eq!(result["error"]["code"], "revision_conflict");
    assert_eq!(
        fs::read_to_string(fixture.path.join("cli.vtt")).unwrap(),
        expected
    );
    assert_eq!(
        fs::read_to_string(fixture.path.join("source.srt")).unwrap(),
        SOURCE
    );
}

#[test]
fn fresh_cli_loss_approval_empty_collapsed_and_invalid_wire_never_silently_publish() {
    kinekino_platform::install();
    let fixture = Fixture::new();
    let mut request = fixture.request("loss.vtt", "1000001", "5000000");
    let (code, result) = fixture.cli(&request);
    assert_eq!(code, 3, "{result}");
    assert_eq!(result["error"]["code"], "needs_input");
    assert!(!fixture.path.join("loss.vtt").exists());
    request.payload["allow_loss"] = json!(true);
    assert_eq!(fixture.cli(&request).0, 0);
    for (start, end) in [
        ("3000000", "4000000"),
        ("1000001", "1000999"),
        ("2000000", "1000000"),
        ("0", "18446744073709551616"),
    ] {
        let request = fixture.request("invalid.vtt", start, end);
        let (code, result) = fixture.cli(&request);
        assert_eq!(code, 2, "{result}");
        assert_eq!(result["error"]["code"], "invalid_argument");
        assert!(!fixture.path.join("invalid.vtt").exists());
    }
    let mut request = fixture.request("invalid.vtt", "0", "5000000");
    request.payload["range"]["in_us"] = json!(0);
    assert_eq!(fixture.cli(&request).0, 2);
    request.payload["range"]["in_us"] = json!("0");
    request.payload["range"]["unexpected"] = json!(true);
    assert_eq!(fixture.cli(&request).0, 2);
    assert!(!fixture.path.join("invalid.vtt").exists());
    assert_eq!(
        fs::read_to_string(fixture.path.join("source.srt")).unwrap(),
        SOURCE
    );
}

#[test]
fn capabilities_schema_adapter_and_context_agree() {
    kinekino_platform::install();
    let fixture = Fixture::new();
    let (_, caps) = fixture.cli(&Request::new("capabilities"));
    assert!(
        caps["data"]["commands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["command"] == "subtitles.export_range" && v["status"] == "supported")
    );
    let mut schema_request = Request::new("schema");
    schema_request.payload = json!({"command":"subtitles.export_range"});
    let (code, schema) = fixture.cli(&schema_request);
    assert_eq!(code, 0);
    let payload = &schema["data"]["request"]["properties"]["payload"];
    assert!(
        schema["data"]["request"]["required"]
            .as_array()
            .unwrap()
            .contains(&json!("payload"))
    );
    assert_eq!(payload["additionalProperties"], false);
    assert_eq!(
        payload["properties"]["range"]["additionalProperties"],
        false
    );
    assert!(
        payload["required"]
            .as_array()
            .unwrap()
            .contains(&json!("range"))
    );
    let unavailable = PlayerSdk::new(UnavailableStorage);
    let unavailable_response = unavailable.execute(fixture.request("none.vtt", "0", "5000000"));
    assert_eq!(
        serde_json::to_value(unavailable_response).unwrap()["error"]["code"],
        "unsupported"
    );
    for field in ["expected_revision", "idempotency_key", "timeout_ms"] {
        assert_eq!(
            schema["data"]["request"]["properties"]["context"]["properties"][field]["type"],
            "null"
        );
        let mut request = fixture.request("context.vtt", "0", "5000000");
        match field {
            "expected_revision" => request.context.expected_revision = Some("1".into()),
            "idempotency_key" => request.context.idempotency_key = Some("key".into()),
            _ => request.context.timeout_ms = Some(1000),
        }
        let (code, result) = fixture.cli(&request);
        assert_eq!(code, 3, "{result}");
        assert_eq!(result["error"]["code"], "unsupported");
        assert!(!fixture.path.join("context.vtt").exists());
    }
    assert_eq!(
        fs::read_to_string(fixture.path.join("source.srt")).unwrap(),
        SOURCE
    );
}

#[test]
fn vtt_to_srt_speaker_policy_and_losses_outside_selection_require_approval() {
    kinekino_platform::install();
    let fixture = Fixture::new();
    let source = "WEBVTT\n\n00:00:01.000 --> 00:00:03.000\n<v 화자>첫째</v>\n\n00:00:04.000 --> 00:00:06.000\n둘째\n\n";
    fs::write(fixture.path.join("source.srt"), source).unwrap();
    let mut request = fixture.request("prefix.srt", "2000000", "5000000");
    request.payload["input_format"] = json!("vtt");
    request.payload["output_format"] = json!("srt");
    request.payload["speaker_policy"] = json!("prefix");
    let (code, result) = fixture.cli(&request);
    assert_eq!(code, 3, "{result}");
    assert_eq!(result["error"]["code"], "needs_input");
    assert!(!fixture.path.join("prefix.srt").exists());
    request.payload["allow_loss"] = json!(true);
    let (code, result) = fixture.cli(&request);
    assert_eq!(code, 0, "{result}");
    assert_eq!(
        fs::read_to_string(fixture.path.join("prefix.srt")).unwrap(),
        "1\n00:00:00,000 --> 00:00:01,000\n[화자] 첫째\n\n2\n00:00:02,000 --> 00:00:03,000\n둘째\n\n"
    );
    request.payload["output_path"] = json!(fixture.path.join("outside.srt"));
    request.payload["range"] = json!({"in_us":"4000000","out_us":"5000000"});
    request.payload["speaker_policy"] = json!("omit");
    request.payload["allow_loss"] = json!(false);
    let (code, result) = fixture.cli(&request);
    assert_eq!(code, 3, "{result}");
    assert!(
        result["error"]["details"]["losses"]
            .as_array()
            .unwrap()
            .iter()
            .any(|loss| loss["code"] == "speaker_structure_removed")
    );
    assert!(!fixture.path.join("outside.srt").exists());
    request.payload["allow_loss"] = json!(true);
    let (code, result) = fixture.cli(&request);
    assert_eq!(code, 0, "{result}");
    assert_eq!(
        fs::read_to_string(fixture.path.join("outside.srt")).unwrap(),
        "1\n00:00:00,000 --> 00:00:01,000\n둘째\n\n"
    );
    assert_eq!(
        fs::read_to_string(fixture.path.join("source.srt")).unwrap(),
        source
    );
}

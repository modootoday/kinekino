//! `production.export_snapshot` through `kinekino --input -` on the build host (roadmap M4-3).
//! The local PNG crop path is already wired on every OS; these fixtures lock the Linux CLI
//! contract (schema, capabilities, publication refusal) without FFmpeg, network fetch or inference.
use serde_json::{Value, json};
use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

// 4x4 RGB PNG (solid red), generated once with ffmpeg; SHA256 below.
const SOURCE_HEX: &str = "89504e470d0a1a0a0000000d49484452000000040000000408020000002693092900000009704859730000000100000001004f25c4d60000001049444154789c63f8cbc000470cc47100ab330fd10691957b0000000049454e44ae426082";
const SOURCE_SHA256: &str = "2ed76eb02adfa38f725e7d01e201a59dcdc81a626784389a64c3db429ca53a32";

fn source_bytes() -> Vec<u8> {
    (0..SOURCE_HEX.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&SOURCE_HEX[i..i + 2], 16).unwrap())
        .collect()
}

fn sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

struct Fixture {
    dir: PathBuf,
    source: PathBuf,
    output: PathBuf,
}

impl Fixture {
    fn new(tag: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("kk-snapshot-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let source = dir.join("src.png");
        std::fs::write(&source, source_bytes()).unwrap();
        Self {
            output: dir.join("out.png"),
            source,
            dir,
        }
    }
    fn payload(&self) -> Value {
        json!({
            "source_path": &self.source,
            "expected_source_sha256": SOURCE_SHA256,
            "output_path": &self.output,
            "crop": {"x": 1, "y": 1, "width": 2, "height": 2}
        })
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

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

fn export(payload: Value) -> Value {
    call(&request("production.export_snapshot", "snap", payload)).1
}

#[test]
fn crops_png_and_refuses_overwrite_and_stale_hash() {
    let fixture = Fixture::new("ok");
    let payload = fixture.payload();

    let ok = export(payload.clone());
    assert_eq!(ok["ok"], true, "{ok}");
    assert_eq!(ok["data"]["source_width"], 4);
    assert_eq!(ok["data"]["source_height"], 4);
    assert_eq!(ok["data"]["source_sha256"], SOURCE_SHA256);
    assert_eq!(ok["data"]["metadata_preserved"], false);
    assert!(ok["data"].get("source_pts_us").is_none());
    assert_eq!(
        ok["data"]["crop"],
        json!({"x":1,"y":1,"width":2,"height":2})
    );
    let published = std::fs::read(&fixture.output).unwrap();
    assert!(published.starts_with(b"\x89PNG"));
    assert_eq!(ok["data"]["output_sha256"], sha256(&published));
    assert_eq!(std::fs::read(&fixture.source).unwrap(), source_bytes());

    let again = export(payload.clone());
    assert_eq!(again["error"]["code"], "revision_conflict");
    assert_eq!(std::fs::read(&fixture.output).unwrap(), published);

    let mut stale = payload;
    stale["expected_source_sha256"] = json!("0".repeat(64));
    stale["output_path"] = json!(fixture.dir.join("other.png"));
    assert_eq!(export(stale)["error"]["code"], "revision_conflict");
    assert!(!fixture.dir.join("other.png").exists());
}

#[test]
fn cli_schema_and_capabilities_mark_snapshot_supported() {
    let (code, schema) = call(&request(
        "schema",
        "schema",
        json!({"command": "production.export_snapshot"}),
    ));
    assert_eq!(code, Some(0), "{schema}");
    assert_eq!(schema["ok"], true, "{schema}");
    assert_eq!(schema["data"]["status"], "supported");
    assert_eq!(schema["data"]["command"], "production.export_snapshot");
    assert_eq!(schema["data"]["source_pts_available"], false);
    assert_eq!(schema["data"]["automatic_retry"], false);

    let (code, capabilities) = call(&request("capabilities", "caps", json!({})));
    assert_eq!(code, Some(0), "{capabilities}");
    assert_eq!(capabilities["ok"], true, "{capabilities}");
    assert_eq!(capabilities["data"]["snapshot_export"]["available"], true);
    assert_eq!(
        capabilities["data"]["snapshot_export"]["ffmpeg_required"],
        false
    );
    assert_eq!(capabilities["data"]["snapshot_export"]["overwrite"], false);
    assert_eq!(
        capabilities["data"]["snapshot_export"]["metadata_preserved"],
        false
    );
    let status = capabilities["data"]["commands"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["command"] == "production.export_snapshot")
        .unwrap()["status"]
        .clone();
    assert_eq!(status, "supported");
}

#[test]
fn invalid_payloads_and_unsupported_context_do_not_publish() {
    let fixture = Fixture::new("invalid");
    for (payload, expected) in [
        (
            {
                let mut payload = fixture.payload();
                payload["unknown"] = json!(true);
                payload
            },
            "invalid_argument",
        ),
        (
            {
                let mut payload = fixture.payload();
                payload["crop"] = json!({"x": u32::MAX, "y": 0, "width": 2, "height": 1});
                payload
            },
            "invalid_argument",
        ),
        (
            {
                let mut payload = fixture.payload();
                payload["output_path"] = json!("relative.png");
                payload
            },
            "invalid_argument",
        ),
        (
            {
                let mut payload = fixture.payload();
                payload["source_path"] = json!("https://example.invalid/frame.png");
                payload
            },
            "invalid_argument",
        ),
    ] {
        let response = export(payload);
        assert_eq!(response["error"]["code"], expected, "{response}");
        assert!(!fixture.output.exists(), "{response}");
    }

    let mut timed = request("production.export_snapshot", "timed", fixture.payload());
    timed["context"]["timeout_ms"] = json!(1000);
    let (code, response) = call(&timed);
    assert_ne!(code, Some(0), "{response}");
    assert_eq!(response["error"]["code"], "unsupported");
    assert!(!fixture.output.exists());

    let mut revisioned = request("production.export_snapshot", "rev", fixture.payload());
    revisioned["context"]["expected_revision"] = json!("1");
    assert_eq!(call(&revisioned).1["error"]["code"], "unsupported");

    let mut keyed = request("production.export_snapshot", "key", fixture.payload());
    keyed["context"]["idempotency_key"] = json!("key");
    assert_eq!(call(&keyed).1["error"]["code"], "unsupported");

    assert_eq!(std::fs::read(&fixture.source).unwrap(), source_bytes());
}

#[test]
fn crc_damage_with_matching_hash_is_not_published() {
    let fixture = Fixture::new("crc");
    let mut damaged = source_bytes();
    damaged[29] ^= 1;
    std::fs::write(&fixture.source, &damaged).unwrap();
    let mut payload = fixture.payload();
    payload["expected_source_sha256"] = json!(sha256(&damaged));
    let response = export(payload);
    assert_eq!(response["error"]["code"], "invalid_argument", "{response}");
    assert!(!fixture.output.exists());
    assert_eq!(std::fs::read(&fixture.source).unwrap(), damaged);
}

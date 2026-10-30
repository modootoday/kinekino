//! `production.export_snapshot` through `kinekino --input -` on every OS (plan P4, inventory #4).
use serde_json::{Value, json};
use std::{io::Write, process::Command, process::Stdio};

// 4x4 RGB PNG (solid red), generated once with ffmpeg; SHA256 below.
const SOURCE_HEX: &str = "89504e470d0a1a0a0000000d49484452000000040000000408020000002693092900000009704859730000000100000001004f25c4d60000001049444154789c63f8cbc000470cc47100ab330fd10691957b0000000049454e44ae426082";
const SOURCE_SHA256: &str = "2ed76eb02adfa38f725e7d01e201a59dcdc81a626784389a64c3db429ca53a32";

fn execute(payload: Value) -> Value {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kinekino"))
        .args(["--input", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let request = json!({"command":"production.export_snapshot","context":{"request_id":"snap","api_version":"1"},"payload":payload});
    child
        .stdin
        .take()
        .unwrap()
        .write_all(request.to_string().as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    serde_json::from_slice(&out.stdout).unwrap()
}

#[test]
fn crops_png_and_refuses_overwrite_and_stale_hash() {
    let dir = std::env::temp_dir().join(format!("kk-snapshot-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let bytes: Vec<u8> = (0..SOURCE_HEX.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&SOURCE_HEX[i..i + 2], 16).unwrap())
        .collect();
    let source = dir.join("src.png");
    std::fs::write(&source, bytes).unwrap();
    let output = dir.join("out.png");
    let payload = json!({"source_path":source,"expected_source_sha256":SOURCE_SHA256,"output_path":output,"crop":{"x":1,"y":1,"width":2,"height":2}});

    let ok = execute(payload.clone());
    assert_eq!(ok["ok"], true, "{ok}");
    assert_eq!(ok["data"]["source_width"], 4);
    assert_eq!(ok["data"]["source_sha256"], SOURCE_SHA256);
    assert!(std::fs::read(&output).unwrap().starts_with(b"\x89PNG"));

    let again = execute(payload.clone());
    assert_eq!(again["error"]["code"], "revision_conflict");

    let mut stale = payload;
    stale["expected_source_sha256"] = json!("0".repeat(64));
    stale["output_path"] = json!(dir.join("other.png"));
    assert_eq!(execute(stale)["error"]["code"], "revision_conflict");
    let _ = std::fs::remove_dir_all(&dir);
}

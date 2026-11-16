//! `--model-root` through the normal CLI on every OS, against the typed SDK on the same root.
//! Synthetic catalog declarations and stub files; no model is loaded or run.
#[path = "support/cli_exe.rs"]
mod cli_exe;
use kinekino_player::{
    application::api::{PlayerSdk, Request},
    infrastructure::model_catalog::LocalModelCatalog,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const FILES: [(&str, &[u8]); 2] = [("weights.onnx", b"stub"), ("config.json", b"{}")];

fn sha(bytes: &[u8]) -> String {
    kinekino_media_contracts::hex_lower(Sha256::digest(bytes))
}

/// A fresh catalog root with one declared model whose files are all present.
fn catalog() -> PathBuf {
    let root = std::env::temp_dir().join(format!("kk-model-root-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let revision = "a".repeat(40);
    let artifacts = root.join("artifacts/technical-only").join(&revision);
    fs::create_dir_all(&artifacts).unwrap();
    let files: Vec<Value> = FILES
        .iter()
        .map(|(path, bytes)| {
            fs::write(artifacts.join(path), bytes).unwrap();
            json!({"path": path, "bytes": bytes.len().to_string(), "sha256": sha(bytes)})
        })
        .collect();
    let catalog = json!({"schema_version":"model-catalog/1","models":[{
        "schema_version":"model-manifest/1","model_id":"technical-only","task":"text","repo_id":"fixture/model","revision":revision,
        "files":files,"format":"onnx","quantization":"fp32","source_model":null,"conversion":null,"tokenizer_files":[],"config_files":["config.json"],
        "license":{"identifier":"fixture-only","reference":"https://example.invalid/LICENSE"},"supported_languages":["ko"],"runtime":"fixture-runtime",
        "min_runtime_version":"1.0","declared_measured_memory":{"ram_bytes":"1","vram_bytes":"0","evidence_id":"fixture-unverified"},"evaluation_id":null}]});
    fs::write(
        root.join("catalog.json"),
        serde_json::to_vec_pretty(&catalog).unwrap(),
    )
    .unwrap();
    root.canonicalize().unwrap()
}

fn request(command: &str, payload: Value) -> Request {
    let mut request = Request::new(command);
    request.payload = payload;
    request
}

/// Runs one request through a fresh CLI process; returns the response and the exit status.
fn cli(root: &Path, request: &Request) -> (Value, bool) {
    let input = root.with_extension(format!("{}.json", request.command));
    fs::write(&input, serde_json::to_vec(request).unwrap()).unwrap();
    let output = Command::new(cli_exe::kinekino_cli_exe())
        .arg("--input")
        .arg(&input)
        .arg("--model-root")
        .arg(root)
        .output()
        .unwrap();
    let _ = fs::remove_file(&input);
    let response = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|_| panic!("{}", String::from_utf8_lossy(&output.stderr)));
    (response, output.status.success())
}

#[test]
fn model_root_reads_and_verifies_through_the_cli_like_the_typed_sdk() {
    kinekino_platform::install();
    let root = catalog();
    let sdk = PlayerSdk::default().with_model_catalog(LocalModelCatalog::open(&root).unwrap());
    let catalog_sha = sha(&fs::read(root.join("catalog.json")).unwrap());
    let model = json!({"model_id":"technical-only","revision":"a".repeat(40)});
    let mut verify = model.clone();
    verify["catalog_sha256"] = json!(catalog_sha);
    verify["maximum_bytes"] = json!("6");
    for request in [
        request("models.list", json!({})),
        request("models.inspect", model),
        request("models.verify", verify.clone()),
    ] {
        let (response, ok) = cli(&root, &request);
        assert!(ok, "{}: {response}", request.command);
        let typed = serde_json::to_value(sdk.execute(request.clone())).unwrap();
        assert_eq!(response["data"], typed["data"], "{}", request.command);
    }
    let (verified, _) = cli(&root, &request("models.verify", verify.clone()));
    assert_eq!(verified["data"]["catalog_sha256"], catalog_sha);
    assert_eq!(verified["data"]["data"]["all_declared_files_matched"], true);
    assert_eq!(verified["data"]["runtime_admitted"], false);
    let mut stale = verify;
    stale["catalog_sha256"] = json!("b".repeat(64));
    let (refused, ok) = cli(&root, &request("models.verify", stale));
    assert!(!ok);
    assert_eq!(refused["error"]["code"], "revision_conflict");
    for (path, bytes) in FILES {
        let stored = root
            .join("artifacts/technical-only")
            .join("a".repeat(40))
            .join(path);
        assert_eq!(fs::read(stored).unwrap(), bytes, "originals untouched");
    }
    let _ = fs::remove_dir_all(&root);
}

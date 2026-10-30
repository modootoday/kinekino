//! Synthetic catalog declarations and ordinary files, never actual model execution.
#![cfg(windows)]
#[path = "support/cli_exe.rs"]
mod cli_exe;
#[path = "sprite_capture_cli/custody.rs"]
mod custody;
use kinekino_player::{
    application::api::{PlayerSdk, Request},
    domain::model_catalog::ModelCatalog,
    infrastructure::model_catalog::LocalModelCatalog,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
struct Owner(Child);
impl Drop for Owner {
    fn drop(&mut self) {
        if !matches!(self.0.try_wait(), Ok(Some(_))) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}
fn root() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "model-catalog-cli-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    root
}
fn fixture() -> Value {
    json!({"schema_version":"model-catalog/1","models":[{
        "schema_version":"model-manifest/1","model_id":"technical-only","task":"text","repo_id":"fixture/model","revision":"a".repeat(40),
        "files":[{"path":"weights.onnx","bytes":"4","sha256":"a".repeat(64)},{"path":"config.json","bytes":"8","sha256":"b".repeat(64)},{"path":"tokenizer.json","bytes":"5","sha256":"c".repeat(64)}],
        "format":"onnx","quantization":"fp32","source_model":null,"conversion":null,"tokenizer_files":["tokenizer.json"],"config_files":["config.json"],"license":{"identifier":"fixture-only","reference":"https://example.invalid/LICENSE"},"supported_languages":["ko","ja"],"runtime":"fixture-runtime","min_runtime_version":"1.0","declared_measured_memory":{"ram_bytes":"123","vram_bytes":"0","evidence_id":"fixture-unverified"},"evaluation_id":null
    }]})
}
fn call(root: &Path, label: &str, catalog: Option<&Path>, request: &Request, ok: bool) -> Value {
    let folder = root.join(label);
    fs::create_dir(&folder).unwrap();
    let input = folder.join("request.json");
    fs::write(&input, serde_json::to_vec_pretty(request).unwrap()).unwrap();
    let mut command = Command::new(crate::cli_exe::kinekino_cli_exe());
    command
        .args(["--input"])
        .arg(&input)
        .stdin(Stdio::null())
        .stdout(File::create(folder.join("stdout.json")).unwrap())
        .stderr(File::create(folder.join("stderr.raw")).unwrap());
    if let Some(catalog) = catalog {
        command.arg("--model-root").arg(catalog);
    }
    let mut child = Owner(command.spawn().unwrap());
    let cutoff = Instant::now() + Duration::from_secs(15);
    let status = loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            break status;
        }
        assert!(Instant::now() < cutoff, "original CLI timed out: {label}");
        std::thread::sleep(Duration::from_millis(20));
    };
    fs::write(folder.join("PROCESS.json"),serde_json::to_vec_pretty(&json!({"pid":child.0.id(),"exit_code":status.code(),"original_child_waited":true,"forced":false})).unwrap()).unwrap();
    let response: Value =
        serde_json::from_slice(&fs::read(folder.join("stdout.json")).unwrap()).unwrap();
    assert_eq!(response["ok"], ok, "{label}: {response}");
    assert_eq!(status.success(), ok, "{label}: {response}");
    response
}
fn request(command: &str, payload: Value) -> Request {
    let mut r = Request::new(command);
    r.payload = payload;
    r
}
#[test]
fn normal_cli_catalog_reads_and_discovery_preserve_declarations_without_admission() {
    kinekino_platform::install();
    let root = root();
    let catalog = root.join("catalog");
    fs::create_dir(&catalog).unwrap();
    let path = catalog.join("catalog.json");
    fs::write(&path, serde_json::to_vec_pretty(&fixture()).unwrap()).unwrap();
    let artifacts = catalog
        .join("artifacts/technical-only")
        .join("a".repeat(40));
    fs::create_dir_all(&artifacts).unwrap();
    fs::write(artifacts.join("weights.onnx"), b"stub").unwrap();
    fs::write(artifacts.join("config.json"), b"{}").unwrap();
    let guard = custody::Guard::acquire(&path);
    let binary = custody::Guard::acquire(Path::new(crate::cli_exe::kinekino_cli_exe()));
    let weights = custody::Guard::acquire(&artifacts.join("weights.onnx"));
    let config = custody::Guard::acquire(&artifacts.join("config.json"));
    fs::write(root.join("WITNESS.json"),serde_json::to_vec_pretty(&json!({"synthetic_catalog":true,"actual_model":false,"catalog":guard.evidence(),"cli":binary.evidence(),"weights_are_four_stub_bytes":true,"quality_approved":false})).unwrap()).unwrap();
    let caps = call(
        &root,
        "capabilities",
        Some(&catalog),
        &Request::new("capabilities"),
        true,
    );
    assert_eq!(caps["data"]["model_catalog"]["read_only"], true);
    let mut seen = BTreeSet::new();
    let mut inventory = Vec::new();
    for item in caps["data"]["commands"].as_array().unwrap() {
        let command = item["command"].as_str().unwrap();
        assert!(seen.insert(command));
        let schema = call(
            &root,
            &format!("schema-{command}"),
            Some(&catalog),
            &request("schema", json!({"command":command})),
            true,
        );
        assert_eq!(schema["data"]["status"], item["status"], "{command}");
        inventory.push(json!({"command":command,"configured_status":item["status"],"functional_acceptance":"not_proven_by_discovery"}));
    }
    assert_eq!(seen.len(), 119);
    fs::write(
        root.join("INVENTORY.json"),
        serde_json::to_vec_pretty(&inventory).unwrap(),
    )
    .unwrap();
    let list = call(
        &root,
        "list",
        Some(&catalog),
        &request("models.list", json!({})),
        true,
    );
    assert_eq!(list["data"]["catalog_sha256"], guard.sha256);
    assert_eq!(list["data"]["data"]["models"].as_array().unwrap().len(), 1);
    let filtered = call(
        &root,
        "filter",
        Some(&catalog),
        &request("models.list", json!({"task":"asr"})),
        true,
    );
    assert!(
        filtered["data"]["data"]["models"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let inspect = request(
        "models.inspect",
        json!({"model_id":"technical-only","revision":"a".repeat(40)}),
    );
    let first = call(&root, "inspect", Some(&catalog), &inspect, true);
    assert_eq!(
        first["data"]["data"]["local_files"][0]["state"],
        "present_unverified"
    );
    assert_eq!(
        first["data"]["data"]["local_files"][1]["state"],
        "size_mismatch"
    );
    assert_eq!(first["data"]["data"]["local_files"][2]["state"], "missing");
    assert_eq!(first["data"]["runtime_admitted"], false);
    assert_eq!(first["data"]["quality_approved"], false);
    assert_eq!(first["data"]["license_permission"], "not_verified");
    assert_eq!(
        first["data"]["data"]["declared_measurements"],
        "not_verified"
    );
    assert_eq!(
        first["data"],
        call(&root, "reopen", Some(&catalog), &inspect, true)["data"]
    );
    let absent = call(&root, "unconfigured", None, &inspect, false);
    assert_eq!(absent["error"]["code"], "unsupported");
    for command in ["models.install", "models.remove"] {
        let response = call(
            &root,
            command,
            Some(&catalog),
            &request(command, json!({})),
            false,
        );
        assert_eq!(response["error"]["code"], "unsupported");
    }
    let checked = call(
        &root,
        "verify-mismatches",
        Some(&catalog),
        &request(
            "models.verify",
            json!({"model_id":"technical-only","revision":"a".repeat(40),"catalog_sha256":guard.sha256,"maximum_bytes":"17"}),
        ),
        false,
    );
    assert_eq!(checked["error"]["code"], "needs_input");
    assert_eq!(
        checked["error"]["details"]["data"]["all_declared_files_matched"],
        false
    );
    assert_eq!(
        checked["error"]["details"]["data"]["verification"]["files"][0]["state"],
        "hash_mismatch"
    );
    assert_eq!(
        checked["error"]["details"]["data"]["verification"]["files"][1]["state"],
        "size_mismatch"
    );
    assert_eq!(
        checked["error"]["details"]["data"]["verification"]["files"][2]["state"],
        "missing"
    );
    let bad = call(
        &root,
        "mutable-revision",
        Some(&catalog),
        &request(
            "models.inspect",
            json!({"model_id":"technical-only","revision":"main"}),
        ),
        false,
    );
    assert_eq!(bad["error"]["code"], "invalid_argument");
    let missing = call(
        &root,
        "unknown-model",
        Some(&catalog),
        &request(
            "models.inspect",
            json!({"model_id":"absent","revision":"a".repeat(40)}),
        ),
        false,
    );
    assert_eq!(missing["error"]["code"], "not_found");
    let mut context = inspect.clone();
    context.context.expected_revision = Some("1".into());
    assert_eq!(
        call(&root, "mutation-context", Some(&catalog), &context, false)["error"]["code"],
        "invalid_argument"
    );
    guard.assert_unchanged();
    binary.assert_unchanged();
    weights.assert_unchanged();
    config.assert_unchanged();
    assert!(!artifacts.join("tokenizer.json").exists());
}

fn sha(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}
fn installed_fixture(root: &Path, weights: &[u8]) -> (PathBuf, Value) {
    let catalog = root.join("catalog");
    fs::create_dir(&catalog).unwrap();
    let artifacts = catalog
        .join("artifacts/technical-only")
        .join("a".repeat(40));
    fs::create_dir_all(&artifacts).unwrap();
    let mut manifest = fixture();
    for (index, (name, bytes)) in [
        ("weights.onnx", weights),
        ("config.json", b"{}".as_slice()),
        ("tokenizer.json", b"tok".as_slice()),
    ]
    .into_iter()
    .enumerate()
    {
        fs::write(artifacts.join(name), bytes).unwrap();
        manifest["models"][0]["files"][index]["bytes"] = json!(bytes.len().to_string());
        manifest["models"][0]["files"][index]["sha256"] = json!(sha(bytes));
    }
    fs::write(
        catalog.join("catalog.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
    (catalog, manifest)
}
#[test]
fn normal_cli_verifies_exact_catalog_files_without_persistent_runtime_admission() {
    kinekino_platform::install();
    let root = root();
    let (catalog, _) = installed_fixture(&root, b"stub");
    let guard = custody::Guard::acquire(&catalog.join("catalog.json"));
    let input = json!({"model_id":"technical-only","revision":"a".repeat(40),"catalog_sha256":guard.sha256,"maximum_bytes":"9"});
    let response = call(
        &root,
        "verify",
        Some(&catalog),
        &request("models.verify", input.clone()),
        true,
    );
    assert_eq!(response["data"]["data"]["all_declared_files_matched"], true);
    assert_eq!(
        response["data"]["data"]["integrity"],
        "matched_at_verification"
    );
    assert_eq!(response["data"]["data"]["persistent_admission"], false);
    assert_eq!(response["data"]["runtime_admitted"], false);
    assert_eq!(response["data"]["quality_approved"], false);
    assert_eq!(
        response["data"],
        call(
            &root,
            "repeat",
            Some(&catalog),
            &request("models.verify", input.clone()),
            true
        )["data"]
    );
    let inspected = call(
        &root,
        "inspect-after-verify",
        Some(&catalog),
        &request(
            "models.inspect",
            json!({"model_id":"technical-only","revision":"a".repeat(40)}),
        ),
        true,
    );
    assert_eq!(inspected["data"]["data"]["integrity"], "not_verified");
    let mut stale = input.clone();
    stale["catalog_sha256"] = json!("b".repeat(64));
    assert_eq!(
        call(
            &root,
            "stale-catalog",
            Some(&catalog),
            &request("models.verify", stale),
            false
        )["error"]["code"],
        "revision_conflict"
    );
    let mut small = input.clone();
    small["maximum_bytes"] = json!("8");
    assert_eq!(
        call(
            &root,
            "small-budget",
            Some(&catalog),
            &request("models.verify", small),
            false
        )["error"]["code"],
        "resource_exhausted"
    );
    let mut timed = request("models.verify", input);
    timed.context.timeout_ms = Some(120001);
    assert_eq!(
        call(&root, "invalid-timeout", Some(&catalog), &timed, false)["error"]["code"],
        "invalid_argument"
    );
    guard.assert_unchanged();
}
#[test]
fn mid_hash_cancel_timeout_and_busy_keep_original_file_guards_until_return() {
    kinekino_platform::install();
    use kinekino_player::application::{
        api::{ApiError, ErrorCode},
        model_catalog::{VerifyModel, verify_model},
        review_source::MutationControl,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct Continue;
    impl MutationControl for Continue {
        fn checkpoint(&self) -> Result<(), ApiError> {
            Ok(())
        }
    }
    struct Stop<'a> {
        store: &'a LocalModelCatalog,
        input: &'a VerifyModel,
        paths: Vec<PathBuf>,
        calls: AtomicUsize,
        timeout: bool,
    }
    impl MutationControl for Stop<'_> {
        fn checkpoint(&self) -> Result<(), ApiError> {
            let n = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
            if n == 20 {
                for path in &self.paths {
                    assert!(
                        fs::OpenOptions::new().write(true).open(path).is_err(),
                        "Original file guard was released early: {}",
                        path.display()
                    );
                }
                assert_eq!(
                    verify_model(self.store, self.input, &Continue)
                        .unwrap_err()
                        .code,
                    ErrorCode::ResourceExhausted
                );
            }
            if n >= 40 {
                Err(ApiError::new(
                    if self.timeout {
                        ErrorCode::Timeout
                    } else {
                        ErrorCode::Cancelled
                    },
                    "explicit mid-hash stop",
                ))
            } else {
                Ok(())
            }
        }
    }
    let root = root();
    let (catalog, _) = installed_fixture(&root, &vec![0x5A; 8 * 1024 * 1024]);
    let catalog_bytes = fs::read(catalog.join("catalog.json")).unwrap();
    let store = LocalModelCatalog::open(&catalog).unwrap();
    let input = VerifyModel {
        model_id: "technical-only".into(),
        revision: "a".repeat(40),
        catalog_sha256: sha(&catalog_bytes),
        maximum_bytes: kinekino_player::domain::api_values::DecimalU64(8 * 1024 * 1024 + 5),
    };
    let artifacts = catalog
        .join("artifacts/technical-only")
        .join("a".repeat(40));
    let paths = vec![
        catalog.join("catalog.json"),
        artifacts.join("weights.onnx"),
        artifacts.join("config.json"),
        artifacts.join("tokenizer.json"),
    ];
    for timeout in [false, true] {
        let control = Stop {
            store: &store,
            input: &input,
            paths: paths.clone(),
            calls: AtomicUsize::new(0),
            timeout,
        };
        assert_eq!(
            verify_model(&store, &input, &control).unwrap_err().code,
            if timeout {
                ErrorCode::Timeout
            } else {
                ErrorCode::Cancelled
            }
        );
        assert_eq!(control.calls.load(Ordering::SeqCst), 40);
        for path in &paths {
            assert!(
                fs::OpenOptions::new().write(true).open(path).is_ok(),
                "Original guard leaked after stopped hash"
            );
        }
    }
    let result = verify_model(&store, &input, &Continue).unwrap();
    assert_eq!(result["data"]["all_declared_files_matched"], true);
    assert_eq!(
        fs::read(catalog.join("catalog.json")).unwrap(),
        catalog_bytes
    );
}
#[test]
fn invalid_catalog_paths_aliases_and_unfixed_provenance_are_rejected() {
    kinekino_platform::install();
    let base = fixture();
    for path in [
        "../escape",
        "/absolute",
        "C:/outside",
        "folder\\outside",
        "NUL.onnx",
        "x/../y",
        "folder/",
        "file.",
    ] {
        let mut value = base.clone();
        value["models"][0]["files"][0]["path"] = json!(path);
        assert!(
            serde_json::from_value::<ModelCatalog>(value)
                .unwrap()
                .validate()
                .is_err(),
            "{path}"
        );
    }
    for revision in ["main", "latest", "A123", "a".repeat(39).as_str()] {
        let mut value = base.clone();
        value["models"][0]["revision"] = json!(revision);
        assert!(
            serde_json::from_value::<ModelCatalog>(value)
                .unwrap()
                .validate()
                .is_err()
        );
    }
    let mut duplicate = base.clone();
    duplicate["models"]
        .as_array_mut()
        .unwrap()
        .push(base["models"][0].clone());
    assert!(
        serde_json::from_value::<ModelCatalog>(duplicate)
            .unwrap()
            .validate()
            .is_err()
    );
    let mut alias = base.clone();
    alias["models"][0]["files"][1]["path"] = json!("WEIGHTS.ONNX");
    assert!(
        serde_json::from_value::<ModelCatalog>(alias)
            .unwrap()
            .validate()
            .is_err()
    );
    let mut unknown = base.clone();
    unknown["models"][0]["remote_code"] = json!(true);
    assert!(serde_json::from_value::<ModelCatalog>(unknown).is_err());
    let mut dangling = base.clone();
    dangling["models"][0]["tokenizer_files"] = json!(["absent.json"]);
    assert!(
        serde_json::from_value::<ModelCatalog>(dangling)
            .unwrap()
            .validate()
            .is_err()
    );
    let root = root();
    fs::write(root.join("catalog.json"), b"{}").unwrap();
    assert!(LocalModelCatalog::open(&root).is_ok());
    let sdk = PlayerSdk::default()
        .with_model_catalog(LocalModelCatalog::open(&root).unwrap())
        .with_frame_capture(kinekino_player::application::frame_capture::UnavailableFrameCapture)
        .with_playback_status(
            kinekino_player::application::playback_status::UnavailablePlaybackStatus,
        )
        .with_playback_control(
            kinekino_player::application::playback_control::UnavailablePlaybackControl,
        )
        .with_review_store(kinekino_player::application::model_review::UnavailableReviewStore);
    assert_eq!(
        sdk.execute(Request::new("capabilities")).data.unwrap()["model_catalog"]["available"],
        true
    );
    assert_eq!(
        sdk.execute(request("models.list", json!({})))
            .error
            .unwrap()
            .code,
        kinekino_player::application::api::ErrorCode::InvalidArgument
    );
}

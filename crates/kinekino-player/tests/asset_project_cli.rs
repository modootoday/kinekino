#![cfg(windows)]
use kinekino_player::application::api::AssetClient;
#[path = "support/owned_process_evidence.rs"]
pub mod owned_process_evidence;
use kinekino_player::{
    application::{
        api::{PlayerSdk, Request, RequestContext},
        asset_projects::InspectAssetProject,
    },
    domain::{api_values::DecimalU64, asset_generation::AssetDocument},
    infrastructure::asset_projects::AssetProjectFiles,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{ffi::OsString, fs, path::PathBuf, time::Duration};
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn document() -> AssetDocument {
    serde_json::from_value(json!({"schema_version":"asset-project/1","document_id":"asset-1","revision":"1","sources":{},"order_basis":"single_frame","frames":[],"layers":[{"layer_id":"foreground","kind":"foreground","visible":true,"locked":false,"transform":{"trim":false,"mask_threshold":0,"feather_radius":0,"scale":{"numerator":1,"denominator":1},"pivot":{"x":500000,"y":1000000},"alpha_mode":"straight","color":"srgb"}}]})).unwrap()
}
struct Harness {
    root: PathBuf,
    store: PathBuf,
    executable: PathBuf,
    digest: String,
    next: usize,
}
impl Harness {
    fn new() -> Self {
        let root = PathBuf::from(
            std::env::var_os("ASSET_PROJECT_CLI_EVIDENCE_DIR")
                .expect("Explicit fresh evidence directory"),
        );
        assert!(root.is_absolute());
        fs::create_dir(&root).unwrap();
        let store = root.join("store");
        fs::create_dir(&store).unwrap();
        let executable = PathBuf::from(std::env::var_os("ASSET_PROJECT_CLI").expect("Pinned CLI"));
        assert!(executable.is_absolute());
        let digest = hash(&fs::read(&executable).unwrap());
        assert_eq!(
            std::env::var("ASSET_PROJECT_CLI_SHA256")
                .unwrap()
                .to_lowercase(),
            digest
        );
        fs::write(root.join("producer.json"), serde_json::to_vec_pretty(&json!({"path":executable,"sha256":digest,"scope":"asset_project_storage_cli","media_provenance_verified":false,"quality_approved":false})).unwrap()).unwrap();
        Self {
            root,
            store,
            executable,
            digest,
            next: 0,
        }
    }
    fn call(&mut self, request: &Request, rooted: bool, extra: &[&str], exit: i32) -> Value {
        let call = self.root.join(format!("call-{:03}", self.next));
        self.next += 1;
        fs::create_dir(&call).unwrap();
        let bytes = serde_json::to_vec(request).unwrap();
        fs::write(call.join("request.json"), &bytes).unwrap();
        let mut args: Vec<OsString> = ["--input", "-", "--json"]
            .into_iter()
            .map(Into::into)
            .collect();
        if rooted {
            args.extend([
                OsString::from("--asset-project-root"),
                self.store.clone().into_os_string(),
            ]);
        }
        args.extend(extra.iter().map(OsString::from));
        let output = match owned_process_evidence::run_process_with_arguments(
            self.executable.clone(),
            &call,
            &bytes,
            args,
            Duration::from_secs(30),
            true,
        ) {
            Ok(output) => output,
            Err(failure) => {
                let failure = failure.closeout();
                if matches!(
                    failure.ownership,
                    owned_process_evidence::Ownership::Live(_)
                ) {
                    eprintln!("Original CLI family remains incomplete: {failure:?}");
                    std::process::exit(101);
                }
                panic!("CLI failed: {failure:?}");
            }
        };
        assert_eq!(
            output.receipt.child_exit.map(i64::from),
            Some(i64::from(exit))
        );
        assert!(!output.receipt.forced);
        output.check_deadline(&call).unwrap();
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        fs::write(
            call.join("response.json"),
            serde_json::to_vec_pretty(&value).unwrap(),
        )
        .unwrap();
        assert_eq!(hash(&fs::read(&self.executable).unwrap()), self.digest);
        value
    }
}
fn request(command: &str, payload: Value) -> Request {
    let mut request = Request::new(command);
    request.payload = payload;
    request
}

#[test]
#[ignore = "Fresh evidence and pinned normal CLI; real storage, draft values, no source/model/GUI admission"]
fn asset_project_cli_and_typed_storage_preserve_cas_history_and_local_boundaries() {
    kinekino_platform::install();
    let mut h = Harness::new();
    let caps = h.call(&Request::new("capabilities"), false, &[], 0);
    assert_eq!(caps["data"]["asset_projects"]["backend_available"], false);
    let inspect = request("asset_projects.inspect", json!({"document_id":"asset-1"}));
    assert_eq!(
        h.call(&inspect, false, &[], 3)["error"]["code"],
        "unsupported"
    );
    for command in [
        "asset_projects.create",
        "asset_projects.inspect",
        "asset_projects.save_revision",
    ] {
        let schema = h.call(&request("schema", json!({"command":command})), true, &[], 0);
        assert_eq!(schema["data"]["status"], "supported");
        assert_eq!(schema["data"]["constraints"]["source_admitted"], false);
        assert_eq!(schema["data"]["constraints"]["review_admitted"], false);
        assert_eq!(
            schema["data"]["constraints"]["build_assets_supported"],
            false
        );
        assert_eq!(schema["data"]["request"]["additionalProperties"], false);
    }
    let caps = h.call(&Request::new("capabilities"), true, &[], 0);
    assert_eq!(caps["data"]["asset_projects"]["backend_available"], true);
    let initial = document();
    let create = request("asset_projects.create", json!({"document":initial}));
    let first = h.call(&create, true, &[], 0);
    let sdk_root = h.root.join("typed-store");
    fs::create_dir(&sdk_root).unwrap();
    let sdk =
        PlayerSdk::default().with_asset_project_store(AssetProjectFiles::open(&sdk_root).unwrap());
    assert_eq!(
        serde_json::to_value(
            sdk.create_asset_project(RequestContext::default(), initial.clone())
                .unwrap()
        )
        .unwrap(),
        first["data"]
    );
    assert_eq!(
        h.call(&create, true, &[], 5)["error"]["code"],
        "revision_conflict"
    );
    let directory = h.store.join("asset-asset-1");
    let original_path = directory.join(format!(
        "r-1-{}.json",
        first["data"]["publication_sha256"].as_str().unwrap()
    ));
    let original = fs::read(&original_path).unwrap();
    let mut edited = initial;
    edited.revision = DecimalU64(2);
    edited.layers[0].transform.pivot.x = 0;
    let mut save = request("asset_projects.save_revision", json!({"document":edited}));
    assert_eq!(
        h.call(&save, true, &[], 2)["error"]["code"],
        "invalid_argument"
    );
    save.context.expected_revision = Some("1".into());
    let second = h.call(&save, true, &[], 0);
    assert_eq!(
        serde_json::to_value(
            sdk.save_asset_project(save.context.clone(), edited)
                .unwrap()
        )
        .unwrap(),
        second["data"]
    );
    assert_eq!(
        h.call(&save, true, &[], 5)["error"]["code"],
        "revision_conflict"
    );
    let historical = request(
        "asset_projects.inspect",
        json!({"document_id":"asset-1","publication_revision":"1"}),
    );
    let prior = h.call(&historical, true, &[], 0);
    assert_eq!(prior["data"]["publication"], first["data"]["publication"]);
    assert_eq!(prior["data"]["latest_publication_revision"], "2");
    assert_eq!(
        serde_json::to_value(
            sdk.inspect_asset_project(
                RequestContext::default(),
                InspectAssetProject {
                    document_id: "asset-1".into(),
                    publication_revision: Some(DecimalU64(1))
                }
            )
            .unwrap()
        )
        .unwrap(),
        prior["data"]
    );
    assert_eq!(h.call(&inspect, true, &[], 0)["data"], second["data"]);
    let mut injected = create.clone();
    injected.payload["root"] = json!("C:/untrusted");
    assert_eq!(
        h.call(&injected, true, &[], 2)["error"]["code"],
        "invalid_argument"
    );
    let mut timed = inspect.clone();
    timed.context.timeout_ms = Some(100);
    assert_eq!(
        h.call(&timed, true, &[], 2)["error"]["code"],
        "invalid_argument"
    );
    let mut replay = inspect.clone();
    replay.context.idempotency_key = Some("unsupported".into());
    assert_eq!(
        h.call(&replay, true, &[], 2)["error"]["code"],
        "invalid_argument"
    );
    assert_eq!(
        h.call(&inspect, true, &["--endpoint", "unused"], 2)["error"]["code"],
        "invalid_argument"
    );
    assert_eq!(
        h.call(&inspect, true, &["--serve-jobs"], 2)["error"]["code"],
        "invalid_argument"
    );
    assert_eq!(fs::read(&original_path).unwrap(), original);
    assert_eq!(h.call(&inspect, true, &[], 0)["data"], second["data"]);
    fs::write(h.root.join("verification.json"),serde_json::to_vec_pretty(&json!({"cli_calls":h.next,"typed_equivalent":true,"original_preserved":true,"reopened_each_call":true,"source_admitted":false,"quality_approved":false,"rendered":false,"gui_verified":false})).unwrap()).unwrap();
}

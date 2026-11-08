//! Asset source reimport through the Linux CLI (roadmap M4-7).
//! Without an asset-build host the command stays unsupported and writes nothing.
//! With `--serve-jobs` plus a pinned `--asset-build-config`, export then import
//! creates a new draft identity from original archive bytes. No inference.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

fn png(width: u32, height: u32, color: png::ColorType, pixels: &[u8], srgb: bool) -> Vec<u8> {
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, width, height);
        encoder.set_color(color);
        encoder.set_depth(png::BitDepth::Eight);
        if srgb {
            encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
        }
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(pixels).unwrap();
        writer.finish().unwrap();
    }
    bytes
}

fn image_png() -> Vec<u8> {
    png(
        2,
        1,
        png::ColorType::Rgba,
        &[255, 0, 0, 255, 0, 0, 255, 255],
        true,
    )
}

fn mask_png() -> Vec<u8> {
    png(2, 1, png::ColorType::Grayscale, &[255, 128], false)
}

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_kinekino"))
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn call(extra: &[&str], request: &Value) -> (Option<i32>, Value) {
    let mut child = bin()
        .args(["--input", "-"])
        .args(extra)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
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
        serde_json::from_slice(&output.stdout).unwrap_or_else(|_| {
            panic!(
                "stdout={} stderr={}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            )
        }),
    )
}

fn request(command: &str, payload: Value) -> Value {
    json!({"context":{"api_version":"1","request_id":command},"command":command,"payload":payload})
}

fn command_status(capabilities: &Value, name: &str) -> Value {
    capabilities["commands"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["command"] == name)
        .unwrap_or_else(|| panic!("{name} listed in {capabilities}"))["status"]
        .clone()
}

fn image_pin() -> Value {
    json!({
        "blob":{"artifact_id":"src-image","revision":"1","sha256":sha256(&image_png())},
        "width":2,
        "height":1,
        "origin":{"kind":"static_png"},
        "declared_color":"srgb"
    })
}

fn mask_pin() -> Value {
    json!({
        "blob":{"artifact_id":"src-mask","revision":"1","sha256":sha256(&mask_png())},
        "source":image_pin(),
        "encoding":"gray8_coverage",
        "origin":"imported_unverified"
    })
}

fn document(document_id: &str) -> Value {
    json!({
        "schema_version":"asset-project/1",
        "document_id":document_id,
        "revision":"1",
        "sources":{"frame":{"position":{"kind":"static_png"},"selected_image":image_pin()}},
        "order_basis":"single_frame",
        "frames":[{
            "state":"available",
            "frame_id":"frame",
            "input":{
                "image":image_pin(),
                "mask":mask_pin(),
                "review":{"kind":"unreviewed_acknowledged","mask":mask_pin()["blob"]}
            }
        }],
        "layers":[{
            "layer_id":"foreground",
            "kind":"foreground",
            "visible":true,
            "locked":false,
            "transform":{
                "trim":false,
                "mask_threshold":0,
                "feather_radius":0,
                "scale":{"numerator":1,"denominator":1},
                "pivot":{"x":500000,"y":1000000},
                "alpha_mode":"straight",
                "color":"srgb"
            }
        }]
    })
}

fn import_payload(
    archive_id: &str,
    manifest_sha256: &str,
    new_document_id: &str,
    ack: bool,
) -> Value {
    json!({
        "archive_id":archive_id,
        "manifest_sha256":manifest_sha256,
        "new_document_id":new_document_id,
        "acknowledge_unreviewed_masks":ack
    })
}

struct Host {
    child: Child,
    endpoint: String,
    projects: PathBuf,
    artifacts: PathBuf,
    base: PathBuf,
}

impl Host {
    fn start(tag: &str) -> Self {
        let base = std::env::temp_dir().join(format!("kk-assetimp-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let projects = base.join("projects");
        let artifacts = base.join("artifacts");
        let jobs = base.join("jobs");
        let inputs = base.join("inputs");
        for path in [&projects, &artifacts, &jobs, &inputs] {
            std::fs::create_dir_all(path).unwrap();
        }
        let image_path = inputs.join("source.png");
        let mask_path = inputs.join("mask.png");
        std::fs::write(&image_path, image_png()).unwrap();
        std::fs::write(&mask_path, mask_png()).unwrap();
        let producer = sha256(&std::fs::read(env!("CARGO_BIN_EXE_kinekino")).unwrap());
        let config = json!({
            "schema_version":"asset-build-host/1",
            "build":{
                "producer_sha256":producer,
                "project_root":projects,
                "artifact_root":artifacts,
                "images":{"src-image":{"pin":image_pin(),"path":image_path}},
                "masks":{"src-mask":{"pin":mask_pin(),"path":mask_path}}
            }
        });
        let bytes = serde_json::to_vec_pretty(&config).unwrap();
        let pin = sha256(&bytes);
        let config_path = base.join("asset-host.json");
        std::fs::write(&config_path, &bytes).unwrap();
        let mut child = bin()
            .args(["--serve-jobs", "--job-root"])
            .arg(&jobs)
            .args(["--asset-build-config"])
            .arg(&config_path)
            .args(["--asset-build-config-sha256", &pin])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().unwrap();
        let mut lines = BufReader::new(stdout).lines();
        let deadline = Instant::now() + Duration::from_secs(30);
        let ready: Value = loop {
            let line = lines
                .next()
                .unwrap_or_else(|| {
                    let stderr = child.stderr.take().map(|mut err| {
                        let mut text = String::new();
                        let _ = std::io::Read::read_to_string(&mut err, &mut text);
                        text
                    });
                    panic!("asset host produced no ready line; stderr={stderr:?}")
                })
                .unwrap();
            let value: Value = serde_json::from_str(&line).unwrap_or_else(|_| panic!("{line}"));
            if value["event"] == "job_host_ready" {
                break value;
            }
            assert!(Instant::now() < deadline, "host ready timed out: {value}");
        };
        let endpoint = ready["endpoint"].as_str().unwrap().to_owned();
        Self {
            child,
            endpoint,
            projects,
            artifacts,
            base,
        }
    }
    fn rpc(&self, request: Value) -> (Option<i32>, Value) {
        call(&["--endpoint", &self.endpoint], &request)
    }
    fn ok(&self, request: Value) -> Value {
        let (code, response) = self.rpc(request);
        assert_eq!(response["ok"], true, "{response}");
        assert_eq!(code, Some(0), "{response}");
        response["data"].clone()
    }
    fn error(&self, request: Value) -> String {
        let (code, response) = self.rpc(request);
        assert_eq!(response["ok"], false, "{response}");
        assert_ne!(code, Some(0), "{response}");
        response["error"]["code"].as_str().unwrap().to_owned()
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        if let Some(stdin) = self.child.stdin.take() {
            drop(stdin);
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.base);
    }
}

#[test]
fn unhosted_cli_refuses_import_and_writes_nothing() {
    let scratch = std::env::temp_dir().join(format!("kk-assetimp-bare-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).unwrap();
    let (_, caps) = call(&[], &request("capabilities", json!({})));
    assert_eq!(caps["ok"], true, "{caps}");
    assert_eq!(
        command_status(&caps["data"], "asset_sources.import"),
        "unsupported"
    );
    let (code, schema) = call(
        &[],
        &request("schema", json!({"command":"asset_sources.import"})),
    );
    assert_eq!(schema["ok"], true, "{schema}");
    assert_eq!(code, Some(0), "{schema}");
    assert_eq!(schema["data"]["status"], "unsupported");
    let required = schema["data"]["request"]["properties"]["payload"]["required"]
        .as_array()
        .unwrap();
    for field in [
        "archive_id",
        "manifest_sha256",
        "new_document_id",
        "acknowledge_unreviewed_masks",
    ] {
        assert!(
            required.iter().any(|value| value == field),
            "{field} in {required:?}"
        );
    }
    assert_eq!(required.len(), 4, "{required:?}");
    let (code, response) = call(
        &[],
        &request(
            "asset_sources.import",
            import_payload("originals", &"a".repeat(64), "asset-imported", true),
        ),
    );
    assert_eq!(response["error"]["code"], "unsupported", "{response}");
    assert_ne!(code, Some(0), "{response}");
    let entries: Vec<_> = std::fs::read_dir(&scratch)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert!(entries.is_empty(), "{entries:?}");
    let _ = std::fs::remove_dir_all(&scratch);
}

#[test]
fn hosted_cli_imports_a_new_draft_and_rejects_reuse() {
    let host = Host::start("import");
    let caps = host.ok(request("capabilities", json!({})));
    assert_eq!(
        command_status(&caps, "asset_sources.import"),
        "experimental"
    );
    assert_eq!(caps["asset_sources"]["quality_approved"], false);
    assert_eq!(caps["asset_sources"]["review_admitted"], false);
    let schema = host.ok(request("schema", json!({"command":"asset_sources.import"})));
    assert_eq!(schema["status"], "experimental");
    assert_eq!(schema["constraints"]["quality_approved"], false);
    assert_eq!(schema["constraints"]["review_admitted"], false);
    let created = host.ok(request(
        "asset_projects.create",
        json!({"document":document("asset-original")}),
    ));
    let exported = host.ok(request(
        "asset_sources.export",
        json!({
            "archive_id":"originals",
            "project":{
                "document_id":"asset-original",
                "publication_revision":created["publication"]["publication_revision"],
                "publication_sha256":created["publication_sha256"]
            },
            "missing_frames":"reject"
        }),
    ));
    let manifest = exported["manifest_sha256"].as_str().unwrap().to_owned();
    assert_eq!(
        host.error(request(
            "asset_sources.import",
            import_payload("originals", &manifest, "asset-imported", false)
        )),
        "needs_input"
    );
    assert_eq!(
        host.error(request(
            "asset_sources.import",
            import_payload("originals", &manifest, "asset-original", true)
        )),
        "invalid_argument"
    );
    let mut timed = request(
        "asset_sources.import",
        import_payload("originals", &manifest, "asset-imported", true),
    );
    timed["context"]["timeout_ms"] = json!(1_000);
    assert_eq!(host.error(timed), "invalid_argument");
    let imported = host.ok(request(
        "asset_sources.import",
        import_payload("originals", &manifest, "asset-imported", true),
    ));
    assert_eq!(
        imported["receipt"]["schema_version"],
        "asset-source-import/1"
    );
    assert_eq!(imported["receipt"]["quality_approved"], false);
    assert_eq!(imported["receipt"]["review_admitted"], false);
    assert_eq!(imported["receipt"]["source"]["archive_id"], "originals");
    assert_eq!(imported["receipt"]["source"]["manifest_sha256"], manifest);
    assert_eq!(
        imported["receipt"]["original_project"]["document_id"],
        "asset-original"
    );
    assert_eq!(
        imported["receipt"]["imported_project"]["document_id"],
        "asset-imported"
    );
    assert_eq!(
        imported["project"]["publication"]["document"]["document_id"],
        "asset-imported"
    );
    assert_eq!(
        imported["project"]["publication"]["document"]["revision"],
        "1"
    );
    assert_eq!(
        imported["project"]["publication"]["document"]["sources"],
        created["publication"]["document"]["sources"]
    );
    assert_eq!(
        imported["project"]["publication"]["document"]["layers"],
        created["publication"]["document"]["layers"]
    );
    assert!(
        imported["project"]["publication"]["parent_publication_sha256"].is_null(),
        "{imported}"
    );
    let inspected = host.ok(request(
        "asset_projects.inspect",
        json!({"document_id":"asset-imported"}),
    ));
    assert_eq!(
        inspected["publication"]["document"]["document_id"],
        "asset-imported"
    );
    assert_eq!(
        host.error(request(
            "asset_sources.import",
            import_payload("originals", &manifest, "asset-imported", true)
        )),
        "revision_conflict"
    );
    let original = host.ok(request(
        "asset_projects.inspect",
        json!({"document_id":"asset-original"}),
    ));
    assert_eq!(
        original["publication_sha256"],
        created["publication_sha256"]
    );
    assert!(host.projects.exists());
    assert!(host.artifacts.exists());
}

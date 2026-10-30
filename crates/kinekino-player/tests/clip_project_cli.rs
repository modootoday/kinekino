#![cfg(windows)]
use kinekino_player::application::api::ClipClient;
#[path = "support/owned_process_evidence.rs"]
pub mod owned_process_evidence;
use kinekino_player::{
    application::{
        api::{PlayerSdk, Request, RequestContext},
        clip_projects::InspectClipProject,
    },
    domain::{
        api_values::DecimalU64,
        clip_document::{ClipDocument, ClipEdit, ClipEditor, CutMode},
    },
    infrastructure::clip_projects::ClipProjectFiles,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{ffi::OsString, fs, path::PathBuf, time::Duration};
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn document() -> ClipDocument {
    serde_json::from_value(json!({"schema_version":"1","document_id":"short-1","revision":"1","sources":[{"media_id":"source-1","sha256":"a".repeat(64),"duration_us":"60000000","width":1920,"height":1080,"clock":{"origin_pts":"-9000","time_base_numerator":1,"time_base_denominator":90000,"observation_sha256":"b".repeat(64)},"tracks":[{"stream_index":0,"kind":"video","codec_id":"h264"}]}],"segments":[{"segment_id":"segment-1","media_id":"source-1","range":{"start_us":"0","end_us":"30000000"},"video_stream_index":0,"crop_keys":[{"source_us":"0","crop":{"left":0,"top":0,"width":1000000,"height":1000000},"fit":"fit"}],"audio":{"kind":"disabled"},"subtitles":null,"subject_ref":null}],"preset":{"catalog":{"artifact_id":"manual-preset","revision":"1","sha256":"c".repeat(64)},"width":1080,"height":1920,"frame_rate":{"numerator":30,"denominator":1},"container":"mp4","video_codec":"h264","audio_codec":null,"color":"sdr_bt709"},"cut_mode":"exact_reencode"})).unwrap()
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
            std::env::var_os("CLIP_PROJECT_CLI_EVIDENCE_DIR")
                .expect("Explicit fresh evidence directory"),
        );
        assert!(root.is_absolute());
        fs::create_dir(&root).unwrap();
        let store = root.join("store");
        fs::create_dir(&store).unwrap();
        let executable = PathBuf::from(std::env::var_os("CLIP_PROJECT_CLI").expect("Pinned CLI"));
        assert!(executable.is_absolute());
        let digest = hash(&fs::read(&executable).unwrap());
        assert_eq!(
            std::env::var("CLIP_PROJECT_CLI_SHA256")
                .unwrap()
                .to_lowercase(),
            digest
        );
        fs::write(root.join("producer.json"), serde_json::to_vec_pretty(&json!({"path":executable,"sha256":digest,"scope":"synthetic_clip_project_cli","media_provenance_verified":false,"quality_approved":false})).unwrap()).unwrap();
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
                OsString::from("--clip-project-root"),
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
#[ignore = "Explicit fresh evidence and pinned normal CLI; synthetic project, no media render"]
fn normal_clip_project_cli_matches_typed_cas_history_and_preserves_originals() {
    kinekino_platform::install();
    let mut h = Harness::new();
    let caps = h.call(&Request::new("capabilities"), false, &[], 0);
    assert_eq!(caps["data"]["commands"].as_array().unwrap().len(), 119);
    assert_eq!(caps["data"]["clip_projects"]["backend_available"], false);
    let inspect = request("clip_projects.inspect", json!({"document_id":"short-1"}));
    assert_eq!(
        h.call(&inspect, false, &[], 3)["error"]["code"],
        "unsupported"
    );
    for command in [
        "clip_projects.create",
        "clip_projects.inspect",
        "clip_projects.save_revision",
    ] {
        let schema = request("schema", json!({"command":command}));
        assert_eq!(
            h.call(&schema, false, &[], 0)["data"]["status"],
            "unsupported"
        );
        let configured = h.call(&schema, true, &[], 0);
        assert_eq!(configured["data"]["status"], "supported");
        assert_eq!(
            configured["data"]["constraints"]["render_short_supported"],
            false
        );
    }
    let caps = h.call(&Request::new("capabilities"), true, &[], 0);
    assert_eq!(caps["data"]["clip_projects"]["backend_available"], true);
    let initial = document();
    initial.validate().unwrap();
    let create = request("clip_projects.create", json!({"document":initial}));
    let first = h.call(&create, true, &[], 0);
    let project = h.store.join("clip-short-1");
    let first_file = fs::read_dir(&project)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| path.extension().is_some_and(|ext| ext == "json"))
        .unwrap();
    let original = fs::read(&first_file).unwrap();
    let sdk_root = h.root.join("typed-store");
    fs::create_dir(&sdk_root).unwrap();
    let sdk =
        PlayerSdk::default().with_clip_project_store(ClipProjectFiles::open(&sdk_root).unwrap());
    assert_eq!(
        serde_json::to_value(
            sdk.create_clip_project(RequestContext::default(), initial.clone())
                .unwrap()
        )
        .unwrap(),
        first["data"]
    );
    assert_eq!(
        h.call(&create, true, &[], 5)["error"]["code"],
        "revision_conflict"
    );
    let mut editor = ClipEditor::open(initial.clone()).unwrap();
    editor
        .apply(
            DecimalU64(1),
            ClipEdit::SetCutMode {
                cut_mode: CutMode::FastKeyframe,
            },
        )
        .unwrap();
    editor
        .apply(
            DecimalU64(2),
            ClipEdit::SetCutMode {
                cut_mode: CutMode::ExactReencode,
            },
        )
        .unwrap();
    let mut save = request(
        "clip_projects.save_revision",
        json!({"document":editor.document()}),
    );
    assert_eq!(
        h.call(&save, true, &[], 2)["error"]["code"],
        "invalid_argument"
    );
    save.context.expected_revision = Some("1".into());
    let second = h.call(&save, true, &[], 0);
    assert_eq!(second["data"]["publication"]["publication_revision"], "2");
    assert_eq!(second["data"]["publication"]["document"]["revision"], "3");
    assert_eq!(
        serde_json::to_value(
            sdk.save_clip_project(save.context.clone(), editor.document().clone())
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
        "clip_projects.inspect",
        json!({"document_id":"short-1","publication_revision":"1"}),
    );
    let prior = h.call(&historical, true, &[], 0);
    assert_eq!(prior["data"]["publication"], first["data"]["publication"]);
    assert_eq!(prior["data"]["latest_publication_revision"], "2");
    assert_eq!(
        serde_json::to_value(
            sdk.inspect_clip_project(
                RequestContext::default(),
                InspectClipProject {
                    document_id: "short-1".into(),
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
    let mut foreign = editor.document().clone();
    foreign.revision = DecimalU64(4);
    foreign.sources[0].sha256 = "d".repeat(64);
    let mut save_foreign = request("clip_projects.save_revision", json!({"document":foreign}));
    save_foreign.context.expected_revision = Some("2".into());
    assert_eq!(
        h.call(&save_foreign, true, &[], 5)["error"]["code"],
        "revision_conflict"
    );
    assert_eq!(
        h.call(&inspect, true, &["--endpoint", "unused"], 2)["error"]["code"],
        "invalid_argument"
    );
    assert_eq!(
        h.call(&inspect, true, &["--serve-jobs"], 2)["error"]["code"],
        "invalid_argument"
    );
    assert_eq!(fs::read(&first_file).unwrap(), original);
    assert_eq!(h.call(&inspect, true, &[], 0)["data"], second["data"]);
    fs::write(h.root.join("verification.json"),serde_json::to_vec_pretty(&json!({"cli_calls":h.next,"original_publication_sha256":hash(&original),"original_preserved":true,"typed_equivalent":true,"reopened_by_each_cli_process":true,"media_provenance_verified":false,"rendered":false,"quality_approved":false})).unwrap()).unwrap();
}

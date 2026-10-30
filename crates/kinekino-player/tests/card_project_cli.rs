#![cfg(windows)]
use kinekino_player::application::api::CardClient;
#[path = "support/cli_exe.rs"]
mod cli_exe;
#[path = "support/owned_process_evidence.rs"]
pub mod owned_process_evidence;
use kinekino_player::{
    application::{
        api::{ApplicationClient, PlayerSdk, Request, RequestContext},
        card_projects::InspectCardProject,
    },
    domain::{
        api_values::DecimalU64,
        card_document::{CardDocument, CardEdit, CardEditor},
    },
    infrastructure::card_projects::CardProjectFiles,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{ffi::OsString, fs, path::PathBuf, time::Duration};

pub fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn document() -> CardDocument {
    serde_json::from_value(json!({"schema_version":"1","document_id":"cards-1","revision":"1",
        "source":{"source_id":"source-1","source_revision":"2","source_sha256":"a".repeat(64),"cues":[{"cue_id":"cue-1","text":"원문 대사","start_us":null,"end_us":null}]},
        "layout":{"canvas":{"width":1080,"height":1080,"template_id":"card.frame-two-slot","template_version":"0.1.0-proposal.1","template_sha256":"b".repeat(64)},"cards":[{"card_id":"first","role":"body","text_kind":"summary","content":{"front":"제목","back":"요약","source_refs":["cue-1"]},"frame":null,"alt_text_draft":"설명"}],"caption_draft":"게시문"}})).unwrap()
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
        let root = std::env::var_os("CARD_PROJECT_CLI_EVIDENCE_DIR")
            .map(PathBuf::from)
            .expect("Explicit fresh evidence directory");
        assert!(root.is_absolute());
        fs::create_dir(&root).unwrap();
        let store = root.join("store");
        fs::create_dir(&store).unwrap();
        let executable = std::env::var_os("CARD_PROJECT_CLI")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(crate::cli_exe::kinekino_cli_exe()));
        assert!(executable.is_absolute());
        let digest = format!("{:x}", Sha256::digest(fs::read(&executable).unwrap()));
        if std::env::var_os("CARD_PROJECT_CLI").is_some() {
            assert_eq!(
                std::env::var("CARD_PROJECT_CLI_SHA256")
                    .unwrap()
                    .to_lowercase(),
                digest
            );
        }
        fs::write(root.join("producer.json"), serde_json::to_vec_pretty(&json!({"path":executable,"sha256":digest,"scope":"synthetic_card_project_sdk_cli","quality_approved":false})).unwrap()).unwrap();
        Self {
            root,
            store,
            executable,
            digest,
            next: 0,
        }
    }
    fn call(&mut self, request: &Request, rooted: bool, expected_code: i32) -> Value {
        let call = self.root.join(format!("call-{:03}", self.next));
        self.next += 1;
        fs::create_dir(&call).unwrap();
        let bytes = serde_json::to_vec(request).unwrap();
        fs::write(call.join("request.json"), &bytes).unwrap();
        let mut arguments: Vec<OsString> = ["--input", "-", "--json"]
            .into_iter()
            .map(Into::into)
            .collect();
        if rooted {
            arguments.extend([
                OsString::from("--card-project-root"),
                self.store.clone().into_os_string(),
            ]);
        }
        let output = match owned_process_evidence::run_process_with_arguments(
            self.executable.clone(),
            &call,
            &bytes,
            arguments,
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
                panic!("CLI family failed: {failure:?}");
            }
        };
        assert_eq!(
            output.receipt.child_exit.map(i64::from),
            Some(i64::from(expected_code))
        );
        assert!(!output.receipt.forced);
        output.check_deadline(&call).unwrap();
        let response: Value = serde_json::from_slice(&output.stdout).unwrap();
        fs::write(
            call.join("response.json"),
            serde_json::to_vec_pretty(&response).unwrap(),
        )
        .unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(fs::read(&self.executable).unwrap())),
            self.digest
        );
        response
    }
}
fn request(command: &str, payload: Value) -> Request {
    let mut r = Request::new(command);
    r.payload = payload;
    r
}
fn rejected(h: &mut Harness, r: &Request, code: &str) {
    let exit = match code {
        "invalid_argument" => 2,
        "unsupported" => 3,
        "not_found" => 4,
        "revision_conflict" => 5,
        _ => panic!("No pinned exit contract for {code}"),
    };
    assert_eq!(h.call(r, true, exit)["error"]["code"], code);
}

#[test]
#[ignore = "Explicit fresh synthetic store/evidence and pinned normal CLI"]
fn normal_cli_card_project_create_save_inspect_matches_typed_sdk_and_preserves_boundaries() {
    kinekino_platform::install();
    let mut h = Harness::new();
    let unavailable = h.call(&Request::new("capabilities"), false, 0);
    assert_eq!(
        unavailable["data"]["card_projects"]["backend_available"],
        false
    );
    assert_eq!(
        unavailable["data"]["commands"].as_array().unwrap().len(),
        119
    );
    let missing = request("card_projects.inspect", json!({"document_id":"cards-1"}));
    assert_eq!(h.call(&missing, false, 3)["error"]["code"], "unsupported");
    for command in [
        "card_projects.create",
        "card_projects.inspect",
        "card_projects.save_revision",
    ] {
        let query = request("schema", json!({"command":command}));
        assert_eq!(h.call(&query, false, 0)["data"]["status"], "unsupported");
        let schema = h.call(&query, true, 0);
        assert_eq!(schema["data"]["status"], "supported");
        assert_eq!(
            schema["data"]["request"]["properties"]["payload"]["additionalProperties"],
            false
        );
    }
    let caps = h.call(&Request::new("capabilities"), true, 0);
    assert_eq!(caps["data"]["card_projects"]["backend_available"], true);
    assert_eq!(
        caps["data"]["card_projects"]["export_cards_supported"],
        false
    );
    let initial = document();
    let create = request("card_projects.create", json!({"document":initial}));
    let first = h.call(&create, true, 0);
    let sdk_root = h.root.join("sdk-store");
    fs::create_dir(&sdk_root).unwrap();
    let sdk: std::sync::Arc<dyn ApplicationClient> = std::sync::Arc::new(
        PlayerSdk::default().with_card_project_store(CardProjectFiles::open(&sdk_root).unwrap()),
    );
    let typed = sdk
        .create_card_project(RequestContext::default(), initial.clone())
        .unwrap();
    assert_eq!(serde_json::to_value(typed).unwrap(), first["data"]);
    rejected(&mut h, &create, "revision_conflict");
    let mut editor = CardEditor::open(initial).unwrap();
    editor
        .apply(
            DecimalU64(1),
            CardEdit::SetCaption {
                caption_draft: "수정 게시문".into(),
            },
        )
        .unwrap();
    editor
        .apply(
            DecimalU64(2),
            CardEdit::Duplicate {
                card_id: "first".into(),
                new_card_id: "second".into(),
                position: 1,
            },
        )
        .unwrap();
    let mut save = request(
        "card_projects.save_revision",
        json!({"document":editor.document()}),
    );
    rejected(&mut h, &save, "invalid_argument");
    save.context.expected_revision = Some("1".into());
    let second = h.call(&save, true, 0);
    assert_eq!(second["data"]["publication"]["publication_revision"], "2");
    assert_eq!(second["data"]["publication"]["document"]["revision"], "3");
    let typed = sdk
        .save_card_project(save.context.clone(), editor.document().clone())
        .unwrap();
    assert_eq!(serde_json::to_value(typed).unwrap(), second["data"]);
    rejected(&mut h, &save, "revision_conflict");
    let historical = request(
        "card_projects.inspect",
        json!({"document_id":"cards-1","publication_revision":"1"}),
    );
    let old = h.call(&historical, true, 0);
    assert_eq!(old["data"]["publication"], first["data"]["publication"]);
    assert_eq!(old["data"]["latest_publication_revision"], "2");
    let head = h.call(&missing, true, 0);
    let typed = sdk
        .inspect_card_project(
            RequestContext::default(),
            InspectCardProject {
                document_id: "cards-1".into(),
                publication_revision: None,
            },
        )
        .unwrap();
    assert_eq!(serde_json::to_value(typed).unwrap(), head["data"]);
    for context in [
        json!({"timeout_ms":1000}),
        json!({"idempotency_key":"no-auto-retry"}),
        json!({"expected_revision":"2"}),
    ] {
        let mut denied = missing.clone();
        if let Some(v) = context["timeout_ms"].as_u64() {
            denied.context.timeout_ms = Some(v);
        }
        if let Some(v) = context["idempotency_key"].as_str() {
            denied.context.idempotency_key = Some(v.into());
        }
        if let Some(v) = context["expected_revision"].as_str() {
            denied.context.expected_revision = Some(v.into());
        }
        rejected(&mut h, &denied, "invalid_argument");
    }
    let mut foreign = save.clone();
    foreign.context.expected_revision = Some("2".into());
    foreign.payload["document"]["revision"] = json!("4");
    foreign.payload["document"]["source"]["cues"][0]["text"] = json!("외부 원문");
    rejected(&mut h, &foreign, "invalid_argument");
    rejected(
        &mut h,
        &request("card_projects.inspect", json!({"document_id":"../foreign"})),
        "invalid_argument",
    );
    rejected(
        &mut h,
        &request(
            "card_projects.inspect",
            json!({"document_id":"cards-1","extra":true}),
        ),
        "invalid_argument",
    );
    rejected(
        &mut h,
        &request(
            "card_projects.inspect",
            json!({"document_id":"cards-1","publication_revision":"999"}),
        ),
        "not_found",
    );
    assert_eq!(h.call(&missing, true, 0)["data"], second["data"]);
    fs::write(h.root.join("summary.json"), serde_json::to_vec_pretty(&json!({"calls":h.next,"normal_cli_sha256":h.digest,"typed_sdk_cli_equal":true,"quality_approved":false,"export_cards_supported":false})).unwrap()).unwrap();
}

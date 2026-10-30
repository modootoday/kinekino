#![cfg(windows)]
use kinekino_player::application::api::ProposalClient;
#[path = "support/cli_exe.rs"]
mod cli_exe;
use kinekino_player::{
    application::{
        api::{PlayerSdk, Request, RequestContext},
        proposals::{ProposalStore, cue_edit::EditCues},
    },
    domain::proposals::Correction,
    infrastructure::{content_digest, proposals::ProposalFiles, subtitle_files::SubtitleFiles},
};
use serde_json::{Value, json};
use std::{
    fs::{self, File},
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

struct Owned(Child);
impl Drop for Owned {
    fn drop(&mut self) {
        if !matches!(self.0.try_wait(), Ok(Some(_))) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}
struct Fixture {
    root: PathBuf,
    store: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "cue-edit-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let store = root.join("store");
        fs::create_dir(&store).unwrap();
        println!("cue edit evidence {}", root.display());
        Self { root, store }
    }
    fn call(&self, label: &str, request: &Request, error: Option<&str>) -> Value {
        let dir = self.root.join(label);
        fs::create_dir(&dir).unwrap();
        let input = dir.join("REQUEST.json");
        fs::write(&input, serde_json::to_vec_pretty(request).unwrap()).unwrap();
        let binary = crate::cli_exe::kinekino_cli_exe();
        let hash = || {
            content_digest::sha256(&mut File::open(binary).unwrap(), 512 * 1024 * 1024)
                .unwrap()
                .1
        };
        let before = hash();
        let mut child = Owned(
            Command::new(binary)
                .arg("--input")
                .arg(&input)
                .arg("--json")
                .arg("--proposal-root")
                .arg(&self.store)
                .stdin(Stdio::null())
                .stdout(File::create(dir.join("stdout.json")).unwrap())
                .stderr(File::create(dir.join("stderr.raw")).unwrap())
                .spawn()
                .unwrap(),
        );
        let deadline = Instant::now() + Duration::from_secs(30);
        let status = loop {
            if let Some(status) = child.0.try_wait().unwrap() {
                break status;
            }
            assert!(
                Instant::now() < deadline,
                "owned CLI deadline: {}",
                dir.display()
            );
            std::thread::sleep(Duration::from_millis(20));
        };
        assert_eq!(hash(), before, "CLI producer changed");
        fs::write(dir.join("PROCESS.json"), serde_json::to_vec_pretty(&json!({"pid":child.0.id(),"exit_code":status.code(),"cli_sha256":before,"original_child_waited":true})).unwrap()).unwrap();
        let bytes = fs::read(dir.join("stdout.json")).unwrap();
        assert!(bytes.len() <= 1024 * 1024);
        let response: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(status.success(), error.is_none(), "{response}");
        assert_eq!(response["ok"], error.is_none(), "{response}");
        if let Some(code) = error {
            assert_eq!(response["error"]["code"], code, "{response}");
        }
        response
    }
    fn seed(&self) -> Value {
        self.call("create", &request("proposals.create", json!({
            "proposal_id":"cue-draft", "source":{"source_id":"source","source_revision":"1","source_sha256":"a".repeat(64),"cues":[
                {"cue_id":"one","text":"Original one","start_us":"1000000","end_us":"2000000"},
                {"cue_id":"two","text":"Original two","start_us":"2500000","end_us":"3500000"}]},
            "generated":{"task":"correction","raw_text":"not valid generated JSON"}
        }),None),None)["data"].clone()
    }
    fn inspect(&self, label: &str) -> Value {
        self.call(
            label,
            &request(
                "proposals.inspect",
                json!({"proposal_id":"cue-draft"}),
                None,
            ),
            None,
        )["data"]
            .clone()
    }
}
fn request(command: &str, payload: Value, expected: Option<&str>) -> Request {
    let mut request = Request::new(command);
    request.payload = payload;
    request.context.expected_revision = expected.map(str::to_owned);
    request
}
fn edit(text: &str, expected: Option<&str>) -> Request {
    request(
        "subtitles.edit_cues",
        json!({"proposal_id":"cue-draft","edits":[{"cue_id":"one","text":text}]}),
        expected,
    )
}

#[test]
fn cli_and_typed_client_share_correction_revision_source_and_acceptance() {
    kinekino_platform::install();
    let f = Fixture::new();
    let original = f.seed();
    assert_eq!(original["state"], "needs_repair");
    let caps = f.call(
        "capabilities",
        &request("capabilities", json!({}), None),
        None,
    );
    assert_eq!(caps["data"]["cue_edit"]["available"], true);
    assert_eq!(caps["data"]["cue_edit"]["timing_edit"], false);
    let schema = f.call(
        "schema",
        &request("schema", json!({"command":"subtitles.edit_cues"}), None),
        None,
    );
    assert_eq!(schema["data"]["status"], "supported");
    let edited = f.call("edit", &edit("Edited one", Some("1")), None);
    assert_eq!(edited["data"]["draft_revision"], "2");
    let client =
        PlayerSdk::new(SubtitleFiles).with_proposal_store(ProposalFiles::open(&f.store).unwrap());
    let document = client
        .edit_subtitle_cues(
            RequestContext {
                expected_revision: Some("2".into()),
                ..Default::default()
            },
            EditCues {
                proposal_id: "cue-draft".into(),
                edits: vec![Correction {
                    cue_id: "two".into(),
                    text: "Edited two".into(),
                }],
            },
        )
        .unwrap();
    assert_eq!(document.draft_revision.0, 3);
    let reloaded = f.inspect("reopen");
    assert_eq!(reloaded["document"]["source"], original["source"]);
    assert_eq!(reloaded["document"]["generated"], original["generated"]);
    assert_eq!(
        reloaded["document"]["draft"]["edits"],
        json!([{"cue_id":"two","text":"Edited two"}])
    );
    let accepted = f.call(
        "accept",
        &request(
            "proposals.accept",
            json!({"proposal_id":"cue-draft","target_source_revision":"1"}),
            Some("3"),
        ),
        None,
    );
    assert_eq!(accepted["data"]["automatic_application_enabled"], false);
    let store = ProposalFiles::open(&f.store).unwrap();
    let state = store.load("cue-draft").unwrap();
    let receipt = serde_json::from_value(accepted["data"].clone()).unwrap();
    let projection =
        kinekino_player::domain::proposals::accepted_subtitle_projection::project(&state, &receipt)
            .unwrap();
    assert_eq!(projection.cues().len(), 2);
    assert_eq!(projection.cues()[0].text(), "Original one");
    assert_eq!(projection.cues()[1].text(), "Edited two");
    assert_eq!(projection.cues()[0].in_us().0, 1_000_000);
    assert_eq!(projection.cues()[1].out_us().0, 3_500_000);
    let revision = accepted["data"]["output_revision"].as_str().unwrap();
    f.call(
        "edit-after-accept",
        &edit("New explicit draft", Some(revision)),
        None,
    );
    let final_state = f.inspect("final-reopen");
    assert!(final_state["acceptance"].is_null());
    assert_eq!(final_state["document"]["source"], original["source"]);
    assert_eq!(final_state["document"]["generated"], original["generated"]);
    let unsupported = PlayerSdk::default().execute(edit("No store", Some("1")));
    assert!(!unsupported.ok);
    assert_eq!(
        unsupported.error.unwrap().code,
        kinekino_player::application::api::ErrorCode::Unsupported
    );
}

#[test]
fn invalid_cues_context_and_stale_revision_do_not_publish() {
    kinekino_platform::install();
    let f = Fixture::new();
    f.seed();
    let before = f.inspect("before");
    f.call("create-cards", &request("proposals.create", json!({"proposal_id":"cards","source":before["document"]["source"],"generated":{"task":"cards","raw_text":"invalid"}}),None), None);
    f.call(
        "wrong-task",
        &request(
            "subtitles.edit_cues",
            json!({"proposal_id":"cards","edits":[{"cue_id":"one","text":"No"}]}),
            Some("1"),
        ),
        Some("invalid_argument"),
    );
    for (i,payload) in [
        json!({"proposal_id":"cue-draft","edits":[]}),
        json!({"proposal_id":"cue-draft","edits":[{"cue_id":"missing","text":"No"}]}),
        json!({"proposal_id":"cue-draft","edits":[{"cue_id":"one","text":"A"},{"cue_id":"one","text":"B"}]}),
        json!({"proposal_id":"cue-draft","edits":[{"cue_id":"one","text":" "}]}),
        json!({"proposal_id":"cue-draft","edits":[{"cue_id":"one","text":"No","start_us":"0"}]}),
    ].into_iter().enumerate() {
        f.call(&format!("invalid-{i}"),&request("subtitles.edit_cues",payload,Some("1")),Some("invalid_argument"));
    }
    f.call(
        "missing-revision",
        &edit("No", None),
        Some("invalid_argument"),
    );
    f.call("stale", &edit("No", Some("9")), Some("revision_conflict"));
    let mut timeout = edit("No", Some("1"));
    timeout.context.timeout_ms = Some(1);
    f.call("timeout-unsupported", &timeout, Some("invalid_argument"));
    let mut idempotency = edit("No", Some("1"));
    idempotency.context.idempotency_key = Some("not-supported".into());
    f.call(
        "idempotency-unsupported",
        &idempotency,
        Some("invalid_argument"),
    );
    assert_eq!(f.inspect("after"), before);
}

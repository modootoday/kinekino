#![cfg(windows)]
use kinekino_player::application::api::ProposalClient;
#[path = "support/cli_exe.rs"]
mod cli_exe;
use kinekino_player::{
    application::{
        api::{PlayerSdk, Request, RequestContext},
        proposals::{CreateProposal, InspectProposal},
    },
    domain::proposals::ProposalDocument,
    infrastructure::{content_digest, proposals::ProposalFiles, subtitle_files::SubtitleFiles},
};
use serde_json::{Value, json};
use std::{
    fs,
    io::{Cursor, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};
struct Fixture {
    root: PathBuf,
    evidence: Option<PathBuf>,
    executable: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let evidence = std::env::var_os("PROPOSAL_EVIDENCE_DIR").map(PathBuf::from);
        let root = if let Some(e) = &evidence {
            assert!(e.is_absolute());
            fs::create_dir(e).unwrap();
            e.join("store")
        } else {
            std::env::temp_dir().join(format!(
                "player-proposal-cli-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ))
        };
        fs::create_dir(&root).unwrap();
        let executable = if let Some(e) = &evidence {
            let p = e.join(kinekino_player::domain::product::PRODUCT.cli_executable);
            fs::copy(crate::cli_exe::kinekino_cli_exe(), &p).unwrap();
            p
        } else {
            PathBuf::from(crate::cli_exe::kinekino_cli_exe())
        };
        Self {
            root,
            evidence,
            executable,
        }
    }
    fn record(&self, name: &str, v: &Value) {
        if let Some(e) = &self.evidence {
            let mut f = fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(e.join(name))
                .unwrap();
            f.write_all(&serde_json::to_vec_pretty(v).unwrap()).unwrap();
        }
    }
    fn invoke(&self, name: &str, request: &Request) -> (i32, Value) {
        let mut child = Command::new(&self.executable)
            .args(["--input", "-", "--proposal-root"])
            .arg(&self.root)
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
        let o = child.wait_with_output().unwrap();
        assert!(o.stderr.is_empty());
        let v = serde_json::from_slice(&o.stdout).unwrap();
        let code = o.status.code().unwrap();
        self.record(
            name,
            &json!({"request":request,"response":v,"exit_code":code}),
        );
        (code, v)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if self.evidence.is_none() {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}
fn request(command: &str, payload: Value, expected: Option<&str>) -> Request {
    let mut r = Request::new(command);
    r.payload = payload;
    r.context.expected_revision = expected.map(str::to_owned);
    r
}
fn hash(path: &Path) -> String {
    content_digest::sha256(&mut Cursor::new(fs::read(path).unwrap()), 536_870_912)
        .unwrap()
        .1
}
#[test]
fn actual_cli_and_typed_ui_port_share_import_edit_accept_revision_and_support_contract() {
    kinekino_platform::install();
    let f = Fixture::new();
    let client =
        PlayerSdk::new(SubtitleFiles).with_proposal_store(ProposalFiles::open(&f.root).unwrap());
    let caps = request("capabilities", json!({}), None);
    let (code, cli) = f.invoke("capabilities.json", &caps);
    assert_eq!(code, 0);
    let sdk = serde_json::to_value(client.execute(caps)).unwrap();
    // Frame adapter availability differs by assembly; proposal support is identical.
    assert_eq!(cli["data"]["proposals"], sdk["data"]["proposals"]);
    assert_eq!(cli["data"]["proposals"]["backend_available"], true);
    for (index, command) in [
        "proposals.create",
        "proposals.inspect",
        "proposals.edit",
        "proposals.accept",
    ]
    .iter()
    .enumerate()
    {
        let r = request("schema", json!({"command":command}), None);
        let (code, c) = f.invoke(&format!("schema-{index}.json"), &r);
        assert_eq!(code, 0);
        assert_eq!(c, serde_json::to_value(client.execute(r)).unwrap());
        assert_eq!(c["data"]["status"], "supported");
    }
    let create = request(
        "proposals.create",
        json!({"proposal_id":"sample","source":{"source_id":"source","source_revision":"3","source_sha256":"b".repeat(64),"cues":[{"cue_id":"c1","text":"immutable source","start_us":null,"end_us":null}]},"generated":{"task":"correction","raw_text":"{\"error\":\"bad model JSON\"}"}}),
        None,
    );
    let (code, c) = f.invoke("create-needs-repair.json", &create);
    assert_eq!(code, 0);
    let before: ProposalDocument = serde_json::from_value(c["data"].clone()).unwrap();
    assert_eq!(c["data"]["state"], "needs_repair");
    let input: CreateProposal = serde_json::from_value(create.payload.clone()).unwrap();
    assert_eq!(
        client
            .create_proposal(RequestContext::default(), input)
            .unwrap_err()
            .code,
        kinekino_player::application::api::ErrorCode::RevisionConflict
    );
    let accept = request(
        "proposals.accept",
        json!({"proposal_id":"sample","target_source_revision":"3"}),
        Some("1"),
    );
    assert_eq!(f.invoke("accept-needs-repair.json", &accept).0, 3);
    let edit = request(
        "proposals.edit",
        json!({"proposal_id":"sample","draft":{"task":"correction","edits":[{"cue_id":"c1","text":"user corrected"}]}}),
        Some("1"),
    );
    let (code, edited) = f.invoke("edit.json", &edit);
    assert_eq!(code, 0);
    assert_eq!(edited["data"]["draft_revision"], "2");
    assert_eq!(f.invoke("stale-edit.json", &edit).0, 5);
    let mut bad = edit.clone();
    bad.context.expected_revision = Some("2".into());
    bad.payload["draft"]["edits"][0]["cue_id"] = json!("not-in-source");
    assert_eq!(f.invoke("unknown-source-ref.json", &bad).0, 2);
    let accept = request(
        "proposals.accept",
        json!({"proposal_id":"sample","target_source_revision":"3"}),
        Some("2"),
    );
    let (code, receipt) = f.invoke("accept.json", &accept);
    assert_eq!(code, 0);
    assert_eq!(receipt["data"]["output_revision"], "3");
    assert_eq!(receipt["data"]["automatic_application_enabled"], false);
    assert_eq!(f.invoke("accept-retry.json", &accept).1, receipt);
    let restored = client
        .inspect_proposal(
            RequestContext::default(),
            InspectProposal {
                proposal_id: "sample".into(),
            },
        )
        .unwrap();
    assert_eq!(
        serde_json::to_value(&restored.acceptance).unwrap(),
        receipt["data"]
    );
    let read = request("proposals.inspect", json!({"proposal_id":"sample"}), None);
    let (code, restarted_cli) = f.invoke("inspect-after-restart.json", &read);
    assert_eq!(code, 0);
    assert_eq!(
        restarted_cli["data"],
        serde_json::to_value(&restored).unwrap()
    );
    let reopened =
        PlayerSdk::new(SubtitleFiles).with_proposal_store(ProposalFiles::open(&f.root).unwrap());
    assert_eq!(
        serde_json::to_value(reopened.execute(read)).unwrap(),
        restarted_cli
    );
    let after = restored.document;
    assert_eq!(
        serde_json::to_value(&before.source).unwrap(),
        serde_json::to_value(&after.source).unwrap()
    );
    assert_eq!(before.generated.raw_text, after.generated.raw_text);
    let source_hash = |d: &ProposalDocument| {
        content_digest::sha256(
            &mut Cursor::new(serde_json::to_vec(&d.source).unwrap()),
            4_194_304,
        )
        .unwrap()
        .1
    };
    let generated_hash = |d: &ProposalDocument| {
        content_digest::sha256(&mut Cursor::new(d.generated.raw_text.as_bytes()), 262_144)
            .unwrap()
            .1
    };
    f.record("summary.json",&json!({"binary_sha256":hash(&f.executable),"source_snapshot_sha256_before":source_hash(&before),"source_snapshot_sha256_after":source_hash(&after),"generated_raw_sha256_before":generated_hash(&before),"generated_raw_sha256_after":generated_hash(&after),"revisions":["1","2","3"],"receipt":receipt["data"],"scope":"actual Rust CLI subprocess and identical typed ApplicationClient port; synthetic input; no GUI visual validation, model inference, source-file rewrite or rendered output"}));
    for (task, payload) in [
        (
            "cards",
            json!({"task":"cards","cards":[{"front":"Synthetic question","back":"Synthetic answer","source_refs":["c1"]}]}),
        ),
        (
            "highlights",
            json!({"task":"highlights","highlights":[{"title":"Synthetic cue highlight","source_refs":["c1"]}]}),
        ),
    ] {
        let r = request(
            "proposals.create",
            json!({"proposal_id":format!("ui-{task}"),"source":create.payload["source"],"generated":{"task":task,"raw_text":serde_json::to_string(&payload).unwrap()}}),
            None,
        );
        let (code, v) = f.invoke(&format!("create-{task}.json"), &r);
        assert_eq!(code, 0);
        assert_eq!(v["data"]["state"], "editable");
    }
}

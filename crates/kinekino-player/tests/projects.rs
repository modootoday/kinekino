#![cfg(windows)]
use kinekino_player::application::api::ProjectClient;
#[path = "support/cli_exe.rs"]
mod cli_exe;
use kinekino_player::{
    application::{
        api::{ApplicationClient, PlayerSdk, Request, RequestContext},
        projects::InspectProject,
        proposals::{CreateProposal, ProposalStore},
    },
    domain::{
        api_values::DecimalU64,
        projects::{StoredProjectRevision, canonical_sha256},
        proposals::*,
    },
    infrastructure::{projects::ProjectFiles, proposals::ProposalFiles},
};
use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

struct Fixture {
    root: PathBuf,
    cli: PathBuf,
    persistent: bool,
}
impl Fixture {
    fn new() -> Self {
        let evidence = std::env::var_os("PROJECT_EVIDENCE_DIR").map(PathBuf::from);
        let persistent = evidence.is_some();
        let root = evidence.unwrap_or_else(|| {
            std::env::temp_dir().join(format!(
                "player-project-cli-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ))
        });
        assert!(root.is_absolute());
        fs::create_dir(&root).unwrap();
        for dir in ["projects", "proposals", "logs"] {
            fs::create_dir(root.join(dir)).unwrap();
        }
        let cli = root.join(kinekino_player::domain::product::PRODUCT.cli_executable);
        let source = std::env::var_os("PROJECT_CLI_EXE")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(crate::cli_exe::kinekino_cli_exe()));
        let digest = |path: &Path| {
            kinekino_player::infrastructure::content_digest::sha256(
                &mut fs::File::open(path).unwrap(),
                1024 * 1024 * 1024,
            )
            .unwrap()
            .1
        };
        let before = digest(&source);
        if let Ok(expected) = std::env::var("PROJECT_CLI_SHA256") {
            assert_eq!(before, expected);
        }
        fs::copy(&source, &cli).unwrap();
        assert_eq!(before, digest(&source));
        assert_eq!(before, digest(&cli));
        fs::write(root.join("cli-sha256.txt"), &before).unwrap();
        Self {
            root,
            cli,
            persistent,
        }
    }
    fn call(
        &self,
        name: &str,
        request: &Request,
        proposal_root: Option<&Path>,
        expected: i32,
    ) -> Value {
        let bytes = serde_json::to_vec(request).unwrap();
        fs::write(self.root.join(format!("logs/{name}.request.json")), &bytes).unwrap();
        let mut command = Command::new(&self.cli);
        command
            .args(["--input", "-", "--project-root"])
            .arg(self.root.join("projects"));
        if let Some(root) = proposal_root {
            command.arg("--proposal-root").arg(root);
        }
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(&bytes).unwrap();
        let output = child.wait_with_output().unwrap();
        fs::write(
            self.root.join(format!("logs/{name}.stdout")),
            &output.stdout,
        )
        .unwrap();
        fs::write(
            self.root.join(format!("logs/{name}.stderr")),
            &output.stderr,
        )
        .unwrap();
        fs::write(
            self.root.join(format!("logs/{name}.exit")),
            format!("{:?}", output.status.code()),
        )
        .unwrap();
        assert_eq!(
            output.status.code(),
            Some(expected),
            "{name}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        serde_json::from_slice(&output.stdout).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if !self.persistent {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}
fn request(command: &str, payload: Value, expected: Option<&str>) -> Request {
    Request {
        command: command.into(),
        payload,
        context: RequestContext {
            expected_revision: expected.map(str::to_owned),
            ..Default::default()
        },
    }
}

#[test]
fn reconnect_locator_preserves_history_and_rejects_stale_selection() {
    kinekino_platform::install();
    let fixture = Fixture::new();
    let proposal_root = fixture.root.join("proposals");
    let captured = fixture_proposal(&proposal_root);
    let proposal_id = &captured[0].document.proposal_id;
    let old = json!({"kind":"local_video","source_id":"movie","path":"D:/old/movie.mp4"});
    let new = json!({"kind":"local_video","source_id":"movie","path":"C:/media/movie.mp4"});
    let first = fixture.call(
        "create",
        &request(
            "projects.create",
            json!({
                "project_id":"reconnect","title":"Keep title","content":{
                    "watching":{"label":"Movie","source":old},"proposals":[{
                        "origin":"live","proposal_id":proposal_id,"draft_revision":"1"}]}
            }),
            None,
        ),
        Some(&proposal_root),
        0,
    );
    assert_eq!(
        first["data"]["project"]["proposals"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let payload = json!({"project_id":"reconnect","expected_source":old,
        "replacement":{"label":"Moved movie","source":new}});
    let reconnect = request("projects.reconnect_source", payload.clone(), Some("1"));
    let second = fixture.call("reconnect", &reconnect, None, 0);
    assert_eq!(second["data"]["scope"], "watching_locator_only");
    for flag in [
        "content_identity_verified",
        "source_opened",
        "playback_rebound",
    ] {
        assert_eq!(second["data"][flag], false);
    }
    let state = &second["data"]["state"];
    let mut expected = first["data"]["project"].clone();
    expected["revision"] = json!("2");
    expected["parent_revision"] = json!("1");
    expected["watching"] = payload["replacement"].clone();
    assert_eq!(state["project"], expected);
    let inspect = request("projects.inspect", json!({"project_id":"reconnect"}), None);
    assert_eq!(
        fixture.call("fresh-inspect", &inspect, None, 0)["data"],
        *state
    );
    assert_eq!(
        fixture.call("stale-revision", &reconnect, None, 5)["error"]["code"],
        "revision_conflict"
    );
    let wrong_source = request("projects.reconnect_source", payload.clone(), Some("2"));
    assert_eq!(
        fixture.call("stale-selection", &wrong_source, None, 5)["error"]["code"],
        "revision_conflict"
    );
    let mut changed_id = payload;
    changed_id["expected_source"] = new.clone();
    changed_id["replacement"]["source"]["source_id"] = json!("different");
    fixture.call(
        "changed-id",
        &request("projects.reconnect_source", changed_id, Some("2")),
        None,
        2,
    );
    assert_eq!(
        fixture.call("unchanged-after-rejection", &inspect, None, 0)["data"],
        *state
    );
    let historical = fixture.call(
        "old-revision",
        &request(
            "projects.inspect",
            json!({"project_id":"reconnect","revision":"1"}),
            None,
        ),
        None,
        0,
    );
    assert_eq!(historical["data"]["project"], first["data"]["project"]);
    let schema = fixture.call(
        "reconnect-schema",
        &request(
            "schema",
            json!({"command":"projects.reconnect_source"}),
            None,
        ),
        None,
        0,
    );
    assert_eq!(
        schema["data"]["request"]["properties"]["payload"]["required"],
        json!(["project_id", "expected_source", "replacement"])
    );
    assert_eq!(
        schema["data"]["response"]["properties"]["data"]["anyOf"][0]["properties"]["scope"]["const"],
        "watching_locator_only"
    );
    let valid = json!({"project_id":"reconnect","expected_source":new,
        "replacement":{"label":"Third","source":new}});
    let mut invalid_payloads = Vec::new();
    let mut relative = valid.clone();
    relative["replacement"]["source"]["path"] = json!("relative.mp4");
    invalid_payloads.push(("relative-path", relative));
    let mut kind = valid.clone();
    kind["replacement"]["source"] = json!({"kind":"episode_page","source_id":"movie","episode_url":"https://catalog.example.test/e/example"});
    invalid_payloads.push(("changed-kind", kind));
    let mut credentials = valid.clone();
    credentials["replacement"]["source"] = json!({"kind":"episode_page","source_id":"movie","episode_url":"https://user:password@catalog.example.test/e/example"});
    invalid_payloads.push(("credential-url", credentials));
    let mut extra = valid.clone();
    extra["unrecognized"] = json!(true);
    invalid_payloads.push(("unknown-field", extra));
    for (name, payload) in invalid_payloads {
        let response = fixture.call(
            name,
            &request("projects.reconnect_source", payload, Some("2")),
            None,
            2,
        );
        assert_eq!(response["error"]["code"], "invalid_argument");
    }
    for (name, revision) in [("missing-revision", None), ("zero-revision", Some("0"))] {
        fixture.call(
            name,
            &request("projects.reconnect_source", valid.clone(), revision),
            None,
            2,
        );
    }
    let mut timeout = request("projects.reconnect_source", valid.clone(), Some("2"));
    timeout.context.timeout_ms = Some(1);
    fixture.call("unsupported-timeout", &timeout, None, 2);
    let mut idempotency = request("projects.reconnect_source", valid.clone(), Some("2"));
    idempotency.context.idempotency_key = Some("no-replay-contract".into());
    fixture.call("unsupported-idempotency", &idempotency, None, 2);
    assert_eq!(
        fixture.call("unchanged-after-errors", &inspect, None, 0)["data"],
        *state
    );
    let sdk = PlayerSdk::default()
        .with_project_store(ProjectFiles::open(&fixture.root.join("projects")).unwrap());
    let client: &dyn ApplicationClient = &sdk;
    let third = client
        .reconnect_project_source(
            RequestContext {
                expected_revision: Some("2".into()),
                ..Default::default()
            },
            serde_json::from_value(valid).unwrap(),
        )
        .unwrap();
    assert_eq!(third.state.project.revision.0, 3);
    assert!(!third.content_identity_verified && !third.source_opened && !third.playback_rebound);
    assert_eq!(
        serde_json::to_value(&third.state.project.proposals).unwrap(),
        state["project"]["proposals"]
    );
    assert_eq!(
        fixture.call("typed-reconnect-fresh-inspect", &inspect, None, 0)["data"],
        serde_json::to_value(third.state).unwrap()
    );
}
fn fixture_proposal(root: &Path) -> Vec<StoredProposal> {
    let store = ProposalFiles::open(root).unwrap();
    kinekino_player::application::proposals::create(
        &store,
        CreateProposal {
            proposal_id: "fixture-correction".into(),
            source: SourceSnapshot::Cue(CueSourceSnapshot {
                source_id: "source-a".into(),
                source_revision: DecimalU64(1),
                source_sha256: "a".repeat(64),
                cues: vec![SourceCue {
                    cue_id: "cue-1".into(),
                    text: "원문 日本語".into(),
                    start_us: None,
                    end_us: None,
                }],
            }),
            generated: GeneratedResult {
                task: ProposalTask::Correction,
                raw_text: "model output requires manual repair".into(),
            },
        },
    )
    .unwrap();
    vec![store.load("fixture-correction").unwrap()]
}

#[test]
fn actual_cli_and_typed_client_persist_complete_snapshots_with_cas_and_fail_closed_corruption() {
    kinekino_platform::install();
    let fixture = Fixture::new();
    let actual_root = std::env::var_os("PROJECT_ACTUAL_PROPOSAL_ROOT").map(PathBuf::from);
    let proposal_root = actual_root
        .as_deref()
        .unwrap_or(fixture.root.as_path())
        .to_owned();
    let (proposal_root, captured) = if actual_root.is_some() {
        let store = ProposalFiles::open(&proposal_root).unwrap();
        let mut captured = Vec::new();
        for model in ["gemma270", "llama1b", "qwen06"] {
            for task in ["correction", "cards", "highlights"] {
                captured.push(store.load(&format!("actual-{model}-{task}")).unwrap());
            }
        }
        (proposal_root, captured)
    } else {
        let root = fixture.root.join("proposals");
        let captured = fixture_proposal(&root);
        (root, captured)
    };
    let refs: Vec<_> = captured.iter().map(|proposal| json!({"origin":"live","proposal_id":proposal.document.proposal_id,"draft_revision":proposal.document.draft_revision,"working_draft":null})).collect();
    let create = request(
        "projects.create",
        json!({"project_id":"actual-work","title":"시청·편집 작업","content":{"watching":null,"proposals":refs}}),
        None,
    );
    let unsupported = PlayerSdk::default().execute(create.clone());
    assert_eq!(
        unsupported.error.unwrap().code,
        kinekino_player::application::api::ErrorCode::Unsupported
    );
    let first: StoredProjectRevision = serde_json::from_value(
        fixture.call("create", &create, Some(&proposal_root), 0)["data"].clone(),
    )
    .unwrap();
    assert_eq!(first.project.proposals.len(), captured.len());
    for (snapshot, original) in first.project.proposals.iter().zip(&captured) {
        assert_eq!(canonical_sha256(original).unwrap(), snapshot.capture_sha256);
        assert_eq!(
            serde_json::to_value(original).unwrap(),
            serde_json::to_value(&snapshot.captured).unwrap()
        );
    }
    let inspect = request(
        "projects.inspect",
        json!({"project_id":"actual-work"}),
        None,
    );
    let reopened = fixture.call("restart-inspect", &inspect, None, 0);
    assert_eq!(reopened["data"], serde_json::to_value(&first).unwrap());
    let client: &dyn ApplicationClient = &PlayerSdk::default()
        .with_project_store(ProjectFiles::open(&fixture.root.join("projects")).unwrap());
    let typed = client
        .inspect_project(
            Default::default(),
            InspectProject {
                project_id: "actual-work".into(),
                revision: None,
            },
        )
        .unwrap();
    assert_eq!(typed.revision_sha256, first.revision_sha256);
    let saved: Vec<_> = first.project.proposals.iter().map(|snapshot| {
        let working = snapshot.captured.document.draft.clone().unwrap_or_else(|| ProposalPayload::Correction { edits: vec![] });
        json!({"origin":"saved","proposal_id":snapshot.captured.document.proposal_id,"capture_sha256":snapshot.capture_sha256,"working_draft":working})
    }).collect();
    let save = request(
        "projects.save_revision",
        json!({"project_id":"actual-work","title":"저장된 편집 작업","content":{"watching":null,"proposals":saved}}),
        Some("1"),
    );
    let second: StoredProjectRevision = serde_json::from_value(
        fixture.call("save-without-proposal-backend", &save, None, 0)["data"].clone(),
    )
    .unwrap();
    assert_eq!(second.project.revision.0, 2);
    for (snapshot, original) in second.project.proposals.iter().zip(&captured) {
        assert_eq!(snapshot.capture_sha256, canonical_sha256(original).unwrap());
    }
    let conflict = fixture.call("stale-save", &save, None, 5);
    assert_eq!(conflict["error"]["code"], "revision_conflict");
    let history = request(
        "projects.inspect",
        json!({"project_id":"actual-work","revision":"1"}),
        None,
    );
    let old = fixture.call("history", &history, None, 0);
    assert_eq!(
        old["data"]["project"],
        serde_json::to_value(&first.project).unwrap()
    );
    assert_eq!(old["data"]["latest_revision"], "2");
    let mut wrong_sha = save.clone();
    wrong_sha.context.expected_revision = Some("2".into());
    wrong_sha.payload["content"]["proposals"][0]["capture_sha256"] = json!("0".repeat(64));
    fixture.call("wrong-saved-sha", &wrong_sha, None, 2);
    let empty = request(
        "projects.create",
        json!({"project_id":"empty","title":"Empty","content":{"watching":null,"proposals":[]}}),
        None,
    );
    fixture.call("empty-context", &empty, None, 2);
    assert!(!fixture.root.join("projects/p-empty").exists());
    let schema = fixture.call(
        "schema",
        &request("schema", json!({"command":"projects.save_revision"}), None),
        None,
        0,
    );
    assert_eq!(schema["data"]["status"], "supported");
    let before = fixture.call("final-restart", &inspect, None, 0);
    assert_eq!(before["data"], serde_json::to_value(&second).unwrap());
    fs::create_dir(fixture.root.join("projects/p-actual-work/r-3")).unwrap();
    let corrupt = fixture.call("corrupt-latest", &inspect, None, 3);
    assert_eq!(corrupt["error"]["code"], "needs_input");
    fixture.call("corrupt-history-no-fallback", &history, None, 3);
    for original in &captured {
        let current = ProposalFiles::open(&proposal_root)
            .unwrap()
            .load(&original.document.proposal_id)
            .unwrap();
        assert_eq!(
            canonical_sha256(original).unwrap(),
            canonical_sha256(&current).unwrap()
        );
    }
}

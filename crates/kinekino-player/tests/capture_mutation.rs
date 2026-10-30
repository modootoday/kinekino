#![cfg(windows)]
#[path = "support/cli_exe.rs"]
mod cli_exe;
#[path = "support/owned_process_evidence.rs"]
pub mod owned_process_evidence;
use kinekino_player::{
    application::{api::Request, projects::ProjectStore, proposals::ProposalStore},
    infrastructure::{content_digest, projects::ProjectFiles, proposals::ProposalFiles},
};
use serde_json::{Value, json};
use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

struct Fixture {
    root: PathBuf,
    executable: PathBuf,
    sha256: String,
}
// The existing owned-process adapter's shared self-tests use this caller hook.
fn hash(bytes: &[u8]) -> String {
    content_digest::sha256(
        &mut std::io::Cursor::new(bytes),
        kinekino_player::application::local_host::MAX_RESPONSE_BYTES as u64,
    )
    .unwrap()
    .1
}
fn digest(path: &Path) -> String {
    content_digest::sha256(&mut fs::File::open(path).unwrap(), 128 * 1024 * 1024)
        .unwrap()
        .1
}
impl Fixture {
    fn new() -> Self {
        let base = PathBuf::from(
            std::env::var_os("CAPTURE_CLI_EVIDENCE_ROOT").expect("Set an explicit evidence root"),
        );
        assert!(base.is_absolute());
        fs::create_dir_all(&base).unwrap();
        let root = base.join(format!(
            "run-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        for name in ["projects", "proposals"] {
            fs::create_dir(root.join(name)).unwrap();
        }
        let original = PathBuf::from(crate::cli_exe::kinekino_cli_exe());
        let sha256 =
            std::env::var("CAPTURE_CLI_SHA256").expect("Pin the freshly built actual CLI SHA256");
        assert_eq!(digest(&original), sha256);
        let executable = root.join(kinekino_player::domain::product::PRODUCT.cli_executable);
        fs::copy(&original, &executable).unwrap();
        assert_eq!(digest(&executable), sha256);
        fs::write(
            root.join("binary.json"),
            serde_json::to_vec_pretty(
                &json!({"original":original,"copy":executable,"sha256":sha256}),
            )
            .unwrap(),
        )
        .unwrap();
        Self {
            root,
            executable,
            sha256,
        }
    }
    fn call(
        &self,
        name: &str,
        command: &str,
        payload: Value,
        proposal_root: &Path,
        exit: u32,
    ) -> Value {
        assert_eq!(digest(&self.executable), self.sha256);
        let evidence = self.root.join(name);
        fs::create_dir(&evidence).unwrap();
        let mut request = Request::new(command);
        request.context.request_id = name.into();
        request.payload = payload;
        let input = serde_json::to_vec(&request).unwrap();
        fs::write(evidence.join("request.json"), &input).unwrap();
        let arguments = vec![
            OsString::from("--input"),
            OsString::from("-"),
            OsString::from("--project-root"),
            self.root.join("projects").into_os_string(),
            OsString::from("--proposal-root"),
            proposal_root.as_os_str().to_owned(),
        ];
        let output = match owned_process_evidence::run_process_with_arguments(
            self.executable.clone(),
            &evidence,
            &input,
            arguments,
            Duration::from_secs(10),
            false,
        ) {
            Ok(output) => output,
            Err(failure) => {
                let failure = failure.closeout();
                fs::write(
                    evidence.join("failure-final.json"),
                    serde_json::to_vec_pretty(&failure.observation()).unwrap(),
                )
                .unwrap();
                let retained = Box::leak(failure);
                panic!("Original CLI failure retained: {retained:?}");
            }
        };
        // Preserve raw process evidence before checking semantic assertions.
        fs::write(evidence.join("observation.json"), serde_json::to_vec_pretty(&json!({"receipt":owned_process_evidence::receipt_json(&output.receipt),"stdout":output.stdout_evidence,"stderr":output.stderr_evidence,"elapsed_ms":output.elapsed_ms})).unwrap()).unwrap();
        assert_eq!(output.receipt.child_exit, Some(exit));
        assert!(!output.receipt.forced);
        assert!(output.receipt.stdin_eof_requested);
        assert!(output.stderr.is_empty());
        assert_eq!(
            output.stdout.iter().filter(|byte| **byte == b'\n').count(),
            1
        );
        assert_eq!(digest(&self.executable), self.sha256);
        let response: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(response["ok"], exit == 0);
        response
    }
}
fn draft(text: &str) -> Value {
    json!({"task":"correction","edits":[{"cue_id":"cue-1","text":text}]})
}

#[test]
#[ignore = "Requires the central actual CLI execution slot and fixed binary/evidence pins"]
fn actual_standalone_cli_capture_edit_edit_accept_retry_and_fail_closed_expectations() {
    kinekino_platform::install();
    let fixture = Fixture::new();
    let proposals = fixture.root.join("proposals");
    fixture.call("01-create-proposal", "proposals.create", json!({"proposal_id":"draft-a","source":{"source_id":"source-a","source_revision":"1","source_sha256":"a".repeat(64),"cues":[{"cue_id":"cue-1","text":"Original.","start_us":null,"end_us":null}]},"generated":{"task":"correction","raw_text":"invalid output"}}), &proposals, 0);
    let project = fixture.call("02-capture", "projects.create", json!({"project_id":"work","title":"Work","content":{"watching":null,"proposals":[{"origin":"live","proposal_id":"draft-a","draft_revision":"1"}]}}), &proposals, 0)["data"].clone();
    assert_eq!(project["project"]["schema_version"], "2");
    assert!(project["project"]["proposals"][0]["proposal_origin"].is_object());
    let anchor = json!({"project_id":"work","project_revision":"1","project_revision_sha256":project["revision_sha256"],"proposal_id":"draft-a","capture_sha256":project["project"]["proposals"][0]["capture_sha256"]});
    let first_payload = json!({"anchor":anchor,"draft":draft("First.")});
    let first = fixture.call(
        "03-edit",
        "proposals.edit_capture",
        first_payload.clone(),
        &proposals,
        0,
    )["data"]
        .clone();
    let second = fixture.call(
        "04-edit-continuation",
        "proposals.edit_capture",
        json!({"anchor":anchor,"expected":first["continuation"],"draft":draft("Second.")}),
        &proposals,
        0,
    )["data"]
        .clone();
    let accept =
        json!({"anchor":anchor,"expected":second["continuation"],"target_source_revision":"1"});
    let accepted = fixture.call(
        "05-accept",
        "proposals.accept_capture",
        accept.clone(),
        &proposals,
        0,
    )["data"]
        .clone();
    let retry = fixture.call(
        "06-exact-retry",
        "proposals.accept_capture",
        accept.clone(),
        &proposals,
        0,
    )["data"]
        .clone();
    assert_eq!(accepted["reused_acceptance"], false);
    assert_eq!(retry["reused_acceptance"], true);
    assert_eq!(accepted["state"], retry["state"]);
    assert_eq!(accepted["continuation"], retry["continuation"]);
    assert_eq!(accepted["continuation"]["draft_revision"], "4");
    let stale = fixture.call(
        "07-stale-edit",
        "proposals.edit_capture",
        first_payload,
        &proposals,
        5,
    );
    assert_eq!(stale["error"]["code"], "revision_conflict");
    let mut forged = accept.clone();
    forged["expected"]["state_sha256"] = json!("0".repeat(64));
    assert_eq!(
        fixture.call(
            "08-forged-retry",
            "proposals.accept_capture",
            forged,
            &proposals,
            5
        )["error"]["code"],
        "revision_conflict"
    );
    let mut wrong_anchor = accept.clone();
    wrong_anchor["anchor"]["project_revision_sha256"] = json!("0".repeat(64));
    assert_eq!(
        fixture.call(
            "09-wrong-project",
            "proposals.accept_capture",
            wrong_anchor,
            &proposals,
            5
        )["error"]["code"],
        "revision_conflict"
    );
    let foreign = fixture.root.join("foreign");
    fs::create_dir(&foreign).unwrap();
    let original = ProposalFiles::open(&proposals).unwrap();
    let foreign_store = ProposalFiles::open(&foreign).unwrap();
    // Equal original capture bytes and an independently issued origin.
    let captured: kinekino_player::domain::proposals::StoredProposal =
        serde_json::from_value(project["project"]["proposals"][0]["captured"].clone()).unwrap();
    foreign_store.publish(None, &captured).unwrap();
    foreign_store.capture_current("draft-a", 1).unwrap();
    drop(foreign_store);
    assert_eq!(
        fixture.call(
            "10-foreign-backend",
            "proposals.accept_capture",
            accept.clone(),
            &foreign,
            5
        )["error"]["code"],
        "revision_conflict"
    );
    let projects = ProjectFiles::open(&fixture.root.join("projects")).unwrap();
    let mut legacy = projects.load("work", Some(1)).unwrap().project;
    legacy.project_id = "legacy".into();
    legacy.schema_version = "1".into();
    legacy.proposals[0].proposal_origin = None;
    let legacy = projects.publish(None, &legacy).unwrap();
    drop(projects);
    let mut legacy_payload = accept;
    legacy_payload["anchor"]["project_id"] = json!("legacy");
    legacy_payload["anchor"]["project_revision_sha256"] = json!(legacy.revision_sha256);
    assert_eq!(
        fixture.call(
            "11-legacy-readonly",
            "proposals.accept_capture",
            legacy_payload,
            &proposals,
            3
        )["error"]["code"],
        "needs_input"
    );
    let reopened = fixture.call(
        "12-project-unchanged",
        "projects.inspect",
        json!({"project_id":"work","revision":"1"}),
        &proposals,
        0,
    );
    assert_eq!(reopened["data"], project);
    assert_eq!(
        original.load("draft-a").unwrap().document.draft_revision.0,
        4
    );
}

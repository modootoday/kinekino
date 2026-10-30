//! Manual imported annotations only. No audio, model, network or GUI execution.
#![cfg(windows)]
#[path = "support/cli_exe.rs"]
mod cli_exe;
#[path = "speaker_annotations_cli/cooperative_execution.rs"]
mod cooperative_execution;
#[path = "sprite_capture_cli/custody.rs"]
mod custody;
use kinekino_player::{
    application::{
        api::{ApiError, ErrorCode, PlayerSdk, Request},
        review_source::MutationControl,
        speaker_annotations::{self, EditSpeakers},
    },
    domain::speaker_annotations::SpeakerDocument,
    infrastructure::speaker_annotation_files::LocalSpeakerFiles,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn root() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "speakers-cli-{}-{}",
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
    json!({"schema_version":"speaker-annotations/1","document_id":"manual-demo","revision":"1","parent_sha256":null,
    "source":{"media_id":"media","media_sha256":"a".repeat(64),"audio_track_id":"audio-0","source_revision":"1","in_us":"0","out_us":"10000000","cue_source_sha256":"b".repeat(64),"model_refs":[]},
    "original":{"speakers":[{"id":"s1","label":"화자1"},{"id":"s2","label":"화자2"}],"spans":[
        {"id":"p1","in_us":"1000000","out_us":"4000000","speaker_ids":["s1"],"origin":"imported","evidence":[],"review_state":"reviewed"},
        {"id":"p2","in_us":"3000000","out_us":"6000000","speaker_ids":["s1","s2"],"origin":"imported","evidence":[],"review_state":"needs_review"},
        {"id":"p3","in_us":"6000000","out_us":"7000000","speaker_ids":[],"origin":"imported_unknown","evidence":[],"review_state":"unreviewed"}
    ],"links":[{"cue_id":"c1","span_ids":["p1"],"speaker_ids":["s1"],"assignment_origin":"imported"},{"cue_id":"c2","span_ids":["p2"],"speaker_ids":["s1","s2"],"assignment_origin":"imported"},{"cue_id":"c3","span_ids":["p3"],"speaker_ids":[],"assignment_origin":"imported_unknown"}]},"edits":[]})
}
struct Owner(Child);
impl Drop for Owner {
    fn drop(&mut self) {
        if !matches!(self.0.try_wait(), Ok(Some(_))) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}
fn call(root: &Path, label: &str, request: &Request, ok: bool) -> Value {
    let folder = root.join(label);
    fs::create_dir(&folder).unwrap();
    let input = folder.join("REQUEST.json");
    fs::write(&input, serde_json::to_vec_pretty(request).unwrap()).unwrap();
    let mut child = Owner(
        Command::new(crate::cli_exe::kinekino_cli_exe())
            .arg("--input")
            .arg(input)
            .stdin(Stdio::null())
            .stdout(File::create(folder.join("stdout.json")).unwrap())
            .stderr(File::create(folder.join("stderr.raw")).unwrap())
            .spawn()
            .unwrap(),
    );
    let cutoff = Instant::now() + Duration::from_secs(15);
    let status = loop {
        if let Some(s) = child.0.try_wait().unwrap() {
            break s;
        }
        assert!(Instant::now() < cutoff, "Original CLI timeout: {label}");
        std::thread::sleep(Duration::from_millis(20));
    };
    fs::write(folder.join("PROCESS.json"),serde_json::to_vec_pretty(&json!({"pid":child.0.id(),"exit_code":status.code(),"forced":false,"original_child_waited":true})).unwrap()).unwrap();
    let response: Value =
        serde_json::from_slice(&fs::read(folder.join("stdout.json")).unwrap()).unwrap();
    assert_eq!(response["ok"], ok, "{label}: {response}");
    assert_eq!(status.success(), ok, "{label}: {response}");
    response
}
fn read(path: &Path, pin: Option<String>) -> Request {
    let mut r = Request::new("subtitles.speakers");
    r.payload = json!({"input_path":path,"input_sha256":pin});
    r
}
fn edit(input: &Path, output: &Path, revision: u64, edits: Value) -> Request {
    let mut r = Request::new("subtitles.edit_speakers");
    r.context.expected_revision = Some(revision.to_string());
    r.payload = json!({"input_path":input,"input_sha256":sha(&fs::read(input).unwrap()),"output_path":output,"edits":edits});
    r
}
#[test]
fn normal_cli_manual_edits_undo_redo_preserve_original_overlap_unknown_and_reopen() {
    kinekino_platform::install();
    let root = root();
    let input = root.join("original.json");
    fs::write(&input, serde_json::to_vec_pretty(&fixture()).unwrap()).unwrap();
    let source = custody::Guard::acquire(&input);
    let binary = custody::Guard::acquire(Path::new(crate::cli_exe::kinekino_cli_exe()));
    fs::write(root.join("WITNESS.json"),serde_json::to_vec_pretty(&json!({"synthetic_annotations":true,"source":source.evidence(),"cli":binary.evidence(),"audio_model_executed":false,"quality_approved":false})).unwrap()).unwrap();
    let original = call(&root, "read-original", &read(&input, None), true);
    assert_eq!(original["data"]["current"], fixture()["original"]);
    for command in ["subtitles.speakers", "subtitles.edit_speakers"] {
        let mut r = Request::new("schema");
        r.payload = json!({"command":command});
        assert_eq!(
            call(&root, command, &r, true)["data"]["status"],
            "supported"
        );
    }
    let out2 = root.join("r2.json");
    let changed = call(
        &root,
        "edit-merge-split",
        &edit(
            &input,
            &out2,
            1,
            json!([
                {"op":"rename","speaker_id":"s1","label":"사용자 이름"},
                {"op":"split","speaker_id":"s1","new_speaker":{"id":"s3","label":"분할"},"span_ids":["p2"],"cue_ids":["c2"]},
                {"op":"merge","speaker_ids":["s2","s3"],"target":"s2"}
            ]),
        ),
        true,
    );
    assert_eq!(
        changed["data"]["document"]["original"],
        fixture()["original"]
    );
    assert_eq!(changed["data"]["document"]["source"], fixture()["source"]);
    assert_eq!(changed["data"]["document"]["parent_sha256"], source.sha256);
    assert_eq!(changed["data"]["document"]["revision"], "2");
    assert_eq!(
        changed["data"]["document"]["original"]["spans"][0]["review_state"],
        "reviewed"
    );
    assert_eq!(
        changed["data"]["current"]["spans"][0]["review_state"],
        "needs_review"
    );
    assert_eq!(changed["data"]["quality_approved"], false);
    assert_eq!(
        changed["data"]["current"]["spans"][1]["speaker_ids"],
        json!(["s2"])
    );
    assert_eq!(
        changed["data"]["current"]["spans"][2]["speaker_ids"],
        json!([])
    );
    let out3 = root.join("r3.json");
    let undo = call(
        &root,
        "undo",
        &edit(&out2, &out3, 2, json!([{"op":"undo"}])),
        true,
    );
    assert_eq!(
        undo["data"]["current"]["spans"][1]["speaker_ids"],
        json!(["s2", "s3"])
    );
    let out4 = root.join("r4.json");
    let redo = call(
        &root,
        "redo",
        &edit(&out3, &out4, 3, json!([{"op":"redo"}])),
        true,
    );
    assert_eq!(redo["data"]["current"], changed["data"]["current"]);
    let denied = call(
        &root,
        "delete-used",
        &edit(
            &out4,
            &root.join("denied.json"),
            4,
            json!([{"op":"delete","speaker_id":"s1","reassign":null}]),
        ),
        false,
    );
    assert_eq!(denied["error"]["code"], "invalid_argument");
    assert!(!root.join("denied.json").exists());
    let out5 = root.join("r5.json");
    let assigned = call(
        &root,
        "explicit-unknown-and-cue",
        &edit(
            &out4,
            &out5,
            4,
            json!([{"op":"delete","speaker_id":"s1","reassign":[]},{"op":"assign_cues","cue_ids":["c3"],"speaker_ids":["s2"]}]),
        ),
        true,
    );
    assert_eq!(
        assigned["data"]["current"]["spans"][0]["speaker_ids"],
        json!([])
    );
    assert_eq!(
        assigned["data"]["current"]["spans"][2]["speaker_ids"],
        json!([])
    );
    assert_eq!(
        assigned["data"]["current"]["links"][2]["speaker_ids"],
        json!(["s2"])
    );
    for (current, raw) in assigned["data"]["current"]["spans"]
        .as_array()
        .unwrap()
        .iter()
        .zip(fixture()["original"]["spans"].as_array().unwrap())
    {
        assert_eq!(current["in_us"], raw["in_us"]);
        assert_eq!(current["out_us"], raw["out_us"]);
    }
    let reopened = call(
        &root,
        "reopen",
        &read(
            &out5,
            Some(assigned["data"]["sha256"].as_str().unwrap().into()),
        ),
        true,
    );
    assert_eq!(reopened["data"]["document"], assigned["data"]["document"]);
    assert_eq!(reopened["data"]["current"], assigned["data"]["current"]);
    assert_eq!(reopened["data"]["quality_approved"], false);
    assert_eq!(reopened["data"]["inference_performed"], false);
    assert_eq!(reopened["data"]["media_binding_verified"], false);
    source.assert_unchanged();
    binary.assert_unchanged();
}
#[test]
fn normal_cli_hash_revision_collision_and_unknown_edits_do_not_publish() {
    kinekino_platform::install();
    let root = root();
    let input = root.join("source.json");
    fs::write(&input, serde_json::to_vec(&fixture()).unwrap()).unwrap();
    let original = fs::read(&input).unwrap();
    let output = root.join("existing.json");
    fs::write(&output, b"original destination").unwrap();
    let request = edit(
        &input,
        &output,
        1,
        json!([{"op":"rename","speaker_id":"s1","label":"new"}]),
    );
    assert_eq!(
        call(&root, "collision", &request, false)["error"]["code"],
        "revision_conflict"
    );
    assert_eq!(fs::read(&output).unwrap(), b"original destination");
    for (label, revision, ops) in [
        ("stale-revision", 2, json!([{"op":"undo"}])),
        (
            "foreign-cue",
            1,
            json!([{"op":"assign_cues","cue_ids":["foreign"],"speaker_ids":[]}]),
        ),
    ] {
        let path = root.join(format!("{label}.json"));
        call(&root, label, &edit(&input, &path, revision, ops), false);
        assert!(!path.exists());
    }
    let mut bad = edit(
        &input,
        &root.join("bad.json"),
        1,
        json!([{"op":"rename","speaker_id":"s1","label":"new"}]),
    );
    bad.payload["input_sha256"] = json!("c".repeat(64));
    assert_eq!(
        call(&root, "stale-hash", &bad, false)["error"]["code"],
        "revision_conflict"
    );
    assert!(!root.join("bad.json").exists());
    bad.payload["input_sha256"] = json!(sha(&original));
    bad.payload["edits"] = json!([{"op":"approve_model"}]);
    assert_eq!(
        call(&root, "unknown-op", &bad, false)["error"]["code"],
        "invalid_argument"
    );
    assert_eq!(
        PlayerSdk::default()
            .execute(read(&input, None))
            .error
            .unwrap()
            .code,
        ErrorCode::Unsupported
    );
    assert_eq!(fs::read(&input).unwrap(), original);
}
#[test]
fn document_branching_never_reuses_historical_ids_or_redoes_discarded_branch() {
    kinekino_platform::install();
    let document: SpeakerDocument = serde_json::from_value(fixture()).unwrap();
    let ops = serde_json::from_value(
        json!([{"op":"add","speaker":{"id":"s3","label":"new"}},{"op":"undo"}]),
    )
    .unwrap();
    let undone = document.edit("c".repeat(64), ops).unwrap();
    assert_eq!(undone.current().unwrap(), document.current().unwrap());
    assert!(
        undone
            .edit(
                "d".repeat(64),
                serde_json::from_value(json!([{"op":"add","speaker":{"id":"s3","label":"reuse"}}]))
                    .unwrap()
            )
            .is_err()
    );
    let branched = undone
        .edit(
            "d".repeat(64),
            serde_json::from_value(json!([{"op":"rename","speaker_id":"s1","label":"branch"}]))
                .unwrap(),
        )
        .unwrap();
    assert!(
        branched
            .edit(
                "e".repeat(64),
                serde_json::from_value(json!([{"op":"redo"}])).unwrap()
            )
            .is_err()
    );
    let mut invalid = fixture();
    invalid["original"]["spans"][0]["out_us"] = json!("11000000");
    assert!(
        serde_json::from_value::<SpeakerDocument>(invalid)
            .unwrap()
            .current()
            .is_err()
    );
}
#[test]
fn cancel_before_atomic_publication_keeps_original_input_lease_and_no_output() {
    kinekino_platform::install();
    struct Cancel {
        root: PathBuf,
        input: PathBuf,
        seen: AtomicBool,
    }
    impl MutationControl for Cancel {
        fn checkpoint(&self) -> Result<(), ApiError> {
            if fs::read_dir(&self.root).unwrap().any(|e| {
                e.unwrap()
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".export-")
            }) {
                assert!(
                    fs::OpenOptions::new()
                        .write(true)
                        .open(&self.input)
                        .is_err()
                );
                self.seen.store(true, Ordering::SeqCst);
                return Err(ApiError::new(
                    ErrorCode::Cancelled,
                    "cancelled after temporary write before commit",
                ));
            }
            Ok(())
        }
    }
    let root = root();
    let input = root.join("source.json");
    let bytes = serde_json::to_vec(&fixture()).unwrap();
    fs::write(&input, &bytes).unwrap();
    let output = root.join("target.json");
    let control = Cancel {
        root: root.clone(),
        input: input.clone(),
        seen: AtomicBool::new(false),
    };
    let request: EditSpeakers = serde_json::from_value(
        edit(
            &input,
            &output,
            1,
            json!([{"op":"rename","speaker_id":"s1","label":"new"}]),
        )
        .payload,
    )
    .unwrap();
    assert_eq!(
        speaker_annotations::edit(&LocalSpeakerFiles, 1, request, &control)
            .unwrap_err()
            .code,
        ErrorCode::Cancelled
    );
    assert!(control.seen.load(Ordering::SeqCst));
    assert!(!output.exists());
    assert_eq!(fs::read(&input).unwrap(), bytes);
    assert!(fs::OpenOptions::new().write(true).open(&input).is_ok());
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
}

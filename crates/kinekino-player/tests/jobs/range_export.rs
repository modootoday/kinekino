use super::fixture::{Fixture, Host, request};
use kinekino_player::application::api::JobClient;
use kinekino_player::{
    application::{
        api::Request,
        jobs::{GetJob, JobId},
    },
    infrastructure::content_digest,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

const SOURCE: &str = "1\n00:00:01,000 --> 00:00:03,000\n첫째 原文\n\n2\n00:00:04,000 --> 00:00:06,000\n둘째 한글\n\n";
const EXPECTED: &str = "WEBVTT\n\n00:00:00.000 --> 00:00:01.000\n첫째 原文\n\n00:00:02.000 --> 00:00:03.000\n둘째 한글\n\n";
const SOURCE_SHA: &str = "2840bb1f0ba39a911fe040b9bb79861486628f0f2bb2e7a5bbd57432782017aa";
const OUTPUT_SHA: &str = "f85e351d515582490a97eb64a3febd6479851fd1993c5bee268262718125288c";
fn digest(path: &Path) -> String {
    content_digest::sha256(&mut fs::File::open(path).unwrap(), 1024 * 1024)
        .unwrap()
        .1
}
fn payload(fixture: &Fixture, output: &str) -> Value {
    json!({"operation":"subtitles.export_range","parameters":{"input_path":fixture.root.join("source.srt"),"output_path":fixture.root.join(output),"input_format":"srt","output_format":"vtt","range":{"in_us":"2000000","out_us":"5000000"},"allow_loss":false,"speaker_policy":"omit"},"expected_source_sha256":SOURCE_SHA,"execution_timeout_ms":"30000"})
}
fn submission(payload: Value, key: &str) -> Request {
    let mut value = request("jobs.submit", payload);
    value.context.idempotency_key = Some(key.into());
    value
}
fn terminal(fixture: &Fixture, host: &Host, job_id: &Value, label: &str) -> Value {
    let id: JobId = serde_json::from_value(job_id.clone()).unwrap();
    let client = fixture.endpoint(host);
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        let job = client
            .get_job(Default::default(), GetJob { job_id: id.clone() })
            .unwrap();
        if job.state.terminal() {
            break;
        }
        assert!(Instant::now() < deadline, "job did not become terminal");
        std::thread::sleep(Duration::from_millis(10));
    }
    fixture.call(
        label,
        &request("jobs.get", json!({"job_id":id})),
        Some(&host.endpoint),
        None,
        0,
    )["data"]
        .clone()
}

#[test]
fn fresh_cli_range_job_publishes_verified_bytes_and_operation_specific_idempotency() {
    kinekino_platform::install();
    let fixture = Fixture::named("range-export-success");
    fs::write(fixture.root.join("source.srt"), SOURCE).unwrap();
    assert_eq!(digest(&fixture.root.join("source.srt")), SOURCE_SHA);
    let host = Host::start(&fixture, "range", &fixture.root.join("store"));
    let ready: Value = serde_json::from_str(
        &fs::read_to_string(fixture.root.join("logs/range-ready.stdout")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        ready["execution_operations"],
        json!([
            "subtitles.convert",
            "subtitles.export_range",
            "subtitles.from_transcript"
        ])
    );
    let caps = fixture.call(
        "capabilities",
        &request("capabilities", json!({})),
        Some(&host.endpoint),
        None,
        0,
    );
    assert_eq!(
        caps["data"]["jobs"]["operations"],
        ready["execution_operations"]
    );
    let schema = fixture.call(
        "schema-submit",
        &request("schema", json!({"command":"jobs.submit"})),
        Some(&host.endpoint),
        None,
        0,
    );
    let variants = schema["data"]["request"]["properties"]["payload"]["oneOf"]
        .as_array()
        .unwrap();
    let range = variants
        .iter()
        .find(|variant| variant["properties"]["operation"]["const"] == "subtitles.export_range")
        .unwrap();
    assert_eq!(
        range["properties"]["parameters"],
        kinekino_player::application::subtitles::export_range::schema()
    );
    let input = payload(&fixture, "range.vtt");
    let submit = submission(input.clone(), "range-first");
    let accepted = fixture.call("submit-range", &submit, Some(&host.endpoint), None, 0);
    assert_eq!(accepted["data"]["created"], true);
    let id = &accepted["data"]["job"]["job_id"];
    assert_eq!(
        accepted["data"]["job"]["input"]["source"],
        json!({"sha256":SOURCE_SHA,"bytes":"94"})
    );
    let job = terminal(&fixture, &host, id, "get-range-terminal");
    assert_eq!(job["state"], "completed");
    assert_eq!(job["output"]["sha256"], OUTPUT_SHA);
    assert_eq!(job["output"]["bytes"], "98");
    assert_eq!(
        job["output"]["receipt"],
        json!({"output_path":fixture.root.join("range.vtt"),"output_format":"vtt","cue_count":2,"output_bytes":98,"losses":[],"encoding":"utf-8","overwrite":false})
    );
    assert_eq!(
        fs::read(fixture.root.join("range.vtt")).unwrap(),
        EXPECTED.as_bytes()
    );
    assert_eq!(digest(&fixture.root.join("range.vtt")), OUTPUT_SHA);
    let duplicate = fixture.call("same-key", &submit, Some(&host.endpoint), None, 0);
    assert_eq!(duplicate["data"]["created"], false);
    assert_eq!(&duplicate["data"]["job"]["job_id"], id);
    let mut changed = input.clone();
    changed["parameters"]["range"]["out_us"] = json!("5001000");
    let conflict = fixture.call(
        "changed-range",
        &submission(changed, "range-first"),
        Some(&host.endpoint),
        None,
        5,
    );
    assert_eq!(
        conflict["error"]["details"]["conflict_kind"],
        "idempotency_conflict"
    );
    let mut convert = input.clone();
    convert["operation"] = json!("subtitles.convert");
    convert["parameters"]
        .as_object_mut()
        .unwrap()
        .remove("range");
    assert_eq!(
        fixture.call(
            "changed-operation",
            &submission(convert, "range-first"),
            Some(&host.endpoint),
            None,
            5
        )["error"]["code"],
        "revision_conflict"
    );
    let list = fixture.call(
        "list-once",
        &request("jobs.list", json!({})),
        Some(&host.endpoint),
        None,
        0,
    );
    assert_eq!(list["data"]["jobs"].as_array().unwrap().len(), 1);
    let mut baseline = input["parameters"].clone();
    baseline["output_path"] = json!(fixture.root.join("baseline.vtt"));
    fixture.call(
        "synchronous-comparison",
        &request("subtitles.export_range", baseline),
        None,
        None,
        0,
    );
    assert_eq!(
        fs::read(fixture.root.join("baseline.vtt")).unwrap(),
        EXPECTED.as_bytes()
    );
    assert_eq!(digest(&fixture.root.join("source.srt")), SOURCE_SHA);
    host.stop();
    let stopped: Value = serde_json::from_str(
        &fs::read_to_string(fixture.root.join("logs/range-stop.stdout")).unwrap(),
    )
    .unwrap();
    assert_eq!(stopped["local_host_shutdown"]["pipe"], "joined");
    assert_eq!(stopped["local_host_shutdown"]["sdk"], "joined");
    assert_eq!(stopped["receipt"]["completed_total"], 1);
}

#[test]
fn fresh_cli_range_job_reports_digest_losses_empty_range_and_no_overwrite() {
    kinekino_platform::install();
    let fixture = Fixture::named("range-export-errors");
    fs::write(fixture.root.join("source.srt"), SOURCE).unwrap();
    let host = Host::start(&fixture, "range", &fixture.root.join("store"));
    let mut wrong = payload(&fixture, "wrong-sha.vtt");
    wrong["expected_source_sha256"] = json!("f".repeat(64));
    assert_eq!(
        fixture.call(
            "wrong-source-sha",
            &submission(wrong, "wrong-sha"),
            Some(&host.endpoint),
            None,
            8
        )["error"]["code"],
        "source_expired"
    );
    assert!(!fixture.root.join("wrong-sha.vtt").exists());
    let mut loss = payload(&fixture, "precision.vtt");
    loss["parameters"]["range"]["in_us"] = json!("2000001");
    let accepted = fixture.call(
        "loss-submit",
        &submission(loss.clone(), "loss-review"),
        Some(&host.endpoint),
        None,
        0,
    );
    let first_id = accepted["data"]["job"]["job_id"].clone();
    let job = terminal(&fixture, &host, &first_id, "loss-terminal");
    assert_eq!(job["state"], "needs_input");
    assert_eq!(job["failure"]["code"], "needs_input");
    assert!(
        job["failure"]["details"]["losses"]
            .as_array()
            .is_some_and(|items| !items.is_empty())
    );
    assert!(!fixture.root.join("precision.vtt").exists());
    loss["parameters"]["allow_loss"] = json!(true);
    let accepted = fixture.call(
        "loss-approved-submit",
        &submission(loss, "loss-approved"),
        Some(&host.endpoint),
        None,
        0,
    );
    let id = &accepted["data"]["job"]["job_id"];
    assert_ne!(&first_id, id);
    let approved = terminal(&fixture, &host, id, "loss-approved-terminal");
    assert_eq!(approved["state"], "completed");
    assert!(
        !approved["output"]["receipt"]["losses"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        approved["output"]["sha256"],
        digest(&fixture.root.join("precision.vtt"))
    );
    fs::write(fixture.root.join("existing.vtt"), "existing").unwrap();
    let before = digest(&fixture.root.join("existing.vtt"));
    let accepted = fixture.call(
        "no-overwrite-submit",
        &submission(payload(&fixture, "existing.vtt"), "no-overwrite"),
        Some(&host.endpoint),
        None,
        0,
    );
    let job = terminal(
        &fixture,
        &host,
        &accepted["data"]["job"]["job_id"],
        "no-overwrite-terminal",
    );
    assert_eq!(job["state"], "needs_input");
    assert_eq!(job["failure"]["code"], "revision_conflict");
    assert_eq!(digest(&fixture.root.join("existing.vtt")), before);
    let mut empty = payload(&fixture, "empty.vtt");
    empty["parameters"]["range"] = json!({"in_us":"9000000","out_us":"10000000"});
    let accepted = fixture.call(
        "empty-submit",
        &submission(empty, "empty"),
        Some(&host.endpoint),
        None,
        0,
    );
    let job = terminal(
        &fixture,
        &host,
        &accepted["data"]["job"]["job_id"],
        "empty-terminal",
    );
    assert_eq!(job["state"], "needs_input");
    assert_eq!(job["failure"]["code"], "invalid_argument");
    assert!(!fixture.root.join("empty.vtt").exists());
    let mut malformed = payload(&fixture, "malformed.vtt");
    malformed["parameters"]
        .as_object_mut()
        .unwrap()
        .remove("range");
    assert_eq!(
        fixture.call(
            "missing-range",
            &submission(malformed, "malformed"),
            Some(&host.endpoint),
            None,
            2
        )["error"]["code"],
        "invalid_argument"
    );
    assert_eq!(digest(&fixture.root.join("source.srt")), SOURCE_SHA);
    host.stop();
}

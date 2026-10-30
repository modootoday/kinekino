#![cfg(windows)]
use kinekino_player::application::api::JobClient;
#[path = "jobs/asset_build.rs"]
mod asset_build;
#[path = "jobs/batches.rs"]
mod batches;
#[path = "jobs/cancel.rs"]
mod cancel;
#[cfg(feature = "card-review-export")]
#[path = "jobs/card_bundle.rs"]
mod card_bundle;
#[path = "jobs/catalog_replay.rs"]
mod catalog_replay;
#[path = "support/cli_exe.rs"]
mod cli_exe;
#[path = "jobs/clip_cancel.rs"]
mod clip_cancel;
#[path = "jobs/clip_render.rs"]
mod clip_render;
#[path = "jobs/episode_capture.rs"]
mod episode_capture;
#[path = "jobs/fixture.rs"]
mod fixture;
#[path = "support/fixture_plugin.rs"]
mod fixture_plugin;
#[path = "jobs/frame_tools.rs"]
mod frame_tools;
#[path = "jobs/kinekino_process.rs"]
mod kinekino_process;
#[path = "jobs/native_episode.rs"]
mod native_episode;
#[path = "jobs/observation_guard.rs"]
mod observation_guard;
#[path = "jobs/playback_observation.rs"]
mod playback_observation;
#[path = "jobs/playback_owner.rs"]
mod playback_owner;
#[path = "jobs/provider.rs"]
mod provider;
#[path = "jobs/range_export.rs"]
mod range_export;
#[path = "jobs/resolve_cli.rs"]
mod resolve_cli;
#[path = "jobs/retaining_consumer.rs"]
mod retaining_consumer;
#[path = "../src/infrastructure/immutable_storage_paths.rs"]
mod retaining_storage_paths;
#[path = "jobs/same_endpoint.rs"]
mod same_endpoint;
#[path = "jobs/series.rs"]
mod series;
#[path = "jobs/synthetic_browser.rs"]
mod synthetic_browser;
use fixture::{Fixture, Host, request};
use kinekino_player::{
    application::{
        api::PlayerSdk,
        jobs::{GetJob, JobId, JobRecord, JobState},
    },
    infrastructure::jobs,
};
use serde_json::json;
use std::{
    fmt::Write,
    fs,
    time::{Duration, Instant},
};

#[test]
fn actual_cli_explicit_host_and_sdk_share_durable_jobs_without_gui() {
    kinekino_platform::install();
    let fixture = Fixture::new();
    let root = fixture.root.join("store");
    let host = Host::start(&fixture, "normal", &root);
    let endpoint = host.endpoint.clone();
    let client = fixture.endpoint(&host);
    let unavailable = fixture.call(
        "default-unsupported",
        &request("jobs.get", json!({})),
        None,
        None,
        3,
    );
    assert_eq!(unavailable["error"]["code"], "unsupported");
    let readonly = fixture.call(
        "read-caps",
        &request("capabilities", json!({})),
        None,
        Some(&root),
        0,
    );
    assert_eq!(readonly["data"]["jobs"]["execution_host_available"], false);
    let caps = fixture.call(
        "host-caps",
        &request("capabilities", json!({})),
        Some(&endpoint),
        None,
        0,
    );
    assert_eq!(caps["data"]["jobs"]["execution_host_available"], true);
    for (index, command) in [
        "jobs.get",
        "jobs.list",
        "jobs.watch",
        "jobs.submit",
        "jobs.cancel",
        "jobs.retry",
    ]
    .iter()
    .enumerate()
    {
        let schema = fixture.call(
            &format!("schema-{index}"),
            &request("schema", json!({"command":command})),
            Some(&endpoint),
            None,
            0,
        );
        assert_eq!(schema["data"]["status"], "supported");
    }
    let source = fixture.root.join("original.srt");
    let output = fixture.root.join("converted.vtt");
    let original = "1\n00:00:01,001 --> 00:00:03,456\n안녕하세요 日本語\n\n";
    fs::write(&source, original).unwrap();
    let mut submit = request(
        "jobs.submit",
        json!({"operation":"subtitles.convert","parameters":{"input_path":source,"output_path":output,"input_format":"srt","output_format":"vtt"}}),
    );
    submit.context.idempotency_key = Some("actual-first".into());
    let accepted = fixture.call("submit", &submit, Some(&endpoint), None, 0);
    assert_eq!(accepted["data"]["created"], true);
    let id: JobId = serde_json::from_value(accepted["data"]["job"]["job_id"].clone()).unwrap();
    let timeout = Instant::now() + Duration::from_secs(8);
    let completed = loop {
        let current = client
            .get_job(Default::default(), GetJob { job_id: id.clone() })
            .unwrap();
        if current.state.terminal() {
            break current;
        }
        assert!(Instant::now() < timeout);
        std::thread::sleep(Duration::from_millis(10));
    };
    assert_eq!(completed.state, JobState::Completed);
    let actual = fixture.call(
        "get-completed",
        &request("jobs.get", json!({"job_id":id})),
        Some(&endpoint),
        None,
        0,
    );
    assert_eq!(actual["data"]["state"], "completed");
    assert_eq!(actual["data"]["output"]["receipt"]["cue_count"], 1);
    assert!(
        fs::read_to_string(&output)
            .unwrap()
            .contains("안녕하세요 日本語")
    );
    assert_eq!(fs::read_to_string(&source).unwrap(), original);
    let duplicate = fixture.call("idempotent-submit", &submit, Some(&endpoint), None, 0);
    assert_eq!(duplicate["data"]["created"], false);
    assert_eq!(duplicate["data"]["job"]["job_id"], json!(id));
    let mut changed = submit.clone();
    changed.payload["execution_timeout_ms"] = json!("20000");
    let conflict = fixture.call("idempotency-conflict", &changed, Some(&endpoint), None, 5);
    assert_eq!(
        conflict["error"]["details"]["conflict_kind"],
        "idempotency_conflict"
    );
    fixture.call("readonly-submit", &submit, None, Some(&root), 3);
    let terminal_cancel = fixture.call(
        "cancel-completed",
        &request("jobs.cancel", json!({"job_id":id,"expected_attempt":"1"})),
        Some(&endpoint),
        None,
        0,
    );
    assert_eq!(terminal_cancel["data"]["disposition"], "already_terminal");
    fixture.call(
        "cancel-stale-attempt",
        &request("jobs.cancel", json!({"job_id":id,"expected_attempt":"2"})),
        Some(&endpoint),
        None,
        5,
    );
    let page = fixture.call(
        "watch-first",
        &request("jobs.watch", json!({"after_sequence":"0","limit":2})),
        Some(&endpoint),
        None,
        0,
    );
    assert_eq!(page["data"]["events"].as_array().unwrap().len(), 2);
    let next = page["data"]["next_sequence"].clone();
    let rest = fixture.call(
        "watch-reconnect",
        &request("jobs.watch", json!({"after_sequence":next,"limit":20})),
        Some(&endpoint),
        None,
        0,
    );
    assert_eq!(rest["data"]["gap"], false);
    assert_eq!(rest["data"]["events"].as_array().unwrap().len(), 3);
    fixture.call(
        "list",
        &request("jobs.list", json!({"limit":1})),
        Some(&endpoint),
        None,
        0,
    );
    let mut unsupported = submit.clone();
    unsupported.context.idempotency_key = Some("no-model".into());
    unsupported.payload["operation"] = json!("subtitles.transcribe");
    fixture.call("model-unsupported", &unsupported, Some(&endpoint), None, 3);
    let loss_source = fixture.root.join("loss.vtt");
    let loss_output = fixture.root.join("loss.srt");
    fs::write(&loss_source,"WEBVTT\n\nNOTE preserve this note\n\ncue-a\n00:00:01.000 --> 00:00:02.000 align:start\n안녕하세요\n").unwrap();
    let mut loss_request = request(
        "jobs.submit",
        json!({"operation":"subtitles.convert","parameters":{"input_path":loss_source,"output_path":loss_output,"input_format":"vtt","output_format":"srt"}}),
    );
    loss_request.context.idempotency_key = Some("loss-pending".into());
    let admitted = fixture.call("loss-submit", &loss_request, Some(&endpoint), None, 0);
    let loss_id: JobId = serde_json::from_value(admitted["data"]["job"]["job_id"].clone()).unwrap();
    let terminal = |id: &JobId| {
        let end = Instant::now() + Duration::from_secs(8);
        loop {
            let job = client
                .get_job(Default::default(), GetJob { job_id: id.clone() })
                .unwrap();
            if job.state.terminal() {
                break job;
            }
            assert!(Instant::now() < end);
            std::thread::sleep(Duration::from_millis(10));
        }
    };
    let needs_input = terminal(&loss_id);
    assert_eq!(needs_input.state, JobState::NeedsInput);
    assert!(!loss_output.exists());
    let mut retry = request("jobs.retry", json!({"job_id":loss_id}));
    retry.context.expected_revision = Some(needs_input.revision.0.to_string());
    let retried = fixture.call("explicit-retry", &retry, Some(&endpoint), None, 0);
    assert_eq!(retried["data"]["attempt"], "2");
    let again = terminal(&loss_id);
    assert_eq!(again.state, JobState::NeedsInput);
    assert_eq!(again.input_sha256, needs_input.input_sha256);
    fixture.call("stale-retry", &retry, Some(&endpoint), None, 5);
    loss_request.context.idempotency_key = Some("loss-explicit".into());
    loss_request.payload["parameters"]["allow_loss"] = json!(true);
    let accepted = fixture.call(
        "explicit-loss-confirmation",
        &loss_request,
        Some(&endpoint),
        None,
        0,
    );
    let explicit_id: JobId =
        serde_json::from_value(accepted["data"]["job"]["job_id"].clone()).unwrap();
    assert_eq!(terminal(&explicit_id).state, JobState::Completed);
    assert!(loss_output.exists());
    host.stop();
    let sdk = PlayerSdk::default().with_jobs(jobs::read_only(&root).unwrap());
    let restored = sdk
        .get_job(Default::default(), GetJob { job_id: id.clone() })
        .unwrap();
    assert_eq!(restored.input_sha256, completed.input_sha256);
    assert_eq!(restored.state, JobState::Completed);
    let fresh = fixture.call(
        "fresh-readonly",
        &request("jobs.get", json!({"job_id":id})),
        None,
        Some(&root),
        0,
    );
    assert_eq!(fresh["data"]["output"], actual["data"]["output"]);

    let forced_root = fixture.root.join("forced-store");
    let forced = Host::start(&fixture, "forced", &forced_root);
    let mut large = String::new();
    let line = "検証用の本文を保持します。原文を変更しません。";
    for cue in 1..=60000 {
        writeln!(large, "{cue}\n00:00:01,001 --> 00:00:03,456\n{line}\n").unwrap();
    }
    assert!(large.len() < 8 * 1024 * 1024);
    let large_path = fixture.root.join("forced-source.srt");
    let forced_output = fixture.root.join("forced-output.vtt");
    fs::write(&large_path, &large).unwrap();
    let mut forced_request = request(
        "jobs.submit",
        json!({"operation":"subtitles.convert","parameters":{"input_path":large_path,"output_path":forced_output,"input_format":"srt","output_format":"vtt"}}),
    );
    forced_request.context.idempotency_key = Some("forced".into());
    let forced_client = fixture.endpoint(&forced);
    let admitted = fixture.call(
        "forced-submit",
        &forced_request,
        Some(&forced.endpoint),
        None,
        0,
    );
    let forced_id = admitted["data"]["job"]["job_id"].clone();
    let forced_typed: JobId = serde_json::from_value(forced_id.clone()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let observed = loop {
        let job = forced_client
            .get_job(
                Default::default(),
                GetJob {
                    job_id: forced_typed.clone(),
                },
            )
            .unwrap();
        if job.state == JobState::Running {
            break job;
        }
        assert_eq!(job.state, JobState::Queued);
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(2));
    };
    assert!(!observed.publication_started);
    fs::write(
        fixture.root.join("logs/forced-observed-sdk.json"),
        serde_json::to_vec(&observed).unwrap(),
    )
    .unwrap();
    forced.kill_owned();
    let before = fixture.call(
        "forced-durable-before-restart",
        &request("jobs.get", json!({"job_id":forced_id})),
        None,
        Some(&forced_root),
        0,
    );
    assert!(
        !matches!(
            before["data"]["state"].as_str(),
            Some("completed" | "failed" | "needs_input")
        ),
        "forced stop did not catch active work: {before}"
    );
    let existed = forced_output.exists();
    let restarted = Host::start(&fixture, "restart", &forced_root);
    let interrupted = fixture.call(
        "restart-interrupted",
        &request("jobs.get", json!({"job_id":forced_id})),
        Some(&restarted.endpoint),
        None,
        0,
    );
    assert_eq!(interrupted["data"]["state"], "interrupted");
    assert_eq!(interrupted["data"]["attempt"], "1");
    assert_eq!(forced_output.exists(), existed);
    restarted.stop();
    let _typed: JobRecord = serde_json::from_value(interrupted["data"].clone()).unwrap();
}
mod infrastructure {
    pub(crate) use kinekino_player::infrastructure::os_policy;
}

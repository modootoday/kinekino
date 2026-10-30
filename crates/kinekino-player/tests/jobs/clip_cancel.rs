//! Actual CLI requests to the original renderer/Jobs; only the pre-CAS pause is a fixture.
use super::{
    cancel::{Gate, Paused, Release},
    fixture::{Fixture, request},
    kinekino_process::CliFailure,
};
use kinekino_jobs::infrastructure::WorkerShutdownOutcome;
use kinekino_player::application::api::JobClient;
use kinekino_player::{
    application::{
        api::PlayerSdk,
        jobs::{GetJob, JobId, JobState},
    },
    infrastructure::{
        clip_render_jobs::OpenedClipRenderHost,
        jobs,
        local_host::{EndpointClient, LocalHost, TransportOptions},
    },
};
use kinekino_process::domain::deadline::Deadline;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
#[test]
#[ignore = "Pinned host/normal CLI, existing media/fonts, real Jobs cancel/retry and fresh evidence; no GUI/models"]
fn actual_clip_cli_cancel_ack_get_and_manual_attempt_two_preserve_original_evidence()
-> Result<(), Box<CliFailure>> {
    kinekino_platform::install();
    let mut fixture = Fixture::named("clip-cancel-retry");
    fixture.keep_evidence();
    let source_config = PathBuf::from(
        std::env::var_os("CLIP_CANCEL_HOST").expect("Pinned existing host configuration"),
    );
    let bytes = fs::read(&source_config).unwrap();
    assert_eq!(
        hash(&bytes),
        std::env::var("CLIP_CANCEL_HOST_SHA256").unwrap()
    );
    let mut config: Value = serde_json::from_slice(&bytes).unwrap();
    let temporary = std::env::temp_dir().join(format!(
        "clip-cli-inputs-{}",
        hash(fixture.root.to_string_lossy().as_bytes())
    ));
    fs::create_dir(&temporary).unwrap();
    fs::create_dir(fixture.root.join("artifacts")).unwrap();
    config["render"]["artifact_root"] = json!(fixture.root.join("artifacts"));
    config["render"]["temporary_root"] = json!(temporary);
    let bytes = serde_json::to_vec_pretty(&config).unwrap();
    let path = fixture.root.join("clip-host.json");
    fs::write(&path, &bytes).unwrap();
    let configured = OpenedClipRenderHost::open(&path, &hash(&bytes)).unwrap();
    let saved = configured
        .projects
        .load(
            "cli-probed-project",
            Some(kinekino_player::domain::api_values::DecimalU64(2)),
        )
        .unwrap();
    let gate = Arc::new(Gate::new());
    let release = Release(gate.clone());
    let executor = Arc::new(Paused {
        inner: configured.executor,
        gate: gate.clone(),
        after: false,
    });
    let host =
        jobs::start_retaining_host_with_executor(&fixture.root.join("store"), executor).unwrap();
    let local = LocalHost::start(
        Arc::new(PlayerSdk::default().with_jobs(host.client())),
        TransportOptions::default(),
    )
    .unwrap();
    let endpoint = local.endpoint().to_owned();
    let client = EndpointClient::new(&endpoint, TransportOptions::default()).unwrap();
    let original = Deadline::new(
        Instant::now(),
        Duration::from_secs(240),
        Duration::from_secs(2),
    )
    .unwrap();
    let call = |label, value: &kinekino_player::application::api::Request, exit| {
        fixture
            .try_call_until(
                label,
                value,
                &endpoint,
                original,
                Duration::from_secs(10),
                exit,
            )
            .map_err(|failure| failure.closeout())
    };
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(
        || -> Result<(), Box<CliFailure>> {
            let mut submit = request(
                "jobs.submit",
                json!({"operation":"production.render_short","parameters":{"project":{"document_id":"cli-probed-project","publication_revision":"2","publication_sha256":saved.publication_sha256},"review_artifact_only":true},"execution_timeout_ms":"120000"}),
            );
            submit.context.idempotency_key = Some("cancel-clip-once".into());
            let accepted = call("submit", &submit, 0)?;
            let id: JobId =
                serde_json::from_value(accepted["data"]["job"]["job_id"].clone()).unwrap();
            gate.wait_render();
            let live = fs::read_dir(&temporary)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .collect::<Vec<_>>();
            let cancelled = call(
                "cancel",
                &request("jobs.cancel", json!({"job_id":id,"expected_attempt":"1"})),
                0,
            )?;
            gate.release();
            assert_eq!(cancelled["data"]["disposition"], "requested");
            assert_eq!(cancelled["data"]["job"]["state"], "cancelling");
            assert_eq!(live.len(), 2);
            fs::write(
                fixture.root.join("live-inputs.json"),
                serde_json::to_vec_pretty(&live).unwrap(),
            )
            .unwrap();
            let terminal = |expected| {
                let deadline = Instant::now() + Duration::from_secs(120);
                loop {
                    let job = client
                        .get_job(Default::default(), GetJob { job_id: id.clone() })
                        .unwrap();
                    if job.state.terminal() {
                        assert_eq!(job.state, expected, "{:?}", job.failure);
                        break job;
                    }
                    assert!(Instant::now() < deadline);
                    std::thread::sleep(Duration::from_millis(20));
                }
            };
            let done = terminal(JobState::Cancelled);
            let observed = call(
                "cancelled-get",
                &request("jobs.get", json!({"job_id":id})),
                0,
            )?;
            assert_eq!(observed["data"]["state"], "cancelled");
            assert!(observed["data"]["output"].is_null());
            let replay = call("submit-ack-replay", &submit, 0)?;
            assert_eq!(replay["data"]["created"], false);
            assert_eq!(replay["data"]["job"]["job_id"], id.as_str());
            let partial = fixture
                .root
                .join("artifacts")
                .join(format!("clip-{}-attempt-1", id.as_str()));
            assert!(!partial.join("manifest.json").exists());
            let preserved = ["clip.mp4", "render.json", "verification.json"]
                .map(|name| (name, hash(&fs::read(partial.join(name)).unwrap())));
            let mut retry = request("jobs.retry", json!({"job_id":id}));
            retry.context.expected_revision = Some((done.revision.0 - 1).to_string());
            assert_eq!(
                call("stale-retry", &retry, 5)?["error"]["code"],
                "revision_conflict"
            );
            retry.context.expected_revision = Some(done.revision.0.to_string());
            assert_eq!(call("manual-retry", &retry, 0)?["data"]["attempt"], "2");
            assert_eq!(
                call("retry-ack-replay", &retry, 5)?["error"]["code"],
                "revision_conflict"
            );
            let second = terminal(JobState::Completed);
            assert_eq!(second.attempt.0, 2);
            assert_eq!(second.input_sha256, done.input_sha256);
            let second_get = call(
                "attempt-two-get",
                &request("jobs.get", json!({"job_id":id})),
                0,
            )?;
            assert_eq!(second_get["data"]["state"], "completed");
            for (name, digest) in preserved {
                assert_eq!(hash(&fs::read(partial.join(name)).unwrap()), digest);
            }
            assert_eq!(fs::read_dir(&temporary).unwrap().count(), 0);
            fs::write(fixture.root.join("verification.json"),serde_json::to_vec_pretty(&json!({"cli_calls":8,"cancel_request_is_not_terminal_ack":true,"cancelled_get_verified":true,"same_submit_ack_replay_did_not_create_job":true,"manual_attempt_two_completed":true,"stale_revision_rejected":true,"attempt_one_evidence_preserved":true,"new_inputs_absent":true,"actual_transport_ack_loss_injected":false,"host_is_in_process_sdk_fixture":true,"quality_approved":false})).unwrap()).unwrap();
            Ok(())
        },
    ));
    drop(release);
    drop(client);
    drop(local);
    let deadline = Instant::now() + Duration::from_secs(15);
    let mut pending = host.begin_shutdown(deadline);
    loop {
        match pending.poll(deadline) {
            WorkerShutdownOutcome::Complete(done) => {
                fs::write(
                    fixture.root.join("shutdown.json"),
                    serde_json::to_vec_pretty(
                        &json!({"receipt":done.receipt,"error":done.errors.first()}),
                    )
                    .unwrap(),
                )
                .unwrap();
                assert!(done.errors.first().is_none());
                break;
            }
            WorkerShutdownOutcome::Pending(owner) => {
                pending = owner;
                assert!(Instant::now() < deadline);
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }
    match outcome {
        Ok(result) => result,
        Err(panic) => std::panic::resume_unwind(panic),
    }
}

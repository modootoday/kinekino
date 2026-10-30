//! Actual pinned CLI and typed client on the same original Jobs host.
use super::{
    fixture::{Fixture, request},
    kinekino_process::{
        CliFailure, OwnedHost, ScenarioFailure, close_request, phase_deadline, transport_at,
    },
};
use kinekino_player::application::api::JobBatchClient;
use kinekino_player::application::{
    api::RequestContext,
    job_batches::service::{BatchIdentity, InspectBatch},
};
use kinekino_process::domain::{deadline::Deadline, observation::FamilyObservation};
use serde_json::{Value, json};
use std::{
    ffi::OsString,
    fs, io,
    panic::{AssertUnwindSafe, catch_unwind},
    time::{Duration, Instant},
};
fn context(revision: u64) -> RequestContext {
    RequestContext {
        expected_revision: Some(revision.to_string()),
        ..RequestContext::default()
    }
}
fn inspect() -> InspectBatch {
    InspectBatch {
        batch_id: "actual-cli".into(),
        offset: 0,
        limit: 8,
    }
}
fn identity() -> BatchIdentity {
    BatchIdentity {
        batch_id: "actual-cli".into(),
    }
}
#[test]
#[ignore = "Central pinned Jobs CLI allocation and retained batch evidence; no GUI/inference/network"]
fn configured_batches_cli_and_typed_client_share_capture_cas_pages_and_owned_shutdown()
-> Result<(), ScenarioFailure> {
    kinekino_platform::install();
    let mut fixture = Fixture::named("job-batches");
    fixture.keep_evidence();
    let original = Deadline::new(
        Instant::now(),
        Duration::from_secs(100),
        Duration::from_secs(10),
    )
    .unwrap();
    let args: Vec<OsString> = vec![
        "--serve-jobs".into(),
        "--job-root".into(),
        fixture.root.join("store").into_os_string(),
    ];
    let cutoff = phase_deadline(original, Duration::from_secs(12)).unwrap();
    let (host, endpoint) = OwnedHost::launch(fixture.cli.clone(), args, &fixture.root, original)
        .and_then(|host| {
            host.ready(cutoff, &fixture.root.join("logs/ready.stdout"), |bytes| {
                let ready: Value = serde_json::from_slice(bytes)?;
                if ready["event"] != "job_host_ready" {
                    return Err(io::Error::other("Wrong batch host readiness"));
                }
                ready["endpoint"]
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| io::Error::other("Missing batch endpoint"))
            })
        })
        .map_err(ScenarioFailure::host)?;
    let action = close_request(catch_unwind(AssertUnwindSafe(
        || -> Result<(), Box<CliFailure>> {
            let call = |label: &str, request: &kinekino_player::application::api::Request, exit| {
                fixture.try_call_until(
                    label,
                    request,
                    &endpoint,
                    original,
                    Duration::from_secs(10),
                    exit,
                )
            };
            let cap = call("capabilities", &request("capabilities", json!({})), 0)?;
            assert_eq!(cap["data"]["commands"].as_array().unwrap().len(), 119);
            assert_eq!(cap["data"]["job_batches"]["backend_available"], true);
            for (index, command) in ["create", "inspect", "submit", "refresh", "retry_failed"]
                .iter()
                .enumerate()
            {
                let schema = call(
                    &format!("schema-{index}"),
                    &request(
                        "schema",
                        json!({"command":format!("job_batches.{command}")}),
                    ),
                    0,
                )?;
                assert_eq!(schema["data"]["status"], "supported");
            }
            let mut items = vec![];
            for item in ["first", "second"] {
                let source = fixture.root.join(format!("{item}.srt"));
                fs::write(
                    &source,
                    format!("1\n00:00:01,000 --> 00:00:02,000\nCLI {item} fixture\n\n"),
                )
                .unwrap();
                items.push(json!({"item_id":item,"request":{"operation":"subtitles.convert","parameters":{"input_path":source,"output_path":fixture.root.join(format!("{item}.vtt")),"input_format":"srt","output_format":"vtt"}}}));
            }
            let create = request(
                "job_batches.create",
                json!({"batch_id":"actual-cli","items":items}),
            );
            let mut injected = create.clone();
            injected.payload["root"] = json!(fixture.root.join("untrusted"));
            let rejected = call("root-rejected", &injected, 2)?;
            assert_eq!(rejected["error"]["code"], "invalid_argument");
            let missing = call(
                "not-created",
                &request("job_batches.inspect", json!({"batch_id":"actual-cli"})),
                4,
            )?;
            assert_eq!(missing["error"]["code"], "not_found");
            let created = call("create", &create, 0)?;
            assert_eq!(created["data"]["revision"], "1");
            assert_eq!(created["data"]["states"]["waiting"], 2);
            assert!(!fixture.root.join("first.vtt").exists());
            let duplicate = call("duplicate", &create, 5)?;
            assert_eq!(duplicate["error"]["code"], "revision_conflict");
            let client = transport_at(
                &endpoint,
                phase_deadline(original, Duration::from_secs(10)).unwrap(),
            )
            .unwrap();
            let first = client
                .inspect_job_batch(
                    context(1),
                    InspectBatch {
                        limit: 1,
                        ..inspect()
                    },
                )
                .unwrap();
            assert_eq!(first.entries.len(), 1);
            assert_eq!(first.next_offset, Some(1));
            assert!(
                first.entries[0]
                    .input
                    .request
                    .expected_source_sha256
                    .is_some()
            );
            let mut draft = first.entries[0].input.request.clone();
            draft.parameters["output_path"] = json!(fixture.root.join("typed-draft.vtt"));
            let created_draft = client
                .create_job_batch(
                    RequestContext::default(),
                    kinekino_player::application::job_batches::service::CreateBatch {
                        batch_id: "typed-draft".into(),
                        items: vec![kinekino_player::application::job_batches::BatchJobInput {
                            item_id: "draft".into(),
                            request: draft,
                        }],
                    },
                )
                .unwrap();
            assert_eq!(created_draft.states.get("waiting"), Some(&1));
            assert!(!fixture.root.join("typed-draft.vtt").exists());
            let second = client
                .inspect_job_batch(
                    context(1),
                    InspectBatch {
                        offset: 1,
                        limit: 1,
                        ..inspect()
                    },
                )
                .unwrap();
            assert_eq!(second.entries[0].input.item_id, "second");
            assert_eq!(second.next_offset, None);
            let submitted = client.submit_job_batch(context(1), identity()).unwrap();
            let mut stale = request("job_batches.refresh", json!({"batch_id":"actual-cli"}));
            stale.context.expected_revision = Some("1".into());
            assert_eq!(
                call("stale-cas", &stale, 5)?["error"]["code"],
                "revision_conflict"
            );
            let missing_cas = request("job_batches.submit", json!({"batch_id":"actual-cli"}));
            assert_eq!(
                call("missing-cas", &missing_cas, 2)?["error"]["code"],
                "invalid_argument"
            );
            let mut current = submitted;
            loop {
                let client = transport_at(
                    &endpoint,
                    phase_deadline(original, Duration::from_secs(10)).unwrap(),
                )
                .unwrap();
                current = client
                    .refresh_job_batch(context(current.revision.0), identity())
                    .unwrap();
                if current.states.get("observed_completed") == Some(&2) {
                    break;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            let client = transport_at(
                &endpoint,
                phase_deadline(original, Duration::from_secs(10)).unwrap(),
            )
            .unwrap();
            let typed = client
                .inspect_job_batch(context(current.revision.0), inspect())
                .unwrap();
            let mut cli_inspect = request(
                "job_batches.inspect",
                json!({"batch_id":"actual-cli","limit":8}),
            );
            cli_inspect.context.expected_revision = Some(current.revision.0.to_string());
            let cli = call("inspect-terminal", &cli_inspect, 0)?;
            assert_eq!(cli["data"], serde_json::to_value(&typed).unwrap());
            for entry in &typed.entries {
                let job = entry.progress.job().unwrap();
                assert_eq!(job.attempt.0, 1);
                let bytes = fs::read(&job.output.as_ref().unwrap().path).unwrap();
                assert!(bytes.starts_with(b"WEBVTT"));
            }
            let no_retry = client
                .retry_failed_job_batch(context(current.revision.0), identity())
                .unwrap();
            assert_eq!(
                serde_json::to_value(&no_retry).unwrap(),
                serde_json::to_value(&current).unwrap()
            );
            let mut replay = request("job_batches.submit", json!({"batch_id":"actual-cli"}));
            replay.context.expected_revision = Some(current.revision.0.to_string());
            assert_eq!(
                call("submit-preserves-complete", &replay, 0)?["data"],
                serde_json::to_value(&current).unwrap()
            );
            let listed = call(
                "jobs-still-two",
                &request("jobs.list", json!({"limit":10})),
                0,
            )?;
            assert_eq!(listed["data"]["jobs"].as_array().unwrap().len(), 2);
            fs::write(
                fixture.root.join("typed-terminal.json"),
                serde_json::to_vec_pretty(&typed).unwrap(),
            )
            .unwrap();
            Ok(())
        },
    )));
    let stopped = host
        .finish(
            Duration::from_secs(15),
            &fixture.root.join("logs/stop.stdout"),
            &fixture.root.join("logs/host.stderr"),
        )
        .and_then(|terminal| {
            let check = (|| -> io::Result<()> {
                let value: Value = serde_json::from_slice(&terminal.tail)?;
                if terminal.receipt.child_exit != Some(0)
                    || terminal.receipt.forced
                    || terminal.receipt.family != FamilyObservation::Empty
                    || !terminal.receipt.stdin_eof_requested
                    || value["event"] != "job_host_stopped"
                    || value["receipt"]["completed_total"] != 2
                    || value["receipt"]["cancelled_total"] != 0
                    || value["receipt"]["interrupted_total"] != 0
                    || value["local_host_shutdown"]["sdk"] != "joined"
                    || value["local_host_shutdown"]["pipe"] != "joined"
                    || value["local_host_shutdown"]["dispatch_poisoned"] != false
                {
                    return Err(io::Error::other("Original batch host shutdown differs"));
                }
                Ok(())
            })();
            check.map_err(|error| {
                CliFailure::terminal(
                    "batch host EOF",
                    error,
                    &fixture.root,
                    terminal.receipt.clone(),
                )
            })?;
            Ok(terminal.receipt)
        });
    ScenarioFailure::conclude(action, stopped, None)
}

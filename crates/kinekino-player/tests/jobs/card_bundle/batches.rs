//! Real card admission conflict; same-key reconciliation preserves completed output.
use super::*;
use kinekino_player::application::api::JobBatchClient;
use kinekino_player::application::job_batches::{
    BatchProgress,
    service::{BatchIdentity, InspectBatch},
};
fn context(revision: u64) -> RequestContext {
    RequestContext {
        expected_revision: Some(revision.to_string()),
        ..RequestContext::default()
    }
}
fn identity() -> BatchIdentity {
    BatchIdentity {
        batch_id: "card-batch".into(),
    }
}
fn inspect() -> InspectBatch {
    InspectBatch {
        batch_id: "card-batch".into(),
        offset: 0,
        limit: 8,
    }
}
#[test]
#[ignore = "Central pinned card CLI/native rendering; retains real admission conflict and original output"]
fn card_batch_cli_partial_admission_reconciles_same_key_and_preserves_completed_bundle()
-> Result<(), ScenarioFailure> {
    kinekino_platform::install();
    let mut fixture = Fixture::named("card-batches");
    fixture.keep_evidence();
    let (projects, first_input) = setup(&fixture);
    let mut second_input = first_input.clone();
    second_input.output_directory = fixture.root.join("second-output").to_str().unwrap().into();
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
        "--card-project-root".into(),
        projects.into_os_string(),
    ];
    let cutoff = phase_deadline(original, Duration::from_secs(12)).unwrap();
    let (host, endpoint) = OwnedHost::launch(fixture.cli.clone(), args, &fixture.root, original)
        .and_then(|host| {
            host.ready(cutoff, &fixture.root.join("logs/ready.stdout"), |bytes| {
                let ready: Value = serde_json::from_slice(bytes)?;
                if ready["event"] != "job_host_ready" || ready["card_bundle_configured"] != true {
                    return Err(io::Error::other("Card batch host differs"));
                }
                ready["endpoint"]
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| io::Error::other("Missing card batch endpoint"))
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
            let capability = call("capabilities", &request("capabilities", json!({})), 0)?;
            assert_eq!(capability["data"]["job_batches"]["backend_available"], true);
            assert_eq!(
                capability["data"]["commands"].as_array().unwrap().len(),
                119
            );
            let create = request(
                "job_batches.create",
                json!({"batch_id":"card-batch","items":[
                    {"item_id":"first","request":{"operation":"production.export_cards","parameters":first_input}},
                    {"item_id":"second","request":{"operation":"production.export_cards","parameters":second_input}}
                ]}),
            );
            let created = call("create", &create, 0)?;
            assert_eq!(created["data"]["revision"], "1");
            assert!(!std::path::Path::new(&first_input.output_directory).exists());
            // A real pre-existing object appears after capture, before second admission.
            fs::write(
                &second_input.output_directory,
                b"original conflicting fixture; preserve me",
            )
            .unwrap();
            let mut submit = request("job_batches.submit", json!({"batch_id":"card-batch"}));
            submit.context.expected_revision = Some("1".into());
            let submitted = call("submit-partial", &submit, 0)?;
            let mut revision = submitted["data"]["revision"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap();
            let partial = loop {
                let client = transport_at(
                    &endpoint,
                    phase_deadline(original, Duration::from_secs(10)).unwrap(),
                )
                .unwrap();
                let receipt = client
                    .refresh_job_batch(context(revision), identity())
                    .unwrap();
                revision = receipt.revision.0;
                if receipt.states.get("observed_completed") == Some(&1)
                    && receipt.states.get("submit_error") == Some(&1)
                {
                    break client
                        .inspect_job_batch(context(revision), inspect())
                        .unwrap();
                }
                std::thread::sleep(Duration::from_millis(10));
            };
            fs::write(
                fixture.root.join("partial.json"),
                serde_json::to_vec_pretty(&partial).unwrap(),
            )
            .unwrap();
            let first = partial.entries[0].progress.job().unwrap();
            assert_eq!(first.state, JobState::Completed);
            let BatchProgress::SubmitError { error } = &partial.entries[1].progress else {
                panic!("Missing original admission error")
            };
            assert_eq!(
                error.code,
                kinekino_player::application::api::ErrorCode::RevisionConflict
            );
            assert!(partial.entries[1].progress.job().is_none());
            let first_record = serde_json::to_vec(first).unwrap();
            let first_output = first.output.as_ref().unwrap();
            let first_manifest = fs::read(&first_output.path).unwrap();
            manifest(
                &first_output.path,
                &first_output.sha256,
                first_output.bytes.0,
            );
            let key_pin = partial.entries[1]
                .input
                .request
                .expected_source_sha256
                .clone()
                .unwrap();
            let client = transport_at(
                &endpoint,
                phase_deadline(original, Duration::from_secs(10)).unwrap(),
            )
            .unwrap();
            // SubmitError may mean unknown admission; retry_failed must not invent a Job.
            let unchanged = client
                .retry_failed_job_batch(context(revision), identity())
                .unwrap();
            assert_eq!(unchanged.revision.0, revision);
            let preserved = fixture.root.join("conflicting-output-preserved");
            fs::rename(&second_input.output_directory, &preserved).unwrap();
            assert_eq!(
                fs::read(&preserved).unwrap(),
                b"original conflicting fixture; preserve me"
            );
            submit.context.expected_revision = Some(revision.to_string());
            let reconciled = call("submit-reconcile-same-key", &submit, 0)?;
            revision = reconciled["data"]["revision"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap();
            let complete = loop {
                let client = transport_at(
                    &endpoint,
                    phase_deadline(original, Duration::from_secs(10)).unwrap(),
                )
                .unwrap();
                let receipt = client
                    .refresh_job_batch(context(revision), identity())
                    .unwrap();
                revision = receipt.revision.0;
                if receipt.states.get("observed_completed") == Some(&2) {
                    break client
                        .inspect_job_batch(context(revision), inspect())
                        .unwrap();
                }
                std::thread::sleep(Duration::from_millis(10));
            };
            assert_eq!(
                serde_json::to_vec(complete.entries[0].progress.job().unwrap()).unwrap(),
                first_record
            );
            assert_eq!(fs::read(&first_output.path).unwrap(), first_manifest);
            manifest(
                &first_output.path,
                &first_output.sha256,
                first_output.bytes.0,
            );
            let second = complete.entries[1].progress.job().unwrap();
            assert_eq!(second.attempt.0, 1); // No second Job was admitted during the first conflict.
            assert_ne!(second.job_id, first.job_id);
            assert_eq!(
                complete.entries[1]
                    .input
                    .request
                    .expected_source_sha256
                    .as_deref(),
                Some(key_pin.as_str())
            );
            assert_eq!(second.input.source.sha256, key_pin);
            let output = second.output.as_ref().unwrap();
            manifest(&output.path, &output.sha256, output.bytes.0);
            let mut query = request(
                "job_batches.inspect",
                json!({"batch_id":"card-batch","limit":8}),
            );
            query.context.expected_revision = Some(revision.to_string());
            assert_eq!(
                call("inspect-final", &query, 0)?["data"],
                serde_json::to_value(&complete).unwrap()
            );
            submit.context.expected_revision = Some(revision.to_string());
            assert_eq!(
                call("submit-complete-preserved", &submit, 0)?["data"]["revision"],
                revision.to_string()
            );
            assert_eq!(
                call(
                    "jobs-still-two",
                    &request("jobs.list", json!({"limit":10})),
                    0
                )?["data"]["jobs"]
                    .as_array()
                    .unwrap()
                    .len(),
                2
            );
            fs::write(
                fixture.root.join("completed.json"),
                serde_json::to_vec_pretty(&complete).unwrap(),
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
            let checked = (|| -> io::Result<()> {
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
                    return Err(io::Error::other(
                        "Original card batch host shutdown differs",
                    ));
                }
                Ok(())
            })();
            checked.map_err(|error| {
                CliFailure::terminal(
                    "card batch EOF",
                    error,
                    &fixture.root,
                    terminal.receipt.clone(),
                )
            })?;
            Ok(terminal.receipt)
        });
    ScenarioFailure::conclude(action, stopped, None)
}

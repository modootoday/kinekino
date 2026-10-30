//! Normal product CLI over the existing local host; fixture source admission only.
use super::*;
use kinekino_player::{
    application::api::{PlayerSdk, Request},
    infrastructure::{
        local_host::{LocalHost, TransportOptions},
        subtitle_files::SubtitleFiles,
    },
};
use serde_json::{Value, json};
use std::os::windows::fs::OpenOptionsExt;
#[path = "../../../../../../../tests/support/owned_process_evidence.rs"]
#[allow(
    dead_code,
    reason = "Shared driver exposes standalone failure consumers outside this fixture"
)]
pub mod owned_process_evidence;
fn hash(bytes: &[u8]) -> String {
    kinekino_player::infrastructure::content_digest::sha256(
        &mut std::io::Cursor::new(bytes),
        bytes.len() as u64,
    )
    .unwrap()
    .1
}
fn rpc(
    cli_endpoint: (&Path, &str),
    root: &Path,
    number: &mut usize,
    original: Deadline,
    command: &str,
    payload: Value,
    key: Option<&str>,
) -> Value {
    let (cli, endpoint) = cli_endpoint;
    *number += 1;
    let evidence = root.join(format!("rpc-{number}"));
    fs::create_dir(&evidence).unwrap();
    let mut request = Request::new(command);
    request.payload = payload;
    request.context.request_id = format!("source-sdk-cli-{number}");
    request.context.idempotency_key = key.map(str::to_owned);
    let input = serde_json::to_vec(&request).unwrap();
    fs::write(evidence.join("request.json"), &input).unwrap();
    let output = match owned_process_evidence::run_process_until(
        cli.into(),
        &evidence,
        &input,
        Some(endpoint),
        original,
        Duration::from_secs(10),
        true,
    ) {
        Ok(output) => output,
        Err(failure) => {
            let failure = failure.closeout();
            let observation = failure.observation();
            Box::leak(failure);
            panic!("Original CLI failure retained: {observation}");
        }
    };
    assert_eq!(
        output.receipt.family,
        kinekino_process::domain::observation::FamilyObservation::Empty
    );
    assert!(!output.receipt.forced);
    assert_eq!(output.stdout_evidence["complete"], true);
    assert_eq!(output.stderr_evidence["complete"], true);
    if let Err(failure) = output.check_deadline(&evidence) {
        let observation = failure.observation();
        Box::leak(failure);
        panic!("Original CLI observation retained: {observation}");
    }
    fs::write(
        evidence.join("elapsed-ms.txt"),
        output.elapsed_ms.to_string(),
    )
    .unwrap();
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["request_id"], request.context.request_id);
    let exit = if response["ok"] == true {
        0
    } else {
        serde_json::from_value::<ApiError>(response["error"].clone())
            .unwrap()
            .exit_code()
    };
    assert_eq!(output.receipt.child_exit, Some(u32::from(exit)));
    fs::write(
        evidence.join("response.json"),
        serde_json::to_vec_pretty(&response).unwrap(),
    )
    .unwrap();
    response
}
fn exercise(expected_exit: u32, admitted: bool, omit_body_end: bool, cancel: bool) {
    let cli = PathBuf::from(std::env::var_os("SOURCE_SDK_CLI_BIN").expect("fixed normal CLI"));
    assert!(cli.is_absolute());
    let mut cli_guard = fs::OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(&cli)
        .unwrap();
    let expected_size: u64 = std::env::var("SOURCE_SDK_CLI_BYTES")
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(cli_guard.metadata().unwrap().len(), expected_size);
    assert_eq!(
        kinekino_player::infrastructure::content_digest::sha256(&mut cli_guard, expected_size)
            .unwrap()
            .1,
        std::env::var("SOURCE_SDK_CLI_SHA256").unwrap()
    );
    let worker = PathBuf::from(std::env::var_os("SOURCE_SDK_FIXTURE_BIN").unwrap());
    let root = std::env::temp_dir().join(format!(
        "sdk-source-cli-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    fs::create_dir(root.join("jobs")).unwrap();
    fs::create_dir(root.join("output")).unwrap();
    eprintln!("sdk_source_evidence={}", root.display());
    let consumed = Arc::new(AtomicUsize::new(0));
    let transport = Arc::new(AtomicUsize::new(0));
    let drops = Arc::new(AtomicUsize::new(0));
    let gate = cancel.then(|| {
        Arc::new(super::cancel::Gate {
            origin_released: std::sync::atomic::AtomicBool::new(false),
            started: std::sync::atomic::AtomicBool::new(false),
            cancelled: std::sync::atomic::AtomicBool::new(false),
            release: std::sync::atomic::AtomicBool::new(false),
        })
    });
    let preparation: Arc<dyn DownloadPreparation> = Arc::new(Preparation {
        replay_failure: None,
        request_failure: None,
        omit_body_end,
        source_exit: expected_exit,
        gate: gate.clone(),
        root: root.clone(),
        worker,
        consumed: consumed.clone(),
        transport: transport.clone(),
        drops: drops.clone(),
        binding: Arc::new(Mutex::new(None)),
    });
    let executor =
        download_registry(Vec::new(), if admitted { Some(preparation) } else { None }).unwrap();
    let host = crate::infrastructure::jobs::start_retaining_host_with_executor(
        &root.join("jobs"),
        executor,
    )
    .unwrap();
    let start = Instant::now();
    let end = start + Duration::from_secs(30);
    let original = Deadline::new(start, Duration::from_secs(30), Duration::from_secs(5)).unwrap();
    let local = match LocalHost::start(
        Arc::new(PlayerSdk::new(SubtitleFiles).with_jobs(host.client())),
        TransportOptions::default(),
    ) {
        Ok(local) => Some(local),
        Err(error) => {
            fs::write(root.join("local-start-error.txt"), format!("{error:?}")).unwrap();
            None
        }
    };
    let result = catch_unwind(AssertUnwindSafe(|| {
        let local = local.as_ref().expect("original local host start");
        let endpoint = local.endpoint();
        let mut number = 0;
        let capabilities = rpc(
            (&cli, endpoint),
            &root,
            &mut number,
            original,
            "capabilities",
            json!({}),
            None,
        );
        assert_eq!(capabilities["ok"], true);
        assert_eq!(
            capabilities["data"]["jobs"]["operations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|op| op == DOWNLOAD_OPERATION),
            admitted
        );
        let schema = rpc(
            (&cli, endpoint),
            &root,
            &mut number,
            original,
            "schema",
            json!({"command":"jobs.submit"}),
            None,
        );
        assert_eq!(schema["ok"], true);
        let submitted = rpc(
            (&cli, endpoint),
            &root,
            &mut number,
            original,
            "jobs.submit",
            json!({"operation":DOWNLOAD_OPERATION,"parameters":{"source":{"selection_id":"fixture-selection","descriptor_sha256":"a".repeat(64)},"destination_id":"fixture-output","maximum_bytes":"1000"},"expected_source_sha256":null,"execution_timeout_ms":"10000"}),
            Some("source-sdk-cli-submit"),
        );
        if !admitted {
            assert_eq!(submitted["ok"], false);
            assert_eq!(submitted["error"]["code"], "unsupported");
            fs::write(root.join("rpc-count.txt"), number.to_string()).unwrap();
            return;
        }
        assert_eq!(submitted["ok"], true, "{submitted}");
        let id: crate::application::jobs::JobId =
            serde_json::from_value(submitted["data"]["job"]["job_id"].clone()).unwrap();
        if let Some(gate) = &gate {
            while !gate.started.load(Ordering::SeqCst) {
                assert!(Instant::now() < end);
                std::thread::sleep(Duration::from_millis(2));
            }
            let cancelled = rpc(
                (&cli, endpoint),
                &root,
                &mut number,
                original,
                "jobs.cancel",
                json!({"job_id":id,"expected_attempt":"1"}),
                None,
            );
            assert_eq!(cancelled["ok"], true, "{cancelled}");
            let source_close_end = Instant::now() + Duration::from_secs(2);
            while !gate.cancelled.load(Ordering::SeqCst) || transport.load(Ordering::SeqCst) != 1 {
                assert!(
                    Instant::now() < source_close_end,
                    "Original CLI cancellation did not close source while probe held"
                );
                std::thread::sleep(Duration::from_millis(2));
            }
            let pending = rpc(
                (&cli, endpoint),
                &root,
                &mut number,
                original,
                "jobs.get",
                json!({"job_id":id}),
                None,
            );
            assert_eq!(pending["data"]["state"], "cancelling");
            assert!(!root.join("output/fixture.bin").exists());
            let parts: Vec<_> = fs::read_dir(root.join("output"))
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .collect();
            assert_eq!(parts.len(), 1);
            assert_eq!(fs::metadata(&parts[0]).unwrap().len(), 16);
            assert_eq!(
                fs::OpenOptions::new()
                    .write(true)
                    .share_mode(7)
                    .open(&parts[0])
                    .unwrap_err()
                    .raw_os_error(),
                Some(32)
            );
            assert_eq!(drops.load(Ordering::SeqCst), 0);
            gate.release.store(true, Ordering::SeqCst);
        }
        loop {
            let job = host.client().get(GetJob { job_id: id.clone() }).unwrap();
            if job.state.terminal() {
                break;
            }
            assert!(Instant::now() < end);
            std::thread::sleep(Duration::from_millis(2));
        }
        let response = rpc(
            (&cli, endpoint),
            &root,
            &mut number,
            original,
            "jobs.get",
            json!({"job_id":id}),
            None,
        );
        assert_eq!(response["ok"], true);
        let job: crate::application::jobs::JobRecord =
            serde_json::from_value(response["data"].clone()).unwrap();
        if cancel {
            assert_eq!(job.state, JobState::Cancelled);
            assert!(job.cancel_requested);
            assert!(!job.publication_started);
            assert!(job.output.is_none());
            assert!(job.failure.is_none());
        } else if expected_exit == 0 && !omit_body_end {
            assert_eq!(job.state, JobState::Completed);
            assert!(job.publication_started);
            assert_eq!(
                fs::read(root.join("output/fixture.bin")).unwrap(),
                b"offline-sdk-body"
            );
        } else {
            assert_eq!(job.state, JobState::Failed);
            assert_eq!(job.failure.as_ref().unwrap().code, ErrorCode::EngineFailed);
            assert!(!job.publication_started);
            assert!(job.output.is_none());
        }
        let events = rpc(
            (&cli, endpoint),
            &root,
            &mut number,
            original,
            "jobs.watch",
            json!({"after_sequence":"0","limit":128}),
            None,
        );
        assert_eq!(events["ok"], true);
        if cancel {
            let rows = events["data"]["events"].as_array().unwrap();
            assert!(rows.iter().any(|row| row["job"]["state"] == "cancelling"));
            assert!(rows.iter().any(|row| row["job"]["state"] == "cancelled"));
            assert!(!rows.iter().any(|row| row["job"]["state"] == "completed"));
        }
        fs::write(root.join("rpc-count.txt"), number.to_string()).unwrap();
    }));
    if let Some(gate) = gate {
        gate.release.store(true, Ordering::SeqCst);
    }
    if let Some(local) = local {
        let mut pending = local.begin_shutdown();
        loop {
            let status = pending.poll();
            if status.is_quiescent() {
                fs::write(root.join("local-close.txt"), format!("{status:?}")).unwrap();
                break;
            }
            if Instant::now() >= end {
                eprintln!("sdk-source-cli:original-local-host-incomplete");
                std::process::exit(124);
            }
            std::thread::sleep(Duration::from_millis(2));
        }
    }
    let mut pending = host.begin_shutdown(end);
    loop {
        match pending.poll(end) {
            WorkerShutdownOutcome::Complete(complete) => {
                assert!(complete.errors.first().is_none());
                assert!(complete.receipt.is_some());
                break;
            }
            WorkerShutdownOutcome::Pending(owner) => {
                pending = owner;
                if Instant::now() >= end {
                    eprintln!("sdk-source-cli:original-jobs-host-incomplete");
                    std::process::exit(124);
                }
                std::thread::sleep(Duration::from_millis(2));
            }
        }
    }
    if let Err(panic) = result {
        std::panic::resume_unwind(panic);
    }
    assert_eq!(consumed.load(Ordering::SeqCst), usize::from(admitted));
    assert_eq!(transport.load(Ordering::SeqCst), usize::from(admitted));
    assert_eq!(drops.load(Ordering::SeqCst), usize::from(admitted));
    if !admitted || expected_exit != 0 || omit_body_end || cancel {
        assert!(fs::read_dir(root.join("output")).unwrap().next().is_none());
    }
}
#[test]
#[ignore = "real pinned normal CLI/local endpoint/source child; centrally assigned"]
fn normal_cli_uses_shared_source_and_original_publication() {
    exercise(0, true, false, false);
}
#[test]
#[ignore = "real pinned CLI/high-bit source failure; centrally assigned"]
fn normal_cli_preserves_source_failure_without_publication() {
    exercise(0x8000_0001, true, false, false);
}
#[test]
#[ignore = "real pinned CLI; absent source factory must remain unsupported"]
fn absent_factory_stays_unsupported_through_normal_cli() {
    exercise(0, false, false, false);
}
#[test]
#[ignore = "real pinned CLI/source IPC EOF without BodyEnd despite matching length"]
fn normal_cli_rejects_matching_length_pipe_eof_without_body_end() {
    exercise(0, true, true, false);
}
#[test]
#[ignore = "real CLI cancellation, original source child and held sealed output lease"]
fn normal_cli_cancel_retains_partial_until_probe_close_and_records_terminal_cancelled() {
    exercise(0, true, false, true);
}

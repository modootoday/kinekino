#![cfg(windows)]
#[path = "support/local_host_cli.rs"]
pub mod cli;
#[path = "support/cli_exe.rs"]
mod cli_exe;
#[path = "sprite_capture_cli/custody.rs"]
mod custody;
use kinekino_player::{
    application::{
        api::{PlayerSdk, Request},
        playback_control::{ControlReceipt, PlaybackControlPort},
        review_playback::{
            ReviewChildOwnership, ReviewRuntimeBackend, ReviewRuntimeProfile, inspect_review_input,
        },
        review_source::{ReviewCaseEvidence, ReviewCases, ReviewPackageMedia, timeline_evidence},
    },
    infrastructure::{
        local_host::{LocalHost, TransportOptions},
        model_review::ReviewFiles,
        mpv::{
            MpvControlHandle, ReviewPlaybackWorker, ReviewWorkerControl, ReviewWorkerLimits,
            TrustedMpvExecutable,
        },
        review_content_classification::IsoBmffReviewClassifier,
        review_runtime::ReviewRuntimePool,
        timeline_evidence::{ReviewTimelineEvidence, TrustedTimelineAdmission},
    },
};
use kinekino_process::infrastructure::family::JobBudget;
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};
enum ActualControl {
    Raw(MpvControlHandle),
    Worker(ReviewWorkerControl),
}
impl PlaybackControlPort for ActualControl {
    fn submit(
        &self,
        intent: kinekino_player::domain::playback_control::PlaybackControl,
    ) -> Result<ControlReceipt, kinekino_player::application::api::ApiError> {
        match self {
            Self::Raw(port) => port.submit(intent),
            Self::Worker(port) => port.submit(intent),
        }
    }
}

#[test]
#[ignore = "central frozen-input actual local CPU media + normal CLI seek; no GUI/model/network"]
fn actual_owned_local_review_cli_seek_keeps_source_and_instance_until_physical_close() {
    kinekino_platform::install();
    run(false, false);
}
#[test]
#[ignore = "central frozen-input actual non-Send timeline proof through local worker physical close; no GUI/model/network"]
fn actual_scoped_timeline_proof_survives_cli_seek_and_original_physical_close() {
    kinekino_platform::install();
    run(true, false);
}
#[test]
#[ignore = "central frozen-input actual per-command original worker proof gate + normal CLI seek; no GUI/model/network"]
fn actual_worker_command_gate_checks_source_instance_proof_before_cli_ipc() {
    kinekino_platform::install();
    run(true, true);
}
fn run(retain_timeline: bool, worker_commands: bool) {
    let workspace = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
    let output = std::path::PathBuf::from(
        std::env::var("KINEKINO_REVIEW_OUTPUT_DIR")
            .expect("explicit fresh private output directory required"),
    );
    assert!(output.starts_with(workspace.join("runtime/verification")));
    let source_receipt_dir = std::path::PathBuf::from(
        std::env::var("KINEKINO_REVIEW_SOURCE_RECEIPT_DIR")
            .expect("explicit frozen source receipt directory required"),
    );
    assert!(source_receipt_dir.starts_with(workspace.join("runtime/verification")));
    let source_receipt = custody::Guard::acquire(&source_receipt_dir.join("SOURCE.md"));
    assert_eq!(
        source_receipt.sha256,
        std::env::var("KINEKINO_REVIEW_SOURCE_RECEIPT_SHA256")
            .expect("explicit frozen source receipt SHA required")
            .to_ascii_lowercase()
    );
    let witness = custody::Guard::acquire(&std::env::current_exe().unwrap());
    let cli_witness = custody::Guard::acquire(Path::new(crate::cli_exe::kinekino_cli_exe()));
    let root = workspace.join("runtime/verification/sprite-timeline-actual-r1");
    let nomination = custody::Guard::acquire(&root.join("NOMINATION.json"));
    assert_eq!(
        nomination.sha256,
        "b1a47d17ad348d6ac7c4c5ec9eb246a54f561babc442cf4310274cdb349be201"
    );
    let record: Value =
        serde_json::from_slice(&fs::read(root.join("NOMINATION.json")).unwrap()).unwrap();
    let case: kinekino_player::domain::review_source::ReviewCase =
        serde_json::from_value(record["case"].clone()).unwrap();
    let media: kinekino_player::domain::review_source::MediaReference =
        serde_json::from_value(record["media"].clone()).unwrap();
    let (status_case, status_media) = (case.clone(), media.clone());
    let allocation = workspace.join("runtime/build-cache/tmp").join(format!(
        "review-worker-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&allocation).unwrap();
    fs::create_dir(allocation.join("evidence")).unwrap();
    println!("original review worker evidence {}", allocation.display());
    let tool_path = workspace.join("runtime/toolchains/mpv-dde0671072-review-r1/mpv.exe");
    let tool_before = custody::Guard::acquire(&tool_path);
    let tool_identity = tool_before.evidence();
    assert_eq!(
        tool_before.sha256,
        "f8bf7e8fd8d6696e35441c55091498060e4594f724a132fd6c46579341bc0790"
    );
    drop(tool_before);
    let before = json!({"source_receipt":source_receipt.evidence(),"test_producer":witness.evidence(),"cli_producer":cli_witness.evidence(),"nomination":nomination.evidence(),"case":record["case"],"media":record["media"],"tool":tool_identity,"requested_retained_timeline":retain_timeline,"requested_worker_commands":worker_commands,"media_process_started":false});
    {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(allocation.join("WITNESS.json"))
            .unwrap();
        file.write_all(&serde_json::to_vec_pretty(&before).unwrap())
            .unwrap();
        file.sync_all().unwrap();
    }
    let witness_record = custody::Guard::acquire(&allocation.join("WITNESS.json"));
    let (prepare_root, prepare_allocation) = (root.clone(), allocation.clone());
    let guard_calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let intent_guard_calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let worker_intent_calls = intent_guard_calls.clone();
    let scope_ended = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let (worker_calls, worker_scope_ended) = (guard_calls.clone(), scope_ended.clone());
    let evidence: kinekino_player::domain::review_source::EvidenceReference =
        serde_json::from_value(record["evidence"].clone()).unwrap();
    let worker = ReviewPlaybackWorker::spawn_scoped(
        move |launch| {
            let cases = ReviewCases::new(ReviewFiles::open(&prepare_root)?);
            let medias = ReviewPackageMedia::new(ReviewFiles::open(&prepare_root)?);
            let resolver = if retain_timeline {
                Some(ReviewTimelineEvidence::new(
                    ReviewFiles::open(&prepare_root)?,
                    TrustedTimelineAdmission {
                        case: case.clone(),
                        media: media.clone(),
                        evidence: evidence.clone(),
                        audio_engine_sha256:
                            "71414777922d05f2639a5cf4a0f9a405f169809cf459000a6f976c9add61f060"
                                .into(),
                        video_engine_sha256:
                            "a6618e99bb58869ded3c6f37b53aa1a8d701c3591dbb7b5b317d47369c112be2"
                                .into(),
                        capture_engine_sha256:
                            "09948d4cdd0650da6ff5a87577469f2a218dc2615ae379f8f734d24c49de0f73"
                                .into(),
                        capture_cli_sha256:
                            "8d06f1919d05abedcdaa799c9a7b43fa42cf8f14b5c092fd6eb4d6d058cf99f9"
                                .into(),
                    },
                )?)
            } else {
                None
            };
            let proof = resolver
                .as_ref()
                .map(|resolver| {
                    timeline_evidence::retain(&cases, &medias, resolver, &case, &media, &evidence)
                })
                .transpose()?;
            let reader = ReviewCaseEvidence::new(ReviewFiles::open(&prepare_root)?);
            let input = inspect_review_input(&reader, &case, &media, &IsoBmffReviewClassifier)?;
            let source = ReviewFiles::open(&prepare_root)?.bind_playback_media(input)?;
            let tool = TrustedMpvExecutable::open(
                &tool_path,
                "f8bf7e8fd8d6696e35441c55091498060e4594f724a132fd6c46579341bc0790",
            )?;
            let (mut mask, mut system) = (0usize, 0usize);
            unsafe {
                windows::Win32::System::Threading::GetProcessAffinityMask(
                    windows::Win32::System::Threading::GetCurrentProcess(),
                    &mut mask,
                    &mut system,
                )
                .map_err(|e| {
                    kinekino_player::application::api::ApiError::new(
                        kinekino_player::application::api::ErrorCode::EngineFailed,
                        e.to_string(),
                    )
                })?;
            }
            let budget = JobBudget {
                max_committed_bytes: 1024 * 1024 * 1024,
                affinity_mask: mask.isolate_lowest_one(),
            };
            let profile = ReviewRuntimeProfile {
                backend: ReviewRuntimeBackend::OwnedLocalMpv,
                engine_sha256: tool.sha256().into(),
                max_source_bytes: 128 * 1024 * 1024,
                max_job_committed_bytes: budget.max_committed_bytes as u64,
                cpu_threads: 1,
                startup_timeout_ms: 10000,
                child_ownership: ReviewChildOwnership::OwnedJobUntilPhysicalExit,
            };
            let admitted = source.admit(
                ReviewRuntimePool::open(&prepare_allocation, tool, profile, budget)?
                    .reserve_owned()?,
            )?;
            let mapping = proof
                .as_ref()
                .map(|proof| proof.mapping_record())
                .transpose()?;
            let result = launch.run_with_control_guard(
                admitted,
                &|| {
                    worker_calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    proof.as_ref().map_or(Ok(()), |proof| proof.check_current())
                },
                &|intent| {
                    worker_intent_calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    if let (Some(proof), Some(mapping)) = (&proof, &mapping) {
                        proof.check_record(mapping)?;
                        if let kinekino_player::domain::playback_control::PlaybackControl::Seek {
                            position_us,
                        } = intent
                            && (position_us.0 < mapping.source_start_us.0
                                || position_us.0 > mapping.source_end_us.0)
                        {
                            return Err(kinekino_player::application::api::ApiError::new(
                                kinekino_player::application::api::ErrorCode::NeedsInput,
                                "Seek point outside retained observed coverage",
                            ));
                        }
                    }
                    Ok(())
                },
            );
            drop(proof);
            worker_scope_ended.store(true, std::sync::atomic::Ordering::SeqCst);
            result
        },
        allocation.join("evidence"),
        ReviewWorkerLimits {
            total: Duration::from_secs(35),
            cleanup: Duration::from_secs(2),
            raw_output_bytes: 1024 * 1024,
        },
    )
    .unwrap();
    let mut retained_status_reader = None;
    let mut retained_control_reader = None;
    let result = (|| -> Result<Value, String> {
        let cutoff = Instant::now() + Duration::from_secs(15);
        loop {
            let state = worker.snapshot().map_err(|e| format!("{e:?}"))?;
            if let Some(error) = state.first_error {
                return Err(error);
            }
            if state.playback.connected
                && state.playback.pause_observed
                && state.playback.paused
                && state.playback.duration.is_some_and(|v| v > 600.)
            {
                break;
            }
            if Instant::now() >= cutoff {
                return Err(format!("startup not observed: {state:?}"));
            }
            thread::sleep(Duration::from_millis(20));
        }
        assert!(!scope_ended.load(std::sync::atomic::Ordering::SeqCst));
        let control = worker.control().map_err(|e| format!("{e:?}"))?;
        let instance = control.instance_id();
        let mut wrong_control = Value::Null;
        let control = if worker_commands {
            let reader = worker
                .scoped_control(status_case.clone(), status_media.clone(), instance)
                .map_err(|e| format!("{e:?}"))?;
            let wrong = worker
                .scoped_control(
                    status_case.clone(),
                    status_media.clone(),
                    instance.checked_add(1).ok_or("instance overflow")?,
                )
                .map_err(|e| format!("{e:?}"))?;
            let rejected = PlayerSdk::default()
                .with_playback_control(wrong)
                .execute(Request::new("playback.pause"));
            if rejected.error.as_ref().map(|e| e.code)
                != Some(kinekino_player::application::api::ErrorCode::RevisionConflict)
            {
                return Err(format!("foreign control not denied: {rejected:?}"));
            }
            wrong_control = serde_json::to_value(rejected).unwrap();
            retained_control_reader = Some(reader.clone());
            ActualControl::Worker(reader)
        } else {
            ActualControl::Raw(control)
        };
        let expected_instance = instance.to_string();
        let status_reader = worker
            .status_reader(status_case.clone(), status_media.clone(), instance)
            .map_err(|e| format!("{e:?}"))?;
        retained_status_reader = Some(status_reader.clone());
        let wrong = worker
            .status_reader(
                status_case.clone(),
                status_media.clone(),
                instance.checked_add(1).ok_or("instance overflow")?,
            )
            .map_err(|e| format!("{e:?}"))?;
        let denied = PlayerSdk::default()
            .with_playback_status(wrong)
            .execute(Request::new("playback.status"));
        if denied.error.as_ref().map(|e| e.code)
            != Some(kinekino_player::application::api::ErrorCode::RevisionConflict)
        {
            return Err(format!("foreign engine not denied: {denied:?}"));
        }
        let host = LocalHost::start(
            Arc::new(
                PlayerSdk::default()
                    .with_playback_control(control)
                    .with_playback_status(status_reader),
            ),
            TransportOptions::default(),
        )
        .map_err(|e| format!("{e:?}"))?;
        let mut outside_denied = Value::Null;
        if worker_commands {
            let mut outside = Request::new("playback.seek");
            outside.payload = json!({"position_us":"26000000"});
            let (code, denied) = cli::invoke(host.endpoint(), &outside);
            if code == 0
                || denied["error"]["code"] != "needs_input"
                || denied["error"]["details"]["delivery"] != "not_sent"
            {
                return Err(format!("outside coverage not denied: {denied}"));
            }
            outside_denied = denied;
        }
        let mut observed = Vec::new();
        for position in [17_000_000u64, 25_500_000] {
            let mut request = Request::new("playback.seek");
            request.payload = json!({"position_us":position.to_string()});
            let (code, response) = cli::invoke(host.endpoint(), &request);
            if code != 0
                || response["ok"] != true
                || response["data"]["engine_instance_id"].as_str()
                    != Some(expected_instance.as_str())
            {
                return Err(format!("CLI receipt: {response}"));
            }
            let cutoff = Instant::now() + Duration::from_secs(5);
            loop {
                let state = worker.snapshot().map_err(|e| format!("{e:?}"))?;
                if state.engine_instance_id != Some(instance) {
                    return Err("engine instance changed".into());
                }
                if let Some(error) = state.first_error {
                    return Err(error);
                }
                if state
                    .playback
                    .time
                    .is_some_and(|v| (v - position as f64 / 1_000_000.).abs() <= 0.1)
                {
                    let (status_code, status) =
                        cli::invoke(host.endpoint(), &Request::new("playback.status"));
                    let position_read = status["data"]["position_us"]
                        .as_str()
                        .and_then(|v| v.parse::<u64>().ok());
                    if status_code != 0
                        || status["ok"] != true
                        || !position_read.is_some_and(|v| v.abs_diff(position) <= 100_000)
                    {
                        return Err(format!("source-bound CLI status differs: {status}"));
                    }
                    observed.push(json!({"requested_us":position.to_string(),"observed_seconds":state.playback.time,"engine_instance_id":instance.to_string(),"cli":response,"cli_status":status}));
                    break;
                }
                if Instant::now() >= cutoff {
                    return Err(format!("seek landing not observed: {state:?}"));
                }
                thread::sleep(Duration::from_millis(20));
            }
        }
        Ok(
            json!({"observations":observed,"foreign_instance_denied":serde_json::to_value(denied).unwrap(),"foreign_control_denied":wrong_control,"outside_coverage_denied":outside_denied,"claim":"normal CLI enqueue/status plus same-instance clock within 100ms; engine time is not frame-exact source PTS proof"}),
        )
    })();
    {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(allocation.join("OBSERVATIONS.json"))
            .unwrap();
        let partial = json!({"result":&result,"before_close":worker.snapshot().unwrap()});
        file.write_all(&serde_json::to_vec_pretty(&partial).unwrap())
            .unwrap();
        file.sync_all().unwrap();
    }
    let observations_record = custody::Guard::acquire(&allocation.join("OBSERVATIONS.json"));
    worker.close();
    let cutoff = Instant::now() + Duration::from_secs(10);
    while !worker.is_finished() && Instant::now() < cutoff {
        thread::sleep(Duration::from_millis(20));
    }
    let closed = worker.snapshot().unwrap();
    println!("worker terminal observation {closed:?}");
    let result = result
        .unwrap_or_else(|error| panic!("actual review worker failed: {error}; cleanup={closed:?}"));
    assert!(
        worker.is_finished() && closed.physical_closed && closed.family_empty && closed.ipc_exited,
        "{closed:?}"
    );
    assert_eq!(closed.child_exit, Some(0), "{closed:?}");
    assert!(!closed.forced, "{closed:?}");
    assert!(closed.first_error.is_none(), "{closed:?}");
    let mut closed_control = Value::Null;
    if let Some(reader) = retained_control_reader {
        let denied = reader
            .submit(kinekino_player::domain::playback_control::PlaybackControl::Pause)
            .unwrap_err();
        assert_eq!(
            denied.code,
            kinekino_player::application::api::ErrorCode::NotFound
        );
        assert_eq!(denied.details["delivery"], "not_sent");
        closed_control = serde_json::to_value(denied).unwrap();
        assert!(intent_guard_calls.load(std::sync::atomic::Ordering::SeqCst) >= 3);
    }
    assert!(scope_ended.load(std::sync::atomic::Ordering::SeqCst));
    assert!(guard_calls.load(std::sync::atomic::Ordering::SeqCst) > 2);
    let closed_host = LocalHost::start(
        Arc::new(PlayerSdk::default().with_playback_status(retained_status_reader.unwrap())),
        TransportOptions::default(),
    )
    .unwrap();
    let (closed_code, closed_status) =
        cli::invoke(closed_host.endpoint(), &Request::new("playback.status"));
    assert_ne!(closed_code, 0);
    assert_eq!(closed_status["error"]["code"], "not_found");
    let tool_after = custody::Guard::acquire(
        &workspace.join("runtime/toolchains/mpv-dde0671072-review-r1/mpv.exe"),
    );
    assert_eq!(tool_after.evidence(), tool_identity);
    for guard in [
        &source_receipt,
        &witness,
        &cli_witness,
        &nomination,
        &witness_record,
        &observations_record,
    ] {
        guard.assert_unchanged();
    }
    fs::create_dir(&output).unwrap();
    let receipt = json!({"result":result,"closed_cli_status":closed_status,"closed_control":closed_control,"source_receipt":source_receipt.evidence(),"source_case":record["case"],"source_media":record["media"],"tool":tool_identity,"witness":witness.evidence(),"cli":cli_witness.evidence(),"nomination":nomination.evidence(),"original_process_pid":closed.pid,"child_exit":closed.child_exit,"physical_closed":closed.physical_closed,"family_empty":closed.family_empty,"ipc_exited":closed.ipc_exited,"forced":closed.forced,"evidence_root":allocation,"retained_timeline_custody":retain_timeline,"worker_guarded_control":worker_commands,"intent_guard_calls":intent_guard_calls.load(std::sync::atomic::Ordering::SeqCst),"custody_check_calls":guard_calls.load(std::sync::atomic::Ordering::SeqCst),"custody_scope_ended_after_run":scope_ended.load(std::sync::atomic::Ordering::SeqCst),"gui_visual_acceptance":false,"verified_timeline":false,"actual9_binding":false,"quality_approved":false});
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output.join("RETURN.json"))
        .unwrap();
    file.write_all(&serde_json::to_vec_pretty(&receipt).unwrap())
        .unwrap();
    file.sync_all().unwrap();
}

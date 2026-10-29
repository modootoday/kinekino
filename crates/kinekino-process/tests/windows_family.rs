#![cfg(all(windows, feature = "test-support"))]
use kinekino_process::{
    application::{
        raw_records::{RecordCursor, RecordPoll},
        supervisor::*,
    },
    domain::{deadline::Deadline, observation::*},
    infrastructure::{evidence_files::PreparedEvidence, family::*},
};
use std::{
    fs, io,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
/// The fixture bin moved to `kinekino-app` (ADR p1-port-inversion); build it first with
/// `cargo build -p kinekino-app --features test-support --bin owned-process-test-child`.
/// `OWNED_PROCESS_TEST_CHILD` overrides the target-dir lookup.
fn owned_process_test_child() -> PathBuf {
    if let Some(path) = std::env::var_os("OWNED_PROCESS_TEST_CHILD") {
        return PathBuf::from(path);
    }
    let exe = std::env::current_exe().unwrap();
    let profile = exe.parent().and_then(|deps| deps.parent()).unwrap();
    profile.join(format!(
        "owned-process-test-child{}",
        std::env::consts::EXE_SUFFIX
    ))
}
#[derive(Default)]
struct Sink(Vec<ProcessReceipt>);
impl ReceiptSink for Sink {
    fn record(&mut self, receipt: &ProcessReceipt) -> io::Result<()> {
        self.0.push(receipt.clone());
        Ok(())
    }
}
fn prepared(mode: &str) -> (PathBuf, SpawnSpec, PreparedEvidence, Deadline) {
    let deadline = Deadline::new(
        Instant::now(),
        Duration::from_secs(5),
        Duration::from_secs(1),
    )
    .unwrap();
    let root = std::env::temp_dir().join(format!(
        "owned-family-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    let evidence = PreparedEvidence::create(&root, None, 64).unwrap();
    let spec = SpawnSpec {
        executable: owned_process_test_child(),
        cwd: root.clone(),
        arguments: vec![mode.into()],
        environment: None,
    };
    (root, spec, evidence, deadline)
}
fn launch(mode: &str) -> (PathBuf, Supervisor<OwnedProcessFamily>, Deadline) {
    let (root, spec, evidence, deadline) = prepared(mode);
    let (process, paths) = spawn(spec, evidence, deadline)
        .unwrap_or_else(|failure| panic!("launch {}: {}", failure.stage, failure.error));
    assert_eq!(process.state(), LaunchState::Running);
    assert!(process.identity().creation_filetime.is_some());
    (root, Supervisor::new(process, paths, 128 * 1024), deadline)
}
fn finish(owner: Supervisor<OwnedProcessFamily>, deadline: Deadline) -> ProcessReceipt {
    match owner.finish(deadline, &SystemClock, &mut Sink::default()) {
        FinishOutcome::Complete(done) => done.receipt().clone(),
        FinishOutcome::Incomplete(failure) => panic!(
            "incomplete {:?}: {}",
            failure.cause.stage, failure.cause.error
        ),
    }
}

#[test]
#[ignore = "Central owned prepared Job budget/transfer/physical EOF cleanup gate; Rust test child only"]
fn pre_reserved_job_budget_survives_original_child_transfer_and_cleanup() {
    kinekino_platform::install();
    use windows::Win32::System::Threading::{GetCurrentProcess, GetProcessAffinityMask};
    let (mut mask, mut system) = (0usize, 0usize);
    unsafe {
        GetProcessAffinityMask(GetCurrentProcess(), &mut mask, &mut system).unwrap();
    }
    let budget = JobBudget {
        max_committed_bytes: 256 * 1024 * 1024,
        affinity_mask: mask.isolate_lowest_one(),
    };
    let job = PreparedFamilyJob::reserve(budget).unwrap();
    job.check_current().unwrap();
    let (root, spec, evidence, deadline) = prepared("eof");
    let (process, paths) = spawn_prepared(spec, evidence, deadline, job)
        .unwrap_or_else(|failure| panic!("launch {}: {}", failure.stage, failure.error));
    assert_eq!(process.state(), LaunchState::Running);
    process.verify_job_budget(budget).unwrap();
    assert!(
        process
            .verify_job_budget(JobBudget {
                max_committed_bytes: budget.max_committed_bytes + 1,
                ..budget
            })
            .is_err()
    );
    let mut owner = Supervisor::new(process, paths, 128 * 1024);
    assert!(
        matches!(owner.wait_stdout_record(&mut RecordCursor::default(), 64, deadline, &SystemClock).unwrap(), RecordPoll::Record { bytes, .. } if bytes == b"ready\n")
    );
    let receipt = finish(owner, deadline);
    assert!(!receipt.forced);
    assert_eq!(receipt.child_exit, Some(0));
    assert_eq!(receipt.family, FamilyObservation::Empty);
    println!("preserved prepared Job child evidence {}", root.display());
}

#[test]
#[ignore = "Explicit owned Windows Job gate; uses only the bundled Rust test child"]
fn stdin_writer_is_not_inherited_and_eof_completes_without_force() {
    kinekino_platform::install();
    let (root, mut owner, deadline) = launch("eof");
    let raw = owner.evidence().stdout.clone();
    let mut cursor = RecordCursor::default();
    assert_eq!(
        owner
            .wait_stdout_record(&mut cursor, 64, deadline, &SystemClock)
            .unwrap(),
        RecordPoll::Record {
            offset: 0,
            bytes: b"ready\n".to_vec()
        }
    );
    let receipt = finish(owner, deadline);
    assert!(!receipt.forced);
    assert_eq!(receipt.child_exit, Some(0));
    assert_eq!(receipt.family, FamilyObservation::Empty);
    assert_eq!(
        cursor.poll(&raw, 64, true).unwrap(),
        RecordPoll::Record {
            offset: 6,
            bytes: b"terminal\n".to_vec()
        }
    );
    fs::remove_dir_all(root).unwrap();
}
#[test]
#[ignore = "Explicit nested family and independently owned sentinel gate"]
fn parent_exit_and_inherited_stdout_do_not_hide_live_family_or_kill_outside_sentinel() {
    kinekino_platform::install();
    let (sentinel_root, mut sentinel, sentinel_deadline) = launch("sentinel");
    let sentinel_identity = sentinel.process().identity().clone();
    let (root, mut owner, deadline) = launch("parent-exit");
    let record = owner
        .wait_stdout_record(&mut RecordCursor::default(), 128, deadline, &SystemClock)
        .unwrap();
    assert!(matches!(record, RecordPoll::Record { .. }));
    loop {
        let state = owner.poll_state().unwrap();
        if state.child_exit == Some(0) {
            assert!(matches!(state.family, FamilyObservation::Active(_)));
            break;
        }
        assert!(!deadline.operation_remaining(Instant::now()).is_zero());
        thread::sleep(Duration::from_millis(10));
    }
    // Child was admitted under the external runner's budget Job before its private nested Job.
    assert!(owner.process().outer_job_observed());
    let receipt = finish(owner, deadline);
    assert!(receipt.forced);
    assert_eq!(receipt.child_exit, Some(0));
    assert_eq!(receipt.family, FamilyObservation::Empty);
    assert_eq!(sentinel.process().identity(), &sentinel_identity);
    assert!(sentinel.poll_state().unwrap().child_exit.is_none());
    let stopped = finish(sentinel, sentinel_deadline);
    assert_eq!(stopped.family, FamilyObservation::Empty);
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(sentinel_root).unwrap();
}
#[test]
#[ignore = "Explicit suspended startup failure gate"]
fn partial_startup_failures_keep_original_handles_and_never_run_child_code() {
    kinekino_platform::install();
    for fault in [
        FaultPoint::AttributePreparation,
        FaultPoint::AttributeInitialized,
        FaultPoint::AfterCreate,
        FaultPoint::Assignment,
        FaultPoint::BeforeResume,
        FaultPoint::UnexpectedResume,
    ] {
        let (root, spec, evidence, deadline) = prepared("eof");
        let failure = match spawn_faulted(spec, evidence, deadline, fault) {
            Err(failure) => failure,
            Ok(_) => panic!("fault did not fail"),
        };
        if matches!(
            fault,
            FaultPoint::AttributePreparation | FaultPoint::AttributeInitialized
        ) {
            assert!(failure.owner.is_none());
        } else {
            let probe = failure.owner.as_ref().unwrap().exit_probe().unwrap();
            drop(failure);
            loop {
                if probe.poll().unwrap().is_some() {
                    break;
                }
                assert!(!deadline.remaining(Instant::now()).is_zero());
                thread::sleep(Duration::from_millis(10));
            }
        }
        assert_eq!(fs::metadata(root.join("stdout.raw")).unwrap().len(), 0);
        fs::remove_dir_all(root).unwrap();
    }
}
#[test]
#[ignore = "Explicit unassigned suspended child and failed termination fallback gate"]
fn failed_assignment_and_failed_termination_remain_unconfirmed_until_drop_fallback_is_observed() {
    kinekino_platform::install();
    let (root, spec, evidence, deadline) = prepared("eof");
    let mut failure = match spawn_faulted(
        spec,
        evidence,
        deadline,
        FaultPoint::AssignmentAndTermination,
    ) {
        Err(failure) => failure,
        Ok(_) => panic!("fault did not fail"),
    };
    assert!(failure.termination_error.is_some());
    let original = failure.owner.as_mut().unwrap();
    assert_eq!(original.state(), LaunchState::UnassignedSuspended);
    assert_eq!(original.poll_family().unwrap(), FamilyObservation::Unknown);
    assert!(original.poll_child().unwrap().is_none());
    let probe = original.exit_probe().unwrap();
    drop(failure);
    loop {
        if probe.poll().unwrap().is_some() {
            break;
        }
        assert!(!deadline.remaining(Instant::now()).is_zero());
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(fs::metadata(root.join("stdout.raw")).unwrap().len(), 0);
    fs::remove_dir_all(root).unwrap();
}
#[test]
#[ignore = "Explicit query failure gate, never an empty-family success"]
fn family_query_failure_returns_original_owner_with_unknown_receipt() {
    kinekino_platform::install();
    let (root, spec, evidence, deadline) = prepared("hang");
    let (process, paths) = spawn_faulted(spec, evidence, deadline, FaultPoint::FamilyQuery)
        .unwrap_or_else(|_| panic!("query injection must not fail launch"));
    let identity = process.identity().clone();
    let probe = process.exit_probe().unwrap();
    match Supervisor::new(process, paths, 1024).finish(deadline, &SystemClock, &mut Sink::default())
    {
        FinishOutcome::Incomplete(failure) => {
            assert!(matches!(
                failure.cause.stage,
                FailureStage::FamilyObservation
            ));
            assert_eq!(failure.owner.process().identity(), &identity);
            assert_eq!(failure.owner.receipt().family, FamilyObservation::Unknown);
            drop(failure);
        }
        _ => panic!("query failure cannot attest family empty"),
    }
    loop {
        if probe.poll().unwrap().is_some() {
            break;
        }
        assert!(!deadline.remaining(Instant::now()).is_zero());
        thread::sleep(Duration::from_millis(10));
    }
    fs::remove_dir_all(root).unwrap();
}
#[test]
#[ignore = "Explicit bounded prefix and operation deadline gate"]
fn incomplete_ready_line_times_out_without_waiting_for_pipe_eof() {
    kinekino_platform::install();
    let (root, mut owner, deadline) = launch("partial");
    let failure = owner
        .wait_stdout_record(&mut RecordCursor::default(), 64, deadline, &SystemClock)
        .unwrap_err();
    assert!(matches!(failure.stage, FailureStage::Deadline));
    let receipt = finish(owner, deadline);
    assert!(receipt.forced);
    assert_eq!(receipt.family, FamilyObservation::Empty);
    fs::remove_dir_all(root).unwrap();
}

#[test]
#[ignore = "Explicit Win32 UTF-16/CRT argument roundtrip gate"]
fn arguments_roundtrip_without_shell_and_nul_is_rejected_before_create() {
    kinekino_platform::install();
    let (invalid_root, mut invalid_spec, invalid_evidence, invalid_deadline) = prepared("args");
    invalid_spec.arguments.push("bad\0argument".into());
    let failure = match spawn(invalid_spec, invalid_evidence, invalid_deadline) {
        Err(failure) => failure,
        Ok(_) => panic!("embedded NUL must not launch"),
    };
    assert!(failure.owner.is_none());
    assert_eq!(
        fs::metadata(invalid_root.join("stdout.raw")).unwrap().len(),
        0
    );
    fs::remove_dir_all(invalid_root).unwrap();

    let (root, mut spec, evidence, deadline) = prepared("args");
    let arguments = ["", "space value", "quote\"value", "trailing\\", "한글😀"];
    spec.arguments.extend(
        arguments
            .iter()
            .map(|value| std::ffi::OsString::from(*value)),
    );
    let (process, paths) =
        spawn(spec, evidence, deadline).unwrap_or_else(|_| panic!("argument launch failed"));
    let raw = paths.stdout.clone();
    let receipt = finish(Supervisor::new(process, paths, 128 * 1024), deadline);
    assert_eq!(receipt.child_exit, Some(0));
    assert!(!receipt.forced);
    let mut cursor = RecordCursor::default();
    for value in arguments {
        let hex = value
            .as_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let expected = format!("arg:{}:{hex}\n", value.len());
        match cursor.poll(&raw, 1024, true).unwrap() {
            RecordPoll::Record { bytes, .. } => assert_eq!(bytes, expected.as_bytes()),
            _ => panic!("missing argument record"),
        }
    }
    assert_eq!(cursor.poll(&raw, 1024, true).unwrap(), RecordPoll::Pending);
    fs::remove_dir_all(root).unwrap();
}

#[test]
#[ignore = "Explicit post-resume late-admission fault gate"]
fn successful_resume_returning_at_cutoff_is_an_owned_launch_failure() {
    kinekino_platform::install();
    let (root, spec, evidence, deadline) = prepared("hang");
    let failure = match spawn_faulted(spec, evidence, deadline, FaultPoint::LateResume) {
        Err(failure) => failure,
        Ok(_) => panic!("successful ResumeThread count must still obey the operation cutoff"),
    };
    assert_eq!(failure.error.kind(), io::ErrorKind::TimedOut);
    assert_eq!(failure.stage, "assign_verify_resume");
    let original = failure
        .owner
        .as_ref()
        .expect("resumed original owner must be returned");
    assert_eq!(original.state(), LaunchState::Running);
    assert!(original.identity().creation_filetime.is_some());
    let probe = original.exit_probe().unwrap();
    drop(failure);
    loop {
        if probe.poll().unwrap().is_some() {
            break;
        }
        assert!(!deadline.remaining(Instant::now()).is_zero());
        thread::sleep(Duration::from_millis(10));
    }
    fs::remove_dir_all(root).unwrap();
}

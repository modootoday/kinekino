//! Native receipt fault characterization; this is not a public SDK recovery API.
use super::*;
use std::fs;

#[test]
#[ignore = "Actual owned child and regular input; callback unwind retained until original family/receipt closure; fresh evidence"]
fn native_driver_unwind_closes_original_family_before_resuming_same_payload() {
    use crate::{
        application::review_source::MutationControl,
        infrastructure::clip_working_file::ClipWorkingFile,
    };
    use std::{
        collections::BTreeSet,
        path::PathBuf,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
    };
    struct Control;
    impl MutationControl for Control {
        fn checkpoint(&self) -> Result<(), ApiError> {
            Ok(())
        }
    }
    struct OriginalPanic(Arc<()>);
    struct Panicking {
        count: AtomicUsize,
        marker: Arc<()>,
        native: crate::application::review_source::NativeProcessLedger,
    }
    impl MutationControl for Panicking {
        fn native_processes(
            &self,
        ) -> Option<&crate::application::review_source::NativeProcessLedger> {
            Some(&self.native)
        }
        fn checkpoint(&self) -> Result<(), ApiError> {
            if self.count.fetch_add(1, Ordering::SeqCst) == 1 {
                std::panic::panic_any(OriginalPanic(self.marker.clone()));
            }
            Ok(())
        }
    }
    let root =
        PathBuf::from(std::env::var_os("NATIVE_UNWIND_EVIDENCE_DIR").expect("Fresh evidence"));
    fs::create_dir(&root).unwrap();
    let prefix = format!("frame-owned-{}-", std::process::id());
    let evidence_roots = || {
        fs::read_dir(std::env::temp_dir())
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| {
                path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with(&prefix)
            })
            .collect::<BTreeSet<_>>()
    };
    let before = evidence_roots();
    let mut working = ClipWorkingFile::create(&std::env::temp_dir(), 4096, &Control).unwrap();
    let facts = working
        .seal(
            b"original retained regular input across native unwind",
            &Control,
        )
        .unwrap();
    let path = working.path().to_path_buf();
    let mut command = Command::new(std::env::current_exe().unwrap());
    command.args(["--exact","infrastructure::frame_capture::process::owned::tests::regular_stdin_consumed_child_fixture","--ignored","--nocapture"]).env("CLIP_WORKING_CHILD_FIXTURE","1");
    let marker = Arc::new(());
    let control = Panicking {
        count: AtomicUsize::new(0),
        marker: marker.clone(),
        native: Default::default(),
    };
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        working.with_reader(|input| {
            run_with_input(
                &command,
                Instant::now() + Duration::from_secs(12),
                8192,
                Some(&control),
                None,
                Some(input),
            )
        })
    }));
    let payload = match caught {
        Err(payload) => payload,
        Ok(_) => panic!("Original callback panic was not resumed"),
    };
    let observations = control.native.observations();
    assert_eq!(observations.len(), 1);
    assert!(matches!(
        observations[0],
        crate::application::review_source::NativeProcessObservation::Observed {
            child_exit: Some(_),
            family_empty: true,
            receipt_recorded: true,
            regular_files_eof: true,
            forced: true,
        }
    ));
    assert!(Arc::ptr_eq(
        &payload
            .downcast_ref::<OriginalPanic>()
            .expect("Original typed payload")
            .0,
        &marker
    ));
    let after = evidence_roots();
    let created = after.difference(&before).collect::<Vec<_>>();
    assert_eq!(created.len(), 1);
    let receipt: serde_json::Value =
        serde_json::from_slice(&fs::read(created[0].join("receipt-0000.json")).unwrap()).unwrap();
    assert!(receipt["child_exit"].is_number());
    assert_eq!(receipt["family"], "Empty");
    assert_eq!(receipt["forced"], true);
    assert!(path.exists());
    drop(working);
    assert!(!path.exists());
    fs::write(
        root.join("input-facts.json"),
        serde_json::to_vec_pretty(&facts).unwrap(),
    )
    .unwrap();
    fs::write(root.join("verification.json"),serde_json::to_vec_pretty(&serde_json::json!({"original_process_evidence":created[0],"receipt":receipt,"same_original_panic_payload_resumed_after_closure":true,"new_input_absent_after_original_outer_owner_release":true,"input_consumption_before_panic_verified":false,"all_jobs_owned_attempt_connected":false})).unwrap()).unwrap();
}

#[test]
fn forced_zero_exit_never_promotes_success_and_original_error_wins() {
    assert!(completion_failure(None, Some(0), false).is_none());
    let forced = completion_failure(None, Some(0), true).unwrap();
    assert_eq!(forced.code, ErrorCode::EngineFailed);
    assert_eq!(forced.details["stage"], "forced_cleanup");
    assert_eq!(
        completion_failure(None, Some(9), true).unwrap().details["native_exit_code"],
        9
    );
    let original = ApiError::new(
        ErrorCode::Cancelled,
        "Original cooperative request before forced cleanup",
    );
    let preserved = completion_failure(Some(original), Some(0), true).unwrap();
    assert_eq!(preserved.code, ErrorCode::Cancelled);
    assert_eq!(
        preserved.message,
        "Original cooperative request before forced cleanup"
    );
}

#[test]
#[ignore = "Owned child fixture, only original parent supplies the marker environment"]
fn regular_stdin_consumed_child_fixture() {
    if std::env::var("CLIP_WORKING_CHILD_FIXTURE").ok().as_deref() != Some("1") {
        return;
    }
    use sha2::{Digest, Sha256};
    use std::io::{Read, Write};
    let mut bytes = Vec::new();
    std::io::stdin().read_to_end(&mut bytes).unwrap();
    println!("clip-input-consumed:{:x}", Sha256::digest(&bytes));
    std::io::stdout().flush().unwrap();
    loop {
        std::thread::sleep(Duration::from_millis(10));
    }
}
#[test]
#[ignore = "Central allocation: real original Windows child consumes regular stdin then is cancelled; fresh evidence"]
fn regular_stdin_cancellation_waits_for_original_family_and_receipt_before_working_file_deletion() {
    use crate::{
        application::review_source::MutationControl,
        infrastructure::clip_working_file::ClipWorkingFile,
    };
    use sha2::{Digest, Sha256};
    let evidence = std::path::PathBuf::from(
        std::env::var_os("CLIP_STDIN_CANCEL_EVIDENCE_DIR").expect("Fresh evidence"),
    );
    fs::create_dir(&evidence).unwrap();
    struct Control;
    impl MutationControl for Control {
        fn checkpoint(&self) -> Result<(), ApiError> {
            Ok(())
        }
    }
    let mut working = ClipWorkingFile::create(&std::env::temp_dir(), 4096, &Control).unwrap();
    let input = b"original parent-owned regular stdin cancellation fixture";
    let facts = working.seal(input, &Control).unwrap();
    let path = working.path().to_path_buf();
    let mut command = Command::new(std::env::current_exe().unwrap());
    command.args(["--exact","infrastructure::frame_capture::process::owned::tests::regular_stdin_consumed_child_fixture","--ignored","--nocapture"]).env("CLIP_WORKING_CHILD_FIXTURE","1");
    let error = working
        .with_reader(|input_file| {
            let deadline =
                Deadline::new(Instant::now(), Duration::from_secs(12), CLEANUP_RESERVE).unwrap();
            let (spec, prepared, root) = launch::prepare_with_input(&command, input_file)?;
            let (owner, first) = match spawn(spec, prepared, deadline) {
                Ok((family, prepared)) => {
                    (Supervisor::new(family, prepared, STDERR_LIMIT as u64), None)
                }
                Err(failure) => {
                    let first = failed(failure.stage);
                    let Some(family) = failure.owner else {
                        return Err(first);
                    };
                    (
                        Supervisor::new(*family, failure.evidence, STDERR_LIMIT as u64),
                        Some(first),
                    )
                }
            };
            struct CancelOnMarker {
                raw: RawFile,
                marker: String,
            }
            impl MutationControl for CancelOnMarker {
                fn checkpoint(&self) -> Result<(), ApiError> {
                    let file = self
                        .raw
                        .open_read()
                        .map_err(|_| failed("fixture_raw_reader"))?;
                    let mut bytes = Vec::new();
                    file.take(8192)
                        .read_to_end(&mut bytes)
                        .map_err(|_| failed("fixture_raw_read"))?;
                    if String::from_utf8_lossy(&bytes).contains(&self.marker) {
                        Err(ApiError::new(
                            ErrorCode::Cancelled,
                            "original child consumed stdin before cancellation",
                        ))
                    } else {
                        Ok(())
                    }
                }
            }
            let control = CancelOnMarker {
                raw: owner.evidence().stdout.clone(),
                marker: format!("clip-input-consumed:{:x}", Sha256::digest(input)),
            };
            let mut session = Session {
                owner,
                deadline,
                max_stdout: STDERR_LIMIT,
                first,
                sink: receipt::FileSink::new(root),
                receipt_attempted: false,
                recovery_generation: None,
                output_drain: Default::default(),
            };
            while !session.poll(Some(&control), None) {
                std::thread::sleep(Duration::from_millis(10));
            }
            let receipt = session.owner.receipt();
            assert!(receipt.child_exit.is_some());
            assert_eq!(receipt.family, FamilyObservation::Empty);
            assert!(session.owner.receipt_recorded());
            session.complete()
        })
        .err()
        .unwrap();
    assert_eq!(error.code, ErrorCode::Cancelled);
    assert_eq!(error.details["kinekino_process"]["physical_closed"], true);
    assert_eq!(error.details["kinekino_process"]["receipt_recorded"], true);
    drop(working);
    assert!(!path.exists());
    fs::write(
        evidence.join("input-facts.json"),
        serde_json::to_vec_pretty(&facts).unwrap(),
    )
    .unwrap();
    fs::write(evidence.join("verification.json"),serde_json::to_vec_pretty(&serde_json::json!({"primary_code":error.code,"original_process_evidence":error.details,"actual_regular_stdin_consumed_before_cancel":true,"original_family_and_receipt_settled":true,"new_working_file_absent_after_original_owner_release":true,"old_temp_and_evidence_not_removed":true,"full_clip_jobs_cancel_or_png_cleanup_verified":false})).unwrap()).unwrap();
}

#[test]
#[ignore = "Central allocation: real owned Windows child and immutable receipt collision"]
fn receipt_failure_stays_pending_until_explicit_same_owner_record_and_keeps_first_error() {
    let mut command = Command::new(std::env::current_exe().unwrap());
    // Run a real harness child with no selected case, so it exits normally without
    // creating another nested test or relying on timing to trigger the fault.
    command.args(["--exact", "frame_receipt_zero_case_fixture"]);
    let deadline = Deadline::new(Instant::now(), Duration::from_secs(7), CLEANUP_RESERVE).unwrap();
    let (spec, evidence, root) = launch::prepare(&command).unwrap();
    let first_path = root.join("receipt-0000.json");
    let sentinel = b"preserved original occupied receipt destination";
    fs::write(&first_path, sentinel).unwrap();
    let (owner, launch_error) = match spawn(spec, evidence, deadline) {
        Ok((process, evidence)) => (
            Supervisor::new(process, evidence, STDERR_LIMIT as u64),
            None,
        ),
        Err(failure) => {
            let first = failed(failure.stage);
            let Some(process) = failure.owner else {
                panic!("No child owner at launch failure stage {}", failure.stage);
            };
            (
                Supervisor::new(*process, failure.evidence, STDERR_LIMIT as u64),
                Some(first),
            )
        }
    };
    let launch_failed = launch_error.is_some();
    let identity = owner.receipt().identity.clone();
    let mut session = Session {
        owner,
        deadline,
        max_stdout: 4096,
        first: launch_error,
        sink: receipt::FileSink::new(root.clone()),
        receipt_attempted: false,
        recovery_generation: None,
        output_drain: Default::default(),
    };
    let cutoff = Instant::now() + Duration::from_secs(3);
    while !session.receipt_attempted {
        assert!(!session.poll(None, None));
        assert!(Instant::now() < cutoff, "original child did not close");
        std::thread::sleep(Duration::from_millis(10));
    }
    let progress = session.owner.cleanup_progress().unwrap();
    assert!(progress.physical_closed());
    assert!(!progress.final_receipt_recorded());
    assert!(!progress.settled());
    assert!(!session.owner.receipt_recorded());
    assert_eq!(
        session.first.as_ref().unwrap().code,
        ErrorCode::EngineFailed
    );
    let first_message = session.first.as_ref().unwrap().message.clone();
    for _ in 0..20 {
        assert!(
            !session.poll(None, None),
            "receipt failure must not acknowledge cleanup"
        );
    }
    assert!(
        !root.join("receipt-0001.json").exists(),
        "no implicit retry"
    );
    assert_eq!(fs::read(&first_path).unwrap(), sentinel);
    // Exercise the existing explicit primitive on the very same owner and sink.
    // Product Session has no public recovery request yet; do not claim it does.
    session
        .owner
        .record_cleanup(&mut session.sink, &SystemClock)
        .unwrap();
    assert!(!session.output_drain.complete());
    assert!(!session.poll(None, None));
    assert!(session.poll(None, None));
    assert!(session.output_drain.complete());
    assert!(session.owner.cleanup_progress().unwrap().settled());
    assert_eq!(session.owner.receipt().identity.pid, identity.pid);
    assert_eq!(
        session.owner.receipt().identity.creation_filetime,
        identity.creation_filetime
    );
    let recovered: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("receipt-0001.json")).unwrap()).unwrap();
    assert_eq!(recovered["identity"]["pid"], identity.pid);
    assert_eq!(recovered["child_exit"], 0);
    assert_eq!(recovered["family"], "Empty");
    assert_eq!(recovered["forced"], false);
    let error = session
        .complete()
        .err()
        .expect("recovery is not operation success");
    assert!(
        !launch_failed,
        "partial launch was retained and cleaned, but is not this receipt case"
    );
    assert_eq!(error.code, ErrorCode::EngineFailed);
    assert_eq!(error.message, first_message);
    assert_eq!(error.details["kinekino_process"]["physical_closed"], true);
    assert_eq!(error.details["kinekino_process"]["receipt_recorded"], true);
    assert_eq!(fs::read(&first_path).unwrap(), sentinel);
    fs::write(
        root.join("recovery-error.json"),
        serde_json::to_vec_pretty(&error).unwrap(),
    )
    .unwrap();
}

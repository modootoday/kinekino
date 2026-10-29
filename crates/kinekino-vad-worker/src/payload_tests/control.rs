use super::*;

struct FaultControl {
    fail: Cell<bool>,
    cutoff: Instant,
}
impl AnalysisControl for FaultControl {
    fn cancellation_requested(&self) -> bool {
        panic!("Fallible consumer used legacy bool instead of checkpoint")
    }
    fn deadline(&self) -> Instant {
        self.cutoff
    }
    fn checkpoint(&self) -> Result<bool, ApiError> {
        if self.fail.get() {
            Err(ApiError::new(
                ErrorCode::PermissionDenied,
                "original control IO failure",
            ))
        } else {
            Ok(false)
        }
    }
}
struct PartialThenFault<'a>(&'a FaultControl);
impl kinekino_vad::application::VadEngine for PartialThenFault<'_> {
    fn run(
        &self,
        _: &VadProfile,
        _: &[u8],
        _: &[u8],
        _: &[i16],
        _: &dyn AnalysisControl,
        scores: &mut dyn kinekino_vad::application::ScoreSink,
    ) -> Result<kinekino_vad::application::EngineCompletion, ApiError> {
        scores.push(0.2)?;
        self.0.fail.set(true);
        Ok(kinekino_vad::application::EngineCompletion::Partial {
            processed_samples: 512,
        })
    }
}
#[test]
fn worker_partial_followed_by_control_io_error_stays_original_failure() {
    let (request, _, source) = inference_fixture();
    let control = FaultControl {
        fail: Cell::new(false),
        cutoff: Instant::now() + Duration::from_secs(1),
    };
    let bytes = crate::run::run_request(
        &request,
        &Exact(request.expected().profile().clone()),
        &source,
        &PartialThenFault(&control),
        &control,
    )
    .unwrap();
    let decoded = kinekino_vad::protocol::application::decode_worker_outcome(
        bytes.as_slice(),
        bytes.len() as u64,
        request.expected(),
    )
    .unwrap();
    match decoded.result() {
        kinekino_vad::protocol::domain::WorkerResult::Failure { code, message } => {
            assert_eq!(*code, ErrorCode::PermissionDenied);
            assert_eq!(message, "original control IO failure");
        }
        _ => panic!("IO fault became cancellation observation or normal output"),
    }
}
#[test]
fn control_io_failure_before_read_does_not_open_artifacts_or_engine() {
    let (request, _, source) = inference_fixture();
    let control = FaultControl {
        fail: Cell::new(true),
        cutoff: Instant::now() + Duration::from_secs(1),
    };
    let bytes = crate::run::run_request(
        &request,
        &Exact(request.expected().profile().clone()),
        &source,
        &PartialThenFault(&control),
        &control,
    )
    .unwrap();
    let decoded = kinekino_vad::protocol::application::decode_worker_outcome(
        bytes.as_slice(),
        bytes.len() as u64,
        request.expected(),
    )
    .unwrap();
    assert!(matches!(
        decoded.result(),
        kinekino_vad::protocol::domain::WorkerResult::Failure {
            code: ErrorCode::PermissionDenied,
            ..
        }
    ));
    assert_eq!(source.opens.get(), 0);
}
#[test]
fn legacy_parent_arguments_without_explicit_control_are_rejected() {
    let root = std::env::temp_dir().join("control-shape-only");
    let old: Vec<std::ffi::OsString> = vec![
        "--vad-request-v1".into(),
        root.join("stdin.raw").into_os_string(),
        "--cancel-file".into(),
        root.join("cancel.signal").into_os_string(),
        "--release-manifest-id".into(),
        "manifest".into(),
        "--release-manifest-sha256".into(),
        "a".repeat(64).into(),
        "--release-manifest-bytes".into(),
        "16".into(),
    ];
    assert_eq!(
        crate::entry::WorkerLaunch::parse(old).err().unwrap().code,
        ErrorCode::InvalidArgument
    );
}

mod files {
    use super::*;
    use crate::control::GuardedCancellation;
    use kinekino_vad_executor::application::WorkerCancellation;
    use std::fs;
    fn root() -> std::path::PathBuf {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "child-control-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir(&path).unwrap();
        path
    }
    #[test]
    fn complete_marker_is_guarded_and_absence_is_not_a_read_error() {
        let root = root();
        let path = root.join("cancel.signal");
        let signal = GuardedCancellation::open(&root, &path).unwrap();
        assert!(!signal.requested().unwrap());
        fs::write(&path, b"cancel\n").unwrap();
        assert!(signal.requested().unwrap());
        crate::infrastructure::os_policy::guarded_mutation(fs::write(&path, b"changed"));
        assert!(signal.requested().unwrap());
        drop(signal);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn malformed_marker_is_source_error_not_cancellation() {
        let root = root();
        let path = root.join("cancel.signal");
        fs::write(&path, b"partial").unwrap();
        let signal = GuardedCancellation::open(&root, &path).unwrap();
        assert_eq!(
            signal.requested().unwrap_err().code,
            ErrorCode::SourceExpired
        );
        drop(signal);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn control_file_actual_pin_request_binding_and_original_guard_are_required() {
        use crate::{control::ChildControlFiles, entry::WorkerLaunch};
        use kinekino_vad_executor::domain::{CounterDomain, CounterReading, WorkerOperationCutoff};
        let root = root();
        let request = support::pin("worker-request", b"original-request");
        let domain = CounterDomain {
            kind: "windows-qpc/1".into(),
            frequency_hz: kinekino_media_contracts::api_values::DecimalU64(10_000_000),
            parent_pid: 123,
            parent_creation_filetime: kinekino_media_contracts::api_values::DecimalU64(456),
        };
        // Synthetic domain is only for metadata/file checks; never a production clock.
        let cutoff = WorkerOperationCutoff::correlate(
            CounterReading {
                domain: domain.clone(),
                ticks: 100,
            },
            CounterReading { domain, ticks: 101 },
            Duration::from_secs(1),
            request.sha256().into(),
        )
        .unwrap();
        let raw = serde_json::to_vec(&cutoff).unwrap();
        let path = root.join("worker-control.v1.json");
        fs::write(&path, &raw).unwrap();
        let mut launch = WorkerLaunch {
            request: root.join("stdin.raw"),
            cancel_signal: root.join("cancel.signal"),
            manifest: support::pin("manifest", b"unadmitted"),
            control_path: path.clone(),
            control_pin: support::pin("worker-control", &raw),
        };
        let files = ChildControlFiles::open(&launch, &request).unwrap();
        assert_eq!(files.cutoff().request_sha256, request.sha256());
        crate::infrastructure::os_policy::guarded_mutation(fs::write(&path, b"changed"));
        drop(files);
        assert_eq!(
            ChildControlFiles::open(&launch, &support::pin("worker-request", b"other"))
                .err()
                .unwrap()
                .code,
            ErrorCode::SourceExpired
        );
        launch.control_pin = support::pin("worker-control", &vec![0; raw.len()]);
        assert_eq!(
            ChildControlFiles::open(&launch, &request)
                .err()
                .unwrap()
                .code,
            ErrorCode::SourceExpired
        );
        fs::remove_dir_all(root).unwrap();
    }
}

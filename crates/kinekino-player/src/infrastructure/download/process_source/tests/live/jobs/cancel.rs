use super::*;
use crate::application::{
    download::{PendingMediaVerification, VerificationPoll},
    jobs::{CancelJob, owned_attempt::OwnerObservation},
};
use std::sync::atomic::AtomicBool;
pub(super) struct Gate {
    pub(super) origin_released: AtomicBool,
    pub(super) started: AtomicBool,
    pub(super) cancelled: AtomicBool,
    pub(super) release: AtomicBool,
}
pub(super) struct GatedProbe(pub(super) Arc<Gate>);
struct Pending {
    gate: Arc<Gate>,
    lease: Arc<dyn VerificationLease>,
}
impl DownloadVerifier for GatedProbe {
    fn begin_owned(
        &self,
        lease: Arc<dyn VerificationLease>,
        _: MediaIdentity,
        control: &dyn ExecutionControl,
    ) -> Result<OwnedVerification, ApiError> {
        control_check(control)?;
        lease.check_current()?;
        self.0.started.store(true, Ordering::SeqCst);
        Ok(OwnedVerification::Pending(Box::new(Pending {
            gate: self.0.clone(),
            lease,
        })))
    }
    fn verify(
        &self,
        _: &Path,
        _: &MediaIdentity,
        _: &dyn ExecutionControl,
    ) -> Result<(), ApiError> {
        Err(failed("Fixture requires original pending lease"))
    }
}
impl PendingMediaVerification for Pending {
    fn owner_observation(&self) -> OwnerObservation {
        if self.gate.release.load(Ordering::SeqCst) {
            OwnerObservation::NotStarted
        } else {
            OwnerObservation::Pending
        }
    }
    fn request_cancel(&mut self) -> Result<(), ApiError> {
        self.gate.cancelled.store(true, Ordering::SeqCst);
        Ok(())
    }
    fn poll(&mut self, _: &dyn ExecutionControl) -> VerificationPoll {
        self.lease.check_current().unwrap();
        if self.gate.release.load(Ordering::SeqCst) {
            VerificationPoll::Closed(Ok(()))
        } else {
            VerificationPoll::Pending
        }
    }
    fn first_error(&self) -> Option<&ApiError> {
        None
    }
}
fn wait_until(end: Instant, mut predicate: impl FnMut() -> bool) {
    while !predicate() {
        assert!(
            Instant::now() < end,
            "original source did not close while probe remained pending"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}
#[test]
#[ignore = "actual original child/coordinator cancellation with held probe lease; centrally assigned"]
fn pending_probe_does_not_block_original_source_cleanup_or_release_partial_early() {
    let worker = PathBuf::from(
        std::env::var_os("SOURCE_SDK_FIXTURE_BIN").expect("fixed hold-capable fixture"),
    );
    let root = std::env::temp_dir().join(format!(
        "sdk-source-probe-pending-{}-{}",
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
    let gate = Arc::new(Gate {
        origin_released: AtomicBool::new(false),
        started: AtomicBool::new(false),
        cancelled: AtomicBool::new(false),
        release: AtomicBool::new(false),
    });
    let consumed = Arc::new(AtomicUsize::new(0));
    let drops = Arc::new(AtomicUsize::new(0));
    let transport = Arc::new(AtomicUsize::new(0));
    let preparation = Arc::new(Preparation {
        replay_failure: None,
        request_failure: None,
        omit_body_end: false,
        source_exit: 0,
        gate: Some(gate.clone()),
        root: root.clone(),
        worker,
        consumed: consumed.clone(),
        drops: drops.clone(),
        transport: transport.clone(),
        binding: Arc::new(Mutex::new(None)),
    });
    let executor = download_registry(Vec::new(), Some(preparation)).unwrap();
    let host = crate::infrastructure::jobs::start_retaining_host_with_executor(
        &root.join("jobs"),
        executor,
    )
    .unwrap();
    let client = host.client();
    let end = Instant::now() + Duration::from_secs(20);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let request = DownloadRequest {
            source: DownloadSourceSelection {
                selection_id: "fixture-selection".into(),
                descriptor_sha256: "a".repeat(64),
            },
            destination_id: "fixture-output".into(),
            maximum_bytes: DecimalU64(1000),
        };
        let submitted = client
            .submit(
                SubmitJob {
                    operation: DOWNLOAD_OPERATION.into(),
                    parameters: serde_json::to_value(request).unwrap(),
                    expected_source_sha256: None,
                    execution_timeout_ms: DecimalU64(10000),
                },
                "sdk-pending-probe-request",
            )
            .unwrap();
        wait_until(end, || gate.started.load(Ordering::SeqCst));
        client
            .cancel(CancelJob {
                job_id: submitted.job.job_id.clone(),
                expected_attempt: DecimalU64(1),
            })
            .unwrap();
        wait_until(end, || gate.cancelled.load(Ordering::SeqCst));
        wait_until(Instant::now() + Duration::from_secs(2), || {
            transport.load(Ordering::SeqCst) == 1
        });
        wait_until(Instant::now() + Duration::from_secs(2), || {
            gate.origin_released.load(Ordering::SeqCst)
        });
        assert!(!gate.release.load(Ordering::SeqCst));
        let held = client
            .get(GetJob {
                job_id: submitted.job.job_id.clone(),
            })
            .unwrap();
        assert_eq!(held.state, JobState::Cancelling);
        assert!(!root.join("output/fixture.bin").exists());
        assert!(fs::read_dir(root.join("output")).unwrap().next().is_some());
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        gate.release.store(true, Ordering::SeqCst);
        let terminal = loop {
            let job = client
                .get(GetJob {
                    job_id: submitted.job.job_id.clone(),
                })
                .unwrap();
            if job.state.terminal() {
                break job;
            }
            assert!(Instant::now() < end);
            std::thread::sleep(Duration::from_millis(2));
        };
        fs::write(
            root.join("terminal-job.json"),
            serde_json::to_vec_pretty(&terminal).unwrap(),
        )
        .unwrap();
        assert_eq!(terminal.state, JobState::Cancelled, "{terminal:?}");
        assert!(!terminal.publication_started);
        assert!(terminal.output.is_none());
        assert!(fs::read_dir(root.join("output")).unwrap().next().is_none());
    }));
    gate.release.store(true, Ordering::SeqCst);
    let mut pending = host.begin_shutdown(end);
    loop {
        match pending.poll(end) {
            WorkerShutdownOutcome::Complete(complete) => {
                assert!(complete.errors.first().is_none());
                assert!(complete.receipt.is_some());
                break;
            }
            WorkerShutdownOutcome::Pending(original) => {
                pending = original;
                if Instant::now() >= end {
                    eprintln!("sdk-source-pending-probe:original-shutdown-incomplete");
                    std::process::exit(124);
                }
                std::thread::sleep(Duration::from_millis(2));
            }
        }
    }
    if let Err(panic) = result {
        std::panic::resume_unwind(panic);
    }
    assert_eq!(consumed.load(Ordering::SeqCst), 1);
    assert_eq!(transport.load(Ordering::SeqCst), 1);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

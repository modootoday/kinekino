//! Real retaining coordinator and filesystem publication, synthetic source/probe.
use super::super::super::super::{StagedDownloadOutput, download_registry};
use super::*;
mod cancel;
mod cli;
use crate::application::{
    download::{
        DOWNLOAD_OPERATION, DownloadPreparation, DownloadRequest, DownloadSourceSelection,
        DownloadVerifier, OwnedVerification, PreparedDownloadResources, SourceCustody,
        VerificationLease,
    },
    jobs::{
        GetJob, JobAttemptIdentityV1, JobState, SourceBlobStore, SubmitJob,
        owned_attempt::AttemptScope,
    },
};
use crate::domain::api_values::DecimalU64;
use kinekino_jobs::{domain::PreparedJobInput, infrastructure::WorkerShutdownOutcome};
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    path::{Path, PathBuf},
};

struct BoundAdmission {
    original: super::super::super::super::browser_replay::BrowserReplayAdmission<
        crate::application::browser_provider::BrowserDocumentSourceOwner,
    >,
    consumed: Arc<AtomicUsize>,
    gate: Option<Arc<cancel::Gate>>,
}
struct BoundControl {
    identity: Option<JobAttemptIdentityV1>,
    deadline: Instant,
}
impl ExecutionControl for BoundControl {
    fn execution_identity(&self) -> Option<&JobAttemptIdentityV1> {
        self.identity.as_ref()
    }
    fn deadline(&self) -> Instant {
        self.deadline
    }
    fn cancellation_requested(&self) -> bool {
        false
    }
    fn reserve_publication(&self) -> Result<(), ApiError> {
        panic!("source cannot publish")
    }
    fn begin_verification(&self) -> Result<(), ApiError> {
        panic!("source cannot verify")
    }
}
impl ReplayAdmission for BoundAdmission {
    fn prepare(&mut self) -> Result<(), ApiError> {
        self.original.prepare()
    }
    fn poll_release(&mut self, deadline: Instant) -> Result<bool, ApiError> {
        let released = self.original.poll_release(deadline)?;
        if released && let Some(gate) = &self.gate {
            gate.origin_released.store(true, Ordering::SeqCst);
        }
        Ok(released)
    }
    fn check_request(
        &self,
        request: &kinekino_source_host::request::HttpBootstrap,
    ) -> Result<(), ApiError> {
        self.original.check_request(request)
    }
    fn check(&self, control: &dyn ExecutionControl) -> Result<(), ApiError> {
        self.original.check(control)
    }
    fn consume(&mut self, control: &dyn ExecutionControl) -> Result<(), ApiError> {
        self.original.consume(control)?;
        if self.consumed.fetch_add(1, Ordering::SeqCst) != 0 {
            return Err(failed("Original fixture attempt already consumed"));
        }
        Ok(())
    }
    fn check_origin(&self, control: &dyn ExecutionControl) -> Result<(), ApiError> {
        self.original.check_origin(control)
    }
}
struct BytesProbe;
impl DownloadVerifier for BytesProbe {
    fn begin_owned(
        &self,
        lease: Arc<dyn VerificationLease>,
        expected: MediaIdentity,
        control: &dyn ExecutionControl,
    ) -> Result<OwnedVerification, ApiError> {
        control_check(control)?;
        lease.check_current()?;
        assert_eq!(expected.media, "fixture-media");
        assert_eq!(fs::read(lease.path()).unwrap(), b"offline-sdk-body");
        Ok(OwnedVerification::Complete(Ok(())))
    }
    fn verify(
        &self,
        _: &Path,
        _: &MediaIdentity,
        _: &dyn ExecutionControl,
    ) -> Result<(), ApiError> {
        Err(ApiError::new(
            ErrorCode::Unsupported,
            "Fixture only accepts original owned verification lease",
        ))
    }
}
struct Preparation {
    replay_failure: Option<bool>,
    request_failure: Option<bool>,
    omit_body_end: bool,
    source_exit: u32,
    gate: Option<Arc<cancel::Gate>>,
    root: PathBuf,
    worker: PathBuf,
    consumed: Arc<AtomicUsize>,
    drops: Arc<AtomicUsize>,
    transport: Arc<AtomicUsize>,
    binding: Arc<Mutex<Option<JobAttemptIdentityV1>>>,
}
impl DownloadPreparation for Preparation {
    fn prepare_in_session(
        &self,
        input: &PreparedJobInput,
        _: &dyn SourceBlobStore,
        scope: &AttemptScope<'_>,
        custody: SourceCustody<'_>,
    ) -> Result<PreparedDownloadResources, ApiError> {
        assert_eq!(scope.identity().input_sha256(), input.sha256().unwrap());
        assert_eq!(scope.identity().attempt(), 1);
        *self.binding.lock().unwrap() = Some(scope.identity().clone());
        let start = Instant::now();
        // All phases fit INSIDE the original core cutoff; reserve fixture cleanup.
        let deadline = Deadline::new(
            start,
            scope.deadline().saturating_duration_since(start),
            Duration::from_secs(3),
        )
        .unwrap();
        let inner = self.root.join("inner");
        fs::create_dir(&inner).unwrap();
        let evidence = PreparedEvidence::create(&inner, None, 64).unwrap();
        let ipc = PreparedSourceIpc::new(deadline).unwrap();
        let launch = ParentLaunch::new(
            SpawnSpec {
                executable: self.worker.clone(),
                cwd: inner.clone(),
                arguments: Vec::new(),
                environment: None,
            },
            evidence,
            deadline,
            ipc,
            65536,
            Guard(self.drops.clone()),
        );
        let target = if self.omit_body_end {
            "https://example.invalid/offline-sdk-without-body-end"
        } else if self.source_exit != 0 {
            "https://example.invalid/offline-sdk-high-exit"
        } else if self.gate.is_some() {
            "https://example.invalid/offline-sdk-hold-after-body"
        } else {
            "https://example.invalid/offline-sdk-body"
        };
        let receipt_file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(inner.join("source-receipts.txt"))
            .unwrap();
        let (provider, stream, owner) =
            crate::application::browser_provider::fixture_stream_for_request(
                target,
                scope.deadline(),
            );
        let original = super::super::super::super::browser_replay::BrowserReplayAdmission::pending(
            provider,
            stream,
            owner,
            Arc::new(ReplayPolicy(self.replay_failure)),
            scope,
            input,
        );
        ProcessSourceOpener::adopt_in_session(
            custody,
            launch,
            BoundAdmission {
                original,
                consumed: self.consumed.clone(),
                gate: self.gate.clone(),
            },
            LiveSink {
                expected_exit: self.source_exit,
                file: receipt_file,
                transport: self.transport.clone(),
            },
            deadline,
            MediaIdentity {
                media: "fixture-media".into(),
                tracks: "fixture-tracks".into(),
            },
            |admission| {
                if let Some(panic) = self.request_failure {
                    if panic {
                        panic!("Original fixture panic during request preparation");
                    }
                    let mut error = ApiError::new(
                        ErrorCode::NeedsInput,
                        "Original fixture request preparation failed",
                    );
                    error.request_id = "original-request-preparation".into();
                    return Err(error);
                }
                let original = &admission.original;
                let binding = original.wire_binding().unwrap();
                let mut changed_input = input.clone();
                changed_input.parameters["maximum_bytes"] = serde_json::json!("1001");
                let (foreign_provider, foreign_grant, foreign_owner) =
                    crate::application::browser_provider::fixture_grant_for_request(
                        target,
                        scope.deadline(),
                    );
                let rejected =
                    super::super::super::super::browser_replay::BrowserReplayAdmission::new(
                        foreign_provider,
                        foreign_grant,
                        foreign_owner,
                        scope,
                        &changed_input,
                    );
                assert_eq!(
                    rejected.wire_binding().unwrap_err().code,
                    ErrorCode::RevisionConflict
                );
                original
                    .check_request(&HttpBootstrap::new(binding, 999, target, None).unwrap())
                    .unwrap();
                let mut wrong_binding = binding;
                wrong_binding[0] ^= 1;
                assert_eq!(
                    original
                        .check_request(
                            &HttpBootstrap::new(wrong_binding, 1000, target, None).unwrap()
                        )
                        .unwrap_err()
                        .code,
                    ErrorCode::RevisionConflict
                );
                assert_eq!(
                    original
                        .check_request(&HttpBootstrap::new(binding, 1001, target, None).unwrap())
                        .unwrap_err()
                        .code,
                    ErrorCode::ResourceExhausted
                );
                let unbound = BoundControl {
                    identity: None,
                    deadline: scope.deadline(),
                };
                assert_eq!(
                    original.check(&unbound).unwrap_err().code,
                    ErrorCode::RevisionConflict
                );
                assert_eq!(
                    original.check_origin(&unbound).unwrap_err().code,
                    ErrorCode::SourceExpired
                );
                let changed_cutoff = BoundControl {
                    identity: Some(scope.identity().clone()),
                    deadline: scope.deadline() + Duration::from_millis(1),
                };
                assert_eq!(
                    original.check(&changed_cutoff).unwrap_err().code,
                    ErrorCode::RevisionConflict
                );
                Ok(HttpBootstrap::new(binding, 1000, target, None).unwrap())
            },
        )?;
        Ok(PreparedDownloadResources {
            output: Arc::new(StagedDownloadOutput::new(
                &self.root.join("output"),
                "fixture.bin",
            )?),
            verifier: match &self.gate {
                Some(gate) => Arc::new(cancel::GatedProbe(gate.clone())),
                None => Arc::new(BytesProbe),
            },
            maximum_bytes: 1000,
        })
    }
}
struct FailAfterAdoption {
    original: Arc<Preparation>,
    panic: bool,
}
impl DownloadPreparation for FailAfterAdoption {
    fn prepare_in_session(
        &self,
        input: &PreparedJobInput,
        blobs: &dyn SourceBlobStore,
        scope: &AttemptScope<'_>,
        custody: SourceCustody<'_>,
    ) -> Result<PreparedDownloadResources, ApiError> {
        let _resources = self
            .original
            .prepare_in_session(input, blobs, scope, custody)?;
        if self.panic {
            panic!("Original fixture panic after source adoption");
        }
        let mut error = ApiError::new(
            ErrorCode::NeedsInput,
            "Original fixture setup failure after source adoption",
        );
        error.request_id = "original-setup-after-adoption".into();
        Err(error)
    }
}
fn exercise_jobs_exit(expected_exit: u32) {
    exercise_jobs_preparation(expected_exit, None);
}
fn exercise_jobs_preparation(expected_exit: u32, prepare_failure: Option<bool>) {
    exercise_jobs_preparation_stage(expected_exit, prepare_failure, FailureStage::AfterAdoption);
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum FailureStage {
    AfterAdoption,
    Request,
    Replay,
}
struct ReplayPolicy(Option<bool>);
impl crate::application::browser_provider::MediaReplayPolicy for ReplayPolicy {
    fn authorize(
        &self,
        _: &kinekino_player::application::browser_network::NetworkResponse,
        _: &kinekino_player::application::browser_network::RequestHeaderEvidence,
    ) -> Result<(), ApiError> {
        if let Some(panic) = self.0 {
            if panic {
                panic!("Original fixture replay policy panicked");
            }
            let mut error = ApiError::new(
                ErrorCode::NeedsInput,
                "Original fixture replay policy rejected",
            );
            error.request_id = "original-replay-policy".into();
            return Err(error);
        }
        Ok(()) // Explicit synthetic fixture only; never a production policy.
    }
}
fn exercise_jobs_preparation_stage(
    expected_exit: u32,
    prepare_failure: Option<bool>,
    stage: FailureStage,
) {
    let worker =
        PathBuf::from(std::env::var_os("SOURCE_SDK_FIXTURE_BIN").expect("fixed fixture worker"));
    assert!(worker.is_absolute());
    let root = std::env::temp_dir().join(format!(
        "sdk-source-jobs-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    fs::create_dir(root.join("output")).unwrap();
    fs::create_dir(root.join("jobs")).unwrap();
    eprintln!("sdk_source_evidence={}", root.display());
    let consumed = Arc::new(AtomicUsize::new(0));
    let drops = Arc::new(AtomicUsize::new(0));
    let transport = Arc::new(AtomicUsize::new(0));
    let binding = Arc::new(Mutex::new(None));
    let preparation = Arc::new(Preparation {
        replay_failure: if stage == FailureStage::Replay {
            prepare_failure
        } else {
            None
        },
        request_failure: if stage == FailureStage::Request {
            prepare_failure
        } else {
            None
        },
        omit_body_end: false,
        source_exit: expected_exit,
        gate: None,
        root: root.clone(),
        worker,
        consumed: consumed.clone(),
        drops: drops.clone(),
        transport: transport.clone(),
        binding: binding.clone(),
    });
    let preparation: Arc<dyn DownloadPreparation> = match prepare_failure {
        Some(panic) if stage == FailureStage::AfterAdoption => Arc::new(FailAfterAdoption {
            original: preparation,
            panic,
        }),
        _ => preparation,
    };
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
                "sdk-fixture-request",
            )
            .unwrap();
        let job = loop {
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
            serde_json::to_vec_pretty(&job).unwrap(),
        )
        .unwrap();
        if let Some(panic) = prepare_failure {
            assert_eq!(
                job.state,
                if panic {
                    JobState::Failed
                } else {
                    JobState::NeedsInput
                }
            );
            assert_eq!(
                job.failure.as_ref().unwrap().code,
                if panic {
                    ErrorCode::Internal
                } else {
                    ErrorCode::NeedsInput
                }
            );
            assert!(!job.publication_started);
            assert!(!root.join("output/fixture.bin").exists());
        } else if expected_exit == 0 {
            assert_eq!(job.state, JobState::Completed, "{job:?}");
            assert_eq!(
                fs::read(root.join("output/fixture.bin")).unwrap(),
                b"offline-sdk-body"
            );
        } else {
            assert_eq!(job.state, JobState::Failed, "{job:?}");
            assert_eq!(job.failure.as_ref().unwrap().code, ErrorCode::EngineFailed);
            assert!(!job.publication_started);
            assert!(job.output.is_none());
            assert!(!root.join("output/fixture.bin").exists());
        }
        let original = binding.lock().unwrap();
        let original = original.as_ref().unwrap();
        assert_eq!(original.job_id(), &job.job_id);
        assert_eq!(original.attempt(), job.attempt.0);
        assert_eq!(original.input_sha256(), job.input_sha256);
        assert_eq!(
            consumed.load(Ordering::SeqCst),
            usize::from(prepare_failure.is_none())
        );
        assert_eq!(transport.load(Ordering::SeqCst), 1);
    }));
    // Keep the original owner and first unwind through real retaining shutdown.
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
                    eprintln!("sdk-source-jobs:original-shutdown-incomplete");
                    std::process::exit(124);
                }
                std::thread::sleep(Duration::from_millis(2));
            }
        }
    }
    if let Err(panic) = result {
        std::panic::resume_unwind(panic);
    }
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    if expected_exit != 0 || prepare_failure.is_some() {
        assert!(fs::read_dir(root.join("output")).unwrap().next().is_none());
    }
}
#[test]
#[ignore = "actual pinned child, retaining jobs host and filesystem CAS; centrally assigned"]
fn original_jobs_core_binding_drives_sdk_source_and_publishes_verified_fixture_bytes() {
    exercise_jobs_exit(0);
}
#[test]
#[ignore = "actual high-bit exit and original coordinator failure settlement; centrally assigned"]
fn high_bit_source_exit_fails_original_job_without_publication() {
    exercise_jobs_exit(0x8000_0001);
}
#[test]
#[ignore = "original prepared handles/document owner; error after custody must retain cleanup"]
fn setup_error_after_adoption_retains_original_source_without_launch() {
    exercise_jobs_preparation(0, Some(false));
}
#[test]
#[ignore = "original prepared handles/document owner; panic after custody must retain cleanup"]
fn setup_panic_after_adoption_retains_original_source_without_launch() {
    exercise_jobs_preparation(0, Some(true));
}
#[test]
#[ignore = "original document and unconnected IPC retained before fallible request preparation"]
fn request_error_retains_original_document_and_unconnected_ipc() {
    exercise_jobs_preparation_stage(0, Some(false), FailureStage::Request);
}
#[test]
#[ignore = "panic under original source mutex must retain document and prepared IPC cleanup"]
fn request_panic_retains_original_document_and_unconnected_ipc() {
    exercise_jobs_preparation_stage(0, Some(true), FailureStage::Request);
}
#[test]
#[ignore = "original replay policy failure occurs only after core custody"]
fn replay_policy_error_retains_original_document_before_grant() {
    exercise_jobs_preparation_stage(0, Some(false), FailureStage::Replay);
}
#[test]
#[ignore = "original replay policy panic retains original core owner and shared browser cleanup"]
fn replay_policy_panic_retains_original_document_before_grant() {
    exercise_jobs_preparation_stage(0, Some(true), FailureStage::Replay);
}

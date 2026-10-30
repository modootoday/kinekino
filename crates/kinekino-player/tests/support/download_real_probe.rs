//! Allocated local-file/real-ffprobe checkpoint. No network/provider or CLI claim.
use super::*;
use download::{OwnedTransferControl, OwnedTransferError};
use std::io::{Read, Seek, SeekFrom};
use std::sync::Mutex;

const MAX_MEDIA_BYTES: u64 = 256 * 1024 * 1024;
const FFPROBE_SHA256: &str = "efb014c8f39091c90f3616154fe20ab8470f5f943f6a48ec4fd30c479c7dd697";
const FFPROBE_BYTES: u64 = 82_620_928;
struct PreparationTrace {
    fixture_started: Instant,
    original_deadline: Mutex<Option<Instant>>,
    entered: AtomicBool,
    returned: AtomicBool,
    entered_us: AtomicU64,
    returned_us: AtomicU64,
    calls_started: AtomicU64,
    calls_completed: AtomicU64,
    control_total_ns: AtomicU64,
    control_max_ns: AtomicU64,
    last_control_returned_us: AtomicU64,
}
impl PreparationTrace {
    fn new() -> Self {
        Self {
            fixture_started: Instant::now(),
            original_deadline: Mutex::new(None),
            entered: AtomicBool::new(false),
            returned: AtomicBool::new(false),
            entered_us: AtomicU64::new(0),
            returned_us: AtomicU64::new(0),
            calls_started: AtomicU64::new(0),
            calls_completed: AtomicU64::new(0),
            control_total_ns: AtomicU64::new(0),
            control_max_ns: AtomicU64::new(0),
            last_control_returned_us: AtomicU64::new(0),
        }
    }
    fn elapsed_us(&self) -> u64 {
        self.fixture_started
            .elapsed()
            .as_micros()
            .min(u64::MAX as u128) as u64
    }
    fn snapshot(&self) -> Value {
        let deadline = self.original_deadline();
        json!({"meaning":"test-only progress; no authority or cleanup ACK",
            "original_deadline_captured":deadline.is_some(),
            "original_cutoff_from_fixture_us":deadline.map(|value| value.saturating_duration_since(self.fixture_started).as_micros()),
            "prepare_entered":self.entered.load(Ordering::Acquire),
            "prepare_returned":self.returned.load(Ordering::Acquire),
            "entered_us":self.entered_us.load(Ordering::Relaxed),
            "returned_us":self.returned_us.load(Ordering::Relaxed),
            "calls_started":self.calls_started.load(Ordering::Relaxed),
            "calls_completed":self.calls_completed.load(Ordering::Relaxed),
            "control_total_ns":self.control_total_ns.load(Ordering::Relaxed),
            "control_max_ns":self.control_max_ns.load(Ordering::Relaxed),
            "last_control_returned_us":self.last_control_returned_us.load(Ordering::Relaxed),
            "fixture_elapsed_us":self.elapsed_us()})
    }
    fn capture_deadline(&self, deadline: Instant) {
        let mut original = self
            .original_deadline
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if let Some(previous) = *original {
            assert_eq!(
                previous, deadline,
                "Original attempt cutoff cannot be renewed"
            );
        } else {
            *original = Some(deadline);
        }
    }
    fn original_deadline(&self) -> Option<Instant> {
        *self
            .original_deadline
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}
struct MeasuredControl<'a> {
    original: &'a dyn ExecutionControl,
    trace: &'a PreparationTrace,
}
impl ExecutionControl for MeasuredControl<'_> {
    fn execution_identity(&self) -> Option<&application::jobs::JobAttemptIdentityV1> {
        self.original.execution_identity()
    }
    fn deadline(&self) -> Instant {
        self.original.deadline()
    }
    fn cancellation_requested(&self) -> bool {
        self.trace.calls_started.fetch_add(1, Ordering::Relaxed);
        let started = Instant::now();
        let cancelled = self.original.cancellation_requested();
        let elapsed = started.elapsed().as_nanos().min(u64::MAX as u128) as u64;
        self.trace
            .control_total_ns
            .fetch_add(elapsed, Ordering::Relaxed);
        self.trace
            .control_max_ns
            .fetch_max(elapsed, Ordering::Relaxed);
        self.trace
            .last_control_returned_us
            .store(self.trace.elapsed_us(), Ordering::Relaxed);
        self.trace.calls_completed.fetch_add(1, Ordering::Release);
        cancelled
    }
    fn reserve_publication(&self) -> Result<(), ApiError> {
        self.original.reserve_publication()
    }
    fn begin_verification(&self) -> Result<(), ApiError> {
        self.original.begin_verification()
    }
}
struct TracedProbe {
    inner: MediaProbe,
    trace: Arc<PreparationTrace>,
}
impl DownloadVerifier for TracedProbe {
    fn verify(
        &self,
        path: &Path,
        expected: &MediaIdentity,
        control: &dyn ExecutionControl,
    ) -> Result<(), ApiError> {
        self.inner.verify(path, expected, control)
    }
    fn begin_owned(
        &self,
        lease: Arc<dyn VerificationLease>,
        expected: MediaIdentity,
        control: &dyn ExecutionControl,
    ) -> Result<OwnedVerification, ApiError> {
        self.trace
            .entered_us
            .store(self.trace.elapsed_us(), Ordering::Relaxed);
        self.trace.entered.store(true, Ordering::Release);
        let result = self.inner.begin_owned(
            lease,
            expected,
            &MeasuredControl {
                original: control,
                trace: self.trace.as_ref(),
            },
        );
        self.trace
            .returned_us
            .store(self.trace.elapsed_us(), Ordering::Relaxed);
        self.trace.returned.store(true, Ordering::Release);
        result
    }
}
fn write_trace(root: &Path, label: &str, trace: &PreparationTrace) -> std::io::Result<()> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root.join(label))?;
    serde_json::to_writer_pretty(&mut file, &trace.snapshot()).map_err(std::io::Error::other)?;
    file.sync_all()
}
fn terminal_at_original_cutoff(
    client: &dyn JobsApi,
    id: &JobId,
    trace: &PreparationTrace,
    root: &Path,
    intermediate: &mut Option<std::io::Result<()>>,
) -> JobRecord {
    let initial_observer = Instant::now() + Duration::from_secs(8);
    loop {
        let record = client.get(GetJob { job_id: id.clone() }).unwrap();
        let now = Instant::now();
        let cutoff = trace.original_deadline().unwrap_or(initial_observer);
        if now >= initial_observer && intermediate.is_none() {
            // This is an observation, not a cancel/re-submit/new deadline.
            *intermediate = Some(write_trace(
                root,
                "original-eight-second-observation.json",
                trace,
            ));
        }
        if record.state.terminal() {
            assert!(
                now <= cutoff,
                "Terminal was observed after original cutoff; preserve late result separately"
            );
            return record;
        }
        assert!(
            now < cutoff,
            "Original attempt cutoff expired without terminal; preserve original owner"
        );
        thread::sleep(Duration::from_millis(2).min(cutoff.saturating_duration_since(now)));
    }
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    path: PathBuf,
    sha256: String,
    bytes: u64,
    container: String,
    tracks: Vec<Track>,
    minimum_duration_us: u64,
    maximum_duration_us: u64,
    provenance: String,
    license: String,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Track {
    index: u32,
    kind: String,
    codec: String,
}
impl Fixture {
    fn plan(&self) -> ProbePlan {
        assert!(!self.provenance.trim().is_empty());
        assert!(!self.license.trim().is_empty());
        assert!((17..=MAX_MEDIA_BYTES).contains(&self.bytes));
        assert!(domain::identifiers::valid_sha256(&self.sha256));
        let plan = ProbePlan {
            identity: identity(),
            container: match self.container.as_str() {
                "mp4" => ProbeContainer::Mp4,
                "matroska" => ProbeContainer::Matroska,
                "mpegts" => ProbeContainer::MpegTs,
                _ => panic!("Explicit supported fixture container required"),
            },
            tracks: self
                .tracks
                .iter()
                .map(|track| ExpectedTrack {
                    index: track.index,
                    kind: match track.kind.as_str() {
                        "video" => TrackKind::Video,
                        "audio" => TrackKind::Audio,
                        "subtitle" => TrackKind::Subtitle,
                        _ => panic!("Explicit fixture track kind required"),
                    },
                    codec: Some(track.codec.clone()),
                })
                .collect(),
            minimum_duration_us: self.minimum_duration_us,
            maximum_duration_us: self.maximum_duration_us,
        };
        plan.validate().unwrap();
        plan
    }
}
#[derive(Clone, Copy)]
enum Case {
    Normal,
    Truncated,
    CancelSource,
    ExternalCancelled,
    CancelCleanupFailure,
}
impl Case {
    fn cancels_source(self) -> bool {
        matches!(
            self,
            Self::CancelSource | Self::ExternalCancelled | Self::CancelCleanupFailure
        )
    }
}
struct Signals {
    reading: AtomicBool,
    release: AtomicBool,
    closed: AtomicBool,
    close_attempts: AtomicU64,
}
fn check(control: &dyn ExecutionControl) -> Result<(), ApiError> {
    if control.cancellation_requested() {
        Err(ApiError::new(
            ErrorCode::Cancelled,
            "Original source cancelled",
        ))
    } else if Instant::now() >= control.deadline() {
        Err(ApiError::new(
            ErrorCode::Timeout,
            "Original source deadline expired",
        ))
    } else {
        Ok(())
    }
}
struct FileSource {
    file: fs::File,
    _guards: Vec<fs::File>,
    evidence: TransferEvidence,
    remaining: u64,
    signals: Arc<Signals>,
    wait_for_cancel: bool,
    external_cancel: bool,
    fail_first_close: bool,
}
impl DownloadSession for FileSource {
    fn read_owned(
        &mut self,
        bytes: &mut [u8],
        control: &dyn OwnedTransferControl,
    ) -> Result<usize, OwnedTransferError> {
        self.signals.reading.store(true, Ordering::Release);
        while self.wait_for_cancel && !self.signals.release.load(Ordering::Acquire) {
            if self.external_cancel {
                // Preserve the adapter's own error even while user cancellation races it.
                check(control)?;
            } else {
                control.checkpoint()?;
            }
            thread::sleep(Duration::from_millis(2));
        }
        control.checkpoint()?;
        self.read(bytes, control)
            .map_err(OwnedTransferError::Failure)
    }
    fn evidence(&self) -> &TransferEvidence {
        &self.evidence
    }
    fn check_current(&self, control: &dyn ExecutionControl) -> Result<(), ApiError> {
        check(control)
    }
    fn read(
        &mut self,
        bytes: &mut [u8],
        control: &dyn ExecutionControl,
    ) -> Result<usize, ApiError> {
        self.signals.reading.store(true, Ordering::Release);
        while self.wait_for_cancel && !self.signals.release.load(Ordering::Acquire) {
            check(control)?;
            thread::sleep(Duration::from_millis(2));
        }
        check(control)?;
        let limit = bytes.len().min(self.remaining as usize);
        let read = self.file.read(&mut bytes[..limit]).map_err(io_error)?;
        self.remaining -= read as u64;
        Ok(read)
    }
    fn close_owned(&mut self) -> Result<(), ApiError> {
        let attempt = self.signals.close_attempts.fetch_add(1, Ordering::AcqRel);
        if self.fail_first_close && attempt == 0 {
            return Err(ApiError::new(
                ErrorCode::EngineFailed,
                "Original source close failed",
            ));
        }
        self.signals.closed.store(true, Ordering::Release);
        Ok(())
    }
}
fn io_error(error: std::io::Error) -> ApiError {
    let mut failure = ApiError::new(ErrorCode::EngineFailed, "Fixed local fixture IO failed");
    failure.details =
        json!({"io_kind":format!("{:?}", error.kind()),"raw_os_error":error.raw_os_error()});
    failure
}
struct FileOpener {
    source: Option<FileSource>,
}
impl SourceOpener for FileOpener {
    fn owner_observation(&self) -> kinekino_jobs::application::owned_attempt::OwnerObservation {
        kinekino_jobs::application::owned_attempt::OwnerObservation::NotStarted
    }
    fn open(
        &mut self,
        control: &dyn ExecutionControl,
    ) -> Result<Box<dyn DownloadSession + Send>, ApiError> {
        check(control)?;
        self.source
            .take()
            .map(|source| Box::new(source) as Box<dyn DownloadSession + Send>)
            .ok_or_else(|| ApiError::new(ErrorCode::SourceExpired, "Original file admission spent"))
    }
    fn poll_close(&mut self, _: Instant) -> Result<bool, ApiError> {
        self.source.take();
        Ok(true)
    }
}
struct RealPreparation {
    fixture: Fixture,
    tool: Arc<ProbeToolPin>,
    root: PathBuf,
    case: Case,
    signals: Arc<Signals>,
    trace: Arc<PreparationTrace>,
}
impl DownloadPreparation for RealPreparation {
    fn prepare(
        &self,
        input: &PreparedJobInput,
        blobs: &dyn SourceBlobStore,
        scope: &AttemptScope<'_>,
    ) -> Result<PreparedDownload, ApiError> {
        self.trace.capture_deadline(scope.deadline());
        if blobs.read_utf8(&input.source)? != "synthetic-original-download-v1" {
            return Err(unsupported());
        }
        let plan = self.fixture.plan();
        let (_, guards) = immutable_storage_paths::root(
            self.fixture.path.parent().expect("Original local parent"),
        )?;
        let mut file = immutable_storage_paths::file(&self.fixture.path)?;
        if file.metadata().map_err(io_error)?.len() != self.fixture.bytes {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Fixture size changed",
            ));
        }
        let mut hash = Sha256::new();
        let mut buffer = [0_u8; 65536];
        loop {
            if scope.cancellation_requested() {
                return Err(ApiError::new(
                    ErrorCode::Cancelled,
                    "Original fixture preparation stopped",
                ));
            }
            if Instant::now() >= scope.deadline() {
                return Err(ApiError::new(
                    ErrorCode::Timeout,
                    "Original fixture preparation deadline expired",
                ));
            }
            let count = file.read(&mut buffer).map_err(io_error)?;
            if count == 0 {
                break;
            }
            hash.update(&buffer[..count]);
        }
        if format!("{:x}", hash.finalize()) != self.fixture.sha256 {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Fixture digest changed",
            ));
        }
        file.seek(SeekFrom::Start(0)).map_err(io_error)?;
        let length = if matches!(self.case, Case::Truncated) {
            16
        } else {
            self.fixture.bytes
        };
        Ok(PreparedDownload {
            source: Box::new(FileOpener {
                source: Some(FileSource {
                    file,
                    _guards: guards,
                    remaining: length,
                    evidence: TransferEvidence {
                        identity: identity(),
                        response_length: Some(length),
                        trusted_sha256: None,
                    },
                    signals: self.signals.clone(),
                    wait_for_cancel: self.case.cancels_source(),
                    external_cancel: matches!(self.case, Case::ExternalCancelled),
                    fail_first_close: matches!(self.case, Case::CancelCleanupFailure),
                }),
            }),
            output: Arc::new(staged_output::StagedDownloadOutput::new(
                &self.root.join("output"),
                "result.media",
            )?),
            verifier: Arc::new(TracedProbe {
                inner: MediaProbe {
                    tool: self.tool.clone(),
                    plan: Arc::new(plan),
                    evidence_root: self.root.join("probe"),
                    cleanup_reserve: Duration::from_secs(2),
                    sequence: AtomicU64::new(0),
                },
                trace: self.trace.clone(),
            }),
            maximum_bytes: MAX_MEDIA_BYTES,
        })
    }
}
fn exercise(case: Case) {
    let manifest = PathBuf::from(
        std::env::var_os("DOWNLOAD_REAL_MEDIA_MANIFEST")
            .expect("Reviewed local fixture manifest required"),
    );
    assert!(fs::metadata(&manifest).unwrap().len() <= 65536);
    let fixture: Fixture = serde_json::from_slice(&fs::read(manifest).unwrap()).unwrap();
    fixture.plan();
    let source_hash = fixture.sha256.clone();
    let source_bytes = fixture.bytes;
    let tool = Arc::new(ProbeToolPin {
        executable: PathBuf::from(
            std::env::var_os("DOWNLOAD_REAL_FFPROBE_BIN").expect("Allocated real ffprobe required"),
        ),
        sha256: std::env::var("DOWNLOAD_REAL_FFPROBE_SHA256").expect("Reviewed tool hash required"),
        bytes: std::env::var("DOWNLOAD_REAL_FFPROBE_BYTES")
            .expect("Reviewed tool size required")
            .parse()
            .unwrap(),
    });
    assert_eq!(
        tool.sha256, FFPROBE_SHA256,
        "Reviewed real tool pin required; no synthetic substitute"
    );
    assert_eq!(tool.bytes, FFPROBE_BYTES);
    assert_eq!(tool.executable.file_name().unwrap(), "ffprobe.exe");
    let label = match case {
        Case::Normal => "real-normal",
        Case::Truncated => "real-truncated",
        Case::CancelSource => "real-source-cancel",
        Case::ExternalCancelled => "external-cancelled-user-race",
        Case::CancelCleanupFailure => "local-cancel-close-failure",
    };
    let root = PathBuf::from(
        std::env::var_os("DOWNLOAD_REAL_EVIDENCE_ROOT")
            .expect("Allocated fresh evidence parent required"),
    )
    .join(label);
    fs::create_dir(&root).unwrap();
    for directory in ["jobs", "output", "probe"] {
        fs::create_dir(root.join(directory)).unwrap();
    }
    let signals = Arc::new(Signals {
        reading: AtomicBool::new(false),
        release: AtomicBool::new(false),
        closed: AtomicBool::new(false),
        close_attempts: AtomicU64::new(0),
    });
    let trace = Arc::new(PreparationTrace::new());
    let host = kinekino_player::infrastructure::jobs::start_retaining_host_with_executor(
        &root.join("jobs"),
        Arc::new(Executor {
            owned: DownloadOwnedExecutor {
                preparation: Arc::new(RealPreparation {
                    fixture,
                    tool,
                    root: root.clone(),
                    case,
                    signals: signals.clone(),
                    trace: trace.clone(),
                }),
            },
        }),
    )
    .unwrap();
    let mut intermediate_trace = None;
    let action = catch_unwind(AssertUnwindSafe(|| {
        let client = host.client();
        let admitted = client
            .submit(
                SubmitJob {
                    operation: "production.download_media".into(),
                    parameters: json!({}),
                    expected_source_sha256: None,
                    execution_timeout_ms: DecimalU64(15_000),
                },
                label,
            )
            .unwrap();
        let id = admitted.job.job_id;
        if case.cancels_source() {
            wait_until(|| signals.reading.load(Ordering::Acquire));
            let current = client.get(GetJob { job_id: id.clone() }).unwrap();
            client
                .cancel(CancelJob {
                    job_id: id.clone(),
                    expected_attempt: current.attempt,
                })
                .unwrap();
        }
        let record = terminal_at_original_cutoff(
            client.as_ref(),
            &id,
            trace.as_ref(),
            &root,
            &mut intermediate_trace,
        );
        assert!(signals.closed.load(Ordering::Acquire));
        if matches!(case, Case::Normal) {
            assert_eq!(record.state, JobState::Completed);
            let output = record.output.as_ref().unwrap();
            assert_eq!(output.bytes, DecimalU64(source_bytes));
            assert_eq!(output.sha256, source_hash);
            assert_eq!(
                format!("{:x}", Sha256::digest(fs::read(&output.path).unwrap())),
                source_hash
            );
            let events = client
                .watch(WatchJobs {
                    after_sequence: DecimalU64(0),
                    limit: Some(128),
                })
                .unwrap();
            let records: Vec<_> = events
                .events
                .iter()
                .filter(|event| event.job.job_id == id)
                .collect();
            assert_eq!(
                records
                    .iter()
                    .filter(|event| event.job.state == JobState::Completed)
                    .count(),
                1
            );
            assert_eq!(
                records
                    .windows(2)
                    .filter(
                        |pair| !pair[0].job.publication_started && pair[1].job.publication_started
                    )
                    .count(),
                1
            );
        } else {
            assert_ne!(record.state, JobState::Completed);
            assert!(record.output.is_none());
            assert!(!record.publication_started);
            assert_eq!(fs::read_dir(root.join("output")).unwrap().count(), 0);
        }
        if case.cancels_source() {
            if matches!(case, Case::CancelSource) {
                assert_eq!(record.state, JobState::Cancelled);
                assert!(record.failure.is_none());
            } else {
                assert_eq!(record.state, JobState::Failed);
                let failure = record.failure.as_ref().unwrap();
                if matches!(case, Case::ExternalCancelled) {
                    assert_eq!(failure.code, ErrorCode::Cancelled);
                    assert_eq!(failure.message, "Original source cancelled");
                } else {
                    assert_eq!(failure.code, ErrorCode::EngineFailed);
                    assert_eq!(failure.message, "Original source close failed");
                    assert!(signals.close_attempts.load(Ordering::Acquire) >= 2);
                }
            }
            assert!(record.cancel_requested);
            assert!(
                descendants(&root.join("probe"), "stdout.raw").is_empty(),
                "Source cancellation precedes probe; no ffprobe cancellation claim"
            );
        } else {
            assert_eq!(
                descendants(&root.join("probe"), "stdout.raw").len(),
                2,
                "Capabilities and inspect raw evidence required"
            );
            assert_eq!(descendants(&root.join("probe"), "stderr.raw").len(), 2);
            for phase in ["capabilities", "inspect"] {
                let receipts = descendants(&root.join("probe"), &format!("{phase}-close-1.json"));
                assert_eq!(receipts.len(), 1, "Original phase close receipt required");
                let receipt: Value =
                    serde_json::from_slice(&fs::read(&receipts[0]).unwrap()).unwrap();
                assert_eq!(receipt["family"], "Empty");
                assert_eq!(receipt["forced"], false);
                if phase == "capabilities" || matches!(case, Case::Normal) {
                    assert_eq!(receipt["child_exit"], 0);
                } else {
                    assert!(receipt["child_exit"].as_u64().is_some_and(|code| code != 0));
                }
            }
        }
    }));
    signals.release.store(true, Ordering::Release);
    // Original host remains outside the action/unwind while diagnostics return
    // plain Results. Do not panic on diagnostics before original close/conclude.
    let before_close_trace = write_trace(&root, "prepare-before-host-close.json", trace.as_ref());
    let close_failure = host_closeout::close(host).err();
    let after_close_trace = write_trace(&root, "prepare-after-host-close.json", trace.as_ref());
    host_closeout::conclude(action, close_failure);
    if let Some(result) = intermediate_trace {
        result.unwrap();
    }
    before_close_trace.unwrap();
    after_close_trace.unwrap();
}
#[test]
#[ignore = "Requires separately allocated pinned real ffprobe and reviewed local media"]
fn real_ffprobe_original_file_single_publication() {
    kinekino_platform::install();
    exercise(Case::Normal);
}
#[test]
#[ignore = "Requires separately allocated pinned real ffprobe and reviewed local media"]
fn real_ffprobe_truncated_original_is_not_published() {
    kinekino_platform::install();
    exercise(Case::Truncated);
}
#[test]
#[ignore = "Requires separately allocated reviewed local media; cancels source before probe"]
fn real_original_source_cancel_closes_without_publication() {
    kinekino_platform::install();
    exercise(Case::CancelSource);
}
#[test]
#[ignore = "Requires separately allocated reviewed local media; external error race before probe"]
fn external_cancelled_error_with_user_cancel_remains_failed() {
    kinekino_platform::install();
    exercise(Case::ExternalCancelled);
}
#[test]
#[ignore = "Requires separately allocated reviewed local media; original close failure before probe"]
fn local_cancel_cleanup_failure_cannot_acknowledge_cancel() {
    kinekino_platform::install();
    exercise(Case::CancelCleanupFailure);
}

#[test]
fn legacy_owned_read_preserves_complete_external_error_during_cancel() {
    kinekino_platform::install();
    struct CancelledControl {
        deadline: Instant,
        checkpoints: AtomicU64,
    }
    impl ExecutionControl for CancelledControl {
        fn cancellation_requested(&self) -> bool {
            true
        }
        fn deadline(&self) -> Instant {
            self.deadline
        }
        fn reserve_publication(&self) -> Result<(), ApiError> {
            panic!("No publication")
        }
        fn begin_verification(&self) -> Result<(), ApiError> {
            panic!("No verification")
        }
    }
    impl OwnedTransferControl for CancelledControl {
        fn checkpoint(&self) -> Result<(), OwnedTransferError> {
            self.checkpoints.fetch_add(1, Ordering::AcqRel);
            panic!("Default adapter must not manufacture local cancellation")
        }
    }
    struct LegacySource {
        evidence: TransferEvidence,
        error: Option<ApiError>,
    }
    // Deliberately inherits the real default read_owned adapter.
    impl DownloadSession for LegacySource {
        fn evidence(&self) -> &TransferEvidence {
            &self.evidence
        }
        fn check_current(&self, _: &dyn ExecutionControl) -> Result<(), ApiError> {
            Ok(())
        }
        fn read(
            &mut self,
            _: &mut [u8],
            control: &dyn ExecutionControl,
        ) -> Result<usize, ApiError> {
            assert!(control.cancellation_requested());
            Err(self.error.take().expect("Original error consumed once"))
        }
    }
    let original = ApiError {
        code: ErrorCode::Cancelled,
        message: "Transport independently stopped".into(),
        retryable: true,
        details: json!({"source":"original-transport", "nested":{"os_code":995}}),
        remediation: "Inspect the original transport receipt".into(),
        request_id: "legacy-read-original-request".into(),
    };
    let expected = serde_json::to_value(&original).unwrap();
    let mut source = LegacySource {
        evidence: TransferEvidence {
            identity: identity(),
            response_length: None,
            trusted_sha256: None,
        },
        error: Some(original),
    };
    let control = CancelledControl {
        deadline: Instant::now() + Duration::from_secs(1),
        checkpoints: AtomicU64::new(0),
    };
    match source.read_owned(&mut [0; 1], &control) {
        Err(OwnedTransferError::Failure(error)) => {
            assert_eq!(serde_json::to_value(error).unwrap(), expected)
        }
        Err(OwnedTransferError::LocalCancellation(_)) => {
            panic!("External error became local marker")
        }
        Ok(_) => panic!("Original transport failure was lost"),
    }
    assert!(source.error.is_none());
    assert_eq!(control.checkpoints.load(Ordering::Acquire), 0);
}

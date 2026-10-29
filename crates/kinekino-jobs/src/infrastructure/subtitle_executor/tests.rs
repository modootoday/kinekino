use super::*;
use crate::{
    application::{JobStore, SourceBlobStore, run_queued},
    domain::{JobId, JobRecord, JobState},
    infrastructure::JobFiles,
};
use kinekino_subtitles::application::{convert, export_range, storage::SubtitleStorage};
use std::{
    fs::{self, File, OpenOptions},
    os::windows::fs::OpenOptionsExt,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};
const SOURCE: &str =
    "1\n00:00:01,000 --> 00:00:02,000\n한국어\n\n2\n00:00:03,000 --> 00:00:04,000\n日本語\n";

// Synthetic rooted-path adapter for this fixture only; not a production security proof.
struct Paths {
    bad_readback: bool,
}
impl RootedJobPaths for Paths {
    fn root(&self, path: &Path) -> Result<(PathBuf, Vec<File>), ApiError> {
        Ok((
            path.canonicalize()
                .map_err(kinekino_core::file_publication::io_error)?,
            vec![self.directory(path)?],
        ))
    }
    fn directory(&self, path: &Path) -> Result<File, ApiError> {
        OpenOptions::new()
            .read(true)
            .share_mode(3)
            .custom_flags(0x02000000)
            .open(path)
            .map_err(kinekino_core::file_publication::io_error)
    }
    fn file(&self, path: &Path) -> Result<File, ApiError> {
        let file = if self.bad_readback && path.file_name().is_some_and(|name| name == "out.vtt") {
            path.with_file_name("wrong-bytes.txt")
        } else {
            path.to_path_buf()
        };
        OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(file)
            .map_err(kinekino_core::file_publication::io_error)
    }
    fn lock(&self, path: &Path) -> Result<File, ApiError> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .share_mode(0)
            .open(path)
            .map_err(kinekino_core::file_publication::io_error)
    }
}
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let parent = std::env::temp_dir();
        let path = parent.join(format!(
            "media-range-job-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        fs::create_dir(path.join("store")).unwrap();
        assert_eq!(
            path.canonicalize().unwrap().parent(),
            Some(parent.canonicalize().unwrap().as_path())
        );
        fs::write(path.join("source.srt"), SOURCE).unwrap();
        fs::write(path.join("wrong-bytes.txt"), "wrong").unwrap();
        Self(path)
    }
    fn store(&self) -> JobFiles {
        JobFiles::open(
            &self.0.join("store"),
            Arc::new(Paths {
                bad_readback: false,
            }),
        )
        .unwrap()
    }
    fn payload(&self) -> Value {
        serde_json::json!({"input_path":self.0.join("source.srt"),"output_path":self.0.join("out.vtt"),"input_format":"srt","output_format":"vtt","range":{"in_us":"1500000","out_us":"3500000"}})
    }
    fn executor(&self) -> SubtitleJobExecutor {
        SubtitleJobExecutor::new(Arc::new(Paths {
            bad_readback: false,
        }))
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
struct Control {
    cancel: bool,
    deadline: Instant,
    reserve_error: Option<ErrorCode>,
    expire_on_reserve: bool,
    reserves: AtomicUsize,
    verifies: AtomicUsize,
}
impl Control {
    fn live() -> Self {
        Self {
            cancel: false,
            deadline: Instant::now() + Duration::from_secs(5),
            reserve_error: None,
            expire_on_reserve: false,
            reserves: AtomicUsize::new(0),
            verifies: AtomicUsize::new(0),
        }
    }
}
impl ExecutionControl for Control {
    fn cancellation_requested(&self) -> bool {
        self.cancel
    }
    fn deadline(&self) -> Instant {
        if self.expire_on_reserve && self.reserves.load(Ordering::SeqCst) > 0 {
            Instant::now() - Duration::from_millis(1)
        } else {
            self.deadline
        }
    }
    fn reserve_publication(&self) -> Result<(), ApiError> {
        self.reserves.fetch_add(1, Ordering::SeqCst);
        if let Some(code) = self.reserve_error {
            Err(ApiError::new(code, "injected publication gate"))
        } else {
            Ok(())
        }
    }
    fn begin_verification(&self) -> Result<(), ApiError> {
        self.verifies.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}
struct Oracle<'a> {
    source: &'a str,
    output: RefCell<Option<String>>,
}
impl SubtitleStorage for Oracle<'_> {
    fn read_utf8(&self, _: &str) -> Result<String, ApiError> {
        Ok(self.source.into())
    }
    fn publish_new(&self, _: &str, text: &str) -> Result<(), ApiError> {
        *self.output.borrow_mut() = Some(text.into());
        Ok(())
    }
}

#[test]
fn range_job_reuses_synchronous_bytes_receipt_and_verified_snapshot() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let executor = fixture.executor();
    let payload = fixture.payload();
    let prepared = executor
        .prepare(
            "subtitles.export_range",
            payload.clone(),
            Some(&digest(SOURCE.as_bytes())),
            &store,
        )
        .unwrap();
    assert_eq!(store.read_utf8(&prepared.source).unwrap(), SOURCE);
    let oracle = Oracle {
        source: SOURCE,
        output: RefCell::new(None),
    };
    let expected = export_range::execute(&oracle, &payload).unwrap();
    let control = Control::live();
    let actual = executor.execute(&prepared, &store, &control).unwrap();
    let bytes = fs::read(fixture.0.join("out.vtt")).unwrap();
    assert_eq!(bytes, oracle.output.into_inner().unwrap().as_bytes());
    assert_eq!(actual.receipt, expected);
    assert_eq!(actual.sha256, digest(&bytes));
    assert_eq!(actual.bytes.0, bytes.len() as u64);
    assert_eq!(actual.receipt["cue_count"], 2);
    assert!(
        String::from_utf8(bytes)
            .unwrap()
            .contains("00:00:00.000 --> 00:00:00.500")
    );
    assert_eq!(
        fs::read_to_string(fixture.0.join("source.srt")).unwrap(),
        SOURCE
    );
    assert_eq!(control.reserves.load(Ordering::SeqCst), 1);
    assert_eq!(control.verifies.load(Ordering::SeqCst), 1);
}

#[test]
fn convert_defaults_and_execution_remain_equal_to_existing_use_case() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let executor = fixture.executor();
    let mut payload = fixture.payload();
    payload.as_object_mut().unwrap().remove("range");
    let typed: convert::ConvertSubtitles = serde_json::from_value(payload.clone()).unwrap();
    assert_eq!(
        executor
            .normalize_parameters("subtitles.convert", payload.clone())
            .unwrap(),
        serde_json::to_value(typed).unwrap()
    );
    let prepared = executor
        .prepare("subtitles.convert", payload.clone(), None, &store)
        .unwrap();
    let oracle = Oracle {
        source: SOURCE,
        output: RefCell::new(None),
    };
    let expected = convert::execute(&oracle, &payload).unwrap();
    let actual = executor
        .execute(&prepared, &store, &Control::live())
        .unwrap();
    assert_eq!(actual.receipt, expected);
    assert_eq!(
        fs::read_to_string(fixture.0.join("out.vtt")).unwrap(),
        oracle.output.into_inner().unwrap()
    );
}

#[test]
fn source_mismatch_preserves_captured_blob_and_does_not_publish() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let executor = fixture.executor();
    assert_eq!(
        executor
            .prepare(
                "subtitles.export_range",
                fixture.payload(),
                Some(&"f".repeat(64)),
                &store
            )
            .unwrap_err()
            .code,
        ErrorCode::SourceExpired
    );
    let prepared = executor
        .prepare("subtitles.export_range", fixture.payload(), None, &store)
        .unwrap();
    fs::write(
        fixture.0.join("source.srt"),
        SOURCE.replace("日本語", "changed"),
    )
    .unwrap();
    assert_eq!(
        executor
            .execute(&prepared, &store, &Control::live())
            .unwrap_err()
            .code,
        ErrorCode::SourceExpired
    );
    assert_eq!(store.read_utf8(&prepared.source).unwrap(), SOURCE);
    assert!(!fixture.0.join("out.vtt").exists());
}

#[test]
fn cancellation_and_deadline_stop_before_or_at_publication_without_output() {
    for phase in 0..4 {
        let fixture = Fixture::new();
        let store = fixture.store();
        let executor = fixture.executor();
        let prepared = executor
            .prepare("subtitles.export_range", fixture.payload(), None, &store)
            .unwrap();
        let mut control = Control::live();
        match phase {
            0 => control.cancel = true,
            1 => control.deadline = Instant::now() - Duration::from_millis(1),
            2 => control.reserve_error = Some(ErrorCode::Cancelled),
            _ => control.expire_on_reserve = true,
        }
        let code = executor
            .execute(&prepared, &store, &control)
            .unwrap_err()
            .code;
        assert_eq!(
            code,
            if phase == 0 || phase == 2 {
                ErrorCode::Cancelled
            } else {
                ErrorCode::Timeout
            }
        );
        assert_eq!(
            control.reserves.load(Ordering::SeqCst),
            if phase < 2 { 0 } else { 1 }
        );
        assert_eq!(control.verifies.load(Ordering::SeqCst), 0);
        assert!(!fixture.0.join("out.vtt").exists());
        assert_eq!(
            fs::read_to_string(fixture.0.join("source.srt")).unwrap(),
            SOURCE
        );
    }
}

#[test]
fn no_overwrite_and_failed_readback_preserve_existing_or_published_file() {
    for bad_readback in [false, true] {
        let fixture = Fixture::new();
        let store = fixture.store();
        let executor = SubtitleJobExecutor::new(Arc::new(Paths { bad_readback }));
        let prepared = executor
            .prepare("subtitles.export_range", fixture.payload(), None, &store)
            .unwrap();
        if !bad_readback {
            fs::write(fixture.0.join("out.vtt"), "existing").unwrap();
        }
        let error = executor
            .execute(&prepared, &store, &Control::live())
            .unwrap_err();
        assert_eq!(
            error.code,
            if bad_readback {
                ErrorCode::EngineFailed
            } else {
                ErrorCode::RevisionConflict
            }
        );
        let output = fs::read_to_string(fixture.0.join("out.vtt")).unwrap();
        if bad_readback {
            assert!(output.starts_with("WEBVTT"));
        } else {
            assert_eq!(output, "existing");
        }
    }
}

#[test]
fn range_losses_need_explicit_approval_and_empty_range_never_publishes() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let executor = fixture.executor();
    let mut payload = fixture.payload();
    payload["range"]["in_us"] = serde_json::json!("1500001");
    let prepared = executor
        .prepare("subtitles.export_range", payload.clone(), None, &store)
        .unwrap();
    let error = executor
        .execute(&prepared, &store, &Control::live())
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::NeedsInput);
    assert!(
        error.details["losses"]
            .as_array()
            .is_some_and(|items| !items.is_empty())
    );
    assert!(!fixture.0.join("out.vtt").exists());
    payload["allow_loss"] = serde_json::json!(true);
    let prepared = executor
        .prepare("subtitles.export_range", payload, None, &store)
        .unwrap();
    executor
        .execute(&prepared, &store, &Control::live())
        .unwrap();
    let empty = Fixture::new();
    let store = empty.store();
    let mut payload = empty.payload();
    payload["range"] = serde_json::json!({"in_us":"9000000","out_us":"10000000"});
    let prepared = executor
        .prepare("subtitles.export_range", payload, None, &store)
        .unwrap();
    assert_eq!(
        executor
            .execute(&prepared, &store, &Control::live())
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert!(!empty.0.join("out.vtt").exists());
}

#[test]
fn publication_cancel_cas_preserves_existing_coordinator_winner_for_range_jobs() {
    use crate::application::request_cancel;
    use crate::domain::CancelDisposition;
    use std::sync::{Mutex, mpsc};
    struct Paused {
        inner: SubtitleJobExecutor,
        arrived: mpsc::Sender<()>,
        release: Mutex<mpsc::Receiver<()>>,
        after: bool,
    }
    struct Gate<'a> {
        inner: &'a dyn ExecutionControl,
        owner: &'a Paused,
    }
    impl ExecutionControl for Gate<'_> {
        fn cancellation_requested(&self) -> bool {
            self.inner.cancellation_requested()
        }
        fn deadline(&self) -> Instant {
            self.inner.deadline()
        }
        fn begin_verification(&self) -> Result<(), ApiError> {
            self.inner.begin_verification()
        }
        fn reserve_publication(&self) -> Result<(), ApiError> {
            if self.owner.after {
                self.inner.reserve_publication()?;
            }
            self.owner
                .arrived
                .send(())
                .map_err(|_| ApiError::new(ErrorCode::Internal, "fixture observer gone"))?;
            self.owner
                .release
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_secs(5))
                .map_err(|_| ApiError::new(ErrorCode::Timeout, "fixture release watchdog"))?;
            if !self.owner.after {
                self.inner.reserve_publication()?;
            }
            Ok(())
        }
    }
    impl JobExecutor for Paused {
        fn supports(&self, operation: &str) -> bool {
            self.inner.supports(operation)
        }
        fn prepare(
            &self,
            operation: &str,
            payload: Value,
            expected: Option<&str>,
            blobs: &dyn SourceBlobStore,
        ) -> Result<PreparedJobInput, ApiError> {
            self.inner.prepare(operation, payload, expected, blobs)
        }
        fn execute(
            &self,
            input: &PreparedJobInput,
            blobs: &dyn SourceBlobStore,
            control: &dyn ExecutionControl,
        ) -> Result<VerifiedJobOutput, ApiError> {
            self.inner.execute(
                input,
                blobs,
                &Gate {
                    inner: control,
                    owner: self,
                },
            )
        }
    }
    for after in [false, true] {
        let fixture = Fixture::new();
        let store = Arc::new(fixture.store());
        let executor = fixture.executor();
        let input = executor
            .prepare(
                "subtitles.export_range",
                fixture.payload(),
                None,
                store.as_ref(),
            )
            .unwrap();
        let id: JobId = format!("job-{:032x}", 1).try_into().unwrap();
        let job = JobRecord::queued(id.clone(), input).unwrap();
        store.admit(&job).unwrap();
        let (arrived, wait) = mpsc::channel();
        let (release, proceed) = mpsc::channel();
        let target = store.clone();
        let worker = std::thread::spawn(move || {
            run_queued(
                target.as_ref(),
                &Paused {
                    inner: executor,
                    arrived,
                    release: Mutex::new(proceed),
                    after,
                },
                target.as_ref(),
                &id,
                1,
            )
        });
        if let Err(error) = wait.recv_timeout(Duration::from_secs(5)) {
            drop(release);
            let result = worker.join();
            panic!("publication boundary missing: {error}; worker={result:?}");
        }
        let (_, disposition) = request_cancel(store.as_ref(), &job.job_id, 1).unwrap();
        assert_eq!(
            disposition,
            if after {
                CancelDisposition::PublicationInProgress
            } else {
                CancelDisposition::Requested
            }
        );
        release.send(()).unwrap();
        let result = worker.join().unwrap().unwrap();
        assert_eq!(
            result.state,
            if after {
                JobState::Completed
            } else {
                JobState::Cancelled
            }
        );
        assert_eq!(fixture.0.join("out.vtt").exists(), after);
    }
}

#[test]
fn capability_and_idempotency_are_operation_specific_without_duplicate_publication() {
    use crate::{
        application::{GetJob, JobIdentitySource, SubmitJob},
        infrastructure::WorkerHost,
    };
    struct Ids(std::sync::atomic::AtomicU64);
    impl JobIdentitySource for Ids {
        fn next(&self) -> Result<JobId, ApiError> {
            Ok(
                format!("job-{:032x}", self.0.fetch_add(1, Ordering::SeqCst))
                    .try_into()
                    .unwrap(),
            )
        }
    }
    let fixture = Fixture::new();
    let store = Arc::new(fixture.store());
    let host = WorkerHost::start(
        store.clone(),
        Arc::new(fixture.executor()),
        store.clone(),
        Arc::new(Ids(std::sync::atomic::AtomicU64::new(1))),
    )
    .unwrap();
    let client = host.client();
    assert_eq!(
        client.capability().operations,
        vec![
            "subtitles.convert",
            "subtitles.export_range",
            "subtitles.from_transcript"
        ]
    );
    let mut convert = fixture.payload();
    convert.as_object_mut().unwrap().remove("range");
    let submit = |operation: &str, parameters: Value| SubmitJob {
        operation: operation.into(),
        parameters,
        expected_source_sha256: None,
        execution_timeout_ms: DecimalU64(30000),
    };
    let first = client
        .submit(submit("subtitles.convert", convert.clone()), "shared-key")
        .unwrap();
    let duplicate = client
        .submit(submit("subtitles.convert", convert), "shared-key")
        .unwrap();
    assert!(!duplicate.created);
    assert_eq!(first.job.job_id, duplicate.job.job_id);
    assert_eq!(
        client
            .submit(
                submit("subtitles.export_range", fixture.payload()),
                "shared-key"
            )
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    let mut range = fixture.payload();
    range["output_path"] = serde_json::json!(fixture.0.join("range-output.vtt"));
    let first = client
        .submit(submit("subtitles.export_range", range.clone()), "range-key")
        .unwrap();
    let same = client
        .submit(submit("subtitles.export_range", range.clone()), "range-key")
        .unwrap();
    assert!(!same.created);
    assert_eq!(same.job.job_id, first.job.job_id);
    range["range"]["out_us"] = serde_json::json!("3600000");
    assert_eq!(
        client
            .submit(submit("subtitles.export_range", range), "range-key")
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let job = client
            .get(GetJob {
                job_id: first.job.job_id.clone(),
            })
            .unwrap();
        if job.state == JobState::Completed {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "range job did not complete: {:?}",
            job.state
        );
        std::thread::yield_now();
    }
    host.shutdown().unwrap();
    let events = store.read_events(0, 200).unwrap().events;
    assert_eq!(
        events
            .iter()
            .filter(|event| event.job.state == JobState::Queued)
            .count(),
        2
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| event.job.state == JobState::Completed)
            .count(),
        2
    );
}

#[test]
fn vtt_to_srt_range_job_matches_original_typed_use_case() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let executor = fixture.executor();
    let source = "WEBVTT\n\n00:00:01.000 --> 00:00:02.000\n한국어\n\n00:00:03.000 --> 00:00:04.000\n日本語\n";
    let input = fixture.0.join("source.vtt");
    fs::write(&input, source).unwrap();
    let mut payload = fixture.payload();
    payload["input_path"] = serde_json::json!(input);
    payload["input_format"] = serde_json::json!("vtt");
    payload["output_path"] = serde_json::json!(fixture.0.join("out.srt"));
    payload["output_format"] = serde_json::json!("srt");
    payload["allow_loss"] = serde_json::json!(true);
    let oracle = Oracle {
        source,
        output: RefCell::new(None),
    };
    let expected = export_range::execute(&oracle, &payload).unwrap();
    let prepared = executor
        .prepare("subtitles.export_range", payload, None, &store)
        .unwrap();
    let result = executor
        .execute(&prepared, &store, &Control::live())
        .unwrap();
    assert_eq!(result.receipt, expected);
    assert_eq!(
        fs::read_to_string(fixture.0.join("out.srt")).unwrap(),
        oracle.output.into_inner().unwrap()
    );
    assert_eq!(fs::read_to_string(input).unwrap(), source);
}

#[test]
fn operation_contracts_remain_distinct_and_unknown_operations_are_not_supported() {
    let fixture = Fixture::new();
    let executor = fixture.executor();
    let payload = fixture.payload();
    assert_eq!(
        executor
            .normalize_parameters("subtitles.convert", payload.clone())
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    let mut missing = payload.clone();
    missing.as_object_mut().unwrap().remove("range");
    assert_eq!(
        executor
            .normalize_parameters("subtitles.export_range", missing)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    let mut extra = payload;
    extra["unknown"] = serde_json::json!(true);
    assert_eq!(
        executor
            .normalize_parameters("subtitles.export_range", extra)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert!(!executor.supports("subtitles.detect"));
    assert_eq!(
        executor.execution_timeout_limit_ms("subtitles.detect"),
        None
    );
    assert_eq!(
        executor.execution_timeout_limit_ms("subtitles.convert"),
        Some(120000)
    );
    assert_eq!(
        executor.execution_timeout_limit_ms("subtitles.export_range"),
        Some(120000)
    );
}

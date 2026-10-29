use super::*;
use crate::{
    application::SourceBlobStore,
    domain::{JobId, PreparedJobInput},
};
use std::{fs::OpenOptions, os::windows::fs::OpenOptionsExt};
struct FixturePaths;
impl RootedJobPaths for FixturePaths {
    fn root(&self, path: &Path) -> Result<(PathBuf, Vec<File>), ApiError> {
        Ok((
            path.canonicalize().map_err(io_error)?,
            vec![self.directory(path)?],
        ))
    }
    fn directory(&self, path: &Path) -> Result<File, ApiError> {
        OpenOptions::new()
            .read(true)
            .share_mode(3)
            .custom_flags(0x02000000)
            .open(path)
            .map_err(io_error)
    }
    fn file(&self, path: &Path) -> Result<File, ApiError> {
        OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(path)
            .map_err(io_error)
    }
    fn lock(&self, path: &Path) -> Result<File, ApiError> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .share_mode(0)
            .open(path)
            .map_err(|error| {
                if error.raw_os_error() == Some(32) {
                    ApiError::new(ErrorCode::RevisionConflict, "fixture writer active")
                } else {
                    io_error(error)
                }
            })
    }
}
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "media-job-files-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn open(&self) -> JobFiles {
        JobFiles::open(&self.0, Arc::new(FixturePaths)).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn queued(store: &JobFiles, n: u64) -> JobRecord {
    let source = store.put_utf8("원문 日本語\n").unwrap();
    let id: JobId = format!("job-{n:032x}").try_into().unwrap();
    JobRecord::queued(
        id,
        PreparedJobInput {
            operation: "subtitles.convert".into(),
            parameters: serde_json::json!({"input_format":"srt"}),
            source,
            execution_timeout_ms: DecimalU64(30000),
        },
    )
    .unwrap()
}
#[test]
fn source_snapshots_events_and_historical_list_cursor_survive_reopen() {
    let fixture = Fixture::new();
    let store = fixture.open();
    let first = queued(&store, 1);
    store.publish(None, &first).unwrap();
    let second = queued(&store, 2);
    store.publish(None, &second).unwrap();
    let page = store.list(None, 1).unwrap();
    assert_eq!(page.jobs.len(), 1);
    let cursor = page.next_cursor.unwrap();
    let third = queued(&store, 3);
    store.publish(None, &third).unwrap();
    let mut running = first.clone();
    running.start().unwrap();
    store.publish(Some(1), &running).unwrap();
    let continued = store.list(Some(&cursor), 20).unwrap();
    assert_eq!(continued.jobs.len(), 1);
    assert_eq!(continued.jobs[0].job_id, second.job_id);
    let events = store.read_events(0, 2).unwrap();
    assert_eq!(events.events.len(), 2);
    assert_eq!(events.next_sequence.0, 2);
    assert_eq!(events.latest_sequence.0, 4);
    let rest = store.read_events(2, 20).unwrap();
    assert_eq!(rest.events.len(), 2);
    assert!(!rest.gap);
    assert_eq!(
        store.read_utf8(&first.input.source).unwrap(),
        "원문 日本語\n"
    );
    drop(store);
    let reopened = fixture.open();
    assert_eq!(
        reopened.get(&first.job_id).unwrap().state,
        JobState::Running
    );
    assert_eq!(reopened.list(None, 200).unwrap().jobs.len(), 3);
    assert_eq!(
        reopened.read_events(5, 1).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
}
#[test]
fn exclusive_execution_owner_and_competing_revision_writers_are_real_file_leases() {
    let fixture = Fixture::new();
    let store = fixture.open();
    let job = queued(&store, 1);
    store.publish(None, &job).unwrap();
    let owner = store.acquire_execution_owner().unwrap();
    assert!(store.acquire_execution_owner().is_err());
    drop(owner);
    drop(store.acquire_execution_owner().unwrap());
    let barrier = Arc::new(std::sync::Barrier::new(2));
    let mut workers = Vec::new();
    for _ in 0..2 {
        let root = fixture.0.clone();
        let store = JobFiles::open(&root, Arc::new(FixturePaths)).unwrap();
        let barrier = barrier.clone();
        let mut next = job.clone();
        next.start().unwrap();
        workers.push(std::thread::spawn(move || {
            barrier.wait();
            store.publish(Some(1), &next)
        }));
    }
    let results: Vec<_> = workers
        .into_iter()
        .map(|worker| worker.join().unwrap())
        .collect();
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|result| result
                .as_ref()
                .is_err_and(|error| error.code == ErrorCode::RevisionConflict))
            .count(),
        1
    );
}
#[test]
fn torn_tail_and_modified_source_fail_without_admission_or_fallback() {
    let fixture = Fixture::new();
    let store = fixture.open();
    let job = queued(&store, 1);
    let source = fixture
        .0
        .join("job-sources")
        .join(format!("{}.utf8", job.input.source.sha256));
    fs::write(&source, b"changed").unwrap();
    assert!(store.publish(None, &job).is_err());
    assert!(store.read_events(0, 10).unwrap().events.is_empty());
    fs::write(&source, "원문 日本語\n").unwrap();
    store.publish(None, &job).unwrap();
    fs::create_dir(
        fixture
            .0
            .join("job-events")
            .join(format!("e-2-{}", job.job_id.as_str())),
    )
    .unwrap();
    assert_eq!(
        store.get(&job.job_id).unwrap_err().code,
        ErrorCode::NeedsInput
    );
    assert_eq!(
        store.read_events(0, 10).unwrap_err().code,
        ErrorCode::NeedsInput
    );
}
#[test]
fn new_owner_marks_old_work_interrupted_without_resubmitting_or_erasing_publication_risk() {
    let fixture = Fixture::new();
    let store = Arc::new(fixture.open());
    let queued = queued(&store, 1);
    store.publish(None, &queued).unwrap();
    let mut publishing = queued.clone();
    publishing.start().unwrap();
    store.publish(Some(1), &publishing).unwrap();
    publishing.begin_publication().unwrap();
    store.publish(Some(2), &publishing).unwrap();
    let (session, recovered) =
        crate::application::ExecutionSession::acquire(store.clone()).unwrap();
    assert_eq!(recovered, vec![queued.job_id.clone()]);
    let old = session.store().get(&queued.job_id).unwrap();
    assert_eq!(old.state, JobState::Interrupted);
    assert_eq!(old.attempt.0, 1);
    assert!(old.recovery_required());
    assert!(old.output.is_none());
    assert!(crate::application::ExecutionSession::acquire(store.clone()).is_err());
    drop(session);
    let (_session, recovered) = crate::application::ExecutionSession::acquire(store).unwrap();
    assert!(recovered.is_empty());
}
fn prepared_conversion(
    fixture: &Fixture,
    store: &JobFiles,
    source: &str,
    input_format: &str,
    allow_loss: bool,
    id: u64,
) -> JobRecord {
    use crate::application::JobExecutor;
    let input = fixture.0.join(format!("source-{id}.txt"));
    let output = fixture.0.join(format!("output-{id}.txt"));
    fs::write(&input, source).unwrap();
    let executor = crate::infrastructure::SubtitleJobExecutor::new(Arc::new(FixturePaths));
    let prepared=executor.prepare("subtitles.convert",serde_json::json!({"input_path":input,"output_path":output,"input_format":input_format,"output_format":if input_format=="srt"{"vtt"}else{"srt"},"allow_loss":allow_loss}),None,store).unwrap();
    let job = JobRecord::queued(format!("job-{id:032x}").try_into().unwrap(), prepared).unwrap();
    store.publish(None, &job).unwrap();
    job
}
const SRT: &str = "1\n00:00:01,001 --> 00:00:03,456\n안녕하세요 日本語\n\n";
#[test]
fn actual_subtitle_use_case_publishes_verified_utf8_result_and_preserves_original() {
    let fixture = Fixture::new();
    let store = fixture.open();
    let job = prepared_conversion(&fixture, &store, SRT, "srt", false, 1);
    let executor = crate::infrastructure::SubtitleJobExecutor::new(Arc::new(FixturePaths));
    let completed =
        crate::application::run_queued(&store, &executor, &store, &job.job_id, 1).unwrap();
    assert_eq!(completed.state, JobState::Completed);
    let output = completed.output.unwrap();
    let bytes = fs::read(&output.path).unwrap();
    assert_eq!(sources::digest(&bytes), output.sha256);
    assert_eq!(output.receipt["cue_count"], 1);
    assert!(
        String::from_utf8(bytes)
            .unwrap()
            .contains("안녕하세요 日本語")
    );
    assert_eq!(
        fs::read_to_string(fixture.0.join("source-1.txt")).unwrap(),
        SRT
    );
}
#[test]
fn changed_source_and_existing_output_are_distinct_needs_input_without_overwrite() {
    let fixture = Fixture::new();
    let store = fixture.open();
    let job = prepared_conversion(&fixture, &store, SRT, "srt", false, 1);
    fs::write(
        fixture.0.join("source-1.txt"),
        SRT.replace("日本語", "changed"),
    )
    .unwrap();
    let executor = crate::infrastructure::SubtitleJobExecutor::new(Arc::new(FixturePaths));
    let result = crate::application::run_queued(&store, &executor, &store, &job.job_id, 1).unwrap();
    assert_eq!(result.state, JobState::NeedsInput);
    assert_eq!(result.failure.unwrap().code, ErrorCode::SourceExpired);
    assert!(!fixture.0.join("output-1.txt").exists());
    assert_eq!(store.read_utf8(&job.input.source).unwrap(), SRT);
    let other = prepared_conversion(&fixture, &store, SRT, "srt", false, 2);
    fs::write(fixture.0.join("output-2.txt"), b"existing").unwrap();
    let result =
        crate::application::run_queued(&store, &executor, &store, &other.job_id, 1).unwrap();
    assert_eq!(result.state, JobState::NeedsInput);
    assert_eq!(result.failure.unwrap().code, ErrorCode::RevisionConflict);
    assert_eq!(
        fs::read(fixture.0.join("output-2.txt")).unwrap(),
        b"existing"
    );
}
#[test]
fn actual_loss_details_remain_needs_input_until_a_separate_explicit_input_is_submitted() {
    let fixture = Fixture::new();
    let store = fixture.open();
    let source = "WEBVTT\n\nNOTE retained note\n\ncue-a\n00:00:01.000 --> 00:00:02.000 align:start\n안녕하세요\n";
    let job = prepared_conversion(&fixture, &store, source, "vtt", false, 1);
    let executor = crate::infrastructure::SubtitleJobExecutor::new(Arc::new(FixturePaths));
    let result = crate::application::run_queued(&store, &executor, &store, &job.job_id, 1).unwrap();
    assert_eq!(result.state, JobState::NeedsInput);
    assert!(
        result.failure.unwrap().details["losses"]
            .as_array()
            .is_some_and(|losses| !losses.is_empty())
    );
    assert!(!fixture.0.join("output-1.txt").exists());
    let explicit = prepared_conversion(&fixture, &store, source, "vtt", true, 2);
    let result =
        crate::application::run_queued(&store, &executor, &store, &explicit.job_id, 1).unwrap();
    assert_eq!(result.state, JobState::Completed);
}
#[test]
fn actual_conversion_cancel_and_publication_race_are_linearized_before_real_file_write() {
    use crate::application::{ExecutionControl, JobExecutor, SourceBlobStore};
    use crate::domain::{CancelDisposition, PreparedJobInput, VerifiedJobOutput};
    use serde_json::Value;
    struct Paused {
        inner: crate::infrastructure::SubtitleJobExecutor,
        barrier: Arc<std::sync::Barrier>,
        after_reservation: bool,
    }
    struct Control<'a> {
        inner: &'a dyn ExecutionControl,
        barrier: &'a std::sync::Barrier,
        after: bool,
    }
    impl ExecutionControl for Control<'_> {
        fn cancellation_requested(&self) -> bool {
            self.inner.cancellation_requested()
        }
        fn deadline(&self) -> std::time::Instant {
            self.inner.deadline()
        }
        fn reserve_publication(&self) -> Result<(), ApiError> {
            if self.after {
                self.inner.reserve_publication()?;
            }
            self.barrier.wait();
            self.barrier.wait();
            if !self.after {
                self.inner.reserve_publication()?;
            }
            Ok(())
        }
        fn begin_verification(&self) -> Result<(), ApiError> {
            self.inner.begin_verification()
        }
    }
    impl JobExecutor for Paused {
        fn supports(&self, operation: &str) -> bool {
            self.inner.supports(operation)
        }
        fn prepare(
            &self,
            operation: &str,
            parameters: Value,
            expected: Option<&str>,
            blobs: &dyn SourceBlobStore,
        ) -> Result<PreparedJobInput, ApiError> {
            self.inner.prepare(operation, parameters, expected, blobs)
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
                &Control {
                    inner: control,
                    barrier: &self.barrier,
                    after: self.after_reservation,
                },
            )
        }
    }
    for after in [false, true] {
        let fixture = Fixture::new();
        let store = Arc::new(fixture.open());
        let job = prepared_conversion(&fixture, &store, SRT, "srt", false, 1);
        let id = job.job_id.clone();
        let target = store.clone();
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let executor = Paused {
            inner: crate::infrastructure::SubtitleJobExecutor::new(Arc::new(FixturePaths)),
            barrier: barrier.clone(),
            after_reservation: after,
        };
        let worker = std::thread::spawn(move || {
            crate::application::run_queued(target.as_ref(), &executor, target.as_ref(), &id, 1)
        });
        barrier.wait();
        let (_, disposition) =
            crate::application::request_cancel(store.as_ref(), &job.job_id, 1).unwrap();
        assert_eq!(
            disposition,
            if after {
                CancelDisposition::PublicationInProgress
            } else {
                CancelDisposition::Requested
            }
        );
        barrier.wait();
        let result = worker.join().unwrap().unwrap();
        assert_eq!(
            result.state,
            if after {
                JobState::Completed
            } else {
                JobState::Cancelled
            }
        );
        assert_eq!(fixture.0.join("output-1.txt").exists(), after);
    }
}
#[test]
fn submission_key_is_durable_first_wins_and_rejects_a_different_request() {
    let fixture = Fixture::new();
    let store = fixture.open();
    let mut first = queued(&store, 1);
    first.submission = Some(crate::domain::SubmissionRef {
        key_sha256: "b".repeat(64),
        request_sha256: "c".repeat(64),
    });
    assert!(store.admit(&first).unwrap().created);
    let mut duplicate = queued(&store, 2);
    duplicate.submission = first.submission.clone();
    let existing = store.admit(&duplicate).unwrap();
    assert!(!existing.created);
    assert_eq!(existing.event.job.job_id, first.job_id);
    duplicate.submission.as_mut().unwrap().request_sha256 = "d".repeat(64);
    let error = store.admit(&duplicate).unwrap_err();
    assert_eq!(error.code, ErrorCode::RevisionConflict);
    assert_eq!(error.details["conflict_kind"], "idempotency_conflict");
    drop(store);
    let reopened = fixture.open();
    assert_eq!(
        reopened
            .find_submission(&"b".repeat(64))
            .unwrap()
            .unwrap()
            .job_id,
        first.job_id
    );
    assert_eq!(reopened.read_events(0, 200).unwrap().events.len(), 1);
}
struct Ids(std::sync::atomic::AtomicU64);
impl crate::application::JobIdentitySource for Ids {
    fn next(&self) -> Result<JobId, ApiError> {
        Ok(format!(
            "job-{:032x}",
            self.0.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        )
        .try_into()
        .unwrap())
    }
}
#[test]
fn explicit_host_executes_durable_queue_and_reuses_key_without_recapturing_a_missing_source() {
    use crate::application::{GetJob, JobsApi, SubmitJob, WatchJobs};
    let fixture = Fixture::new();
    let store = Arc::new(fixture.open());
    let source = fixture.0.join("native-source.srt");
    let output = fixture.0.join("native-output.vtt");
    fs::write(&source, SRT).unwrap();
    let executor = Arc::new(crate::infrastructure::SubtitleJobExecutor::new(Arc::new(
        FixturePaths,
    )));
    let host = crate::infrastructure::WorkerHost::start(
        store.clone(),
        executor,
        store.clone(),
        Arc::new(Ids(std::sync::atomic::AtomicU64::new(1))),
    )
    .unwrap();
    let client = host.client();
    let input = SubmitJob {
        operation: "subtitles.convert".into(),
        parameters: serde_json::json!({"input_path":source,"output_path":output,"input_format":"srt","output_format":"vtt"}),
        expected_source_sha256: None,
        execution_timeout_ms: DecimalU64(30000),
    };
    let admitted = client.submit(input.clone(), "first").unwrap();
    assert!(admitted.created);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    let completed = loop {
        let job = client
            .get(GetJob {
                job_id: admitted.job.job_id.clone(),
            })
            .unwrap();
        if job.state.terminal() {
            break job;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
    };
    assert_eq!(completed.state, JobState::Completed);
    fs::remove_file(&source).unwrap();
    let duplicate = client.submit(input.clone(), "first").unwrap();
    assert!(!duplicate.created);
    assert_eq!(duplicate.job.job_id, completed.job_id);
    let mut changed = input.clone();
    changed.execution_timeout_ms = DecimalU64(20000);
    assert_eq!(
        client.submit(changed, "first").unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    let events = client
        .watch(WatchJobs {
            after_sequence: DecimalU64(0),
            limit: Some(20),
        })
        .unwrap();
    assert_eq!(events.events.len(), 5);
    assert_eq!(events.events.last().unwrap().job.state, JobState::Completed);
    let readonly = crate::application::JobService::read_only(store.clone());
    assert!(!readonly.capability().execution_available);
    assert_eq!(
        readonly.submit(input.clone(), "first").unwrap_err().code,
        ErrorCode::Unsupported
    );
    let receipt = host.shutdown().unwrap();
    assert_eq!(receipt.completed_total, 1);
    assert_eq!(
        client.submit(input, "first").unwrap_err().code,
        ErrorCode::Unsupported
    );
    assert!(store.acquire_execution_owner().is_ok());
    assert!(fs::read_to_string(output).unwrap().contains("日本語"));
}

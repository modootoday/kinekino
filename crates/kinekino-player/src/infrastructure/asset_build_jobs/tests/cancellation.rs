//! Pauses/delegates the original CPU control, never supplies a replacement CAS.
use super::*;
use crate::application::jobs::{CancelDisposition, CancelJob, RetryJob};
use std::{
    path::Path,
    sync::{
        Condvar, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Point {
    BeforeCas,
    AfterCommit,
}
#[derive(Default)]
struct State {
    entered: bool,
    released: bool,
    finished: bool,
}
pub(super) struct Gate {
    point: Point,
    state: Mutex<State>,
    changed: Condvar,
    calls: AtomicU64,
}
impl Gate {
    pub fn new(point: Point) -> Self {
        Self {
            point,
            state: Mutex::new(State::default()),
            changed: Condvar::new(),
            calls: AtomicU64::new(0),
        }
    }
    fn pause(&self) -> Result<(), ApiError> {
        let mut state = self.state.lock().unwrap();
        state.entered = true;
        self.changed.notify_all();
        let (state, _) = self
            .changed
            .wait_timeout_while(state, Duration::from_secs(20), |state| !state.released)
            .unwrap();
        if state.released {
            Ok(())
        } else {
            Err(error(
                ErrorCode::Timeout,
                "Original CPU publication test pause expired",
            ))
        }
    }
    fn wait(&self) {
        let (state, _) = self
            .changed
            .wait_timeout_while(
                self.state.lock().unwrap(),
                Duration::from_secs(110),
                |state| !state.entered && !state.finished,
            )
            .unwrap();
        assert!(
            state.entered,
            "Original CPU attempt did not reach publication pause; preserve adapter error"
        );
    }
    fn release(&self) {
        self.state.lock().unwrap().released = true;
        self.changed.notify_all();
    }
    fn finished(&self) {
        self.state.lock().unwrap().finished = true;
        self.changed.notify_all();
    }
    pub fn calls(&self) -> u64 {
        self.calls.load(Ordering::SeqCst)
    }
}
struct Release(Arc<Gate>);
impl Drop for Release {
    fn drop(&mut self) {
        self.0.release();
    }
}
struct Control<'a> {
    inner: &'a dyn ExecutionControl,
    gate: &'a Gate,
}
impl ExecutionControl for Control<'_> {
    fn execution_identity(&self) -> Option<&kinekino_jobs::application::JobAttemptIdentityV1> {
        self.inner.execution_identity()
    }
    fn cancellation_requested(&self) -> bool {
        self.inner.cancellation_requested()
    }
    fn deadline(&self) -> Instant {
        self.inner.deadline()
    }
    fn reserve_publication(&self) -> Result<(), ApiError> {
        if self.gate.point == Point::BeforeCas {
            self.gate.pause()?;
        }
        self.gate.calls.fetch_add(1, Ordering::SeqCst);
        self.inner.reserve_publication()
    }
    fn begin_verification(&self) -> Result<(), ApiError> {
        if self.gate.point == Point::AfterCommit {
            self.gate.pause()?;
        }
        self.inner.begin_verification()
    }
}
pub(super) struct Paused {
    pub inner: Arc<dyn JobExecutor>,
    pub gate: Arc<Gate>,
    pub root: PathBuf,
}
impl JobExecutor for Paused {
    fn operations(&self) -> Vec<String> {
        self.inner.operations()
    }
    fn supports(&self, operation: &str) -> bool {
        self.inner.supports(operation)
    }
    fn supports_batch_capture(&self, operation: &str) -> bool {
        self.inner.supports_batch_capture(operation)
    }
    fn execution_timeout_limit_ms(&self, operation: &str) -> Option<u64> {
        self.inner.execution_timeout_limit_ms(operation)
    }
    fn normalize_parameters(&self, operation: &str, value: Value) -> Result<Value, ApiError> {
        self.inner.normalize_parameters(operation, value)
    }
    fn prepare(
        &self,
        operation: &str,
        value: Value,
        expected: Option<&str>,
        blobs: &dyn SourceBlobStore,
    ) -> Result<PreparedJobInput, ApiError> {
        self.inner.prepare(operation, value, expected, blobs)
    }
    fn execute(
        &self,
        input: &PreparedJobInput,
        blobs: &dyn SourceBlobStore,
        control: &dyn ExecutionControl,
    ) -> Result<VerifiedJobOutput, ApiError> {
        assert!(self.inner.owned_executor(&input.operation).is_none());
        let result = self.inner.execute(
            input,
            blobs,
            &Control {
                inner: control,
                gate: &self.gate,
            },
        );
        if let Err(failure) = &result {
            fs::write(
                self.root.join("adapter-error.json"),
                serde_json::to_vec_pretty(failure).unwrap(),
            )
            .unwrap();
        }
        self.gate.finished();
        result
    }
}
fn inventory(directory: &Path) -> BTreeMap<String, String> {
    fs::read_dir(directory)
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            assert!(entry.file_type().unwrap().is_file());
            (
                entry.file_name().into_string().unwrap(),
                hash(&fs::read(entry.path()).unwrap()),
            )
        })
        .collect()
}
pub(super) fn cancel_at_gate(
    gate: &Arc<Gate>,
    root: &Path,
    client: &dyn crate::application::jobs::JobsApi,
    submitted: &kinekino_jobs::application::SubmittedJob,
    image: &Path,
    mask: &Path,
    project: &Path,
) {
    let release = Release(gate.clone());
    gate.wait();
    let directory = root
        .join("artifacts")
        .join(format!("asset-{}-attempt-1", submitted.job.job_id.as_str()));
    let files = inventory(&directory);
    assert_eq!(
        files.len(),
        if gate.point == Point::BeforeCas { 8 } else { 9 }
    );
    fs::write(
        root.join("staged-before-cancel.json"),
        serde_json::to_vec_pretty(&files).unwrap(),
    )
    .unwrap();
    for path in [image, mask, project] {
        assert!(
            OpenOptions::new().write(true).open(path).is_err(),
            "Original source owner released before publication boundary"
        );
    }
    let result = client.cancel(CancelJob {
        job_id: submitted.job.job_id.clone(),
        expected_attempt: submitted.job.attempt,
    });
    gate.release();
    drop(release);
    let result = result.unwrap();
    fs::write(
        root.join("cancel-response.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    assert_eq!(
        result.disposition,
        if gate.point == Point::BeforeCas {
            CancelDisposition::Requested
        } else {
            CancelDisposition::PublicationInProgress
        }
    );
}
pub(super) fn verify_cancel_and_retry(
    root: &Path,
    cancelled: &kinekino_jobs::domain::JobRecord,
    executor: Arc<dyn JobExecutor>,
    image: &Path,
    mask: &Path,
) {
    assert_eq!(
        cancelled.state,
        JobState::Cancelled,
        "{:?}",
        cancelled.failure
    );
    assert!(!cancelled.publication_started);
    assert!(cancelled.output.is_none());
    let partial = root
        .join("artifacts")
        .join(format!("asset-{}-attempt-1", cancelled.job_id.as_str()));
    let original = inventory(&partial);
    assert_eq!(original.len(), 8);
    assert!(!partial.join("publication.json").exists());
    let before: BTreeMap<String, String> =
        serde_json::from_slice(&fs::read(root.join("staged-before-cancel.json")).unwrap()).unwrap();
    assert_eq!(original, before);
    let failure: Value =
        serde_json::from_slice(&fs::read(root.join("adapter-error.json")).unwrap()).unwrap();
    assert_eq!(failure["code"], "cancelled");
    assert_eq!(
        failure["details"]["artifact_id"],
        format!("{}-attempt-1", cancelled.job_id.as_str())
    );
    assert_eq!(failure["details"]["publication_committed"], false);
    assert_eq!(failure["details"]["publication_may_exist"], false);
    assert_eq!(failure["details"]["partial_cleanup_supported"], false);
    for path in [image, mask] {
        assert!(OpenOptions::new().write(true).open(path).is_ok());
    }
    let host = jobs::start_retaining_host_with_executor(&root.join("jobs"), executor).unwrap();
    let client = host.client();
    let before = client
        .get(GetJob {
            job_id: cancelled.job_id.clone(),
        })
        .unwrap();
    assert_eq!(before.state, JobState::Cancelled);
    assert_eq!(before.attempt.0, 1, "No automatic retry on restart");
    let request = RetryJob {
        job_id: cancelled.job_id.clone(),
    };
    assert_eq!(
        client
            .retry(request.clone(), before.revision.0 - 1)
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    let admitted = client.retry(request.clone(), before.revision.0).unwrap();
    assert_eq!(admitted.attempt.0, 2);
    assert_eq!(admitted.input_sha256, cancelled.input_sha256);
    assert_eq!(admitted.input.source.sha256, cancelled.input.source.sha256);
    assert_eq!(
        client.retry(request, before.revision.0).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    let deadline = Instant::now() + Duration::from_secs(120);
    let done = loop {
        let record = client
            .get(GetJob {
                job_id: cancelled.job_id.clone(),
            })
            .unwrap();
        fs::write(
            root.join("attempt-2-job.json"),
            serde_json::to_vec_pretty(&record).unwrap(),
        )
        .unwrap();
        if record.state.terminal() {
            break record;
        }
        assert!(
            Instant::now() < deadline,
            "Original manual retry remains live"
        );
        std::thread::sleep(Duration::from_millis(20));
    };
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut pending = host.begin_shutdown(deadline);
    loop {
        match pending.poll(deadline) {
            WorkerShutdownOutcome::Complete(closed) => {
                fs::write(
                    root.join("attempt-2-shutdown.json"),
                    serde_json::to_vec_pretty(
                        &json!({"receipt":closed.receipt,"error":closed.errors.first()}),
                    )
                    .unwrap(),
                )
                .unwrap();
                assert!(closed.errors.first().is_none());
                break;
            }
            WorkerShutdownOutcome::Pending(original) => {
                pending = original;
                assert!(Instant::now() < deadline);
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }
    assert_eq!(done.state, JobState::Completed, "{:?}", done.failure);
    assert_eq!(done.attempt.0, 2);
    assert!(done.publication_started);
    assert_eq!(inventory(&partial), original);
    assert!(!partial.join("publication.json").exists());
    let output = done.output.as_ref().unwrap();
    let bytes = fs::read(&output.path).unwrap();
    assert_eq!(hash(&bytes), output.sha256);
    let publication: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(publication["execution"]["attempt"], "2");
    assert_eq!(
        publication["execution"]["input_sha256"],
        cancelled.input_sha256
    );
    assert_eq!(
        publication["execution"]["source_blob_sha256"],
        cancelled.input.source.sha256
    );
    let reopened = jobs::read_only(&root.join("jobs"))
        .unwrap()
        .get(GetJob {
            job_id: done.job_id.clone(),
        })
        .unwrap();
    assert_eq!(reopened.output.as_ref().unwrap().sha256, output.sha256);
    fs::write(root.join("cancel-retry-verification.json"),serde_json::to_vec_pretty(&json!({"original_cancel_won_before_publication_cas":true,"original_cas_delegated_once":true,"partial_eight_files_preserved":true,"adapter_primary_recovery_metadata_retained":true,"source_guards_retained_until_cancel_settled":true,"orderly_same_process_host_restart":true,"no_automatic_retry":true,"stale_retry_rejected_before_and_after":true,"same_input_manual_attempt_2_completed":true,"attempt_2_publication_sha256":output.sha256,"unexpected_crash_or_ack_loss_verified":false,"cli_verified":false,"gui_verified":false,"quality_approved":false})).unwrap()).unwrap();
}

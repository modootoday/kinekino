//! Test-only pause before the original publication gate; no replacement CAS.
use super::*;
use std::sync::{Condvar, Mutex};
pub(super) struct Gate {
    state: Mutex<(bool, bool)>,
    changed: Condvar,
}
impl Gate {
    pub(super) fn new() -> Self {
        Self {
            state: Mutex::new((false, false)),
            changed: Condvar::new(),
        }
    }
    fn pause(&self) -> Result<(), ApiError> {
        let mut state = self.state.lock().unwrap();
        state.0 = true;
        self.changed.notify_all();
        let (state, _) = self
            .changed
            .wait_timeout_while(state, Duration::from_secs(20), |state| !state.1)
            .unwrap();
        if state.1 {
            Ok(())
        } else {
            Err(error(ErrorCode::Timeout, "Test publication pause expired"))
        }
    }
    pub(super) fn wait(&self) {
        let (state, _) = self
            .changed
            .wait_timeout_while(
                self.state.lock().unwrap(),
                Duration::from_secs(110),
                |state| !state.0,
            )
            .unwrap();
        assert!(state.0, "Original render did not reach publication gate");
    }
    pub(super) fn release(&self) {
        self.state.lock().unwrap().1 = true;
        self.changed.notify_all();
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
        self.gate.pause()?;
        self.inner.reserve_publication()
    }
    fn begin_verification(&self) -> Result<(), ApiError> {
        self.inner.begin_verification()
    }
}
pub(super) struct Paused {
    pub inner: Arc<dyn JobExecutor>,
    pub gate: Arc<Gate>,
}
impl JobExecutor for Paused {
    fn owned_executor(
        &self,
        operation: &str,
    ) -> Option<&kinekino_jobs::application::owned_attempt::OwnedExecutorPort> {
        self.inner
            .owned_executor(operation)
            .map(|_| self as &kinekino_jobs::application::owned_attempt::OwnedExecutorPort)
    }
    fn supports_batch_capture(&self, operation: &str) -> bool {
        self.inner.supports_batch_capture(operation)
    }
    fn operations(&self) -> Vec<String> {
        self.inner.operations()
    }
    fn supports(&self, operation: &str) -> bool {
        self.inner.supports(operation)
    }
    fn normalize_parameters(&self, operation: &str, value: Value) -> Result<Value, ApiError> {
        self.inner.normalize_parameters(operation, value)
    }
    fn execution_timeout_limit_ms(&self, operation: &str) -> Option<u64> {
        self.inner.execution_timeout_limit_ms(operation)
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
        self.inner.execute(
            input,
            blobs,
            &Control {
                inner: control,
                gate: &self.gate,
            },
        )
    }
}
impl kinekino_jobs::application::owned_attempt::OwnedAttemptExecutor for Paused {
    type Candidate = Box<dyn kinekino_jobs::application::owned_attempt::OwnedOutputCandidate>;
    fn prepare_owned(
        &self,
        input: &PreparedJobInput,
        blobs: &dyn SourceBlobStore,
        scope: &kinekino_jobs::application::owned_attempt::AttemptScope<'_>,
    ) -> Result<
        Box<
            dyn kinekino_jobs::application::owned_attempt::OwnedAttemptSession<
                    Candidate = Self::Candidate,
                >,
        >,
        ApiError,
    > {
        let inner = self
            .inner
            .owned_executor(&input.operation)
            .expect("Original Core clip executor")
            .prepare_owned(input, blobs, scope)?;
        Ok(Box::new(PausedSession {
            inner,
            gate: self.gate.clone(),
            candidate: None,
        }))
    }
}
struct PausedSession {
    inner: Box<
        dyn kinekino_jobs::application::owned_attempt::OwnedAttemptSession<
                Candidate = Box<
                    dyn kinekino_jobs::application::owned_attempt::OwnedOutputCandidate,
                >,
            >,
    >,
    gate: Arc<Gate>,
    candidate: Option<Box<dyn kinekino_jobs::application::owned_attempt::OwnedOutputCandidate>>,
}
impl kinekino_jobs::application::owned_attempt::OwnedAttemptSession for PausedSession {
    type Candidate = Box<dyn kinekino_jobs::application::owned_attempt::OwnedOutputCandidate>;
    fn run(
        &mut self,
        input: &PreparedJobInput,
        blobs: &dyn SourceBlobStore,
        scope: &kinekino_jobs::application::owned_attempt::AttemptScope<'_>,
    ) -> kinekino_jobs::application::owned_attempt::SessionRun<Self::Candidate> {
        use kinekino_jobs::application::owned_attempt::SessionRun;
        match self.inner.run(input, blobs, scope) {
            SessionRun::Candidate(candidate) => {
                self.candidate = Some(candidate);
                match self.gate.pause() {
                    Ok(()) => SessionRun::Candidate(self.candidate.take().unwrap()),
                    Err(error) => SessionRun::Failed(error),
                }
            }
            other => other,
        }
    }
    fn poll_close(
        &mut self,
        deadline: Instant,
    ) -> Result<kinekino_jobs::application::owned_attempt::OwnerObservation, ApiError> {
        if let Some(candidate) = &mut self.candidate
            && !candidate.poll_release(deadline)?
        {
            return Ok(kinekino_jobs::application::owned_attempt::OwnerObservation::Pending);
        }
        self.inner.poll_close(deadline)
    }
}
pub(super) fn verify(
    root: &std::path::Path,
    temporary: &std::path::Path,
    done: &kinekino_jobs::domain::JobRecord,
) {
    assert_eq!(done.state, JobState::Cancelled, "{:?}", done.failure);
    assert!(done.output.is_none());
    assert_eq!(fs::read_dir(temporary).unwrap().count(), 0);
    let directory = root.join("artifacts").join(format!(
        "clip-{}-attempt-{}",
        done.job_id.as_str(),
        done.attempt.0
    ));
    assert!(!directory.join("manifest.json").exists());
    for name in ["clip.mp4", "render.json", "verification.json"] {
        assert!(
            directory.join(name).is_file(),
            "Preserve original partial output evidence {name}"
        );
    }
    let reopened = jobs::read_only(&root.join("jobs"))
        .unwrap()
        .get(GetJob {
            job_id: done.job_id.clone(),
        })
        .unwrap();
    assert_eq!(reopened.state, JobState::Cancelled);
    assert!(reopened.output.is_none());
    fs::write(root.join("verification.json"), serde_json::to_vec_pretty(&json!({"original_jobs_cancelled_before_publication_cas":true,"render_and_output_decode_completed_before_cancel":true,"new_png_and_mp4_inputs_absent_after_original_worker_shutdown":true,"partial_artifact_evidence_preserved_without_manifest":true,"cancelled_state_reopened":true,"encoder_mid_process_cancel_verified":false,"full_cleanup_restart_or_ack_loss_verified":false,"cli_verified":false,"quality_approved":false})).unwrap()).unwrap();
}

pub(super) fn retry_after_orderly_restart(
    root: &std::path::Path,
    temporary: &std::path::Path,
    cancelled: &kinekino_jobs::domain::JobRecord,
    executor: Arc<dyn JobExecutor>,
) {
    use crate::application::jobs::RetryJob;
    let partial = root
        .join("artifacts")
        .join(format!("clip-{}-attempt-1", cancelled.job_id.as_str()));
    let original = ["clip.mp4", "render.json", "verification.json"]
        .map(|name| (name, hash(&fs::read(partial.join(name)).unwrap())));
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
        let job = client
            .get(GetJob {
                job_id: cancelled.job_id.clone(),
            })
            .unwrap();
        if job.state.terminal() {
            break job;
        }
        assert!(Instant::now() < deadline, "Original retry did not finish");
        std::thread::sleep(Duration::from_millis(20));
    };
    fs::write(
        root.join("attempt-2-job.json"),
        serde_json::to_vec_pretty(&done).unwrap(),
    )
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut pending = host.begin_shutdown(deadline);
    loop {
        match pending.poll(deadline) {
            WorkerShutdownOutcome::Complete(done) => {
                fs::write(
                    root.join("attempt-2-shutdown.json"),
                    serde_json::to_vec_pretty(
                        &json!({"receipt":done.receipt,"error":done.errors.first()}),
                    )
                    .unwrap(),
                )
                .unwrap();
                assert!(done.errors.first().is_none());
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
    assert_eq!(fs::read_dir(temporary).unwrap().count(), 0);
    for (name, digest) in original {
        assert_eq!(
            hash(&fs::read(partial.join(name)).unwrap()),
            digest,
            "Attempt 1 evidence changed"
        );
    }
    assert!(!partial.join("manifest.json").exists());
    let output = done.output.as_ref().unwrap();
    let manifest_bytes = fs::read(&output.path).unwrap();
    assert_eq!(hash(&manifest_bytes), output.sha256);
    let manifest: Value = serde_json::from_slice(&manifest_bytes).unwrap();
    assert_eq!(manifest["execution"]["job_id"], cancelled.job_id.as_str());
    assert_eq!(manifest["execution"]["attempt"], "2");
    assert_eq!(
        manifest["execution"]["input_sha256"],
        cancelled.input_sha256
    );
    let reopened = jobs::read_only(&root.join("jobs"))
        .unwrap()
        .get(GetJob {
            job_id: done.job_id.clone(),
        })
        .unwrap();
    assert_eq!(reopened.output.unwrap().sha256, output.sha256);
    fs::write(root.join("retry-verification.json"), serde_json::to_vec_pretty(&json!({"orderly_worker_restart_same_process":true,"cancelled_attempt_not_automatically_retried":true,"stale_revision_rejected_before_and_after_manual_retry":true,"same_job_attempt_2_completed":true,"captured_input_and_source_hashes_preserved":true,"attempt_1_partial_evidence_preserved":true,"attempt_2_manifest_sha256":output.sha256,"new_working_inputs_absent":true,"unexpected_crash_restart_or_ack_loss_verified":false,"cli_verified":false,"quality_approved":false})).unwrap()).unwrap();
}

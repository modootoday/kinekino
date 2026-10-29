use super::*;
use crate::{
    application::{
        EventPage, ExecutionOwner, JobAdmission, JobEvent, JobListCursor, JobPage,
        owned_attempt::{
            AttemptScope, OwnedAttemptExecutor, OwnedAttemptSession, OwnerObservation, SessionRun,
        },
    },
    domain::PreparedJobInput,
};
use kinekino_media_contracts::api_values::DecimalU64;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    sync::{
        Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

struct Store {
    job: Mutex<JobRecord>,
    fail_failure_cas: AtomicBool,
    panic_failure_cas: AtomicBool,
    panic_get: AtomicBool,
    panic_after_publication_cas: AtomicBool,
}
impl Store {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            job: Mutex::new(
                JobRecord::queued(
                    "job-00000000000000000000000000000002"
                        .to_owned()
                        .try_into()
                        .unwrap(),
                    PreparedJobInput {
                        operation: "test.owned".into(),
                        parameters: serde_json::json!({}),
                        source: SourceBlobRef {
                            sha256: "a".repeat(64),
                            bytes: DecimalU64(1),
                        },
                        execution_timeout_ms: DecimalU64(30000),
                    },
                )
                .unwrap(),
            ),
            fail_failure_cas: AtomicBool::new(false),
            panic_failure_cas: AtomicBool::new(false),
            panic_get: AtomicBool::new(false),
            panic_after_publication_cas: AtomicBool::new(false),
        })
    }
    fn current(&self) -> JobRecord {
        self.job.lock().unwrap().clone()
    }
    fn cancel(&self) {
        crate::application::request_cancel(self, &self.current().job_id, 1).unwrap();
    }
}
impl JobStore for Store {
    fn find_submission(&self, _: &str) -> Result<Option<JobRecord>, ApiError> {
        unreachable!()
    }
    fn admit(&self, _: &JobRecord) -> Result<JobAdmission, ApiError> {
        unreachable!()
    }
    fn get(&self, _: &JobId) -> Result<JobRecord, ApiError> {
        if self.panic_get.swap(false, Ordering::SeqCst) {
            panic!("Original record observation panicked");
        }
        Ok(self.current())
    }
    fn publish(&self, expected: Option<u64>, job: &JobRecord) -> Result<JobEvent, ApiError> {
        if job.state == JobState::Failed && self.panic_failure_cas.swap(false, Ordering::SeqCst) {
            panic!("Original failure record adapter panicked");
        }
        if job.state == JobState::Failed && self.fail_failure_cas.swap(false, Ordering::SeqCst) {
            return Err(ApiError::new(
                ErrorCode::EngineFailed,
                "Original failure record write failed",
            ));
        }
        let mut current = self.job.lock().unwrap();
        if expected != Some(current.revision.0) {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Original revision changed",
            ));
        }
        job.validate_successor(&current).unwrap();
        *current = job.clone();
        drop(current);
        if job.publication_started
            && self
                .panic_after_publication_cas
                .swap(false, Ordering::SeqCst)
        {
            panic!("Original publication CAS committed before adapter panic");
        }
        Ok(JobEvent {
            sequence: job.revision,
            job: job.clone(),
        })
    }
    fn read_events(&self, _: u64, _: usize) -> Result<EventPage, ApiError> {
        unreachable!()
    }
    fn list(&self, _: Option<&JobListCursor>, _: usize) -> Result<JobPage, ApiError> {
        unreachable!()
    }
    fn acquire_execution_owner(&self) -> Result<Box<dyn ExecutionOwner>, ApiError> {
        unreachable!()
    }
}
#[derive(Default)]
struct Blobs {
    bodies: Mutex<BTreeMap<String, String>>,
    fail_write: AtomicBool,
    panic_write: AtomicBool,
}
impl SourceBlobStore for Blobs {
    fn put_utf8(&self, body: &str) -> Result<SourceBlobRef, ApiError> {
        if self.panic_write.swap(false, Ordering::SeqCst) {
            panic!("Original diagnostic adapter panicked");
        }
        if self.fail_write.load(Ordering::SeqCst) {
            return Err(ApiError::new(
                ErrorCode::EngineFailed,
                "Original diagnostic store unavailable",
            ));
        }
        let sha256 = format!("{:x}", Sha256::digest(body.as_bytes()));
        self.bodies
            .lock()
            .unwrap()
            .insert(sha256.clone(), body.into());
        Ok(SourceBlobRef {
            sha256,
            bytes: DecimalU64(body.len() as u64),
        })
    }
    fn read_utf8(&self, reference: &SourceBlobRef) -> Result<String, ApiError> {
        self.bodies
            .lock()
            .unwrap()
            .get(&reference.sha256)
            .cloned()
            .ok_or_else(|| ApiError::new(ErrorCode::NotFound, "Original diagnostic missing"))
    }
}
#[derive(Clone, Copy)]
enum Mode {
    Success,
    Cooperative { forced: bool },
    CancelledError,
    Panic,
}
#[derive(Default)]
struct Counts {
    hold_release: AtomicBool,
    fail_publish: AtomicBool,
    panic_release_once: AtomicBool,
    fail_release_once: AtomicBool,
    release_calls: AtomicUsize,
    prepare: AtomicUsize,
    run: AtomicUsize,
    publish: AtomicUsize,
    verify: AtomicUsize,
    dropped: AtomicUsize,
    candidate_dropped: AtomicUsize,
}
struct Executor {
    store: Arc<Store>,
    counts: Arc<Counts>,
    closed: Arc<AtomicBool>,
    mode: Mode,
}
impl OwnedAttemptExecutor for Executor {
    type Candidate = Box<dyn OwnedOutputCandidate>;
    fn prepare_owned(
        &self,
        _: &PreparedJobInput,
        _: &dyn SourceBlobStore,
        _: &AttemptScope<'_>,
    ) -> Result<Box<dyn OwnedAttemptSession<Candidate = Self::Candidate>>, ApiError> {
        self.counts.prepare.fetch_add(1, Ordering::SeqCst);
        if matches!(self.mode, Mode::Cooperative { .. } | Mode::CancelledError) {
            self.store.cancel();
        }
        Ok(Box::new(Session {
            counts: Arc::clone(&self.counts),
            closed: Arc::clone(&self.closed),
            mode: self.mode,
        }))
    }
}
struct Session {
    counts: Arc<Counts>,
    closed: Arc<AtomicBool>,
    mode: Mode,
}
impl Drop for Session {
    fn drop(&mut self) {
        self.counts.dropped.fetch_add(1, Ordering::SeqCst);
    }
}
impl OwnedAttemptSession for Session {
    type Candidate = Box<dyn OwnedOutputCandidate>;
    fn run(
        &mut self,
        _: &PreparedJobInput,
        _: &dyn SourceBlobStore,
        scope: &AttemptScope<'_>,
    ) -> SessionRun<Self::Candidate> {
        self.counts.run.fetch_add(1, Ordering::SeqCst);
        match self.mode {
            Mode::Success => {
                SessionRun::Candidate(Box::new(OutputCandidate(Arc::clone(&self.counts))))
            }
            Mode::Cooperative { .. } => {
                SessionRun::CooperativeCancelled(scope.observe_cooperative_cancellation().unwrap())
            }
            Mode::CancelledError => {
                let mut error = ApiError::new(
                    ErrorCode::Cancelled,
                    "Original engine failure while cancelling",
                );
                error.request_id = "original-engine-diagnostic".into();
                SessionRun::Failed(error)
            }
            Mode::Panic => panic!("Synthetic original session panic"),
        }
    }
    fn poll_close(&mut self, _: Instant) -> Result<OwnerObservation, ApiError> {
        if !self.closed.load(Ordering::SeqCst) {
            return Ok(OwnerObservation::Pending);
        }
        Ok(OwnerObservation::Exited {
            exit_code: 0,
            family_empty: true,
            streams_eof: true,
            forced: matches!(self.mode, Mode::Cooperative { forced: true }),
        })
    }
}
struct OutputCandidate(Arc<Counts>);
impl Drop for OutputCandidate {
    fn drop(&mut self) {
        self.0.candidate_dropped.fetch_add(1, Ordering::SeqCst);
    }
}
impl OwnedOutputCandidate for OutputCandidate {
    fn poll_release(&mut self, _: Instant) -> Result<bool, ApiError> {
        self.0.release_calls.fetch_add(1, Ordering::SeqCst);
        if self.0.panic_release_once.swap(false, Ordering::SeqCst) {
            panic!("Original fixture release panic");
        }
        if self.0.fail_release_once.swap(false, Ordering::SeqCst) {
            return Err(ApiError::new(
                ErrorCode::EngineFailed,
                "Original fixture release failed",
            ));
        }
        Ok(!self.0.hold_release.load(Ordering::SeqCst))
    }
    fn publish(&mut self) -> Result<(), ApiError> {
        self.0.publish.fetch_add(1, Ordering::SeqCst);
        if self.0.fail_publish.load(Ordering::SeqCst) {
            return Err(ApiError::new(
                ErrorCode::EngineFailed,
                "Original fixture publish failure",
            ));
        }
        Ok(())
    }
    fn verify(&mut self) -> Result<VerifiedJobOutput, ApiError> {
        self.0.verify.fetch_add(1, Ordering::SeqCst);
        Ok(VerifiedJobOutput {
            path: "fixture-output.json".into(),
            sha256: "b".repeat(64),
            bytes: DecimalU64(2),
            receipt: serde_json::json!({"fixture":true}),
        })
    }
}
fn fixture(mode: Mode) -> (Arc<Store>, Arc<Blobs>, Executor) {
    let store = Store::new();
    let executor = Executor {
        store: Arc::clone(&store),
        counts: Arc::new(Counts::default()),
        closed: Arc::new(AtomicBool::new(false)),
        mode,
    };
    (store, Arc::new(Blobs::default()), executor)
}
fn start(store: &Arc<Store>, blobs: &Arc<Blobs>, executor: &Executor) -> OwnedRunOutcome {
    run_owned(
        store.clone(),
        executor,
        blobs.clone(),
        &store.current().job_id,
        1,
    )
    .unwrap()
}
fn pending(outcome: OwnedRunOutcome) -> Box<PendingOwnedAttempt> {
    match outcome {
        OwnedRunOutcome::Pending(owner) => owner,
        _ => panic!("Original attempt must remain pending"),
    }
}
fn finish(mut outcome: OwnedRunOutcome) -> JobRecord {
    for _ in 0..20 {
        match outcome {
            OwnedRunOutcome::Finished(job) => return *job,
            OwnedRunOutcome::Pending(owner) => outcome = owner.poll(),
        }
    }
    panic!("Fixture did not settle original attempt")
}

#[test]
fn verified_candidate_and_result_remain_owned_until_release_without_republishing() {
    let (store, blobs, executor) = fixture(Mode::Success);
    executor.closed.store(true, Ordering::SeqCst);
    executor.counts.hold_release.store(true, Ordering::SeqCst);
    let mut outcome = start(&store, &blobs, &executor);
    for _ in 0..10 {
        outcome = pending(outcome).poll();
    }
    let owner = pending(outcome);
    assert_eq!(store.current().state, JobState::Verifying);
    assert_eq!(executor.counts.publish.load(Ordering::SeqCst), 1);
    assert_eq!(executor.counts.verify.load(Ordering::SeqCst), 1);
    assert_eq!(executor.counts.candidate_dropped.load(Ordering::SeqCst), 0);
    assert!(owner.verified().is_some());
    executor.counts.hold_release.store(false, Ordering::SeqCst);
    assert_eq!(
        finish(OwnedRunOutcome::Pending(owner)).state,
        JobState::Completed
    );
    assert_eq!(executor.counts.publish.load(Ordering::SeqCst), 1);
    assert_eq!(executor.counts.verify.load(Ordering::SeqCst), 1);
    assert_eq!(executor.counts.candidate_dropped.load(Ordering::SeqCst), 1);
}

#[test]
fn release_error_retains_verified_output_and_owner_until_cleanup_is_observed() {
    let (store, blobs, executor) = fixture(Mode::Success);
    executor.closed.store(true, Ordering::SeqCst);
    executor.counts.hold_release.store(true, Ordering::SeqCst);
    executor
        .counts
        .fail_release_once
        .store(true, Ordering::SeqCst);
    let mut outcome = start(&store, &blobs, &executor);
    for _ in 0..10 {
        outcome = pending(outcome).poll();
    }
    let owner = pending(outcome);
    assert!(owner.first_error().is_some());
    assert!(owner.verified().is_some());
    assert_eq!(executor.counts.candidate_dropped.load(Ordering::SeqCst), 0);
    assert_eq!(store.current().state, JobState::Verifying);
    executor.counts.hold_release.store(false, Ordering::SeqCst);
    assert_eq!(
        finish(OwnedRunOutcome::Pending(owner)).state,
        JobState::Failed
    );
    assert_eq!(executor.counts.publish.load(Ordering::SeqCst), 1);
    assert_eq!(executor.counts.verify.load(Ordering::SeqCst), 1);
}

#[test]
fn late_cancel_keeps_unpublished_candidate_until_original_release() {
    let (store, blobs, executor) = fixture(Mode::Success);
    executor.closed.store(true, Ordering::SeqCst);
    executor.counts.hold_release.store(true, Ordering::SeqCst);
    let mut outcome = start(&store, &blobs, &executor);
    store.cancel();
    for _ in 0..8 {
        outcome = pending(outcome).poll();
    }
    let owner = pending(outcome);
    assert_eq!(store.current().state, JobState::Cancelling);
    assert_eq!(executor.counts.candidate_dropped.load(Ordering::SeqCst), 0);
    assert_eq!(executor.counts.publish.load(Ordering::SeqCst), 0);
    executor.counts.hold_release.store(false, Ordering::SeqCst);
    assert_eq!(
        finish(OwnedRunOutcome::Pending(owner)).state,
        JobState::Cancelled
    );
    assert_eq!(executor.counts.candidate_dropped.load(Ordering::SeqCst), 1);
}

#[test]
fn release_panic_retains_original_candidate_and_is_not_repolled_as_closed() {
    let (store, blobs, executor) = fixture(Mode::Success);
    executor.closed.store(true, Ordering::SeqCst);
    executor
        .counts
        .panic_release_once
        .store(true, Ordering::SeqCst);
    let mut outcome = start(&store, &blobs, &executor);
    for _ in 0..10 {
        outcome = pending(outcome).poll();
    }
    let owner = pending(outcome);
    assert!(owner.release_poisoned && owner.panic_payload.is_some());
    assert!(owner.verified().is_some());
    assert_eq!(executor.counts.release_calls.load(Ordering::SeqCst), 1);
    assert_eq!(executor.counts.candidate_dropped.load(Ordering::SeqCst), 0);
    assert_eq!(store.current().state, JobState::Verifying);
    assert_eq!(executor.counts.publish.load(Ordering::SeqCst), 1);
    assert_eq!(executor.counts.verify.load(Ordering::SeqCst), 1);
}

#[test]
fn publication_failure_stays_primary_while_candidate_cleanup_is_pending() {
    let (store, blobs, executor) = fixture(Mode::Success);
    executor.closed.store(true, Ordering::SeqCst);
    executor.counts.fail_publish.store(true, Ordering::SeqCst);
    executor
        .counts
        .fail_release_once
        .store(true, Ordering::SeqCst);
    executor.counts.hold_release.store(true, Ordering::SeqCst);
    let mut outcome = start(&store, &blobs, &executor);
    for _ in 0..10 {
        outcome = pending(outcome).poll();
    }
    let owner = pending(outcome);
    assert_eq!(
        owner.first_error().unwrap().message,
        "Original fixture publish failure"
    );
    assert!(owner.verified().is_none());
    assert_eq!(executor.counts.candidate_dropped.load(Ordering::SeqCst), 0);
    executor.counts.hold_release.store(false, Ordering::SeqCst);
    assert_eq!(
        finish(OwnedRunOutcome::Pending(owner)).state,
        JobState::Failed
    );
    assert_eq!(executor.counts.publish.load(Ordering::SeqCst), 1);
    assert_eq!(executor.counts.verify.load(Ordering::SeqCst), 0);
}

#[test]
fn original_start_cas_session_and_publication_are_each_used_once() {
    let (store, blobs, executor) = fixture(Mode::Success);
    let mut owner = pending(start(&store, &blobs, &executor));
    for _ in 0..3 {
        owner = pending(owner.poll());
    }
    assert_eq!(store.current().state, JobState::Running);
    assert_eq!(executor.counts.publish.load(Ordering::SeqCst), 0);
    assert_eq!(executor.counts.dropped.load(Ordering::SeqCst), 0);
    executor.closed.store(true, Ordering::SeqCst);
    let job = finish(OwnedRunOutcome::Pending(owner));
    assert_eq!(job.state, JobState::Completed);
    for count in [
        &executor.counts.prepare,
        &executor.counts.run,
        &executor.counts.publish,
        &executor.counts.verify,
        &executor.counts.dropped,
    ] {
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn cooperative_cancel_ack_requires_original_graceful_owner_closure() {
    for forced in [false, true] {
        let (store, blobs, executor) = fixture(Mode::Cooperative { forced });
        let owner = pending(start(&store, &blobs, &executor)).poll();
        assert_eq!(store.current().state, JobState::Cancelling);
        assert_eq!(executor.counts.dropped.load(Ordering::SeqCst), 0);
        executor.closed.store(true, Ordering::SeqCst);
        let job = finish(owner);
        assert_eq!(
            job.state,
            if forced {
                JobState::Failed
            } else {
                JobState::Cancelled
            }
        );
        assert_eq!(executor.counts.publish.load(Ordering::SeqCst), 0);
    }
}

#[test]
fn cancelled_error_is_durably_failed_instead_of_becoming_cancel_ack() {
    let (store, blobs, executor) = fixture(Mode::CancelledError);
    let outcome = start(&store, &blobs, &executor);
    executor.closed.store(true, Ordering::SeqCst);
    let job = finish(outcome);
    assert_eq!(job.state, JobState::Failed);
    let failure = job.failure.unwrap();
    assert_eq!(failure.code, ErrorCode::Cancelled);
    let reference: SourceBlobRef =
        serde_json::from_value(failure.details["owned_attempt_diagnostic"].clone()).unwrap();
    let body: serde_json::Value =
        serde_json::from_str(&blobs.read_utf8(&reference).unwrap()).unwrap();
    assert_eq!(
        body["first_error"]["reported_error_request_id"],
        "original-engine-diagnostic"
    );
    assert_eq!(body["input_sha256"], job.input_sha256);
    assert_eq!(executor.counts.publish.load(Ordering::SeqCst), 0);
}

#[test]
fn diagnostic_and_record_write_failures_keep_original_attempt_and_first_error() {
    let (store, blobs, executor) = fixture(Mode::CancelledError);
    let outcome = start(&store, &blobs, &executor);
    executor.closed.store(true, Ordering::SeqCst);
    blobs.fail_write.store(true, Ordering::SeqCst);
    let owner = pending(pending(outcome).poll());
    let owner = pending(owner.poll());
    assert_eq!(store.current().state, JobState::Cancelling);
    assert_eq!(
        owner.first_error.as_ref().unwrap().code,
        ErrorCode::Cancelled
    );
    blobs.fail_write.store(false, Ordering::SeqCst);
    store.fail_failure_cas.store(true, Ordering::SeqCst);
    let owner = pending(owner.poll());
    assert_eq!(store.current().state, JobState::Cancelling);
    let job = finish(OwnedRunOutcome::Pending(owner));
    assert_eq!(
        job.failure.unwrap().message,
        "Original engine failure while cancelling"
    );
    assert!(blobs.bodies.lock().unwrap().len() >= 2);
}

#[test]
fn changed_input_on_original_attempt_never_reaches_publication() {
    let (store, blobs, executor) = fixture(Mode::Success);
    let outcome = start(&store, &blobs, &executor);
    {
        let mut job = store.job.lock().unwrap();
        job.input.parameters = serde_json::json!({"changed":true});
        job.input_sha256 = job.input.sha256().unwrap();
    }
    executor.closed.store(true, Ordering::SeqCst);
    let mut owner = pending(outcome);
    for _ in 0..4 {
        owner = pending(owner.poll());
    }
    assert!(owner.first_error.is_some());
    assert_eq!(executor.counts.publish.load(Ordering::SeqCst), 0);
    assert!(store.current().output.is_none());
}

#[test]
fn actual_driver_preserves_session_and_panic_payload_before_recording_failure() {
    let (store, blobs, executor) = fixture(Mode::Panic);
    let owner = pending(start(&store, &blobs, &executor));
    assert!(owner.panic_payload.is_some());
    assert_eq!(executor.counts.dropped.load(Ordering::SeqCst), 0);
    executor.closed.store(true, Ordering::SeqCst);
    let job = finish(OwnedRunOutcome::Pending(owner));
    assert_eq!(job.state, JobState::Failed);
    assert_eq!(job.failure.unwrap().code, ErrorCode::Internal);
    assert_eq!(executor.counts.dropped.load(Ordering::SeqCst), 1);
}

#[test]
fn late_parent_cancel_discards_unpublished_success_after_graceful_closure() {
    let (store, blobs, executor) = fixture(Mode::Success);
    let outcome = start(&store, &blobs, &executor);
    store.cancel();
    executor.closed.store(true, Ordering::SeqCst);
    let job = finish(outcome);
    assert_eq!(job.state, JobState::Cancelled);
    assert_eq!(executor.counts.publish.load(Ordering::SeqCst), 0);
    assert_eq!(executor.counts.dropped.load(Ordering::SeqCst), 1);
}

#[test]
fn persistence_panics_retain_original_carrier_and_first_engine_error() {
    let (store, blobs, executor) = fixture(Mode::CancelledError);
    let outcome = start(&store, &blobs, &executor);
    executor.closed.store(true, Ordering::SeqCst);
    let owner = pending(pending(outcome).poll());
    let original = std::ptr::from_ref(owner.as_ref());
    blobs.panic_write.store(true, Ordering::SeqCst);
    let owner = pending(owner.poll());
    assert!(std::ptr::eq(original, owner.as_ref()));
    assert!(owner.panic_payload.is_some());
    assert_eq!(owner.first_error().unwrap().code, ErrorCode::Cancelled);
    assert_eq!(store.current().state, JobState::Cancelling);
    assert_eq!(executor.counts.dropped.load(Ordering::SeqCst), 1);
    store.panic_failure_cas.store(true, Ordering::SeqCst);
    let owner = pending(owner.poll());
    assert!(std::ptr::eq(original, owner.as_ref()));
    assert_eq!(owner.first_error().unwrap().code, ErrorCode::Cancelled);
    assert_eq!(store.current().state, JobState::Cancelling);
    assert_eq!(executor.counts.dropped.load(Ordering::SeqCst), 1);
    let job = finish(OwnedRunOutcome::Pending(owner));
    assert_eq!(job.state, JobState::Failed);
    assert_eq!(job.failure.unwrap().code, ErrorCode::Cancelled);
    assert_eq!(executor.counts.prepare.load(Ordering::SeqCst), 1);
    assert_eq!(executor.counts.run.load(Ordering::SeqCst), 1);
    assert_eq!(executor.counts.publish.load(Ordering::SeqCst), 0);
    assert_eq!(executor.counts.dropped.load(Ordering::SeqCst), 1);
}

#[test]
fn settled_observation_panic_keeps_original_candidate_until_durable_failure() {
    let (store, blobs, executor) = fixture(Mode::Success);
    let owner = pending(start(&store, &blobs, &executor));
    executor.closed.store(true, Ordering::SeqCst);
    let owner = pending(owner.poll());
    let original = std::ptr::from_ref(owner.as_ref());
    blobs.fail_write.store(true, Ordering::SeqCst);
    store.panic_get.store(true, Ordering::SeqCst);
    let owner = pending(owner.poll());
    assert!(std::ptr::eq(original, owner.as_ref()));
    assert!(owner.panic_payload.is_some());
    assert_eq!(owner.first_error().unwrap().code, ErrorCode::Internal);
    assert_eq!(store.current().state, JobState::Running);
    assert_eq!(executor.counts.dropped.load(Ordering::SeqCst), 1);
    assert_eq!(executor.counts.candidate_dropped.load(Ordering::SeqCst), 0);
    assert_eq!(executor.counts.publish.load(Ordering::SeqCst), 0);
    blobs.fail_write.store(false, Ordering::SeqCst);
    let job = finish(OwnedRunOutcome::Pending(owner));
    assert_eq!(job.state, JobState::Failed);
    assert_eq!(executor.counts.dropped.load(Ordering::SeqCst), 1);
    assert_eq!(executor.counts.candidate_dropped.load(Ordering::SeqCst), 1);
}

#[test]
fn committed_publication_cas_panic_cannot_release_original_candidate() {
    let (store, blobs, executor) = fixture(Mode::Success);
    let owner = pending(start(&store, &blobs, &executor));
    executor.closed.store(true, Ordering::SeqCst);
    let owner = pending(owner.poll());
    let original = std::ptr::from_ref(owner.as_ref());
    blobs.fail_write.store(true, Ordering::SeqCst);
    store
        .panic_after_publication_cas
        .store(true, Ordering::SeqCst);
    let owner = pending(owner.poll());
    assert!(std::ptr::eq(original, owner.as_ref()));
    assert!(owner.panic_payload.is_some());
    assert!(store.current().publication_started);
    assert_eq!(executor.counts.dropped.load(Ordering::SeqCst), 1);
    assert_eq!(executor.counts.candidate_dropped.load(Ordering::SeqCst), 0);
    assert_eq!(executor.counts.publish.load(Ordering::SeqCst), 0);
    blobs.fail_write.store(false, Ordering::SeqCst);
    let job = finish(OwnedRunOutcome::Pending(owner));
    assert_eq!(job.state, JobState::Failed);
    assert!(job.output.is_none());
    assert_eq!(executor.counts.prepare.load(Ordering::SeqCst), 1);
    assert_eq!(executor.counts.run.load(Ordering::SeqCst), 1);
    assert_eq!(executor.counts.publish.load(Ordering::SeqCst), 0);
    assert_eq!(executor.counts.dropped.load(Ordering::SeqCst), 1);
    assert_eq!(executor.counts.candidate_dropped.load(Ordering::SeqCst), 1);
}

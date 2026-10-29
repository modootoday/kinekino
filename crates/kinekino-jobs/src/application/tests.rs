use super::*;
use crate::domain::*;
use kinekino_media_contracts::{
    api_values::DecimalU64,
    errors::{ApiError, ErrorCode},
};
use serde_json::{Value, json};
use std::sync::{
    Arc, Barrier, Mutex,
    atomic::{AtomicBool, Ordering},
};
struct Store(Mutex<Vec<JobEvent>>);
impl Store {
    fn new() -> Self {
        let job = JobRecord::queued(
            "job-00000000000000000000000000000001"
                .to_owned()
                .try_into()
                .unwrap(),
            PreparedJobInput {
                operation: "subtitles.convert".into(),
                parameters: json!({}),
                source: SourceBlobRef {
                    sha256: "a".repeat(64),
                    bytes: DecimalU64(1),
                },
                execution_timeout_ms: DecimalU64(30000),
            },
        )
        .unwrap();
        Self(Mutex::new(vec![JobEvent {
            sequence: DecimalU64(1),
            job,
        }]))
    }
    fn id(&self) -> JobId {
        self.0.lock().unwrap()[0].job.job_id.clone()
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
        Ok(self.0.lock().unwrap().last().unwrap().job.clone())
    }
    fn publish(&self, expected: Option<u64>, job: &JobRecord) -> Result<JobEvent, ApiError> {
        let mut events = self.0.lock().unwrap();
        if Some(events.last().unwrap().job.revision.0) != expected {
            return Err(ApiError::new(ErrorCode::RevisionConflict, "changed"));
        }
        job.validate_successor(&events.last().unwrap().job).unwrap();
        let event = JobEvent {
            sequence: DecimalU64(events.len() as u64 + 1),
            job: job.clone(),
        };
        events.push(event.clone());
        Ok(event)
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
struct Blobs;
impl SourceBlobStore for Blobs {
    fn put_utf8(&self, _: &str) -> Result<SourceBlobRef, ApiError> {
        unreachable!()
    }
    fn read_utf8(&self, _: &SourceBlobRef) -> Result<String, ApiError> {
        unreachable!()
    }
}
struct Executor {
    barrier: Arc<Barrier>,
    published: Arc<AtomicBool>,
    reserve_first: bool,
}
fn output() -> VerifiedJobOutput {
    VerifiedJobOutput {
        path: "output.vtt".into(),
        sha256: "b".repeat(64),
        bytes: DecimalU64(20),
        receipt: json!({"cue_count":1}),
    }
}
impl JobExecutor for Executor {
    fn supports(&self, _: &str) -> bool {
        true
    }
    fn prepare(
        &self,
        _: &str,
        _: Value,
        _: Option<&str>,
        _: &dyn SourceBlobStore,
    ) -> Result<PreparedJobInput, ApiError> {
        unreachable!()
    }
    fn execute(
        &self,
        _: &PreparedJobInput,
        _: &dyn SourceBlobStore,
        control: &dyn ExecutionControl,
    ) -> Result<VerifiedJobOutput, ApiError> {
        if self.reserve_first {
            control.reserve_publication()?;
        }
        self.barrier.wait();
        self.barrier.wait();
        if !self.reserve_first {
            control.reserve_publication()?;
        }
        self.published.store(true, Ordering::SeqCst);
        control.begin_verification()?;
        Ok(output())
    }
}
#[test]
fn worker_acknowledges_cancel_before_publication_without_claiming_output() {
    let store = Arc::new(Store::new());
    let id = store.id();
    let barrier = Arc::new(Barrier::new(2));
    let published = Arc::new(AtomicBool::new(false));
    let executor = Executor {
        barrier: barrier.clone(),
        published: published.clone(),
        reserve_first: false,
    };
    let target = store.clone();
    let worker_id = id.clone();
    let worker =
        std::thread::spawn(move || run_queued(target.as_ref(), &executor, &Blobs, &worker_id, 1));
    barrier.wait();
    let (requested, disposition) = request_cancel(store.as_ref(), &id, 1).unwrap();
    assert_eq!(requested.state, JobState::Cancelling);
    assert_eq!(disposition, CancelDisposition::Requested);
    assert!(!published.load(Ordering::SeqCst));
    barrier.wait();
    let acknowledged = worker.join().unwrap().unwrap();
    assert_eq!(acknowledged.state, JobState::Cancelled);
    assert!(acknowledged.output.is_none());
    assert!(!published.load(Ordering::SeqCst));
}
#[test]
fn publication_reservation_wins_race_and_preserves_completed_receipt() {
    let store = Arc::new(Store::new());
    let id = store.id();
    let barrier = Arc::new(Barrier::new(2));
    let published = Arc::new(AtomicBool::new(false));
    let executor = Executor {
        barrier: barrier.clone(),
        published: published.clone(),
        reserve_first: true,
    };
    let target = store.clone();
    let worker_id = id.clone();
    let worker =
        std::thread::spawn(move || run_queued(target.as_ref(), &executor, &Blobs, &worker_id, 1));
    barrier.wait();
    let (current, disposition) = request_cancel(store.as_ref(), &id, 1).unwrap();
    assert_eq!(disposition, CancelDisposition::PublicationInProgress);
    assert!(current.publication_started);
    barrier.wait();
    let completed = worker.join().unwrap().unwrap();
    assert_eq!(completed.state, JobState::Completed);
    assert!(published.load(Ordering::SeqCst));
    assert!(completed.output.is_some());
    assert_eq!(
        request_cancel(store.as_ref(), &id, 1).unwrap().1,
        CancelDisposition::AlreadyTerminal
    );
    assert_eq!(
        request_cancel(store.as_ref(), &id, 2).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
}
struct InvalidExecutor;
impl JobExecutor for InvalidExecutor {
    fn supports(&self, _: &str) -> bool {
        true
    }
    fn prepare(
        &self,
        _: &str,
        _: Value,
        _: Option<&str>,
        _: &dyn SourceBlobStore,
    ) -> Result<PreparedJobInput, ApiError> {
        unreachable!()
    }
    fn execute(
        &self,
        _: &PreparedJobInput,
        _: &dyn SourceBlobStore,
        _: &dyn ExecutionControl,
    ) -> Result<VerifiedJobOutput, ApiError> {
        Ok(output())
    }
}
#[test]
fn executor_result_without_publication_verification_never_becomes_completed() {
    let store = Store::new();
    let id = store.id();
    let result = run_queued(&store, &InvalidExecutor, &Blobs, &id, 1).unwrap();
    assert_eq!(result.state, JobState::Failed);
    assert!(result.output.is_none());
    assert_eq!(result.failure.unwrap().code, ErrorCode::EngineFailed);
}

struct IdentityExecutor {
    store: Arc<Store>,
    captured: Arc<Mutex<Vec<JobAttemptIdentityV1>>>,
    cancel_inside: bool,
}
impl JobExecutor for IdentityExecutor {
    fn supports(&self, _: &str) -> bool {
        true
    }
    fn prepare(
        &self,
        _: &str,
        _: Value,
        _: Option<&str>,
        _: &dyn SourceBlobStore,
    ) -> Result<PreparedJobInput, ApiError> {
        unreachable!()
    }
    fn execute(
        &self,
        input: &PreparedJobInput,
        _: &dyn SourceBlobStore,
        control: &dyn ExecutionControl,
    ) -> Result<VerifiedJobOutput, ApiError> {
        let identity = control
            .execution_identity()
            .expect("Original worker must bind its running record")
            .clone();
        assert_eq!(identity.input_sha256(), input.sha256().unwrap());
        assert_eq!(identity.source_blob_sha256(), input.source.sha256);
        let current = self.store.get(identity.job_id())?;
        assert_eq!(current.state, JobState::Running);
        assert_eq!(identity.attempt(), current.attempt.0);
        self.captured.lock().unwrap().push(identity.clone());
        if self.cancel_inside {
            request_cancel(self.store.as_ref(), identity.job_id(), identity.attempt())?;
            assert!(control.cancellation_requested());
            assert_eq!(control.execution_identity(), Some(&identity));
        }
        Err(ApiError::new(
            ErrorCode::NeedsInput,
            "synthetic identity fixture, no external execution",
        ))
    }
}
fn identity_fixture(cancel_inside: bool) -> (Arc<Store>, IdentityExecutor) {
    let store = Arc::new(Store::new());
    let executor = IdentityExecutor {
        store: store.clone(),
        captured: Arc::new(Mutex::new(Vec::new())),
        cancel_inside,
    };
    (store, executor)
}

#[test]
fn original_worker_identity_distinguishes_first_and_retry_attempt_with_same_input() {
    let (store, executor) = identity_fixture(false);
    let id = store.id();
    let original = store.get(&id).unwrap();
    let first = run_queued(store.as_ref(), &executor, &Blobs, &id, 1).unwrap();
    assert_eq!(first.state, JobState::NeedsInput);
    let retried =
        super::mutations::change_attempt(store.as_ref(), &id, 1, JobRecord::retry).unwrap();
    assert_eq!(retried.attempt.0, 2);
    assert_eq!(
        run_queued(store.as_ref(), &executor, &Blobs, &id, 1)
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    let second = run_queued(store.as_ref(), &executor, &Blobs, &id, 2).unwrap();
    assert_eq!(second.state, JobState::NeedsInput);
    let identities = executor.captured.lock().unwrap();
    assert_eq!(identities.len(), 2);
    for (index, identity) in identities.iter().enumerate() {
        assert_eq!(identity.job_id(), &id);
        assert_eq!(identity.attempt(), index as u64 + 1);
        assert_eq!(identity.input_sha256(), original.input_sha256);
        assert_eq!(identity.source_blob_sha256(), original.input.source.sha256);
        assert_ne!(identity.input_sha256(), identity.source_blob_sha256());
        assert_eq!(identity.submission_request_sha256(), None);
    }
    assert_eq!(identities[0].attempt(), 1);
}
#[test]
fn original_submission_fingerprint_is_preserved_without_inventing_request_id() {
    let (store, executor) = identity_fixture(false);
    let id = store.id();
    store.0.lock().unwrap()[0].job.submission = Some(SubmissionRef {
        key_sha256: "c".repeat(64),
        request_sha256: "d".repeat(64),
    });
    run_queued(store.as_ref(), &executor, &Blobs, &id, 1).unwrap();
    let captured = executor.captured.lock().unwrap();
    assert_eq!(
        captured[0].submission_request_sha256(),
        Some("d".repeat(64).as_str())
    );
    assert_ne!(
        captured[0].submission_request_sha256(),
        Some("c".repeat(64).as_str())
    );
}
#[test]
fn unbound_control_keeps_default_none_instead_of_a_synthetic_identity() {
    struct Unbound;
    impl ExecutionControl for Unbound {
        fn cancellation_requested(&self) -> bool {
            false
        }
        fn deadline(&self) -> std::time::Instant {
            std::time::Instant::now()
        }
        fn reserve_publication(&self) -> Result<(), ApiError> {
            unreachable!()
        }
        fn begin_verification(&self) -> Result<(), ApiError> {
            unreachable!()
        }
    }
    assert_eq!(Unbound.execution_identity(), None);
}
#[test]
fn missing_or_invalid_source_and_input_hashes_never_reach_executor_identity() {
    for field in ["source", "input"] {
        let (store, executor) = identity_fixture(false);
        let id = store.id();
        {
            let mut events = store.0.lock().unwrap();
            if field == "source" {
                events[0].job.input.source.sha256.clear();
            } else {
                events[0].job.input_sha256.clear();
            }
        }
        assert_eq!(
            run_queued(store.as_ref(), &executor, &Blobs, &id, 1)
                .unwrap_err()
                .code,
            ErrorCode::NeedsInput
        );
        assert!(executor.captured.lock().unwrap().is_empty());
        assert_eq!(store.get(&id).unwrap().state, JobState::Queued);
    }
}
#[test]
fn captured_identity_is_immutable_during_cancel_without_changing_legacy_ack() {
    let (store, executor) = identity_fixture(true);
    let id = store.id();
    let cancelled = run_queued(store.as_ref(), &executor, &Blobs, &id, 1).unwrap();
    assert_eq!(cancelled.state, JobState::Cancelled);
    assert!(cancelled.output.is_none());
    let captured = executor.captured.lock().unwrap();
    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0].job_id(), &id);
    assert_eq!(captured[0].attempt(), 1);
    assert_eq!(captured[0].input_sha256(), cancelled.input_sha256);
}

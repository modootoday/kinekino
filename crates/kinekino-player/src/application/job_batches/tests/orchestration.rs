//! Fault-injected port tests, not native persistence or actual worker acceptance.
use super::*;
use crate::application::jobs::{GetJob, JobsApi, RetryJob, SubmittedJob};
use std::sync::{
    Arc, Mutex, MutexGuard,
    atomic::{AtomicUsize, Ordering},
};

struct Store {
    document: Mutex<BatchDocument>,
    events: Arc<Mutex<Vec<String>>>,
    fail_publish: usize,
    publishes: AtomicUsize,
}
struct Lease<'a> {
    store: &'a Store,
    document: MutexGuard<'a, BatchDocument>,
}
impl BatchStore for Store {
    fn scope_sha256(&self) -> &str {
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    }
    fn create(&self, _: &BatchDocument) -> Result<(), ApiError> {
        panic!("Not used")
    }
    fn acquire(&self, _: &str) -> Result<Box<dyn BatchLease + '_>, ApiError> {
        Ok(Box::new(Lease {
            store: self,
            document: self.document.lock().unwrap(),
        }))
    }
}
impl BatchLease for Lease<'_> {
    fn load(&self) -> Result<BatchDocument, ApiError> {
        Ok(reopen(&self.document))
    }
    fn publish(&mut self, revision: u64, next: &BatchDocument) -> Result<(), ApiError> {
        assert_eq!(revision, self.document.revision.0);
        validate_successor(&self.document, next)?;
        let ordinal = self.store.publishes.fetch_add(1, Ordering::SeqCst) + 1;
        if ordinal == self.store.fail_publish {
            return Err(ApiError::new(
                ErrorCode::EngineFailed,
                "fixture durable publish failure",
            ));
        }
        self.store.events.lock().unwrap().push("publish".into());
        *self.document = reopen(next);
        Ok(())
    }
}
struct Jobs {
    job: Mutex<JobRecord>,
    events: Arc<Mutex<Vec<String>>>,
    retry_calls: AtomicUsize,
    retry_ack_lost: bool,
    get_failed: bool,
}
impl JobsApi for Jobs {
    fn submit(&self, _: SubmitJob, key: &str) -> Result<SubmittedJob, ApiError> {
        self.events.lock().unwrap().push(format!("submit:{key}"));
        Ok(SubmittedJob {
            job: self.job.lock().unwrap().clone(),
            created: false,
        })
    }
    fn get(&self, request: GetJob) -> Result<JobRecord, ApiError> {
        self.events.lock().unwrap().push("get".into());
        if self.get_failed {
            return Err(ApiError::new(ErrorCode::Timeout, "fixture get timeout"));
        }
        let job = self.job.lock().unwrap();
        assert_eq!(request.job_id, job.job_id);
        Ok(job.clone())
    }
    fn retry(&self, request: RetryJob, revision: u64) -> Result<JobRecord, ApiError> {
        self.events.lock().unwrap().push("retry".into());
        self.retry_calls.fetch_add(1, Ordering::SeqCst);
        let mut job = self.job.lock().unwrap();
        assert_eq!(request.job_id, job.job_id);
        assert_eq!(revision, job.revision.0);
        job.retry().map_err(super::super::document::invalid)?;
        if self.retry_ack_lost {
            let mut error = ApiError::new(ErrorCode::Timeout, "fixture retry ACK unobserved");
            error.details = json!({"delivery":"execution_unknown"});
            return Err(error);
        }
        Ok(job.clone())
    }
}
fn setup(document: BatchDocument, fail_publish: usize) -> (Store, Jobs) {
    let value = document.entries[0]
        .progress
        .job()
        .cloned()
        .unwrap_or_else(|| job(&document));
    let events = Arc::new(Mutex::new(vec![]));
    (
        Store {
            document: Mutex::new(document),
            events: events.clone(),
            fail_publish,
            publishes: AtomicUsize::new(0),
        },
        Jobs {
            job: Mutex::new(value),
            events,
            retry_calls: AtomicUsize::new(0),
            retry_ack_lost: false,
            get_failed: false,
        },
    )
}
fn failed() -> BatchDocument {
    let bound = bound();
    let mut value = bound.entries[0].progress.job().unwrap().clone();
    value.start().unwrap();
    value
        .fail(
            kinekino_jobs::domain::JobFailure {
                code: ErrorCode::EngineFailed,
                message: "fixture failed".into(),
                remediation: String::new(),
                details: json!({}),
                retryable: false,
            },
            false,
        )
        .unwrap();
    next(&bound, observed(value))
}
#[test]
fn failed_intent_never_calls_jobs_and_failed_ack_publish_replays_same_key() {
    let initial = document();
    let (store, jobs) = setup(reopen(&initial), 1);
    assert!(run(&store, &jobs, "batch-1", 1, BatchAction::Submit).is_err());
    assert!(store.events.lock().unwrap().is_empty());
    let (store, jobs) = setup(initial, 2);
    assert!(run(&store, &jobs, "batch-1", 1, BatchAction::Submit).is_err());
    assert!(matches!(
        store.document.lock().unwrap().entries[0].progress,
        BatchProgress::SubmitPending
    ));
    let result = run(&store, &jobs, "batch-1", 2, BatchAction::Submit).unwrap();
    assert!(matches!(
        result.entries[0].progress,
        BatchProgress::Observed { .. }
    ));
    let events = store.events.lock().unwrap();
    assert_eq!(events[0], "publish");
    assert_eq!(events[1], events[2]); // Explicit replay uses the original key.
}
#[test]
fn retry_ack_loss_is_reconciled_without_a_second_retry() {
    let original = failed();
    let revision = original.revision.0;
    let (store, mut jobs) = setup(original, 0);
    jobs.retry_ack_lost = true;
    let pending = run(&store, &jobs, "batch-1", revision, BatchAction::RetryFailed).unwrap();
    let BatchProgress::RetryPending {
        last_error: Some(error),
        ..
    } = &pending.entries[0].progress
    else {
        panic!("Missing retained retry intent")
    };
    assert_eq!(error.details["delivery"], "execution_unknown");
    let reconciled = run(
        &store,
        &jobs,
        "batch-1",
        pending.revision.0,
        BatchAction::RetryFailed,
    )
    .unwrap();
    assert_eq!(jobs.retry_calls.load(Ordering::SeqCst), 1);
    assert_eq!(reconciled.entries[0].progress.job().unwrap().attempt.0, 2);
    assert_eq!(
        reconciled.entries[0].progress.job().unwrap().state,
        JobState::Queued
    );
}
#[test]
fn failed_refresh_never_grants_retry_and_stale_batch_revision_never_calls_jobs() {
    let original = failed();
    let revision = original.revision.0;
    let (store, mut jobs) = setup(original, 0);
    assert!(
        run(
            &store,
            &jobs,
            "batch-1",
            revision - 1,
            BatchAction::RetryFailed
        )
        .is_err()
    );
    assert!(store.events.lock().unwrap().is_empty());
    jobs.get_failed = true;
    let result = run(&store, &jobs, "batch-1", revision, BatchAction::RetryFailed).unwrap();
    assert!(matches!(
        result.entries[0].progress,
        BatchProgress::Observed {
            last_error: Some(_),
            ..
        }
    ));
    assert_eq!(jobs.retry_calls.load(Ordering::SeqCst), 0);
}
#[test]
fn completed_output_is_preserved_without_any_jobs_call() {
    let original = bound();
    let mut value = original.entries[0].progress.job().unwrap().clone();
    value.start().unwrap();
    value.begin_publication().unwrap();
    value.begin_verification().unwrap();
    value
        .complete(VerifiedJobOutput {
            path: "C:/fixture/result.vtt".into(),
            sha256: "e".repeat(64),
            bytes: DecimalU64(42),
            receipt: json!({"fixture":true}),
        })
        .unwrap();
    let complete = next(&original, observed(value));
    let expected = serde_json::to_vec(&complete).unwrap();
    let (store, jobs) = setup(complete, 0);
    for action in [
        BatchAction::Submit,
        BatchAction::Refresh,
        BatchAction::RetryFailed,
    ] {
        let revision = store.document.lock().unwrap().revision.0;
        let result = run(&store, &jobs, "batch-1", revision, action).unwrap();
        assert_eq!(serde_json::to_vec(&result).unwrap(), expected);
    }
    assert!(store.events.lock().unwrap().is_empty());
}

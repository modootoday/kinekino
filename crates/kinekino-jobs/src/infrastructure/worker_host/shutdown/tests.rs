use super::*;
use crate::{
    application::{
        EventPage, ExecutionOwner, ExecutionSession, JobAdmission, JobEvent, JobListCursor,
        JobPage, JobService, JobStore,
    },
    domain::{JobId, JobRecord},
    infrastructure::worker_host::Wake,
};
use std::{
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};

struct Lease(Arc<AtomicBool>);
impl ExecutionOwner for Lease {}
impl Drop for Lease {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}
struct Store {
    leased: Arc<AtomicBool>,
    fail_list: AtomicBool,
    panic_list: AtomicBool,
    lists: AtomicUsize,
}
impl JobStore for Store {
    fn find_submission(&self, _: &str) -> Result<Option<JobRecord>, ApiError> {
        unreachable!()
    }
    fn admit(&self, _: &JobRecord) -> Result<JobAdmission, ApiError> {
        unreachable!()
    }
    fn get(&self, _: &JobId) -> Result<JobRecord, ApiError> {
        unreachable!()
    }
    fn publish(&self, _: Option<u64>, _: &JobRecord) -> Result<JobEvent, ApiError> {
        unreachable!()
    }
    fn read_events(&self, _: u64, _: usize) -> Result<EventPage, ApiError> {
        unreachable!()
    }
    fn list(&self, _: Option<&JobListCursor>, _: usize) -> Result<JobPage, ApiError> {
        self.lists.fetch_add(1, Ordering::SeqCst);
        assert!(
            !self.panic_list.swap(false, Ordering::SeqCst),
            "Synthetic original store adapter panic"
        );
        if self.fail_list.load(Ordering::SeqCst) {
            Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Original store traversal failed",
            ))
        } else {
            Ok(JobPage {
                jobs: vec![],
                next_cursor: None,
            })
        }
    }
    fn acquire_execution_owner(&self) -> Result<Box<dyn ExecutionOwner>, ApiError> {
        self.leased
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .map_err(|_| {
                ApiError::new(ErrorCode::RevisionConflict, "Original execution lease held")
            })?;
        Ok(Box::new(Lease(Arc::clone(&self.leased))))
    }
}
fn fixture(panic: bool) -> (WorkerHost, Arc<Store>, mpsc::SyncSender<()>) {
    let store = Arc::new(Store {
        leased: Arc::new(AtomicBool::new(false)),
        fail_list: AtomicBool::new(false),
        panic_list: AtomicBool::new(false),
        lists: AtomicUsize::new(0),
    });
    let (session, _) = ExecutionSession::acquire(store.clone()).unwrap();
    let (release, ready) = mpsc::sync_channel(0);
    let worker = thread::spawn(move || {
        ready.recv().unwrap();
        assert!(!panic, "Synthetic original worker panic");
        crate::infrastructure::worker_host::WorkerExit::Finished(Ok(()))
    });
    let host = WorkerHost {
        service: Arc::new(JobService::read_only(store.clone())),
        session: Some(session),
        worker: Some(worker),
        retained: None,
        retained_attempt: Arc::new(AtomicBool::new(false)),
        allow_owned: false,
        store: store.clone(),
        running: Arc::new(AtomicBool::new(true)),
        stop: Arc::new(AtomicBool::new(false)),
        wake: Arc::new(Wake {
            signal: Condvar::new(),
            mutex: Mutex::new(()),
        }),
        admission: Arc::new(Mutex::new(())),
        closed: false,
    };
    (host, store, release)
}
fn pending(outcome: WorkerShutdownOutcome) -> PendingWorkerHost {
    match outcome {
        WorkerShutdownOutcome::Pending(owner) => owner,
        _ => panic!("Original owner must remain pending"),
    }
}
fn complete(outcome: WorkerShutdownOutcome) -> WorkerShutdownComplete {
    match outcome {
        WorkerShutdownOutcome::Complete(receipt) => receipt,
        _ => panic!("Expected original completed shutdown"),
    }
}
fn release_worker(owner: &PendingWorkerHost, release: mpsc::SyncSender<()>) {
    release.send(()).unwrap();
    let cutoff = Instant::now() + Duration::from_secs(5);
    while !owner.worker_finished() && Instant::now() < cutoff {
        thread::yield_now();
    }
    assert!(owner.worker_finished(), "Fixture worker did not finish");
}

#[test]
fn unfinished_original_handle_and_session_survive_poll_without_join() {
    let (host, store, release) = fixture(false);
    let original_thread = host.worker.as_ref().unwrap().thread().id();
    let deadline = Instant::now() + Duration::from_secs(30);
    let owner = pending(host.begin_shutdown(deadline).poll(deadline));
    assert_eq!(
        owner.host.worker.as_ref().unwrap().thread().id(),
        original_thread
    );
    assert!(owner.host.session.is_some());
    assert!(owner.host.closed);
    assert!(!owner.host.running.load(Ordering::Acquire));
    assert!(store.acquire_execution_owner().is_err());
    release_worker(&owner, release);
    let result = complete(owner.poll(deadline));
    assert!(result.errors.first().is_none());
    assert!(result.receipt.is_some());
    assert!(!store.leased.load(Ordering::SeqCst));
    assert!(store.acquire_execution_owner().is_ok());
}

#[test]
fn durable_finalization_failure_retains_session_and_bounded_original_errors() {
    let (host, store, release) = fixture(false);
    let deadline = Instant::now() + Duration::from_secs(30);
    let owner = host.begin_shutdown(deadline);
    release_worker(&owner, release);
    store.fail_list.store(true, Ordering::SeqCst);
    let mut owner = pending(owner.poll(deadline));
    for _ in 0..20 {
        owner = pending(owner.poll(deadline));
    }
    assert!(owner.host.worker.is_none());
    assert!(owner.host.session.is_some());
    assert!(store.acquire_execution_owner().is_err());
    assert_eq!(
        owner.errors().first().unwrap().message,
        "Original store traversal failed"
    );
    assert_eq!(owner.errors().secondary().len(), 8);
    assert!(owner.errors().overflow());
    store.fail_list.store(false, Ordering::SeqCst);
    let result = complete(owner.poll(deadline));
    assert!(result.receipt.is_some());
    assert_eq!(
        result.errors.first().unwrap().code,
        ErrorCode::SourceExpired
    );
    assert!(!store.leased.load(Ordering::SeqCst));
}

#[test]
fn original_join_panic_never_runs_terminal_ack_sweep_or_releases_lease() {
    let (host, store, release) = fixture(true);
    let deadline = Instant::now() + Duration::from_secs(30);
    let owner = host.begin_shutdown(deadline);
    release_worker(&owner, release);
    let lists = store.lists.load(Ordering::SeqCst);
    let owner = pending(owner.poll(deadline));
    assert!(owner.join_uncertain());
    assert!(owner.host.worker.is_none());
    assert!(owner.host.session.is_some());
    let owner = pending(owner.poll(deadline));
    assert!(store.acquire_execution_owner().is_err());
    assert_eq!(store.lists.load(Ordering::SeqCst), lists);
    assert_eq!(
        owner.errors().first().unwrap().message,
        "Owned job worker panicked"
    );
}

#[test]
fn original_shutdown_deadline_only_shortens_and_timeout_survives_eventual_close() {
    let (host, store, release) = fixture(false);
    let deadline = Instant::now().checked_sub(Duration::from_secs(1)).unwrap();
    let owner = pending(
        host.begin_shutdown(deadline)
            .poll(Instant::now() + Duration::from_secs(90)),
    );
    assert_eq!(owner.deadline(), deadline);
    assert_eq!(owner.errors().first().unwrap().code, ErrorCode::Timeout);
    assert!(store.acquire_execution_owner().is_err());
    release_worker(&owner, release);
    let result = complete(owner.poll(deadline));
    assert_eq!(result.errors.first().unwrap().code, ErrorCode::Timeout);
    assert!(result.receipt.is_some());
}

#[test]
fn admission_and_cancel_errors_remain_distinct_in_pending_shutdown() {
    let (host, store, release) = fixture(false);
    let admission = Arc::clone(&host.admission);
    assert!(
        std::panic::catch_unwind(move || {
            let _guard = admission.lock().unwrap();
            panic!("Synthetic admission poison");
        })
        .is_err()
    );
    store.fail_list.store(true, Ordering::SeqCst);
    let deadline = Instant::now() + Duration::from_secs(30);
    let owner = pending(host.begin_shutdown(deadline).poll(deadline));
    assert_eq!(
        owner.errors().first().unwrap().message,
        "Job shutdown gate unavailable"
    );
    assert_eq!(
        owner.errors().secondary()[0].message(),
        "Original store traversal failed"
    );
    store.fail_list.store(false, Ordering::SeqCst);
    release_worker(&owner, release);
    let result = complete(owner.poll(deadline));
    assert_eq!(
        result.errors.first().unwrap().message,
        "Job shutdown gate unavailable"
    );
    assert_eq!(result.errors.secondary().len(), 1);
}

#[test]
fn preparation_store_panic_signals_same_worker_and_retains_original_lease() {
    let (host, store, release) = fixture(false);
    let original_thread = host.worker.as_ref().unwrap().thread().id();
    store.panic_list.store(true, Ordering::SeqCst);
    let deadline = Instant::now() + Duration::from_secs(30);
    let owner = host.begin_shutdown(deadline);
    assert!(owner.panic_payload.is_some());
    assert!(owner.host.closed);
    assert!(!owner.host.running.load(Ordering::Acquire));
    assert!(owner.host.stop.load(Ordering::Acquire));
    assert_eq!(
        owner.host.worker.as_ref().unwrap().thread().id(),
        original_thread
    );
    assert!(store.acquire_execution_owner().is_err());
    let lists = store.lists.load(Ordering::SeqCst);
    let owner = pending(owner.poll(deadline));
    assert_eq!(store.lists.load(Ordering::SeqCst), lists);
    assert_eq!(owner.errors().first().unwrap().code, ErrorCode::Internal);
    release_worker(&owner, release);
    let result = complete(owner.poll(deadline));
    assert!(result.receipt.is_some());
    assert_eq!(result.errors.first().unwrap().code, ErrorCode::Internal);
    assert!(store.acquire_execution_owner().is_ok());
}

#[test]
fn finalization_store_panic_retains_original_session_until_durable_retry() {
    let (host, store, release) = fixture(false);
    let deadline = Instant::now() + Duration::from_secs(30);
    let owner = host.begin_shutdown(deadline);
    release_worker(&owner, release);
    store.panic_list.store(true, Ordering::SeqCst);
    let owner = pending(owner.poll(deadline));
    assert!(owner.panic_payload.is_some());
    assert!(owner.host.worker.is_none());
    assert!(owner.host.session.is_some());
    assert_eq!(owner.deadline(), deadline);
    assert!(!owner.join_uncertain());
    assert!(store.acquire_execution_owner().is_err());
    assert_eq!(
        owner.errors().first().unwrap().message,
        "Original job shutdown finalization panicked"
    );
    let result = complete(owner.poll(deadline));
    assert!(result.receipt.is_some());
    assert_eq!(result.errors.first().unwrap().code, ErrorCode::Internal);
    assert!(store.acquire_execution_owner().is_ok());
}

#[test]
fn admission_error_remains_first_when_original_cancellation_store_panics() {
    let (host, store, release) = fixture(false);
    let admission = Arc::clone(&host.admission);
    assert!(
        catch_unwind(move || {
            let _guard = admission.lock().unwrap();
            panic!("Synthetic original admission poison");
        })
        .is_err()
    );
    store.panic_list.store(true, Ordering::SeqCst);
    let deadline = Instant::now() + Duration::from_secs(30);
    let owner = host.begin_shutdown(deadline);
    assert_eq!(
        owner.errors().first().unwrap().message,
        "Job shutdown gate unavailable"
    );
    assert_eq!(owner.errors().secondary()[0].code(), ErrorCode::Internal);
    assert!(owner.panic_payload.is_some());
    assert!(store.acquire_execution_owner().is_err());
    release_worker(&owner, release);
    let result = complete(owner.poll(deadline));
    assert!(result.receipt.is_some());
    assert_eq!(
        result.errors.first().unwrap().message,
        "Job shutdown gate unavailable"
    );
    assert_eq!(result.errors.secondary().len(), 1);
    assert!(store.acquire_execution_owner().is_ok());
}

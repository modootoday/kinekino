use super::*;
use crate::{
    application::{EventPage, ExecutionOwner, JobAdmission, JobEvent, JobListCursor, JobPage},
    domain::{JobId, JobRecord},
};
use std::sync::atomic::AtomicUsize;

struct Store {
    calls: AtomicUsize,
    first_list_fails: bool,
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
    fn acquire_execution_owner(&self) -> Result<Box<dyn ExecutionOwner>, ApiError> {
        unreachable!()
    }
    fn list(&self, _: Option<&JobListCursor>, _: usize) -> Result<JobPage, ApiError> {
        let call = self.calls.fetch_add(1, Ordering::SeqCst);
        if call == 0 && !self.first_list_fails {
            return Ok(JobPage {
                jobs: vec![],
                next_cursor: None,
            });
        }
        Err(ApiError::new(
            ErrorCode::SourceExpired,
            if call == 0 {
                "Original cancellation enumeration failed"
            } else {
                "Finalization enumeration failed"
            },
        ))
    }
}

fn fixture(first_list_fails: bool) -> (WorkerHost, Arc<Store>) {
    let store = Arc::new(Store {
        calls: AtomicUsize::new(0),
        first_list_fails,
    });
    let host = WorkerHost {
        service: Arc::new(JobService::read_only(store.clone())),
        session: None,
        worker: Some(thread::spawn(|| {
            super::WorkerExit::Finished(Err(ApiError::new(
                ErrorCode::EngineFailed,
                "Original worker failure",
            )))
        })),
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
    (host, store)
}

#[test]
fn admission_poison_remains_first_through_cancel_worker_and_finalize_errors() {
    let (mut host, store) = fixture(true);
    let admission = Arc::clone(&host.admission);
    assert!(
        std::panic::catch_unwind(move || {
            let _guard = admission.lock().unwrap();
            panic!("Synthetic admission poison");
        })
        .is_err()
    );
    let error = host.stop_owned().err().unwrap();
    assert_eq!(error.code, ErrorCode::EngineFailed);
    assert_eq!(error.message, "Job shutdown gate unavailable");
    assert_eq!(store.calls.load(Ordering::SeqCst), 2);
    assert!(host.worker.is_none());
    assert!(!host.running.load(Ordering::Acquire));
    assert!(host.closed);
}

#[test]
fn cancellation_enumeration_failure_remains_first_after_worker_join_failure() {
    let (mut host, store) = fixture(true);
    let error = host.stop_owned().err().unwrap();
    assert_eq!(error.code, ErrorCode::SourceExpired);
    assert_eq!(error.message, "Original cancellation enumeration failed");
    assert_eq!(store.calls.load(Ordering::SeqCst), 2);
    assert!(host.worker.is_none());
}

#[test]
fn worker_failure_remains_first_when_only_finalization_enumeration_fails() {
    let (mut host, store) = fixture(false);
    let error = host.stop_owned().err().unwrap();
    assert_eq!(error.code, ErrorCode::EngineFailed);
    assert_eq!(error.message, "Original worker failure");
    assert_eq!(store.calls.load(Ordering::SeqCst), 2);
    assert!(host.worker.is_none());
}

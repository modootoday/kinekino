use crate::{
    application::{
        ExecutionResources, ExecutionSession, JobExecutor, JobIdentitySource, JobService, JobStore,
        JobWake, JobsApi, OwnedRunOutcome, PendingOwnedAttempt, SourceBlobStore, request_cancel,
        run_owned, run_queued,
    },
    domain::JobState,
};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use serde::{Deserialize, Serialize};
use std::sync::{
    Arc, Condvar, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::Duration;
mod shutdown;
#[cfg(test)]
mod tests;
pub use shutdown::{
    PendingWorkerHost, ShutdownErrors, WorkerShutdownComplete, WorkerShutdownOutcome,
};
struct Wake {
    signal: Condvar,
    mutex: Mutex<()>,
}
impl JobWake for Wake {
    fn wake(&self) {
        self.signal.notify_all();
    }
}
struct Live(Arc<AtomicBool>);
impl Drop for Live {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShutdownReceipt {
    pub completed_total: usize,
    pub cancelled_total: usize,
    pub interrupted_total: usize,
}
pub struct WorkerHost {
    service: Arc<JobService>,
    session: Option<ExecutionSession>,
    worker: Option<JoinHandle<WorkerExit>>,
    retained: Option<Box<PendingOwnedAttempt>>,
    retained_attempt: Arc<AtomicBool>,
    allow_owned: bool,
    store: Arc<dyn JobStore>,
    running: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    wake: Arc<Wake>,
    admission: Arc<Mutex<()>>,
    closed: bool,
}
enum WorkerExit {
    Finished(Result<(), ApiError>),
    Retained(Box<PendingOwnedAttempt>),
}
/// An owned-mode host exposes retaining shutdown only, never the consuming legacy Result API.
#[must_use]
pub struct RetainingWorkerHost {
    inner: WorkerHost,
}
/// Typed dependency ownership shared by legacy and retaining assemblies.
/// Retaining implementations deliberately provide no synchronous Result shutdown.
pub trait JobHostOwner: Sized + 'static {
    fn client(&self) -> Arc<dyn JobsApi>;
    fn begin_shutdown(self, deadline: std::time::Instant) -> PendingWorkerHost;
}
impl JobHostOwner for WorkerHost {
    fn client(&self) -> Arc<dyn JobsApi> {
        WorkerHost::client(self)
    }
    fn begin_shutdown(self, deadline: std::time::Instant) -> PendingWorkerHost {
        WorkerHost::begin_shutdown(self, deadline)
    }
}
impl JobHostOwner for RetainingWorkerHost {
    fn client(&self) -> Arc<dyn JobsApi> {
        RetainingWorkerHost::client(self)
    }
    fn begin_shutdown(self, deadline: std::time::Instant) -> PendingWorkerHost {
        RetainingWorkerHost::begin_shutdown(self, deadline)
    }
}
impl RetainingWorkerHost {
    pub fn client(&self) -> Arc<dyn JobsApi> {
        self.inner.client()
    }
    pub fn begin_shutdown(self, deadline: std::time::Instant) -> PendingWorkerHost {
        self.inner.begin_shutdown(deadline)
    }
}
impl WorkerHost {
    pub fn start(
        store: Arc<dyn JobStore>,
        executor: Arc<dyn JobExecutor>,
        blobs: Arc<dyn SourceBlobStore>,
        identities: Arc<dyn JobIdentitySource>,
    ) -> Result<Self, ApiError> {
        Self::start_inner(store, executor, blobs, identities, false)
    }
    pub fn start_retaining(
        store: Arc<dyn JobStore>,
        executor: Arc<dyn JobExecutor>,
        blobs: Arc<dyn SourceBlobStore>,
        identities: Arc<dyn JobIdentitySource>,
    ) -> Result<RetainingWorkerHost, ApiError> {
        Self::start_inner(store, executor, blobs, identities, true)
            .map(|inner| RetainingWorkerHost { inner })
    }
    fn start_inner(
        store: Arc<dyn JobStore>,
        executor: Arc<dyn JobExecutor>,
        blobs: Arc<dyn SourceBlobStore>,
        identities: Arc<dyn JobIdentitySource>,
        allow_owned: bool,
    ) -> Result<Self, ApiError> {
        let (session, _) = ExecutionSession::acquire(store.clone())?;
        let cursor = store.read_events(0, 1)?.latest_sequence.0;
        let running = Arc::new(AtomicBool::new(true));
        let stop = Arc::new(AtomicBool::new(false));
        let wake = Arc::new(Wake {
            signal: Condvar::new(),
            mutex: Mutex::new(()),
        });
        let admission = Arc::new(Mutex::new(()));
        let retained_attempt = Arc::new(AtomicBool::new(false));
        let service = Arc::new(JobService::host(
            store.clone(),
            ExecutionResources {
                executor: executor.clone(),
                blobs: blobs.clone(),
                identities,
                wake: wake.clone(),
                running: running.clone(),
                admission: admission.clone(),
                retained_attempt: retained_attempt.clone(),
            },
        ));
        let target = store.clone();
        let worker_running = running.clone();
        let worker_stop = stop.clone();
        let worker_wake = wake.clone();
        let worker_retained = retained_attempt.clone();
        let worker_admission = admission.clone();
        let worker = thread::Builder::new()
            .name("media-job-worker".into())
            .spawn(move || {
                let _live = Live(worker_running);
                let result = (|| -> Result<Option<Box<PendingOwnedAttempt>>, ApiError> {
                let mut cursor = cursor;
                while !worker_stop.load(Ordering::Acquire) {
                    let page = target.read_events(cursor, 100)?;
                    cursor = page.next_sequence.0;
                    for event in page.events {
                        if worker_stop.load(Ordering::Acquire) {
                            break;
                        }
                        if event.job.state != JobState::Queued {
                            continue;
                        }
                        let current = target.get(&event.job.job_id)?;
                        if current.attempt != event.job.attempt
                            || !matches!(current.state, JobState::Queued | JobState::Cancelling)
                        {
                            continue;
                        }
                        if let Some(owned) = executor.owned_executor(&current.input.operation) {
                            if !allow_owned {
                                return Err(ApiError::new(ErrorCode::Unsupported, "Owned executor requires retaining host construction"));
                            }
                            let mut outcome = run_owned(target.clone(), owned, blobs.clone(), &current.job_id, current.attempt.0)?;
                            loop {
                                match outcome {
                                        OwnedRunOutcome::Finished(job) => {
                                            if !job.state.terminal() {
                                                worker_retained.store(true, Ordering::Release);
                                                return Err(ApiError::new(ErrorCode::Internal, "Original owned driver returned a nonterminal record"));
                                            }
                                        let _gate = worker_admission.lock().map_err(|_| ApiError::new(ErrorCode::EngineFailed, "Owned retry gate unavailable after settlement"))?;
                                        worker_retained.store(false, Ordering::Release);
                                        break;
                                    }
                                    OwnedRunOutcome::Pending(mut owner) => {
                                        if !worker_retained.load(Ordering::Acquire) {
                                            match worker_admission.lock() {
                                                Ok(_gate) => worker_retained.store(true, Ordering::Release),
                                                Err(_) => {
                                                    worker_retained.store(true, Ordering::Release);
                                                    owner.record(ApiError::new(ErrorCode::EngineFailed, "Owned retry gate poisoned while retaining original owner"));
                                                }
                                            }
                                        }
                                        if worker_stop.load(Ordering::Acquire) {return Ok(Some(owner));}
                                        outcome = owner.poll();
                                        if matches!(&outcome, OwnedRunOutcome::Pending(_)) {thread::sleep(Duration::from_millis(10));}
                                    }
                                }
                            }
                        } else {
                            run_queued(
                            target.as_ref(),
                            executor.as_ref(),
                            blobs.as_ref(),
                            &current.job_id,
                            current.attempt.0,
                        )?;
                        }
                    }
                    if cursor < page.latest_sequence.0 {
                        continue;
                    }
                    let lock = worker_wake.mutex.lock().map_err(|_| {
                        ApiError::new(ErrorCode::EngineFailed, "Job worker wake gate unavailable")
                    })?;
                    let _wait = worker_wake
                        .signal
                        .wait_timeout(lock, Duration::from_millis(100))
                        .map_err(|_| {
                            ApiError::new(ErrorCode::EngineFailed, "Job worker wait failed")
                        })?;
                }
                Ok(None)
                })();
                match result {
                    Ok(Some(owner)) => WorkerExit::Retained(owner),
                    Ok(None) => WorkerExit::Finished(Ok(())),
                    Err(error) => WorkerExit::Finished(Err(error)),
                }
            })
            .map_err(|error| {
                ApiError::new(
                    ErrorCode::EngineFailed,
                    format!("Cannot start owned job worker: {error}"),
                )
            })?;
        Ok(Self {
            service,
            session: Some(session),
            worker: Some(worker),
            retained: None,
            retained_attempt,
            allow_owned,
            store,
            running,
            stop,
            wake,
            admission,
            closed: false,
        })
    }
    pub fn client(&self) -> Arc<dyn JobsApi> {
        self.service.clone()
    }
    pub fn shutdown(mut self) -> Result<ShutdownReceipt, ApiError> {
        self.stop_owned()
    }
    fn stop_owned(&mut self) -> Result<ShutdownReceipt, ApiError> {
        assert!(
            !self.allow_owned,
            "Retaining hosts require begin/poll shutdown"
        );
        if self.closed {
            return Ok(ShutdownReceipt::default());
        }
        let mut errors = self.request_stop();
        if let (Some(error), _) = self.join_worker() {
            errors.record(error);
        }
        let finalized = self.finalize_owned();
        if let Some(error) = errors.into_parts().0 {
            return Err(error);
        }
        finalized
    }
    fn request_stop(&mut self) -> ShutdownErrors {
        let mut errors = ShutdownErrors::default();
        self.request_stop_into(&mut errors);
        errors
    }
    fn request_stop_into(&mut self, errors: &mut ShutdownErrors) {
        self.closed = true;
        {
            match self.admission.lock() {
                Ok(_gate) => self.running.store(false, Ordering::Release),
                Err(_) => {
                    self.running.store(false, Ordering::Release);
                    errors.record(ApiError::new(
                        ErrorCode::EngineFailed,
                        "Job shutdown gate unavailable",
                    ));
                }
            }
        }
        if let Err(error) = visit(self.store.as_ref(), |job| {
            if !job.state.terminal() {
                request_cancel(self.store.as_ref(), &job.job_id, job.attempt.0)?;
            }
            Ok(())
        }) {
            errors.record(error);
        }
        self.stop.store(true, Ordering::Release);
        self.wake.wake();
    }
    /// Legacy synchronous shutdown only. The pending driver checks is_finished first.
    fn join_worker(&mut self) -> (Option<ApiError>, bool) {
        if let Some(worker) = self.worker.take() {
            match worker.join() {
                Ok(WorkerExit::Finished(Ok(()))) => {}
                Ok(WorkerExit::Finished(Err(error))) => {
                    let uncertain = self.retained_attempt.load(Ordering::Acquire);
                    return (Some(error), uncertain);
                }
                Ok(WorkerExit::Retained(owner)) => {
                    self.retained = Some(owner);
                }
                Err(_) => {
                    return (
                        Some(ApiError::new(
                            ErrorCode::EngineFailed,
                            "Owned job worker panicked",
                        )),
                        true,
                    );
                }
            }
        }
        (None, false)
    }
    fn finalize_owned(&mut self) -> Result<ShutdownReceipt, ApiError> {
        let result = self.finalize_records();
        drop(self.session.take());
        result
    }
    fn finalize_records(&mut self) -> Result<ShutdownReceipt, ApiError> {
        let mut receipt = ShutdownReceipt::default();
        let finalized = visit(self.store.as_ref(), |mut job| {
            if !job.state.terminal() {
                let previous = job.revision.0;
                if !job.publication_started {
                    let (current, _) =
                        request_cancel(self.store.as_ref(), &job.job_id, job.attempt.0)?;
                    job = current;
                    let previous = job.revision.0;
                    job.acknowledge_cancel()
                        .map_err(|message| ApiError::new(ErrorCode::NeedsInput, message))?;
                    self.store.publish(Some(previous), &job)?;
                } else {
                    job.interrupt()
                        .map_err(|message| ApiError::new(ErrorCode::NeedsInput, message))?;
                    self.store.publish(Some(previous), &job)?;
                }
            }
            match job.state {
                JobState::Completed => receipt.completed_total += 1,
                JobState::Cancelled => receipt.cancelled_total += 1,
                JobState::Interrupted => receipt.interrupted_total += 1,
                _ => {}
            }
            Ok(())
        });
        finalized?;
        Ok(receipt)
    }
}
impl Drop for WorkerHost {
    fn drop(&mut self) {
        if !self.closed && !self.allow_owned {
            let _ = self.stop_owned();
        } else if !self.closed {
            // Abandoning an owned host is not cleanup. Explicit callers must keep
            // its pending carrier; do not manufacture ACKs or block in Drop.
            self.running.store(false, Ordering::Release);
            self.stop.store(true, Ordering::Release);
            self.wake.wake();
        }
    }
}
fn visit(
    store: &dyn JobStore,
    mut action: impl FnMut(crate::domain::JobRecord) -> Result<(), ApiError>,
) -> Result<(), ApiError> {
    let mut cursor = None;
    loop {
        let page = store.list(cursor.as_ref(), 100)?;
        for snapshot in page.jobs {
            action(store.get(&snapshot.job_id)?)?;
        }
        match page.next_cursor {
            Some(next) => cursor = Some(next),
            None => return Ok(()),
        }
    }
}

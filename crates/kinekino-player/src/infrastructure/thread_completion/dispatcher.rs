use super::completion::{Job, JoinRequest, JoinTicket, notify};
use std::{
    num::NonZeroUsize,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
        mpsc::{SyncSender, TrySendError, sync_channel},
    },
    thread::JoinHandle,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubmitRejection {
    Full,
    Closed,
    Faulted,
}
pub struct SubmitFailure<T> {
    pub reason: SubmitRejection,
    pub request: JoinRequest<T>,
}
impl<T> std::fmt::Debug for SubmitFailure<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SubmitFailure")
            .field("reason", &self.reason)
            .finish_non_exhaustive()
    }
}
struct Shared {
    sender: Mutex<Option<SyncSender<Job>>>,
    outstanding: AtomicUsize,
    faults: AtomicUsize,
}
#[derive(Clone)]
pub struct DispatcherHandle(Arc<Shared>);
#[derive(Debug, Clone, Copy)]
pub struct DispatcherStatus {
    pub admission_open: bool,
    pub outstanding: usize,
    pub faults: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatcherShutdown {
    AdmissionOpen,
    Pending,
    Joined {
        worker_panicked: bool,
        faults: usize,
    },
}
/// One dispatcher thread per explicit host, never one supervisor per child.
#[must_use = "retain the dispatcher host until its own shutdown is observed or joined"]
pub struct DispatcherHost {
    handle: DispatcherHandle,
    worker: Option<JoinHandle<()>>,
    joined: Option<bool>,
}

impl DispatcherHandle {
    pub fn submit<T>(&self, request: JoinRequest<T>) -> Result<JoinTicket<T>, SubmitFailure<T>> {
        let sender = match self.0.sender.lock() {
            Ok(sender) => sender,
            Err(_) => {
                return Err(SubmitFailure {
                    reason: SubmitRejection::Faulted,
                    request,
                });
            }
        };
        let Some(sender) = sender.as_ref() else {
            return Err(SubmitFailure {
                reason: SubmitRejection::Closed,
                request,
            });
        };
        let JoinRequest { job, ticket } = request;
        self.0.outstanding.fetch_add(1, Ordering::AcqRel);
        match sender.try_send(job) {
            Ok(()) => Ok(ticket),
            Err(error) => {
                self.0.outstanding.fetch_sub(1, Ordering::AcqRel);
                let (reason, job) = match error {
                    TrySendError::Full(job) => (SubmitRejection::Full, job),
                    TrySendError::Disconnected(job) => (SubmitRejection::Closed, job),
                };
                Err(SubmitFailure {
                    reason,
                    request: JoinRequest { job, ticket },
                })
            }
        }
    }
    pub fn status(&self) -> DispatcherStatus {
        DispatcherStatus {
            admission_open: self.0.sender.lock().is_ok_and(|sender| sender.is_some()),
            outstanding: self.0.outstanding.load(Ordering::Acquire),
            faults: self.0.faults.load(Ordering::Acquire),
        }
    }
    fn close(&self) {
        self.0
            .sender
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take();
    }
}
impl DispatcherHost {
    pub fn start(capacity: NonZeroUsize) -> std::io::Result<Self> {
        let (sender, receiver) = sync_channel::<Job>(capacity.get());
        let handle = DispatcherHandle(Arc::new(Shared {
            sender: Mutex::new(Some(sender)),
            outstanding: AtomicUsize::new(0),
            faults: AtomicUsize::new(0),
        }));
        let state = handle.clone();
        let worker = std::thread::Builder::new()
            .name("join-completion-dispatcher".into())
            .spawn(move || {
                while let Ok(job) = receiver.recv() {
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(job.run));
                    match result {
                        Ok(true) => {}
                        Ok(false) => {
                            state.0.faults.fetch_add(1, Ordering::AcqRel);
                        }
                        Err(_) => {
                            job.fault.store(true, Ordering::Release);
                            state.0.faults.fetch_add(1, Ordering::AcqRel);
                            state.close();
                            notify(&job.wake, &job.notification);
                        }
                    }
                    state.0.outstanding.fetch_sub(1, Ordering::AcqRel);
                }
            })?;
        Ok(Self {
            handle,
            worker: Some(worker),
            joined: None,
        })
    }
    pub fn handle(&self) -> DispatcherHandle {
        self.handle.clone()
    }
    /// Close admission and drain accepted work. This does not join or claim host exit.
    pub fn request_shutdown(&self) {
        self.handle.close();
    }
    pub fn poll_shutdown(&mut self) -> DispatcherShutdown {
        if self.handle.status().admission_open {
            return DispatcherShutdown::AdmissionOpen;
        }
        if let Some(worker) = &self.worker {
            if !worker.is_finished() {
                return DispatcherShutdown::Pending;
            }
            self.joined = Some(
                self.worker
                    .take()
                    .expect("Finished dispatcher remains owned")
                    .join()
                    .is_err(),
            );
        }
        DispatcherShutdown::Joined {
            worker_panicked: self.joined.unwrap_or(false),
            faults: self.handle.status().faults,
        }
    }
}
impl Drop for DispatcherHost {
    fn drop(&mut self) {
        // No UI-thread join. Dropping this owner is never a Joined receipt.
        self.request_shutdown();
    }
}

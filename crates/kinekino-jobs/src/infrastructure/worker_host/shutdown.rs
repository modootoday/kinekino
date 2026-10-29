use super::{ShutdownReceipt, WorkerHost};
use crate::application::owned_attempt::AttemptFailure;
use crate::application::{JobWake, OwnedRunOutcome};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use std::{
    any::Any,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::atomic::Ordering,
    time::Instant,
};

#[cfg(test)]
mod tests;

/// Original first error and bounded secondary observations; never cleanup authority.
#[derive(Default)]
pub struct ShutdownErrors {
    first: Option<ApiError>,
    secondary: Vec<AttemptFailure>,
    overflow: bool,
}
impl ShutdownErrors {
    pub fn first(&self) -> Option<&ApiError> {
        self.first.as_ref()
    }
    pub fn secondary(&self) -> &[AttemptFailure] {
        &self.secondary
    }
    pub fn overflow(&self) -> bool {
        self.overflow
    }
    pub fn record(&mut self, error: ApiError) {
        if self.first.is_none() {
            self.first = Some(error);
        } else if self.secondary.len() < 8 {
            self.secondary.push(AttemptFailure::capture(&error));
        } else {
            self.overflow = true;
        }
    }
    pub fn into_parts(self) -> (Option<ApiError>, Vec<AttemptFailure>, bool) {
        (self.first, self.secondary, self.overflow)
    }
    pub fn merge(&mut self, other: Self) {
        let (first, secondary, overflow) = other.into_parts();
        if let Some(error) = first {
            self.record(error);
        }
        for failure in secondary {
            if self.secondary.len() < 8 {
                self.secondary.push(failure);
            } else {
                self.overflow = true;
            }
        }
        self.overflow |= overflow;
    }
}

#[must_use = "A pending original worker still owns its execution session"]
pub enum WorkerShutdownOutcome {
    Complete(WorkerShutdownComplete),
    Pending(PendingWorkerHost),
}

pub struct WorkerShutdownComplete {
    pub receipt: Option<ShutdownReceipt>,
    pub errors: ShutdownErrors,
}

/// Holds the original host, worker handle and exclusive session without spawning
/// another worker. Dropping this value abandons ownership; callers must retain it
/// through an incomplete process boundary, never convert it to an ApiError.
#[must_use]
pub struct PendingWorkerHost {
    host: WorkerHost,
    deadline: Instant,
    errors: ShutdownErrors,
    cutoff_observed: bool,
    join_uncertain: bool,
    panic_payload: Option<Box<dyn Any + Send>>,
}

impl WorkerHost {
    pub fn begin_shutdown(mut self, original_deadline: Instant) -> PendingWorkerHost {
        let mut errors = ShutdownErrors::default();
        let preparation = catch_unwind(AssertUnwindSafe(|| self.request_stop_into(&mut errors)));
        // Even if cancellation-store traversal unwinds, the original host remains
        // here and its worker must be signalled before returning the carrier.
        if preparation.is_err() {
            self.closed = true;
            self.running.store(false, Ordering::Release);
            self.stop.store(true, Ordering::Release);
            self.wake.wake();
        }
        let mut pending = PendingWorkerHost {
            host: self,
            deadline: original_deadline,
            errors,
            cutoff_observed: false,
            join_uncertain: false,
            panic_payload: None,
        };
        if let Err(payload) = preparation {
            pending.record_adapter_panic(payload, "Original job shutdown preparation panicked");
        }
        pending
    }
}

impl PendingWorkerHost {
    fn record_adapter_panic(&mut self, payload: Box<dyn Any + Send>, message: &'static str) {
        if self.panic_payload.is_none() {
            self.panic_payload = Some(payload);
        }
        self.errors
            .record(ApiError::new(ErrorCode::Internal, message));
    }
    pub fn deadline(&self) -> Instant {
        self.deadline
    }
    pub fn errors(&self) -> &ShutdownErrors {
        &self.errors
    }
    pub fn original_attempt_error(&self) -> Option<&ApiError> {
        self.host
            .retained
            .as_ref()
            .and_then(|owner| owner.first_error())
    }
    pub fn join_uncertain(&self) -> bool {
        self.join_uncertain
    }
    pub fn worker_finished(&self) -> bool {
        self.host
            .worker
            .as_ref()
            .is_none_or(std::thread::JoinHandle::is_finished)
    }

    /// At most one observation. No sleep, unfinished join or replacement handle.
    pub fn poll(mut self, constrained_deadline: Instant) -> WorkerShutdownOutcome {
        self.deadline = self.deadline.min(constrained_deadline);
        if Instant::now() >= self.deadline && !self.cutoff_observed {
            self.cutoff_observed = true;
            self.errors.record(ApiError::new(
                ErrorCode::Timeout,
                "Original job shutdown observation deadline expired",
            ));
        }
        if self.join_uncertain || !self.worker_finished() {
            return WorkerShutdownOutcome::Pending(self);
        }
        let (error, uncertain) = self.host.join_worker();
        if let Some(error) = error {
            self.errors.record(error);
        }
        if uncertain {
            self.join_uncertain = true;
            return WorkerShutdownOutcome::Pending(self);
        }
        if let Some(owner) = self.host.retained.take() {
            match owner.poll() {
                OwnedRunOutcome::Pending(owner) => {
                    self.host.retained = Some(owner);
                    return WorkerShutdownOutcome::Pending(self);
                }
                OwnedRunOutcome::Finished(job) => {
                    if !job.state.terminal() {
                        self.join_uncertain = true;
                        self.errors.record(ApiError::new(
                            ErrorCode::Internal,
                            "Original retained driver returned a nonterminal record",
                        ));
                        return WorkerShutdownOutcome::Pending(self);
                    }
                    self.host
                        .retained_attempt
                        .store(false, std::sync::atomic::Ordering::Release);
                }
            }
        }
        // Borrow the host under the boundary: session/lease/handle ownership
        // stays in this carrier if a durable store adapter unwinds.
        let finalized = catch_unwind(AssertUnwindSafe(|| self.host.finalize_records()));
        let receipt = match finalized {
            Ok(Ok(receipt)) => Some(receipt),
            Ok(Err(error)) => {
                self.errors.record(error);
                return WorkerShutdownOutcome::Pending(self);
            }
            Err(payload) => {
                self.record_adapter_panic(payload, "Original job shutdown finalization panicked");
                return WorkerShutdownOutcome::Pending(self);
            }
        };
        drop(self.host.session.take());
        WorkerShutdownOutcome::Complete(WorkerShutdownComplete {
            receipt,
            errors: self.errors,
        })
    }
}

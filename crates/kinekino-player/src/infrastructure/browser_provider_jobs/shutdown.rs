use super::{ProviderJobsHost, ProviderShutdownReceipt};
use crate::application::{
    api::{ApiError, ErrorCode},
    browser_provider::BrowserHostState,
};
use kinekino_jobs::infrastructure::{
    JobHostOwner, PendingWorkerHost, ShutdownErrors, ShutdownReceipt, WorkerHost,
    WorkerShutdownOutcome,
};
use std::time::Instant;
#[cfg(test)]
mod tests;

#[must_use]
pub enum ProviderShutdownOutcome<W: JobHostOwner = WorkerHost> {
    Complete(ProviderShutdownComplete),
    Pending(Box<PendingProviderHost<W>>),
}
pub struct ProviderShutdownComplete {
    pub receipt: Option<ProviderShutdownReceipt>,
    pub errors: ShutdownErrors,
}
#[must_use = "Retain original jobs, browser and profile guards until closed"]
pub struct PendingProviderHost<W: JobHostOwner = WorkerHost> {
    host: ProviderJobsHost<W>,
    jobs: Option<PendingWorkerHost>,
    jobs_receipt: Option<ShutdownReceipt>,
    errors: ShutdownErrors,
    deadline: Instant,
    cutoff_observed: bool,
    browser_requested: bool,
}
impl<W: JobHostOwner> ProviderJobsHost<W> {
    pub fn begin_shutdown(mut self, original_deadline: Instant) -> Box<PendingProviderHost<W>> {
        let jobs = self
            .jobs
            .take()
            .map(|jobs| jobs.begin_shutdown(original_deadline));
        let mut errors = ShutdownErrors::default();
        let jobs_receipt = if jobs.is_none() && self.jobs_never_started {
            // Original constructor never produced a worker. Zero job counts do
            // not invent a process exit or a cooperative cancellation receipt.
            Some(ShutdownReceipt::default())
        } else {
            None
        };
        if jobs.is_none() && !self.jobs_never_started {
            errors.record(ApiError::new(
                ErrorCode::Internal,
                "Original provider job owner is missing",
            ));
        }
        Box::new(PendingProviderHost {
            host: self,
            jobs,
            jobs_receipt,
            errors,
            deadline: original_deadline,
            cutoff_observed: false,
            browser_requested: false,
        })
    }
}
impl<W: JobHostOwner> PendingProviderHost<W> {
    pub fn original_attempt_error(&self) -> Option<&ApiError> {
        self.jobs
            .as_ref()
            .and_then(|jobs| jobs.original_attempt_error())
    }
    pub fn errors(&self) -> &ShutdownErrors {
        if self.errors.first().is_none()
            && let Some(jobs) = &self.jobs
        {
            return jobs.errors();
        }
        &self.errors
    }
    pub fn deadline(&self) -> Instant {
        self.deadline
    }
    pub fn poll(mut self: Box<Self>, constrained_deadline: Instant) -> ProviderShutdownOutcome<W> {
        self.deadline = self.deadline.min(constrained_deadline);
        if let Some(jobs) = self.jobs.take() {
            match jobs.poll(self.deadline) {
                WorkerShutdownOutcome::Pending(jobs) => {
                    self.jobs = Some(jobs);
                    return ProviderShutdownOutcome::Pending(self);
                }
                WorkerShutdownOutcome::Complete(result) => {
                    self.errors.merge(result.errors);
                    self.jobs_receipt = result.receipt;
                }
            }
        }
        if Instant::now() >= self.deadline && !self.cutoff_observed {
            self.cutoff_observed = true;
            self.errors.record(ApiError::new(
                ErrorCode::Timeout,
                "Original provider shutdown deadline expired",
            ));
        }
        if self.jobs_receipt.is_none() {
            return ProviderShutdownOutcome::Pending(self);
        }
        let owner = self.host.browser.owner();
        if !self.browser_requested {
            match owner.request_shutdown() {
                Ok(()) => self.browser_requested = true,
                Err(error) => {
                    self.errors.record(error);
                    return ProviderShutdownOutcome::Pending(self);
                }
            }
        }
        match owner.shutdown_snapshot() {
            Ok(snapshot) if snapshot.worker_exited => {
                if snapshot.state == BrowserHostState::Failed {
                    let error = match snapshot.failure {
                        Some(original) => ApiError {
                            code: original.code,
                            message: original.message.clone(),
                            retryable: original.retryable,
                            details: original.details.clone(),
                            remediation: original.remediation.clone(),
                            request_id: original.request_id.clone(),
                        },
                        None => ApiError::new(
                            ErrorCode::EngineFailed,
                            "Original provider worker exited with failure",
                        ),
                    };
                    self.errors.record(error);
                }
                // This is the existing browser worker-exited observation, not an OS join.
                ProviderShutdownOutcome::Complete(ProviderShutdownComplete {
                    receipt: self
                        .jobs_receipt
                        .take()
                        .map(|jobs| ProviderShutdownReceipt {
                            jobs,
                            browser_worker_exited: true,
                        }),
                    errors: self.errors,
                })
            }
            Ok(_) => ProviderShutdownOutcome::Pending(self),
            Err(error) => {
                self.errors.record(error);
                ProviderShutdownOutcome::Pending(self)
            }
        }
    }
}

//! Original provider shutdown carrier; a deadline never permits dropping a pending owner.
use crate::{
    assembly::meari::native_cleanup_driver::{
        CleanupAdapter, CleanupDriverOutcome, CleanupPoll, drive_cleanup,
    },
    infrastructure::browser_provider_jobs::{
        PendingProviderHost, ProviderShutdownComplete, ProviderShutdownOutcome,
        RetainingProviderJobsHost,
    },
};
use kinekino_jobs::infrastructure::RetainingWorkerHost;
use meari_native::infrastructure::native_host::HostRetry;
use std::{
    convert::Infallible,
    time::{Duration, Instant},
};
type Pending = PendingProviderHost<RetainingWorkerHost>;
pub(super) type Outcome = ProviderShutdownOutcome<RetainingWorkerHost>;
pub(super) struct ProviderCleanup {
    pending: Option<Box<Pending>>,
    completed: Option<ProviderShutdownComplete>,
    deadline: Instant,
}
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Status {
    Complete,
    Faulted,
    BudgetExhausted,
}
pub(super) fn original_deadline() -> Instant {
    // Same dependency observation budget as the original CLI provider closeout. It does not
    // change a browser job attempt deadline or the native-window fixture's own budgets.
    Instant::now() + Duration::from_secs(30)
}
pub(super) fn close(owner: RetainingProviderJobsHost) -> CleanupDriverOutcome<ProviderCleanup> {
    observe(owner.begin_shutdown(original_deadline()))
}
pub(super) fn observe(pending: Box<Pending>) -> CleanupDriverOutcome<ProviderCleanup> {
    let deadline = pending.deadline();
    drive_cleanup(
        ProviderCleanup {
            pending: Some(pending),
            completed: None,
            deadline,
        },
        Duration::from_millis(10),
        std::thread::sleep,
    )
}
impl CleanupAdapter for ProviderCleanup {
    type Owners = Outcome;
    type Evidence = serde_json::Value;
    type Status = Status;
    type Error = Infallible;
    fn poll(&mut self) -> CleanupPoll<Status> {
        let pending = self
            .pending
            .take()
            .expect("original provider pending owner retained");
        match pending.poll(self.deadline) {
            ProviderShutdownOutcome::Complete(completed) => {
                let status = if completed.errors.first().is_none() && completed.receipt.is_some() {
                    Status::Complete
                } else {
                    Status::Faulted
                };
                self.completed = Some(completed);
                CleanupPoll::Terminal(status)
            }
            ProviderShutdownOutcome::Pending(pending) => {
                self.pending = Some(pending);
                if Instant::now() >= self.deadline {
                    CleanupPoll::Terminal(Status::BudgetExhausted)
                } else {
                    CleanupPoll::Pending(HostRetry::After(Duration::from_millis(10)))
                }
            }
        }
    }
    fn accept_terminal_for_shutdown(&mut self) -> Result<(), Infallible> {
        Ok(())
    }
    fn acceptance_failure_status(&self) -> Status {
        Status::Faulted
    }
    fn into_parts(self) -> (Outcome, serde_json::Value) {
        if let Some(completed) = self.completed {
            let evidence = serde_json::json!({"receipt":completed.receipt,"first_error":completed.errors.first(),"secondary_overflow":completed.errors.overflow()});
            (ProviderShutdownOutcome::Complete(completed), evidence)
        } else {
            let pending = self.pending.expect("original provider retained at cutoff");
            let evidence = serde_json::json!({"pending":true,"first_error":pending.errors().first(),"original_attempt_error":pending.original_attempt_error(),"secondary_overflow":pending.errors().overflow()});
            (ProviderShutdownOutcome::Pending(pending), evidence)
        }
    }
}

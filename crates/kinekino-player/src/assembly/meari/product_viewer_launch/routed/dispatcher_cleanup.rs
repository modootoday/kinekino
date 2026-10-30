//! Observe the one dispatcher after original child/execution cleanup; no replacement worker.
use crate::{
    assembly::meari::native_cleanup_driver::{CleanupAdapter, CleanupPoll},
    infrastructure::thread_completion::{DispatcherHost, DispatcherShutdown},
};
use meari_native::infrastructure::native_host::HostRetry;
use std::{
    convert::Infallible,
    time::{Duration, Instant},
};

pub(super) struct DispatcherCleanup {
    owner: DispatcherHost,
    started: Instant,
    observed: DispatcherShutdown,
}
#[derive(Debug, PartialEq, Eq)]
pub(super) enum DispatcherCleanupStatus {
    Joined,
    Faulted,
    BudgetExhausted,
}
impl DispatcherCleanup {
    pub(super) fn new(owner: DispatcherHost) -> Self {
        owner.request_shutdown();
        Self {
            owner,
            started: Instant::now(),
            observed: DispatcherShutdown::Pending,
        }
    }
}
impl CleanupAdapter for DispatcherCleanup {
    type Owners = DispatcherHost;
    type Evidence = DispatcherShutdown;
    type Status = DispatcherCleanupStatus;
    type Error = Infallible;
    fn poll(&mut self) -> CleanupPoll<Self::Status> {
        self.observed = self.owner.poll_shutdown();
        match self.observed {
            DispatcherShutdown::Joined {
                worker_panicked: false,
                faults: 0,
            } => CleanupPoll::Terminal(DispatcherCleanupStatus::Joined),
            DispatcherShutdown::Joined { .. } | DispatcherShutdown::AdmissionOpen => {
                CleanupPoll::Terminal(DispatcherCleanupStatus::Faulted)
            }
            DispatcherShutdown::Pending if self.started.elapsed() >= Duration::from_secs(2) => {
                CleanupPoll::Terminal(DispatcherCleanupStatus::BudgetExhausted)
            }
            DispatcherShutdown::Pending => {
                CleanupPoll::Pending(HostRetry::After(Duration::from_millis(2)))
            }
        }
    }
    fn accept_terminal_for_shutdown(&mut self) -> Result<(), Infallible> {
        Ok(())
    }
    fn acceptance_failure_status(&self) -> Self::Status {
        DispatcherCleanupStatus::Faulted
    }
    fn into_parts(self) -> (Self::Owners, Self::Evidence) {
        (self.owner, self.observed)
    }
}

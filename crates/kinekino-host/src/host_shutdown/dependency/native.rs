//! Retain the original standalone native dependency on cleanup error.
//! The existing shutdown call is synchronous; polling is not hard preemption.
use super::{DependencyPoll, PendingDependency, completion};
#[cfg(test)]
mod tests;
use kinekino_jobs::infrastructure::ShutdownErrors;
use kinekino_player::{
    application::{
        api::{ApiError, ErrorCode},
        skip_segments::host::SkipHostOwner,
    },
    infrastructure::mpv::NativePlaybackHost,
};
use std::{
    any::Any,
    panic::{AssertUnwindSafe, catch_unwind},
    time::Instant,
};

struct NativeDependency {
    host: NativePlaybackHost,
    _skip_owner: SkipHostOwner,
    errors: ShutdownErrors,
    deadline: Instant,
    cutoff_observed: bool,
    panic_payload: Option<Box<dyn Any + Send>>,
}
impl NativeDependency {
    fn observe_cutoff(&mut self) -> bool {
        let expired = Instant::now() >= self.deadline;
        if expired && !self.cutoff_observed {
            self.cutoff_observed = true;
            self.errors.record(ApiError::new(
                ErrorCode::Timeout,
                "Original native cleanup observation deadline expired",
            ));
        }
        expired
    }
}
impl PendingDependency for NativeDependency {
    fn errors(&self) -> &ShutdownErrors {
        &self.errors
    }
    fn poll(mut self: Box<Self>, constrained_deadline: Instant) -> DependencyPoll {
        self.deadline = self.deadline.min(constrained_deadline);
        if self.observe_cutoff() || self.panic_payload.is_some() {
            return DependencyPoll::Pending(self);
        }
        // Borrow the original host across unwind; never move it into a worker.
        // This call can block in the existing child/IO joins.
        let result = catch_unwind(AssertUnwindSafe(|| self.host.shutdown()));
        match result {
            Ok(Ok(receipt)) => {
                self.observe_cutoff();
                DependencyPoll::Complete(completion(receipt, std::mem::take(&mut self.errors)))
            }
            Ok(Err(error)) => {
                self.errors.record(error);
                self.observe_cutoff();
                DependencyPoll::Pending(self)
            }
            Err(payload) => {
                self.panic_payload = Some(payload);
                self.errors.record(ApiError::new(
                    ErrorCode::EngineFailed,
                    "Native cleanup panicked; original host and payload remain owned",
                ));
                self.observe_cutoff();
                DependencyPoll::Pending(self)
            }
        }
    }
}
pub(crate) fn pending_native(
    host: NativePlaybackHost,
    skip_owner: SkipHostOwner,
    deadline: Instant,
) -> Box<dyn PendingDependency> {
    Box::new(NativeDependency {
        host,
        _skip_owner: skip_owner,
        errors: ShutdownErrors::default(),
        deadline,
        cutoff_observed: false,
        panic_payload: None,
    })
}

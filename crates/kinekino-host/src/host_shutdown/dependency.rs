use super::carrier::LocalHostShutdownView;
#[cfg(windows)]
mod native;
#[cfg(windows)]
mod windows_hosts;
use kinekino_jobs::infrastructure::{
    PendingWorkerHost, RetainingWorkerHost, ShutdownErrors, WorkerShutdownOutcome,
};
use kinekino_player::{
    application::api::{ApiError, ErrorCode},
    infrastructure::local_host::LocalHostShutdownStatus,
};
#[cfg(windows)]
pub(crate) use native::pending_native;
use serde_json::{Value, json};
use std::{
    io::Write,
    time::{Duration, Instant},
};
#[cfg(windows)]
pub(crate) use windows_hosts::{pending_playback, pending_provider, pending_workspace};

#[cfg(test)]
mod tests;

// A distinct dependency observation stage; it never refreshes a job attempt deadline.
const DEPENDENCY_OBSERVATION_BUDGET: Duration = Duration::from_secs(30);
const POLL_INTERVAL: Duration = Duration::from_millis(10);
pub(crate) fn dependency_deadline() -> Instant {
    Instant::now() + DEPENDENCY_OBSERVATION_BUDGET
}

pub(crate) trait PendingDependency {
    fn errors(&self) -> &ShutdownErrors;
    fn original_attempt_error(&self) -> Option<&ApiError> {
        None
    }
    fn poll(self: Box<Self>, deadline: Instant) -> DependencyPoll;
}
pub(crate) enum DependencyPoll {
    Complete(DependencyCompletion),
    Pending(Box<dyn PendingDependency>),
}
pub(crate) struct DependencyCompletion {
    receipt: Value,
    errors: ShutdownErrors,
}
#[must_use]
pub(crate) enum DependencyObservation {
    Complete(DependencyCompletion),
    Incomplete(DependencyIncomplete),
}
#[must_use = "Retain the original dependency through the terminal process boundary"]
pub(crate) struct DependencyIncomplete {
    owner: Box<dyn PendingDependency>,
    first_error: Option<ApiError>,
    local_status: Option<LocalHostShutdownStatus>,
    deadline: Instant,
}
impl DependencyIncomplete {
    pub(crate) fn exit_code(&self) -> u8 {
        self.first_error
            .as_ref()
            .or_else(|| self.owner.errors().first())
            .map_or(8, ApiError::exit_code)
    }
    pub(crate) fn diagnostic(&self, host: &str) -> Value {
        json!({"event":"host_shutdown_incomplete","host":host,
            "reason":"dependency_shutdown_incomplete",
            "first_error":self.first_error.as_ref().or_else(|| self.owner.errors().first()),
            "dependency_first_error":self.owner.errors().first(),
            "original_attempt_error":self.owner.original_attempt_error(),
            "secondary":secondary(self.owner.errors()),
            "secondary_overflow":self.owner.errors().overflow(),
            "local_host_shutdown":self.local_status.map(LocalHostShutdownView::from),
            "budget_expired":Instant::now() >= self.deadline,
            "cleanup_complete":false,"sdk_cancellation_confirmed":false})
    }
}
fn secondary(errors: &ShutdownErrors) -> Vec<Value> {
    errors
        .secondary()
        .iter()
        .map(|error| {
            json!({
                "code":error.code(),"message":error.message(),"remediation":error.remediation()
            })
        })
        .collect()
}
fn completion<T: serde::Serialize>(receipt: T, mut errors: ShutdownErrors) -> DependencyCompletion {
    let receipt = serde_json::to_value(receipt).unwrap_or_else(|error| {
        errors.record(ApiError::new(
            ErrorCode::Internal,
            format!("Closed dependency receipt serialization failed: {error}"),
        ));
        Value::Null
    });
    DependencyCompletion { receipt, errors }
}
impl PendingDependency for PendingWorkerHost {
    fn original_attempt_error(&self) -> Option<&ApiError> {
        self.original_attempt_error()
    }
    fn errors(&self) -> &ShutdownErrors {
        self.errors()
    }
    fn poll(self: Box<Self>, deadline: Instant) -> DependencyPoll {
        match (*self).poll(deadline) {
            WorkerShutdownOutcome::Pending(owner) => DependencyPoll::Pending(Box::new(owner)),
            WorkerShutdownOutcome::Complete(result) => {
                DependencyPoll::Complete(completion(result.receipt, result.errors))
            }
        }
    }
}
pub(crate) fn pending_jobs(
    owner: RetainingWorkerHost,
    deadline: Instant,
) -> Box<dyn PendingDependency> {
    Box::new(owner.begin_shutdown(deadline))
}
pub(crate) fn failed_dependency<D>(
    owner: Box<dyn PendingDependency>,
    first_error: ApiError,
    deadline: Instant,
) -> super::HostRunOutcome<D> {
    match observe_dependency(owner, Some(first_error), None, deadline) {
        DependencyObservation::Incomplete(owner) => {
            super::HostRunOutcome::DependencyIncomplete(owner)
        }
        DependencyObservation::Complete(result) => super::HostRunOutcome::Complete {
            result: finish_dependency(result).map(|_| ()),
            local_host_shutdown: None,
        },
    }
}
pub(crate) fn observe_dependency(
    owner: Box<dyn PendingDependency>,
    first_error: Option<ApiError>,
    local_status: Option<LocalHostShutdownStatus>,
    deadline: Instant,
) -> DependencyObservation {
    let mut pending = DependencyIncomplete {
        owner,
        first_error,
        local_status,
        deadline,
    };
    loop {
        match advance_dependency(pending) {
            DependencyObservation::Complete(result) => {
                return DependencyObservation::Complete(result);
            }
            DependencyObservation::Incomplete(owner) => pending = owner,
        }
        let now = Instant::now();
        if now >= pending.deadline {
            return DependencyObservation::Incomplete(pending);
        }
        std::thread::sleep(POLL_INTERVAL.min(pending.deadline.saturating_duration_since(now)));
    }
}
fn advance_dependency(pending: DependencyIncomplete) -> DependencyObservation {
    let DependencyIncomplete {
        owner,
        first_error,
        local_status,
        deadline,
    } = pending;
    match owner.poll(deadline) {
        DependencyPoll::Pending(owner) => DependencyObservation::Incomplete(DependencyIncomplete {
            owner,
            first_error,
            local_status,
            deadline,
        }),
        DependencyPoll::Complete(mut result) => {
            let mut errors = ShutdownErrors::default();
            if let Some(error) = first_error {
                errors.record(error);
            }
            errors.merge(result.errors);
            result.errors = errors;
            DependencyObservation::Complete(result)
        }
    }
}
pub(crate) fn finish_dependency(result: DependencyCompletion) -> Result<Value, ApiError> {
    if result.errors.first().is_some() {
        let diagnostic = json!({"event":"dependency_shutdown_observation",
            "first_error":result.errors.first(),"secondary":secondary(&result.errors),
            "secondary_overflow":result.errors.overflow(),"cleanup_complete":true});
        let _ = writeln!(std::io::stderr().lock(), "{diagnostic}");
    }
    match result.errors.into_parts().0 {
        Some(error) => Err(error),
        None => Ok(result.receipt),
    }
}

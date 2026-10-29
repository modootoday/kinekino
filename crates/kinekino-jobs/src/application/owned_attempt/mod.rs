//! Inert owned-executor contracts. Production admission/registry remains disabled
//! until the host, retry lease and incomplete shutdown carrier are integrated.
mod control;
mod diagnostics;
mod outcome;
mod publication;
mod session;
#[cfg(test)]
mod tests;

pub use control::{AttemptControl, AttemptScope};
pub use diagnostics::{AttemptDiagnostic, AttemptFailure};
pub use outcome::{
    CandidateReleaseFailure, OwnedAttemptExecutor, OwnedAttemptOutcome, OwnerObservation,
    RetainedAttempt, RetainedAttemptOwner, SettledAttempt, SettlementKind,
};
pub use publication::{PublicationPermit, SettlementPublicationGate};
pub use session::{
    CooperativeCancellation, OwnedAttemptSession, OwnedExecutorPort, OwnedOutputCandidate,
    SessionCandidate, SessionRun,
};
pub(in crate::application) use session::{SessionExecution, run_prepared};

use super::JobAttemptIdentityV1;
use std::{sync::Arc, time::Instant};

struct AttemptBinding {
    identity: JobAttemptIdentityV1,
    deadline: Instant,
    clock: Arc<dyn AttemptClock>,
}

/// Core-only monotonic clock; engines cannot replace it or refresh the deadline.
trait AttemptClock: Send + Sync {
    fn now(&self) -> Instant;
}

fn same_binding(left: &Arc<AttemptBinding>, right: &Arc<AttemptBinding>) -> bool {
    Arc::ptr_eq(left, right)
}

struct OriginalClock;
impl AttemptClock for OriginalClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}

/// Only the application worker can mint this from its original start-CAS record.
pub(in crate::application) struct CoreAttemptBinding(Arc<AttemptBinding>);
impl CoreAttemptBinding {
    pub(in crate::application) fn from_running(
        running: &crate::domain::JobRecord,
        control: &dyn super::ExecutionControl,
    ) -> Result<Self, kinekino_media_contracts::errors::ApiError> {
        use kinekino_media_contracts::errors::{ApiError, ErrorCode};
        running
            .validate()
            .map_err(|message| ApiError::new(ErrorCode::NeedsInput, message))?;
        let identity = JobAttemptIdentityV1::from_running_record(running);
        if running.state != crate::domain::JobState::Running
            || running.publication_started
            || running.cancel_requested
            || control.execution_identity() != Some(&identity)
        {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Owned execution requires original running CAS provenance",
            ));
        }
        Ok(Self(Arc::new(AttemptBinding {
            identity,
            deadline: control.deadline(),
            clock: Arc::new(OriginalClock),
        })))
    }
    fn check(
        &self,
        control: &dyn super::ExecutionControl,
    ) -> Result<(), kinekino_media_contracts::errors::ApiError> {
        if control.execution_identity() != Some(&self.0.identity)
            || control.deadline() != self.0.deadline
        {
            return Err(kinekino_media_contracts::errors::ApiError::new(
                kinekino_media_contracts::errors::ErrorCode::RevisionConflict,
                "Original attempt control changed",
            ));
        }
        Ok(())
    }
    pub(in crate::application) fn scope<'a>(
        &self,
        control: &'a dyn super::ExecutionControl,
    ) -> Result<AttemptScope<'a>, kinekino_media_contracts::errors::ApiError> {
        self.check(control)?;
        Ok(AttemptScope {
            binding: Arc::clone(&self.0),
            control: control::ControlSource::Original(control),
        })
    }
    pub(in crate::application) fn gate<'a>(
        &self,
        control: &'a dyn super::ExecutionControl,
    ) -> Result<SettlementPublicationGate<'a>, kinekino_media_contracts::errors::ApiError> {
        self.check(control)?;
        Ok(SettlementPublicationGate {
            binding: Arc::clone(&self.0),
            control,
        })
    }
    pub(in crate::application) fn cancellation_matches(
        &self,
        marker: &CooperativeCancellation,
    ) -> bool {
        same_binding(&self.0, &marker.binding)
    }
}

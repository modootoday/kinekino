use super::{
    AttemptBinding, AttemptDiagnostic, AttemptScope, OwnedAttemptSession, PublicationPermit,
    same_binding,
};
use crate::{
    application::{JobAttemptIdentityV1, SourceBlobStore},
    domain::PreparedJobInput,
};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use std::{
    any::Any,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::Arc,
    time::Instant,
};

/// Adapter type owns original handles. Worker JSON cannot implement this port.
/// Poll implementations must preserve ownership on every error and must not panic.
pub trait RetainedAttemptOwner: Send {
    fn poll_close(&mut self, deadline: Instant) -> Result<OwnerObservation, ApiError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnerObservation {
    /// Parent's original session never created a child, or original spawn failed
    /// with no retained owner. Never derive this from worker JSON or missing PID.
    NotStarted,
    Pending,
    Exited {
        exit_code: i32,
        family_empty: bool,
        streams_eof: bool,
        forced: bool,
    },
}

/// Candidate is a typed, unpublished value, never VerifiedJobOutput by coercion.
pub trait OwnedAttemptExecutor: Send + Sync {
    type Candidate: Send + 'static;

    /// Preparation must not start a process. Core owns the returned session before
    /// calling run, so a run panic cannot unwind away that original owner.
    fn prepare_owned(
        &self,
        input: &PreparedJobInput,
        blobs: &dyn SourceBlobStore,
        scope: &AttemptScope<'_>,
    ) -> Result<Box<dyn OwnedAttemptSession<Candidate = Self::Candidate>>, ApiError>;
}

#[must_use = "retain the original owner until explicit settlement"]
pub enum OwnedAttemptOutcome<T> {
    Retained(RetainedAttempt<T>),
    Settled(SettledAttempt<T>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettlementKind {
    NotStarted,
    Graceful,
    ForcedObserved,
}

#[must_use = "dropping this carrier is not process cleanup"]
pub struct RetainedAttempt<T> {
    binding: Arc<AttemptBinding>,
    owner: Box<dyn RetainedAttemptOwner>,
    candidate: Option<T>,
    diagnostic: AttemptDiagnostic,
    close_deadline: Instant,
    poll_poisoned: bool,
    panic_payload: Option<Box<dyn Any + Send>>,
    forced_seen: bool,
    observed_exit: Option<i32>,
}

#[must_use]
pub struct SettledAttempt<T> {
    pub(super) binding: Arc<AttemptBinding>,
    pub(super) candidate: Option<T>,
    pub(super) diagnostic: AttemptDiagnostic,
    pub(super) kind: SettlementKind,
    pub(super) permit_issued: bool,
    pub(super) publication_reserved: bool,
    observed_exit: Option<i32>,
}

#[must_use = "preserve both original settled candidate and unused permit"]
pub struct CandidateReleaseFailure<T> {
    settled: SettledAttempt<T>,
    permit: PublicationPermit,
}

impl<T> CandidateReleaseFailure<T> {
    pub fn into_parts(self) -> (SettledAttempt<T>, PublicationPermit) {
        (self.settled, self.permit)
    }
}

impl AttemptScope<'_> {
    /// Called by the trusted adapter once it owns the original execution family.
    pub fn retain<T>(
        self,
        owner: Box<dyn RetainedAttemptOwner>,
        candidate: Result<T, ApiError>,
    ) -> RetainedAttempt<T> {
        let mut diagnostic = AttemptDiagnostic::default();
        let candidate = match candidate {
            Ok(candidate) => Some(candidate),
            Err(error) => {
                diagnostic.record(&error);
                None
            }
        };
        RetainedAttempt {
            binding: Arc::clone(&self.binding),
            owner,
            candidate,
            diagnostic,
            close_deadline: self.deadline(),
            poll_poisoned: false,
            panic_payload: None,
            forced_seen: false,
            observed_exit: None,
        }
    }

    /// A trusted adapter's prelaunch failure can never grant publication.
    pub fn not_started<T>(self, error: ApiError) -> SettledAttempt<T> {
        let mut diagnostic = AttemptDiagnostic::default();
        diagnostic.record(&error);
        SettledAttempt {
            binding: Arc::clone(&self.binding),
            candidate: None,
            diagnostic,
            kind: SettlementKind::NotStarted,
            permit_issued: false,
            publication_reserved: false,
            observed_exit: None,
        }
    }
}

impl<T> RetainedAttempt<T> {
    pub fn identity(&self) -> &JobAttemptIdentityV1 {
        &self.binding.identity
    }

    pub fn diagnostic(&self) -> &AttemptDiagnostic {
        &self.diagnostic
    }

    pub fn close_deadline(&self) -> Instant {
        self.close_deadline
    }

    pub fn poll_poisoned(&self) -> bool {
        self.poll_poisoned
    }

    pub fn panic_payload_retained(&self) -> bool {
        self.panic_payload.is_some()
    }

    /// Persistence/cleanup failures stay secondary to the original engine error.
    pub fn record_failure(&mut self, error: &ApiError) {
        self.diagnostic.record(error);
    }

    pub fn poll_close(mut self, requested_deadline: Instant) -> OwnedAttemptOutcome<T> {
        self.close_deadline = self.close_deadline.min(requested_deadline);
        if self.poll_poisoned {
            return OwnedAttemptOutcome::Retained(self);
        }
        let observation = match catch_unwind(AssertUnwindSafe(|| {
            self.owner.poll_close(self.close_deadline)
        })) {
            Ok(Ok(observation)) => observation,
            Ok(Err(error)) => {
                self.diagnostic.record(&error);
                return OwnedAttemptOutcome::Retained(self);
            }
            Err(payload) => {
                self.poll_poisoned = true;
                // A custom panic payload can itself have a panicking destructor.
                // Keep it with the original owner instead of dropping while catching.
                self.panic_payload = Some(payload);
                self.diagnostic.record(&ApiError::new(
                    ErrorCode::Internal,
                    "Original owner poll panicked; owner remains retained",
                ));
                return OwnedAttemptOutcome::Retained(self);
            }
        };
        if observation == OwnerObservation::NotStarted {
            if self.observed_exit.is_some() || self.forced_seen {
                self.diagnostic.record(&ApiError::new(
                    ErrorCode::Internal,
                    "No-child observation contradicts original process observations",
                ));
                return OwnedAttemptOutcome::Retained(self);
            }
            return OwnedAttemptOutcome::Settled(SettledAttempt {
                binding: self.binding,
                candidate: self.candidate,
                diagnostic: self.diagnostic,
                kind: SettlementKind::NotStarted,
                permit_issued: false,
                publication_reserved: false,
                observed_exit: None,
            });
        }
        if let OwnerObservation::Exited {
            exit_code, forced, ..
        } = observation
        {
            self.forced_seen |= forced;
            if self
                .observed_exit
                .is_some_and(|original| original != exit_code)
            {
                self.diagnostic.record(&ApiError::new(
                    ErrorCode::Internal,
                    "Original owner exit observation changed",
                ));
            }
            self.observed_exit.get_or_insert(exit_code);
        }
        let OwnerObservation::Exited {
            exit_code,
            family_empty: true,
            streams_eof: true,
            ..
        } = observation
        else {
            return OwnedAttemptOutcome::Retained(self);
        };
        if self.forced_seen || exit_code != 0 {
            self.diagnostic.record(&ApiError::new(
                ErrorCode::EngineFailed,
                "Original execution did not settle successfully without force",
            ));
        }
        // Original handles are dropped only after observed exit, empty family and EOF.
        OwnedAttemptOutcome::Settled(SettledAttempt {
            binding: self.binding,
            candidate: self.candidate,
            diagnostic: self.diagnostic,
            kind: if self.forced_seen {
                SettlementKind::ForcedObserved
            } else {
                SettlementKind::Graceful
            },
            permit_issued: false,
            publication_reserved: false,
            observed_exit: self.observed_exit,
        })
    }
}

impl<T> SettledAttempt<T> {
    pub(in crate::application) fn candidate_ref(&self) -> Option<&T> {
        self.candidate.as_ref()
    }
    pub(in crate::application) fn candidate_mut(&mut self) -> Option<&mut T> {
        self.candidate.as_mut()
    }
    pub fn identity(&self) -> &JobAttemptIdentityV1 {
        &self.binding.identity
    }

    pub fn kind(&self) -> SettlementKind {
        self.kind
    }

    pub fn diagnostic(&self) -> &AttemptDiagnostic {
        &self.diagnostic
    }

    pub fn observed_exit(&self) -> Option<i32> {
        self.observed_exit
    }

    pub fn publication_reserved(&self) -> bool {
        self.publication_reserved
    }

    pub fn record_failure(&mut self, error: &ApiError) {
        self.diagnostic.record(error);
    }

    /// A one-use core CAS permit releases the unpublished candidate.
    pub fn into_candidate(
        mut self,
        permit: PublicationPermit,
    ) -> Result<T, Box<CandidateReleaseFailure<T>>> {
        if !same_binding(&self.binding, &permit.binding)
            || !self.permit_issued
            || !self.publication_reserved
            || self.diagnostic.first().is_some()
            || self.kind != SettlementKind::Graceful
        {
            return Err(Box::new(CandidateReleaseFailure {
                settled: self,
                permit,
            }));
        }
        if self.binding.clock.now() >= self.binding.deadline {
            self.record_failure(&ApiError::new(
                ErrorCode::Timeout,
                "Original deadline expired before candidate release",
            ));
            return Err(Box::new(CandidateReleaseFailure {
                settled: self,
                permit,
            }));
        }
        match self.candidate {
            Some(candidate) => Ok(candidate),
            None => Err(Box::new(CandidateReleaseFailure {
                settled: self,
                permit,
            })),
        }
    }
}

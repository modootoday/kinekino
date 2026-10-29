use super::{change_attempt, read_attempt};
use crate::{
    application::{
        ExecutionControl, JobAttemptIdentityV1, JobStore, SourceBlobStore,
        owned_attempt::{
            AttemptDiagnostic, CoreAttemptBinding, OwnedAttemptOutcome, OwnedExecutorPort,
            OwnedOutputCandidate, PublicationPermit, RetainedAttempt, SessionCandidate,
            SessionExecution, SettledAttempt, SettlementKind, run_prepared,
        },
    },
    domain::{JobId, JobRecord, JobState, SourceBlobRef, VerifiedJobOutput},
};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use std::{
    any::Any,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::Arc,
    time::{Duration, Instant},
};
mod persistence;
#[cfg(test)]
mod tests;

type Candidate = SessionCandidate<Box<dyn OwnedOutputCandidate>>;
enum Stage {
    Closing(Option<RetainedAttempt<Candidate>>),
    Settled(Option<SettledAttempt<Candidate>>),
    Output(Box<Output>),
}
struct Output {
    candidate: Box<dyn OwnedOutputCandidate>,
    published: bool,
    verifying: bool,
    verified: Option<VerifiedJobOutput>,
}
struct OriginalControl {
    store: Arc<dyn JobStore>,
    identity: JobAttemptIdentityV1,
    deadline: Instant,
}
impl OriginalControl {
    fn current(&self) -> Result<JobRecord, ApiError> {
        let current = read_attempt(
            self.store.as_ref(),
            self.identity.job_id(),
            self.identity.attempt(),
        )?;
        if JobAttemptIdentityV1::from_running_record(&current) != self.identity {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Original owned attempt provenance changed",
            ));
        }
        Ok(current)
    }
    fn change(
        &self,
        change: impl Fn(&mut JobRecord) -> Result<(), &'static str>,
    ) -> Result<JobRecord, ApiError> {
        change_attempt(
            self.store.as_ref(),
            self.identity.job_id(),
            self.identity.attempt(),
            |job| {
                if JobAttemptIdentityV1::from_running_record(job) != self.identity {
                    return Err("Original owned attempt provenance changed before CAS");
                }
                change(job)
            },
        )
    }
}
impl ExecutionControl for OriginalControl {
    fn execution_identity(&self) -> Option<&JobAttemptIdentityV1> {
        Some(&self.identity)
    }
    fn cancellation_requested(&self) -> bool {
        self.current().map_or(true, |job| {
            matches!(
                job.state,
                JobState::Cancelling | JobState::Cancelled | JobState::Interrupted
            )
        })
    }
    fn deadline(&self) -> Instant {
        self.deadline
    }
    fn reserve_publication(&self) -> Result<(), ApiError> {
        if Instant::now() >= self.deadline {
            return Err(ApiError::new(
                ErrorCode::Timeout,
                "Original owned publication deadline expired",
            ));
        }
        if self.current()?.cancel_requested {
            return Err(ApiError::new(
                ErrorCode::Cancelled,
                "Cancellation won before owned publication",
            ));
        }
        self.change(JobRecord::begin_publication).map(|_| ())
    }
    fn begin_verification(&self) -> Result<(), ApiError> {
        self.change(JobRecord::begin_verification).map(|_| ())
    }
}
#[must_use]
pub(crate) enum OwnedRunOutcome {
    Finished(Box<JobRecord>),
    Pending(Box<PendingOwnedAttempt>),
}
#[must_use = "Keep original owner, candidate, diagnostics and store binding"]
pub(crate) struct PendingOwnedAttempt {
    control: OriginalControl,
    binding: CoreAttemptBinding,
    blobs: Arc<dyn SourceBlobStore>,
    stage: Stage,
    first_error: Option<ApiError>,
    diagnostic: AttemptDiagnostic,
    diagnostic_blob: Option<SourceBlobRef>,
    diagnostic_history: Vec<SourceBlobRef>,
    rejected_permit: Option<PublicationPermit>,
    faulted: bool,
    release_closed: bool,
    release_poisoned: bool,
    release_poll_active: bool,
    panic_payload: Option<Box<dyn Any + Send>>,
}
impl PendingOwnedAttempt {
    fn poll_candidate_release(&mut self) -> bool {
        if self.release_closed {
            return true;
        }
        if self.release_poisoned {
            return false;
        }
        let candidate = match &mut self.stage {
            Stage::Closing(_) => return false,
            Stage::Settled(slot) => match slot.as_mut().and_then(SettledAttempt::candidate_mut) {
                Some(SessionCandidate::Output(candidate)) => Some(candidate),
                _ => None,
            },
            Stage::Output(output) => Some(&mut output.candidate),
        };
        let Some(candidate) = candidate else {
            self.release_closed = true;
            return true;
        };
        let result = catch_unwind(AssertUnwindSafe(|| {
            candidate.poll_release(self.control.deadline)
        }));
        match result {
            Ok(Ok(closed)) => {
                self.release_closed = closed;
                closed
            }
            Ok(Err(error)) => {
                self.record(error);
                self.faulted = true;
                false
            }
            Err(payload) => {
                if self.panic_payload.is_none() {
                    self.panic_payload = Some(payload);
                }
                self.release_poisoned = true;
                self.faulted = true;
                self.record(ApiError::new(
                    ErrorCode::Internal,
                    "Original candidate release panicked; owner remains retained",
                ));
                false
            }
        }
    }
    pub(crate) fn first_error(&self) -> Option<&ApiError> {
        self.first_error.as_ref()
    }
    pub(crate) fn record(&mut self, error: ApiError) {
        self.diagnostic.record(&error);
        if self.first_error.is_none() {
            self.first_error = Some(error);
        }
        self.diagnostic_blob = None;
    }
    fn capture_port_result<T>(
        &mut self,
        result: std::thread::Result<Result<T, ApiError>>,
    ) -> Result<T, ApiError> {
        match result {
            Ok(result) => result,
            Err(payload) => {
                if self.panic_payload.is_none() {
                    self.panic_payload = Some(payload);
                }
                Err(ApiError::new(
                    ErrorCode::Internal,
                    "Original owned settlement store or control panicked",
                ))
            }
        }
    }
    pub(crate) fn poll(mut self: Box<Self>) -> OwnedRunOutcome {
        if self.faulted {
            return self.persist_failure();
        }
        match &mut self.stage {
            Stage::Closing(owner) => {
                let outcome = owner
                    .take()
                    .expect("original retained owner")
                    .poll_close(self.control.deadline);
                match outcome {
                    OwnedAttemptOutcome::Retained(owner) => {
                        self.stage = Stage::Closing(Some(owner))
                    }
                    OwnedAttemptOutcome::Settled(settled) => {
                        self.stage = Stage::Settled(Some(settled))
                    }
                }
                OwnedRunOutcome::Pending(self)
            }
            Stage::Settled(slot) => {
                let mut settled = slot.take().expect("original settled attempt");
                if self.first_error.is_none()
                    && let Some(error) = settled.diagnostic().first()
                {
                    self.record(ApiError::new(error.code(), error.message()));
                }
                if self.first_error.is_some() {
                    self.stage = Stage::Settled(Some(settled));
                    self.faulted = true;
                    return self.persist_failure();
                }
                if Instant::now() >= self.control.deadline {
                    self.record(ApiError::new(
                        ErrorCode::Timeout,
                        "Original deadline expired before owned settlement admission",
                    ));
                    self.stage = Stage::Settled(Some(settled));
                    self.faulted = true;
                    return self.persist_failure();
                }
                let marker = match settled.candidate_ref() {
                    Some(SessionCandidate::CooperativeCancelled(marker)) => {
                        Some(self.binding.cancellation_matches(marker))
                    }
                    _ => None,
                };
                let observation = catch_unwind(AssertUnwindSafe(|| self.control.current()));
                let current = match self.capture_port_result(observation) {
                    Ok(current) => current,
                    Err(error) => {
                        self.record(error);
                        self.stage = Stage::Settled(Some(settled));
                        self.faulted = true;
                        return self.persist_failure();
                    }
                };
                if current.state == JobState::Cancelling || marker.is_some() {
                    self.stage = Stage::Settled(Some(settled));
                    if !self.poll_candidate_release() {
                        return OwnedRunOutcome::Pending(self);
                    }
                    let settled = match &mut self.stage {
                        Stage::Settled(slot) => slot
                            .take()
                            .expect("original settled candidate retained through release"),
                        _ => unreachable!(),
                    };
                    // A late parent cancel can discard a successful unpublished
                    // candidate after graceful closure. Engine errors were already
                    // handled above and can never be converted into this ACK.
                    let clean = (settled.kind() == SettlementKind::Graceful
                        && settled.observed_exit() == Some(0))
                        || (settled.kind() == SettlementKind::NotStarted
                            && settled.observed_exit().is_none());
                    let valid = marker != Some(false)
                        && clean
                        && current.state == JobState::Cancelling
                        && !current.publication_started;
                    let result = if valid {
                        let changed = catch_unwind(AssertUnwindSafe(|| {
                            self.control.change(JobRecord::acknowledge_cancel)
                        }));
                        self.capture_port_result(changed)
                    } else {
                        Err(ApiError::new(
                            ErrorCode::RevisionConflict,
                            "Cooperative settlement does not match original cancelling attempt",
                        ))
                    };
                    match result {
                        Ok(job) => return OwnedRunOutcome::Finished(Box::new(job)),
                        Err(error) => {
                            self.record(error);
                            self.stage = Stage::Settled(Some(settled));
                            self.faulted = true;
                            return self.persist_failure();
                        }
                    }
                }
                // The original settled owner stays outside this adapter boundary,
                // including a CAS that commits and then unwinds before returning.
                let reservation = catch_unwind(AssertUnwindSafe(|| {
                    self.binding
                        .gate(&self.control)
                        .and_then(|gate| gate.reserve(&mut settled))
                }));
                let permit = match self.capture_port_result(reservation) {
                    Ok(permit) => permit,
                    Err(error) => {
                        self.record(error);
                        self.stage = Stage::Settled(Some(settled));
                        self.faulted = true;
                        return self.persist_failure();
                    }
                };
                match settled.into_candidate(permit) {
                    Ok(SessionCandidate::Output(candidate)) => {
                        self.stage = Stage::Output(Box::new(Output {
                            candidate,
                            published: false,
                            verifying: false,
                            verified: None,
                        }));
                    }
                    Ok(SessionCandidate::CooperativeCancelled(_)) => {
                        unreachable!("cooperative candidate handled before publication")
                    }
                    Err(rejected) => {
                        let (settled, permit) = rejected.into_parts();
                        self.rejected_permit = Some(permit);
                        self.record(ApiError::new(
                            ErrorCode::Timeout,
                            "Original candidate release failed after publication CAS",
                        ));
                        self.stage = Stage::Settled(Some(settled));
                        self.faulted = true;
                    }
                }
                OwnedRunOutcome::Pending(self)
            }
            Stage::Output(output) => {
                let result = catch_unwind(AssertUnwindSafe(
                    || -> Result<Option<JobRecord>, ApiError> {
                        if Instant::now() >= self.control.deadline {
                            return Err(ApiError::new(
                                ErrorCode::Timeout,
                                "Original deadline expired before output completion",
                            ));
                        }
                        if !output.published {
                            output.candidate.publish()?;
                            output.published = true;
                            return Ok(None);
                        }
                        if !output.verifying {
                            self.control.begin_verification()?;
                            output.verifying = true;
                            return Ok(None);
                        }
                        if output.verified.is_none() {
                            let verified = output.candidate.verify()?;
                            verified.validate().map_err(|message| {
                                ApiError::new(ErrorCode::EngineFailed, message)
                            })?;
                            output.verified = Some(verified);
                            return Ok(None);
                        }
                        if !self.release_closed {
                            self.release_poll_active = true;
                            let closed = output.candidate.poll_release(self.control.deadline);
                            self.release_poll_active = false;
                            if !closed? {
                                return Ok(None);
                            }
                            self.release_closed = true;
                            return Ok(None);
                        }
                        if Instant::now() >= self.control.deadline {
                            return Err(ApiError::new(
                                ErrorCode::Timeout,
                                "Original deadline expired after output verification",
                            ));
                        }
                        let verified = output
                            .verified
                            .as_ref()
                            .expect("original verified candidate");
                        self.control
                            .change(|job| job.complete(verified.clone()))
                            .map(Some)
                    },
                ));
                match result {
                    Ok(Ok(Some(job))) => OwnedRunOutcome::Finished(Box::new(job)),
                    Ok(Ok(None)) => OwnedRunOutcome::Pending(self),
                    Ok(Err(error)) => {
                        self.record(error);
                        self.faulted = true;
                        self.persist_failure()
                    }
                    Err(payload) => {
                        if self.release_poll_active {
                            self.release_poisoned = true;
                        }
                        self.panic_payload = Some(payload);
                        self.record(ApiError::new(
                            ErrorCode::Internal,
                            "Original output publisher or verification panicked",
                        ));
                        self.faulted = true;
                        self.persist_failure()
                    }
                }
            }
        }
    }
}

pub(crate) fn run_owned(
    store: Arc<dyn JobStore>,
    executor: &OwnedExecutorPort,
    blobs: Arc<dyn SourceBlobStore>,
    id: &JobId,
    attempt: u64,
) -> Result<OwnedRunOutcome, ApiError> {
    let current = read_attempt(store.as_ref(), id, attempt)?;
    if current.state == JobState::Cancelling {
        return change_attempt(store.as_ref(), id, attempt, JobRecord::acknowledge_cancel)
            .map(|job| OwnedRunOutcome::Finished(Box::new(job)));
    }
    let running = change_attempt(store.as_ref(), id, attempt, JobRecord::start)?;
    let control = OriginalControl {
        store,
        identity: JobAttemptIdentityV1::from_running_record(&running),
        deadline: Instant::now()
            .checked_add(Duration::from_millis(running.input.execution_timeout_ms.0))
            .ok_or_else(|| {
                ApiError::new(
                    ErrorCode::InvalidArgument,
                    "Invalid original owned deadline",
                )
            })?,
    };
    let binding = CoreAttemptBinding::from_running(&running, &control)?;
    // Core observes a cancel before preparation/launch, so no process can exist yet.
    if control.current()?.state == JobState::Cancelling {
        return change_attempt(
            control.store.as_ref(),
            id,
            attempt,
            JobRecord::acknowledge_cancel,
        )
        .map(|job| OwnedRunOutcome::Finished(Box::new(job)));
    }
    let scope = binding.scope(&control)?;
    let prepared = catch_unwind(AssertUnwindSafe(|| {
        executor.prepare_owned(&running.input, blobs.as_ref(), &scope)
    }));
    let (stage, first_error, panic_payload) = match prepared {
        Ok(Ok(session)) => {
            let execution: SessionExecution<Box<dyn OwnedOutputCandidate>> =
                run_prepared(session, &running.input, blobs.as_ref(), scope);
            (
                Stage::Closing(Some(execution.retained)),
                execution.first_error,
                execution.panic_payload,
            )
        }
        Ok(Err(error)) => {
            let copy = ApiError::new(error.code, &error.message);
            (
                Stage::Settled(Some(scope.not_started(copy))),
                Some(error),
                None,
            )
        }
        Err(payload) => {
            let error = ApiError::new(ErrorCode::Internal, "Owned prelaunch preparation panicked");
            let copy = ApiError::new(error.code, &error.message);
            (
                Stage::Settled(Some(scope.not_started(copy))),
                Some(error),
                Some(payload),
            )
        }
    };
    let mut pending = Box::new(PendingOwnedAttempt {
        control,
        binding,
        blobs,
        stage,
        first_error: None,
        diagnostic: AttemptDiagnostic::default(),
        diagnostic_blob: None,
        diagnostic_history: Vec::new(),
        rejected_permit: None,
        faulted: false,
        release_closed: false,
        release_poisoned: false,
        release_poll_active: false,
        panic_payload,
    });
    if let Some(error) = first_error {
        pending.record(error);
    }
    Ok(OwnedRunOutcome::Pending(pending))
}

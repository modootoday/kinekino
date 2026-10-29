use super::{
    candidate::{CompleteCandidate, PublicationState},
    driver::Driver,
};
use crate::application::{OutputPublication, RunControl, WorkerTransport};
use kinekino_jobs::{
    application::{
        JobAttemptIdentityV1, SourceBlobStore,
        owned_attempt::{
            AttemptScope, OwnedAttemptSession, OwnedOutputCandidate, OwnerObservation, SessionRun,
        },
    },
    domain::PreparedJobInput,
};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_vad::protocol::domain::{ExpectedWorkerContext, WorkerResult};
use std::{sync::Arc, time::Instant};

pub struct PreparedVadSession {
    driver: Driver,
    identity: JobAttemptIdentityV1,
    deadline: Instant,
    publication: Arc<dyn OutputPublication>,
}
impl PreparedVadSession {
    pub(crate) fn new(
        transport: Box<dyn WorkerTransport>,
        request: Vec<u8>,
        context: ExpectedWorkerContext,
        identity: JobAttemptIdentityV1,
        deadline: Instant,
        publication: Arc<dyn OutputPublication>,
    ) -> Self {
        Self {
            driver: Driver::new(transport, request, context),
            identity,
            deadline,
            publication,
        }
    }
}
struct ScopeView<'a, 'b>(&'a AttemptScope<'b>);
impl RunControl for ScopeView<'_, '_> {
    fn deadline(&self) -> Instant {
        self.0.deadline()
    }
    fn cancellation_requested(&self) -> bool {
        self.0.cancellation_requested()
    }
}
impl OwnedAttemptSession for PreparedVadSession {
    type Candidate = Box<dyn OwnedOutputCandidate>;
    fn run(
        &mut self,
        input: &PreparedJobInput,
        _blobs: &dyn SourceBlobStore,
        scope: &AttemptScope<'_>,
    ) -> SessionRun<Self::Candidate> {
        if scope.deadline() != self.deadline
            || scope.identity() != &self.identity
            || input.sha256().ok().as_deref() != Some(self.identity.input_sha256())
        {
            return SessionRun::Failed(ApiError::new(
                ErrorCode::RevisionConflict,
                "Original VAD input or attempt changed",
            ));
        }
        if let Err(error) = self.driver.run(&ScopeView(scope)) {
            if error.code == ErrorCode::Cancelled {
                if cooperative_partial(
                    self.driver.observation.as_ref(),
                    scope.cancellation_requested(),
                    self.driver.transport.cancellation_signal_sent(),
                ) && let Some(marker) = scope.observe_cooperative_cancellation()
                {
                    return SessionRun::CooperativeCancelled(marker);
                }
                if self.driver.observation.as_ref().is_some_and(|wire| {
                    matches!(
                        wire.result(),
                        WorkerResult::Complete { .. } | WorkerResult::Failure { .. }
                    )
                }) {
                    // Core handles a completed candidate followed by parent cancel;
                    // worker Failure remains its original error, never a marker.
                    return map_outcome(&mut self.driver, self.publication.clone(), self.deadline);
                }
            }
            return SessionRun::Failed(error);
        }
        map_outcome(&mut self.driver, self.publication.clone(), self.deadline)
    }
    fn poll_close(&mut self, deadline: Instant) -> Result<OwnerObservation, ApiError> {
        // SDK supplies its close cap; transport constrains the original owned
        // cleanup end. Execution cutoff is not a replacement cleanup deadline.
        self.driver.transport.poll_close(deadline)
    }
}
pub(super) fn cooperative_partial(
    observation: Option<&kinekino_vad::protocol::domain::WorkerOutcomeCandidate>,
    parent_cancelled: bool,
    signal_sent: bool,
) -> bool {
    parent_cancelled
        && signal_sent
        && observation.is_some_and(|wire| matches!(wire.result(), WorkerResult::Partial { .. }))
}
pub(super) fn map_outcome(
    driver: &mut Driver,
    publication: Arc<dyn OutputPublication>,
    deadline: Instant,
) -> SessionRun<Box<dyn OwnedOutputCandidate>> {
    let wire = driver
        .observation
        .as_ref()
        .expect("Decoded outcome stored before result mapping");
    match wire.result() {
        WorkerResult::Complete { .. } => {
            let wire = driver.observation.take().expect("Original complete wire");
            SessionRun::Candidate(Box::new(CompleteCandidate {
                wire,
                publication,
                deadline,
                state: PublicationState::Fresh,
            }))
        }
        WorkerResult::Partial { .. } => SessionRun::Failed(ApiError::new(
            ErrorCode::EngineFailed,
            "Partial VAD observation retained; completion and cancellation ACK unproven",
        )),
        WorkerResult::Failure { code, message } => {
            SessionRun::Failed(ApiError::new(*code, message))
        }
    }
}

use crate::application::OutputPublication;
use kinekino_jobs::{application::owned_attempt::OwnedOutputCandidate, domain::VerifiedJobOutput};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_vad::protocol::domain::WorkerOutcomeCandidate;
use std::{sync::Arc, time::Instant};

pub(super) struct CompleteCandidate {
    pub wire: WorkerOutcomeCandidate,
    pub publication: Arc<dyn OutputPublication>,
    pub deadline: Instant,
    pub state: PublicationState,
}
#[derive(PartialEq, Eq)]
pub(super) enum PublicationState {
    Fresh,
    Attempted,
    Published,
}
impl OwnedOutputCandidate for CompleteCandidate {
    fn poll_release(&mut self, _: Instant) -> Result<bool, ApiError> {
        // Original VAD process is already settled; candidate owns immutable output only.
        Ok(true)
    }
    fn publish(&mut self) -> Result<(), ApiError> {
        if self.state != PublicationState::Fresh || Instant::now() >= self.deadline {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "VAD publication repeated or expired",
            ));
        }
        // Even a failing publication may have created durable output. No retry.
        self.state = PublicationState::Attempted;
        self.publication.publish(&self.wire, self.deadline)?;
        self.state = PublicationState::Published;
        Ok(())
    }
    fn verify(&mut self) -> Result<VerifiedJobOutput, ApiError> {
        if self.state != PublicationState::Published || Instant::now() >= self.deadline {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "VAD output not published or expired",
            ));
        }
        self.publication.verify(&self.wire, self.deadline)
    }
}

//! Joined child results are delivered by an explicit host-owned dispatcher.
use super::{ParentBinding, ParentCleanupStatus, internal};
use crate::{
    application::api::{ApiError, ErrorCode},
    infrastructure::thread_completion::{
        CompletionWake, DispatcherHandle, JoinFailure, JoinRequest, JoinTicket, NotificationState,
        SubmitRejection,
    },
};
use std::{num::NonZeroU64, sync::atomic::Ordering};

pub enum ParentCleanupObservation<'a> {
    NotStarted,
    Pending,
    Finished(&'a Result<ParentCleanupStatus, ApiError>),
}
type CleanupResult = Result<ParentCleanupStatus, ApiError>;

/// Retain this child owner and the separate DispatcherHost until their own completion.
#[must_use = "retain cleanup ownership until the joined result has been handled"]
pub struct ParentCleanupTask {
    binding: ParentBinding,
    epoch: NonZeroU64,
    dispatcher: DispatcherHandle,
    wake: CompletionWake,
    request: Option<JoinRequest<CleanupResult>>,
    ticket: Option<JoinTicket<CleanupResult>>,
    result: Option<CleanupResult>,
}
impl ParentCleanupTask {
    pub fn new(
        binding: ParentBinding,
        epoch: NonZeroU64,
        dispatcher: DispatcherHandle,
        wake: CompletionWake,
    ) -> Self {
        Self {
            binding,
            epoch,
            dispatcher,
            wake,
            request: None,
            ticket: None,
            result: None,
        }
    }

    /// Queue rejection starts no child and retains the original request for explicit retry.
    pub fn start_attempt(&mut self) -> Result<(), ApiError> {
        self.binding.require_epoch(self.epoch)?;
        if !self.binding.0.revoked.load(Ordering::Acquire) {
            return Err(ApiError::new(
                ErrorCode::NeedsInput,
                "Revoke admission first",
            ));
        }
        if self.ticket.is_some() && self.result.is_none()
            || matches!(self.result, Some(Ok(ParentCleanupStatus::Complete(_))))
        {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Cleanup is active or complete",
            ));
        }
        let request = self.request.take().unwrap_or_else(|| {
            let binding = self.binding.clone();
            let epoch = self.epoch;
            JoinRequest::new(move || binding.cleanup_wait(epoch), self.wake.clone())
        });
        match self.dispatcher.submit(request) {
            Ok(ticket) => {
                self.ticket = Some(ticket);
                self.result = None;
                Ok(())
            }
            Err(refused) => {
                self.request = Some(refused.request);
                let code = match refused.reason {
                    SubmitRejection::Full => ErrorCode::ResourceExhausted,
                    SubmitRejection::Closed => ErrorCode::Cancelled,
                    SubmitRejection::Faulted => ErrorCode::Internal,
                };
                Err(ApiError::new(
                    code,
                    "Cleanup dispatch was refused; request remains owned",
                ))
            }
        }
    }

    /// Reads a joined-result cell only; never waits or joins on the calling UI thread.
    pub fn observe(&mut self) -> ParentCleanupObservation<'_> {
        if self.result.is_none()
            && let Some(ticket) = &mut self.ticket
        {
            match ticket.try_take() {
                None => return ParentCleanupObservation::Pending,
                Some(Ok(result)) => self.result = Some(result),
                Some(Err(error)) => {
                    self.result = Some(Err(match error {
                        JoinFailure::DispatcherPanicked => internal(),
                        JoinFailure::WorkerPanicked => {
                            ApiError::new(ErrorCode::EngineFailed, "Cleanup worker panicked")
                        }
                        JoinFailure::Spawn(message) => {
                            ApiError::new(ErrorCode::EngineFailed, message)
                        }
                    }))
                }
            }
        }
        match &self.result {
            Some(result) => ParentCleanupObservation::Finished(result),
            None => ParentCleanupObservation::NotStarted,
        }
    }

    /// This is notification execution status, not a GUI delivery acknowledgement.
    pub fn notification(&self) -> Option<NotificationState> {
        self.ticket.as_ref().map(JoinTicket::notification)
    }
}

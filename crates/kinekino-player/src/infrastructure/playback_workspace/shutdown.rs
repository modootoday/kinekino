use super::{PlaybackWorkspaceHost, PlaybackWorkspaceShutdown};
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        jobs::JobId,
        playback_session::ClosedEpisodePlayback,
    },
    infrastructure::browser_provider_jobs::{
        PendingProviderHost, ProviderShutdownOutcome, ProviderShutdownReceipt,
    },
};
use kinekino_jobs::infrastructure::{JobHostOwner, ShutdownErrors, WorkerHost};
use std::time::Instant;

#[must_use]
pub enum WorkspaceShutdownOutcome<W: JobHostOwner = WorkerHost> {
    Complete(Box<WorkspaceShutdownComplete>),
    Pending(Box<PendingPlaybackWorkspace<W>>),
}
pub struct WorkspaceShutdownComplete {
    pub receipt: PlaybackWorkspaceShutdown,
    pub errors: ShutdownErrors,
}
#[must_use = "Retain original native, provider and transition owners"]
pub struct PendingPlaybackWorkspace<W: JobHostOwner = WorkerHost> {
    host: PlaybackWorkspaceHost<W>,
    native: Option<Option<ClosedEpisodePlayback>>,
    transition: Option<Option<JobId>>,
    provider: Option<Box<PendingProviderHost<W>>>,
    provider_receipt: Option<ProviderShutdownReceipt>,
    errors: ShutdownErrors,
    deadline: Instant,
    cutoff_observed: bool,
}
impl<W: JobHostOwner> PlaybackWorkspaceHost<W> {
    pub fn begin_shutdown(self, original_deadline: Instant) -> Box<PendingPlaybackWorkspace<W>> {
        Box::new(PendingPlaybackWorkspace {
            host: self,
            native: None,
            transition: None,
            provider: None,
            provider_receipt: None,
            errors: ShutdownErrors::default(),
            deadline: original_deadline,
            cutoff_observed: false,
        })
    }
}
impl<W: JobHostOwner> PendingPlaybackWorkspace<W> {
    pub fn original_attempt_error(&self) -> Option<&ApiError> {
        self.provider
            .as_ref()
            .and_then(|provider| provider.original_attempt_error())
    }
    pub fn errors(&self) -> &ShutdownErrors {
        if self.errors.first().is_none()
            && let Some(provider) = &self.provider
        {
            return provider.errors();
        }
        &self.errors
    }
    pub fn deadline(&self) -> Instant {
        self.deadline
    }
    pub fn poll(mut self: Box<Self>, constrained_deadline: Instant) -> WorkspaceShutdownOutcome<W> {
        self.deadline = self.deadline.min(constrained_deadline);
        if self.provider.is_none() && Instant::now() >= self.deadline && !self.cutoff_observed {
            self.cutoff_observed = true;
            self.errors.record(ApiError::new(
                ErrorCode::Timeout,
                "Original workspace shutdown deadline expired",
            ));
        }
        if self.native.is_none() {
            match self.host.native.shutdown() {
                Ok(receipt) => self.native = Some(receipt),
                Err(error) => {
                    self.errors.record(error);
                    return WorkspaceShutdownOutcome::Pending(self);
                }
            }
        }
        if self.transition.is_none() {
            match self.host.transitions.end() {
                Ok(receipt) => self.transition = Some(receipt),
                Err(error) => {
                    self.errors.record(error);
                    return WorkspaceShutdownOutcome::Pending(self);
                }
            }
        }
        if self.provider.is_none() && self.provider_receipt.is_none() {
            let Some(provider) = self.host.provider.take() else {
                self.errors.record(ApiError::new(
                    ErrorCode::Internal,
                    "Original workspace provider is missing",
                ));
                return WorkspaceShutdownOutcome::Pending(self);
            };
            self.provider = Some(provider.begin_shutdown(self.deadline));
        }
        if let Some(provider) = self.provider.take() {
            match provider.poll(self.deadline) {
                ProviderShutdownOutcome::Pending(provider) => {
                    self.provider = Some(provider);
                    return WorkspaceShutdownOutcome::Pending(self);
                }
                ProviderShutdownOutcome::Complete(result) => {
                    self.errors.merge(result.errors);
                    self.provider_receipt = result.receipt;
                }
            }
        }
        let Some(provider) = self.provider_receipt.take() else {
            return WorkspaceShutdownOutcome::Pending(self);
        };
        if Instant::now() >= self.deadline && !self.cutoff_observed {
            self.cutoff_observed = true;
            self.errors.record(ApiError::new(
                ErrorCode::Timeout,
                "Original workspace shutdown deadline expired",
            ));
        }
        WorkspaceShutdownOutcome::Complete(Box::new(WorkspaceShutdownComplete {
            receipt: PlaybackWorkspaceShutdown {
                native: Ok(self.native.take().expect("observed native closure")),
                pending_transition_job: Ok(self
                    .transition
                    .take()
                    .expect("observed transition closure")),
                provider: Ok(provider),
            },
            errors: self.errors,
        }))
    }
}

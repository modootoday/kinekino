//! Shutdown dependencies for the Windows-only provider and playback-workspace hosts.
use super::{DependencyPoll, PendingDependency, completion};
use kinekino_jobs::infrastructure::{JobHostOwner, RetainingWorkerHost, ShutdownErrors};
use kinekino_player::{
    application::{api::ApiError, skip_segments::host::SkipHostOwner},
    infrastructure::{
        browser_provider_jobs::{
            PendingProviderHost, ProviderShutdownOutcome, RetainingProviderJobsHost,
        },
        playback_workspace::{
            PendingPlaybackWorkspace, RetainingPlaybackWorkspaceHost, WorkspaceShutdownOutcome,
        },
    },
};
use std::time::Instant;

impl<W: JobHostOwner> PendingDependency for PendingProviderHost<W> {
    fn original_attempt_error(&self) -> Option<&ApiError> {
        self.original_attempt_error()
    }
    fn errors(&self) -> &ShutdownErrors {
        self.errors()
    }
    fn poll(self: Box<Self>, deadline: Instant) -> DependencyPoll {
        match PendingProviderHost::poll(self, deadline) {
            ProviderShutdownOutcome::Pending(owner) => DependencyPoll::Pending(owner),
            ProviderShutdownOutcome::Complete(result) => {
                DependencyPoll::Complete(completion(result.receipt, result.errors))
            }
        }
    }
}
impl<W: JobHostOwner> PendingDependency for PendingPlaybackWorkspace<W> {
    fn original_attempt_error(&self) -> Option<&ApiError> {
        self.original_attempt_error()
    }
    fn errors(&self) -> &ShutdownErrors {
        self.errors()
    }
    fn poll(self: Box<Self>, deadline: Instant) -> DependencyPoll {
        match PendingPlaybackWorkspace::poll(self, deadline) {
            WorkspaceShutdownOutcome::Pending(owner) => DependencyPoll::Pending(owner),
            WorkspaceShutdownOutcome::Complete(result) => {
                DependencyPoll::Complete(completion(result.receipt, result.errors))
            }
        }
    }
}
struct PlaybackDependency {
    workspace: Box<PendingPlaybackWorkspace<RetainingWorkerHost>>,
    _skip_owner: SkipHostOwner,
}
impl PendingDependency for PlaybackDependency {
    fn original_attempt_error(&self) -> Option<&ApiError> {
        self.workspace.original_attempt_error()
    }
    fn errors(&self) -> &ShutdownErrors {
        self.workspace.errors()
    }
    fn poll(self: Box<Self>, deadline: Instant) -> DependencyPoll {
        let Self {
            workspace,
            _skip_owner,
        } = *self;
        match workspace.poll(deadline) {
            WorkspaceShutdownOutcome::Pending(workspace) => {
                DependencyPoll::Pending(Box::new(Self {
                    workspace,
                    _skip_owner,
                }))
            }
            WorkspaceShutdownOutcome::Complete(result) => {
                DependencyPoll::Complete(completion(result.receipt, result.errors))
            }
        }
    }
}
pub(crate) fn pending_provider(
    owner: RetainingProviderJobsHost,
    deadline: Instant,
) -> Box<dyn PendingDependency> {
    owner.begin_shutdown(deadline)
}
pub(crate) fn pending_playback(
    owner: RetainingPlaybackWorkspaceHost,
    skip_owner: SkipHostOwner,
    deadline: Instant,
) -> Box<dyn PendingDependency> {
    Box::new(PlaybackDependency {
        workspace: owner.begin_shutdown(deadline),
        _skip_owner: skip_owner,
    })
}
pub(crate) fn pending_workspace(
    owner: RetainingPlaybackWorkspaceHost,
    deadline: Instant,
) -> Box<dyn PendingDependency> {
    owner.begin_shutdown(deadline)
}

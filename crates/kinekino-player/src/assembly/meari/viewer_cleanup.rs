//! Returned viewer ownership carrier; common driver scheduling, no second shutdown executor.
use super::{
    native_cleanup_driver::{CleanupAdapter, CleanupPoll},
    player_shell_document::DocumentFailure,
    player_shell_host::ShellHostController,
    player_shell_startup::ShellPreparationFailure,
    viewer_host::{ViewerCloseProgress, ViewerHost, ViewerHostError},
};
use crate::application::{
    api::{ApiError, ErrorCode},
    sdk_execution::ExecutionCloseProgress,
};
use meari_native::{
    application::window_lifecycle::*,
    infrastructure::{
        native_host::{CloseCause, HostLifecycle, HostRetry},
        window::{NativeLoopOwner, NativeRunOutcome, NativeStartup},
    },
};
use std::{
    convert::Infallible,
    rc::Rc,
    time::{Duration, Instant},
};

#[derive(Clone, Copy)]
pub enum ViewerNoChild {}
impl NativeChild for ViewerNoChild {
    type Error = Infallible;
    fn parent_lost(&mut self) {
        match *self {}
    }
    fn begin_cleanup(
        &mut self,
        _: CleanupRequest,
        _: CompletionSignal,
    ) -> Result<CleanupProgress, Self::Error> {
        match *self {}
    }
    fn observe_cleanup(&mut self, _: CleanupRequest) -> Result<CleanupProgress, Self::Error> {
        match *self {}
    }
}
pub type TypedViewerController<C> = ShellHostController<ViewerHost, C>;
pub type ViewerController = TypedViewerController<ViewerNoChild>;
pub struct UnrunNative {
    pub owner: NativeLoopOwner,
    pub startup: NativeStartup,
}
#[must_use = "Preserve every original owner on failure, pending cleanup and abnormal Native return"]
pub enum TypedViewerOwners<C: NativeChild> {
    ShellFailure {
        failure: Box<ShellPreparationFailure<ViewerHost>>,
        native: Box<UnrunNative>,
    },
    DocumentFailure {
        failure: Box<DocumentFailure<ViewerHost>>,
        native: Box<UnrunNative>,
    },
    Unrun {
        controller: Box<TypedViewerController<C>>,
        native: Box<UnrunNative>,
    },
    Returned(Box<NativeRunOutcome<TypedViewerController<C>>>),
}
pub type ViewerOwners = TypedViewerOwners<ViewerNoChild>;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewerNativeState {
    NotRun,
    Closed,
    Unconfirmed,
    ForcedIncomplete,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewerCleanupStatus {
    WorkersQuiescent(ViewerNativeState),
    NeedsTerminalAck,
    Faulted,
    BudgetExhausted,
}
#[derive(Debug)]
pub enum ViewerCleanupError {
    Host(ViewerHostError),
    Runtime(String),
    RuntimeMismatch,
    MissingOwner,
    ExplicitRecoveryRequired(HostLifecycle),
    CloseRejected,
}
#[derive(Default)]
pub struct ViewerCleanupEvidence {
    pub observations: usize,
    pub last_workers: Option<ViewerCloseProgress>,
    pub last_native: Option<ViewerNativeState>,
    pub error: Option<ViewerCleanupError>,
}
pub struct ViewerCleanupLimits {
    duration: Duration,
    observations: usize,
    delay: Duration,
}
impl ViewerCleanupLimits {
    pub fn new(duration: Duration, observations: usize, delay: Duration) -> Result<Self, ApiError> {
        if duration.is_zero() || observations == 0 || delay.is_zero() || delay > duration {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Positive bounded viewer cleanup limits required",
            ));
        }
        Ok(Self {
            duration,
            observations,
            delay,
        })
    }
}
pub struct TypedViewerCleanup<C: NativeChild> {
    owners: TypedViewerOwners<C>,
    limits: ViewerCleanupLimits,
    started: Instant,
    evidence: ViewerCleanupEvidence,
}
pub type ViewerCleanup = TypedViewerCleanup<ViewerNoChild>;
impl<C: NativeChild> TypedViewerCleanup<C> {
    pub fn new(owners: TypedViewerOwners<C>, limits: ViewerCleanupLimits) -> Self {
        Self {
            owners,
            limits,
            started: Instant::now(),
            evidence: Default::default(),
        }
    }
}
fn unrun(
    host: &mut ViewerHost,
    native: ViewerNativeState,
) -> Result<(Option<ViewerCloseProgress>, ViewerNativeState), ViewerCleanupError> {
    host.poll_unrun_shutdown()
        .map(|p| (Some(p), native))
        .map_err(ViewerCleanupError::Host)
}
fn observe_owners<C: NativeChild>(
    owners: &mut TypedViewerOwners<C>,
) -> Result<(Option<ViewerCloseProgress>, ViewerNativeState), ViewerCleanupError> {
    match owners {
        TypedViewerOwners::ShellFailure { failure, .. } => {
            unrun(&mut failure.inputs.host_owner, ViewerNativeState::NotRun)
        }
        TypedViewerOwners::DocumentFailure { failure, .. } => {
            unrun(&mut failure.inputs.host_owner, ViewerNativeState::NotRun)
        }
        TypedViewerOwners::Unrun { controller, .. } => unrun(
            controller
                .pending_owner_mut()
                .ok_or(ViewerCleanupError::MissingOwner)?,
            ViewerNativeState::NotRun,
        ),
        TypedViewerOwners::Returned(outcome) => {
            if let (Some(a), Some(b)) = (&outcome.runtime, &outcome.controller.runtime)
                && !Rc::ptr_eq(a, b)
            {
                return Err(ViewerCleanupError::RuntimeMismatch);
            }
            let runtime = outcome
                .runtime
                .as_ref()
                .or(outcome.controller.runtime.as_ref())
                .cloned();
            if let Some(runtime) = runtime {
                let mut runtime = runtime
                    .try_borrow_mut()
                    .map_err(|_| ViewerCleanupError::Runtime("runtime busy".into()))?;
                let lifecycle = runtime.document().lifecycle();
                let close = match close_action(lifecycle) {
                    CloseAction::Request => Some(runtime.request_close(CloseCause::Host)),
                    CloseAction::Observe => Some(runtime.observe_close(CloseCause::Host)),
                    CloseAction::Retain => None,
                    CloseAction::ExplicitRecovery => {
                        return Err(ViewerCleanupError::ExplicitRecoveryRequired(lifecycle));
                    }
                };
                if let Some(close) = close {
                    let status =
                        close.map_err(|error| ViewerCleanupError::Runtime(format!("{error:?}")))?;
                    if status
                        == meari_native::infrastructure::native_host::HostCloseStatus::KeepOpen
                    {
                        return Err(ViewerCleanupError::CloseRejected);
                    }
                }
                if let Some(error) = runtime.host().host().last_close_error() {
                    return Err(ViewerCleanupError::Host(error.clone()));
                }
                let native = match runtime.document().lifecycle() {
                    HostLifecycle::Closed if outcome.result.is_ok() => ViewerNativeState::Closed,
                    HostLifecycle::ForcedIncomplete => ViewerNativeState::ForcedIncomplete,
                    _ => ViewerNativeState::Unconfirmed,
                };
                Ok((runtime.host().host().last_close_progress().cloned(), native))
            } else {
                unrun(
                    outcome
                        .controller
                        .pending_owner_mut()
                        .ok_or(ViewerCleanupError::MissingOwner)?,
                    ViewerNativeState::Unconfirmed,
                )
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CloseAction {
    Request,
    Observe,
    Retain,
    ExplicitRecovery,
}
fn close_action(lifecycle: HostLifecycle) -> CloseAction {
    match lifecycle {
        HostLifecycle::Open => CloseAction::Request,
        HostLifecycle::Closing => CloseAction::Observe,
        HostLifecycle::HostCleaned | HostLifecycle::Closed => CloseAction::Retain,
        HostLifecycle::CleanupFailed | HostLifecycle::ForcedIncomplete => {
            CloseAction::ExplicitRecovery
        }
    }
}
impl<C: NativeChild> CleanupAdapter for TypedViewerCleanup<C> {
    type Owners = TypedViewerOwners<C>;
    type Evidence = ViewerCleanupEvidence;
    type Status = ViewerCleanupStatus;
    type Error = ApiError;
    fn poll(&mut self) -> CleanupPoll<ViewerCleanupStatus> {
        if self.evidence.observations >= self.limits.observations
            || self.started.elapsed() >= self.limits.duration
        {
            return CleanupPoll::Terminal(ViewerCleanupStatus::BudgetExhausted);
        }
        self.evidence.observations += 1;
        match observe_owners(&mut self.owners) {
            Ok((progress, native)) => {
                self.evidence.last_native = Some(native);
                self.evidence.last_workers = progress.clone();
                project_progress(progress, native, self.limits.delay)
            }
            Err(error) => {
                self.evidence.error = Some(error);
                CleanupPoll::Terminal(ViewerCleanupStatus::Faulted)
            }
        }
    }
    fn accept_terminal_for_shutdown(&mut self) -> Result<(), ApiError> {
        Err(ApiError::new(
            ErrorCode::NeedsInput,
            "Viewer terminal acknowledgement requires an explicit consumer decision",
        ))
    }
    fn acceptance_failure_status(&self) -> ViewerCleanupStatus {
        ViewerCleanupStatus::NeedsTerminalAck
    }
    fn into_parts(self) -> (TypedViewerOwners<C>, ViewerCleanupEvidence) {
        (self.owners, self.evidence)
    }
}

fn project_progress(
    progress: Option<ViewerCloseProgress>,
    native: ViewerNativeState,
    delay: Duration,
) -> CleanupPoll<ViewerCleanupStatus> {
    match progress {
        Some(ViewerCloseProgress::CallbackPending { tickets }) if !tickets.is_empty() => {
            CleanupPoll::Terminal(ViewerCleanupStatus::NeedsTerminalAck)
        }
        Some(ViewerCloseProgress::Complete { .. }) => {
            CleanupPoll::Terminal(ViewerCleanupStatus::WorkersQuiescent(native))
        }
        Some(ViewerCloseProgress::ExecutionPending(
            ExecutionCloseProgress::TerminalAckPending { .. },
        )) => CleanupPoll::Terminal(ViewerCleanupStatus::NeedsTerminalAck),
        _ => CleanupPoll::Pending(HostRetry::After(delay)),
    }
}

#[cfg(test)]
mod tests;

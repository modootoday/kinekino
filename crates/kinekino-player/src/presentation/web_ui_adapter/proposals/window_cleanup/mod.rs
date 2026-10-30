//! Bounded observations over returned UI owners; never a replacement event loop or SDK executor.
use super::{
    document_factory::ParentCloseProgress,
    initial_panel::AcceptedShutdownTerminal,
    product_window::ProductWindowBuildFailure,
    window_startup::{OwnedSdkCleanupError, ProposalWindowStartup},
};
use kinekino_player::application::{
    api::{ApiError, ErrorCode},
    sdk_execution::{ExecutionCloseProgress, ExecutionCloseReceipt},
};
use meari_native::infrastructure::{
    native_host::{CloseCause, HostAccessError, HostCloseStatus, HostLifecycle, HostRetry},
    window::NativeRunOutcome,
};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

pub enum ProductWindowOwners {
    BuildFailure(Box<ProductWindowBuildFailure>),
    Headless(Box<ProposalWindowStartup>),
    Returned(Box<NativeRunOutcome<ProposalWindowStartup>>),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeCleanupState {
    NoDocument,
    HeadlessClosed,
    Closed,
    ForcedIncomplete,
    Unconfirmed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CleanupStatus {
    Pending(HostRetry),
    NeedsTerminalAcceptance,
    SdkQuiescent { native: NativeCleanupState },
    Faulted,
    BudgetExhausted,
}
pub struct CleanupLimits {
    duration: Duration,
    observations: usize,
    worker_exit_delay: Duration,
}
impl CleanupLimits {
    pub fn new(
        duration: Duration,
        observations: usize,
        worker_exit_delay: Duration,
    ) -> Result<Self, ApiError> {
        if duration.is_zero()
            || observations == 0
            || worker_exit_delay.is_zero()
            || worker_exit_delay > duration
        {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Positive bounded cleanup limits are required",
            ));
        }
        Ok(Self {
            duration,
            observations,
            worker_exit_delay,
        })
    }
}
#[derive(Default)]
pub struct CleanupEvidence {
    pub observations: usize,
    pub last_sdk: Option<ExecutionCloseProgress>,
    pub last_error: Option<OwnedSdkCleanupError>,
    pub accepted_initial: Option<Arc<AcceptedShutdownTerminal>>,
    pub status: Option<CleanupStatus>,
}
pub struct OwnedProductCleanup {
    owners: ProductWindowOwners,
    limits: CleanupLimits,
    started: Instant,
    pub evidence: CleanupEvidence,
}
impl OwnedProductCleanup {
    pub fn new(owners: ProductWindowOwners, limits: CleanupLimits) -> Self {
        Self {
            owners,
            limits,
            started: Instant::now(),
            evidence: CleanupEvidence::default(),
        }
    }
    pub fn owners(&self) -> &ProductWindowOwners {
        &self.owners
    }
    /// Transfer both authority and diagnostics, including budget/fault evidence, for explicit recovery.
    pub fn into_parts(self) -> (ProductWindowOwners, CleanupEvidence) {
        (self.owners, self.evidence)
    }
    /// Explicit typed acceptance. An invalid UI selection is quarantined, never mounted via this path.
    pub fn accept_initial_for_shutdown(
        &mut self,
    ) -> Result<Arc<AcceptedShutdownTerminal>, ApiError> {
        let accepted = match &mut self.owners {
            ProductWindowOwners::Headless(controller) => {
                controller.acknowledge_initial_for_shutdown()
            }
            ProductWindowOwners::Returned(outcome) => {
                outcome.controller.acknowledge_initial_for_shutdown()
            }
            ProductWindowOwners::BuildFailure(failure) => match failure.as_mut() {
                ProductWindowBuildFailure::Configuration(input) => {
                    input.initial.acknowledge_for_shutdown()
                }
                ProductWindowBuildFailure::Input(_) => Err(ApiError::new(
                    ErrorCode::NeedsInput,
                    "No initial inspection port exists; retained SDK owner requires explicit diagnostics",
                )),
            },
        }?;
        self.evidence.accepted_initial = Some(accepted.clone());
        Ok(accepted)
    }
    pub fn poll(&mut self) -> CleanupStatus {
        if let Some(status @ CleanupStatus::SdkQuiescent { .. }) = self.evidence.status {
            return status;
        }
        if self.evidence.observations >= self.limits.observations
            || self.started.elapsed() >= self.limits.duration
        {
            self.evidence.status = Some(CleanupStatus::BudgetExhausted);
            return CleanupStatus::BudgetExhausted;
        }
        self.evidence.observations += 1;
        let result = match &mut self.owners {
            ProductWindowOwners::BuildFailure(failure) => match failure.as_mut() {
                ProductWindowBuildFailure::Input(failure) => {
                    observe_owner(failure.owner.poll_close(), &mut self.evidence.last_sdk)
                        .map(|progress| (progress, NativeCleanupState::NoDocument))
                }
                ProductWindowBuildFailure::Configuration(input) => {
                    observe_owner(input.initial.poll_close(), &mut self.evidence.last_sdk)
                        .map(|progress| (progress, NativeCleanupState::NoDocument))
                }
            },
            ProductWindowOwners::Headless(controller) => {
                observe_controller(controller, true, &mut self.evidence.last_sdk)
            }
            ProductWindowOwners::Returned(outcome) => {
                if outcome.runtime.is_some() && outcome.controller.runtime.is_none() {
                    self.evidence.last_error = Some(OwnedSdkCleanupError::MissingOwner);
                    self.evidence.status = Some(CleanupStatus::Faulted);
                    return CleanupStatus::Faulted;
                }
                if let (Some(returned), Some(controller)) =
                    (&outcome.runtime, &outcome.controller.runtime)
                    && !std::rc::Rc::ptr_eq(returned, controller)
                {
                    self.evidence.last_error =
                        Some(OwnedSdkCleanupError::Host(HostAccessError::Stale));
                    self.evidence.status = Some(CleanupStatus::Faulted);
                    return CleanupStatus::Faulted;
                }
                observe_controller(&mut outcome.controller, false, &mut self.evidence.last_sdk)
            }
        };
        let status = match result {
            Ok((ParentCloseProgress::Complete, native)) => CleanupStatus::SdkQuiescent { native },
            Ok((ParentCloseProgress::TerminalAckPending, _)) => {
                CleanupStatus::NeedsTerminalAcceptance
            }
            Ok((ParentCloseProgress::WorkerExitPending, _)) => {
                CleanupStatus::Pending(HostRetry::After(self.limits.worker_exit_delay))
            }
            Err(error) => {
                self.evidence.last_error = Some(error);
                CleanupStatus::Faulted
            }
        };
        self.evidence.status = Some(status);
        status
    }
}
fn observe_owner(
    progress: Result<ExecutionCloseProgress, ApiError>,
    retained: &mut Option<ExecutionCloseProgress>,
) -> Result<ParentCloseProgress, OwnedSdkCleanupError> {
    let progress = progress.map_err(|error| OwnedSdkCleanupError::Sdk(Arc::new(error)))?;
    *retained = Some(progress.clone());
    match progress {
        ExecutionCloseProgress::Closed(ExecutionCloseReceipt {
            worker_stopped: true,
            unacknowledged: 0,
        }) => Ok(ParentCloseProgress::Complete),
        ExecutionCloseProgress::Closed(_) => Err(OwnedSdkCleanupError::Host(
            HostAccessError::Invalid("SDK did not return a quiescent join receipt".into()),
        )),
        ExecutionCloseProgress::WorkerExitPending { .. } => {
            Ok(ParentCloseProgress::WorkerExitPending)
        }
        ExecutionCloseProgress::TerminalAckPending { .. } => {
            Ok(ParentCloseProgress::TerminalAckPending)
        }
        ExecutionCloseProgress::Faulted { error, .. } => Err(OwnedSdkCleanupError::Sdk(error)),
    }
}
fn observe_controller(
    controller: &mut ProposalWindowStartup,
    headless: bool,
    retained: &mut Option<ExecutionCloseProgress>,
) -> Result<(ParentCloseProgress, NativeCleanupState), OwnedSdkCleanupError> {
    controller.validate_cleanup_ownership()?;
    if let Some(runtime) = controller.runtime.clone() {
        let mut runtime = runtime
            .try_borrow_mut()
            .map_err(|_| OwnedSdkCleanupError::Host(HostAccessError::Busy))?;
        let lifecycle = runtime.document().lifecycle();
        if headless
            && !matches!(
                lifecycle,
                HostLifecycle::Closed | HostLifecycle::ForcedIncomplete
            )
        {
            let status = runtime
                .request_close(CloseCause::Host)
                .map_err(OwnedSdkCleanupError::Host)?;
            *retained = runtime.host().parent_close_progress().cloned();
            return match status {
                HostCloseStatus::HostCleanupComplete => {
                    runtime
                        .finish_headless_close()
                        .map_err(OwnedSdkCleanupError::Host)?;
                    Ok((
                        ParentCloseProgress::Complete,
                        NativeCleanupState::HeadlessClosed,
                    ))
                }
                HostCloseStatus::Pending => Ok((
                    match retained {
                        Some(ExecutionCloseProgress::WorkerExitPending { .. }) => {
                            ParentCloseProgress::WorkerExitPending
                        }
                        _ => ParentCloseProgress::TerminalAckPending,
                    },
                    NativeCleanupState::Unconfirmed,
                )),
                _ => Err(OwnedSdkCleanupError::Host(HostAccessError::Invalid(
                    "Headless host cleanup did not complete; original host diagnostics retained"
                        .into(),
                ))),
            };
        }
        let result = runtime.host_mut().poll_owned_cleanup();
        *retained = runtime.host().parent_close_progress().cloned();
        let native = match lifecycle {
            HostLifecycle::Closed if headless => NativeCleanupState::HeadlessClosed,
            HostLifecycle::Closed => NativeCleanupState::Closed,
            HostLifecycle::ForcedIncomplete => NativeCleanupState::ForcedIncomplete,
            _ => NativeCleanupState::Unconfirmed,
        };
        return result
            .map(|progress| (progress, native))
            .map_err(OwnedSdkCleanupError::Bridge);
    }
    let result = controller.poll_owned_sdk_cleanup();
    *retained = controller.close_progress.clone();
    result.map(|progress| (progress, NativeCleanupState::NoDocument))
}

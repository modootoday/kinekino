//! Product startup over the strict Native loop; SDK ownership is transferred only once.
use super::{
    IntentPins,
    bootstrap::BootstrapLabels,
    correction_panel::CorrectionLabels,
    initial_panel::{
        InitialExecutionParent, InitialPanelHandoff, InitialPanelMount, InitialPanelMountFailure,
        InitialPanelStartup, InitialPreparationState, InitialPrepareError, PreparedInitialPanel,
    },
    window_bridge::{
        BridgeAssemblyFailure, CorrectionLabelProvider, PanelClosePolicy, PanelCommitPort,
        PanelLabelProvider, ProposalWindowBridge,
    },
};
use kinekino_player::application::{api::ApiError, sdk_execution::ExecutionCloseProgress};
use meari_native::{
    application::WindowOptions,
    infrastructure::{
        native_host::{HostRetry, HostScheduleLimits, NativeHostRuntime},
        window::{
            NativeStartupAttachFailure, NativeStartupContext, NativeStartupController,
            NativeStartupPoll, NativeWindowObserver, NativeWindowWake,
        },
    },
};
use std::{cell::RefCell, fmt, rc::Rc, sync::Arc, time::Duration};

#[derive(Debug)]
pub enum OwnedSdkCleanupError {
    Sdk(Arc<ApiError>),
    Plugin(meari::application::plugins::PluginError),
    Host(meari_native::infrastructure::native_host::HostAccessError),
    Bridge(super::window_bridge::PanelBridgeError),
    MissingOwner,
}

pub type ProposalRuntime = NativeHostRuntime<ProposalWindowBridge<InitialExecutionParent>>;
pub type PanelMount =
    Box<dyn FnOnce(InitialPanelHandoff) -> Result<InitialPanelMount, InitialPanelMountFailure>>;
pub struct WindowStartupInput {
    pub initial: InitialPanelStartup,
    pub initial_labels: Box<dyn Fn() -> BootstrapLabels>,
    pub correction_labels: Box<dyn Fn() -> Option<CorrectionLabels>>,
    pub mount: PanelMount,
    pub labels: PanelLabelProvider,
    pub live_correction_labels: Option<CorrectionLabelProvider>,
    pub close_policy: PanelClosePolicy,
    pub pins: IntentPins,
    pub options: WindowOptions,
    pub observer: NativeWindowObserver,
    pub limits: HostScheduleLimits,
    pub worker_exit_delay: Duration,
    pub commit: Option<PanelCommitPort>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowStartupError {
    InvalidRetryDelay,
    Preparation,
    InitialFailure,
    Acknowledgement,
    Mount,
    Bridge,
    Attach,
    Close,
    AlreadyTransferred,
}
impl fmt::Display for WindowStartupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "proposal window startup: {self:?}")
    }
}
impl std::error::Error for WindowStartupError {}

/// Failure fields retain the actual owner and canonical evidence for explicit recovery.
/// An attached hook is not Ready or frame-submission evidence; use the injected observer.
pub struct ProposalWindowStartup {
    initial: Option<InitialPanelStartup>,
    handoff: Option<InitialPanelHandoff>,
    input: WindowStartupConfiguration,
    mount: Option<PanelMount>,
    labels: Option<PanelLabelProvider>,
    correction_labels: Option<CorrectionLabelProvider>,
    close_policy: Option<PanelClosePolicy>,
    commit: Option<PanelCommitPort>,
    observer: Option<NativeWindowObserver>,
    pub preparation_error: Option<InitialPrepareError>,
    pub initial_failure:
        Option<Arc<Result<kinekino_player::domain::proposals::StoredProposal, ApiError>>>,
    pub sdk_error: Option<Arc<ApiError>>,
    pub close_progress: Option<ExecutionCloseProgress>,
    pub mount_failure: Option<Box<InitialPanelMountFailure>>,
    pub bridge_failure: Option<Box<BridgeAssemblyFailure<InitialExecutionParent>>>,
    pub attach_failure:
        Option<Box<NativeStartupAttachFailure<ProposalWindowBridge<InitialExecutionParent>>>>,
    pub runtime: Option<Rc<RefCell<ProposalRuntime>>>,
    pub wake: Option<NativeWindowWake>,
    pub prepared: Option<Arc<PreparedInitialPanel>>,
    closing: bool,
    attachment_attempted: bool,
    pub closed: bool,
}
struct WindowStartupConfiguration {
    initial_labels: Box<dyn Fn() -> BootstrapLabels>,
    correction_labels: Box<dyn Fn() -> Option<CorrectionLabels>>,
    pins: Option<IntentPins>,
    options: WindowOptions,
    limits: HostScheduleLimits,
    worker_exit_delay: Duration,
}
impl ProposalWindowStartup {
    pub fn validate_cleanup_ownership(&self) -> Result<(), OwnedSdkCleanupError> {
        let owners = usize::from(self.initial.is_some())
            + usize::from(self.handoff.is_some())
            + usize::from(self.mount_failure.is_some())
            + usize::from(self.bridge_failure.is_some())
            + usize::from(self.runtime.is_some());
        if owners == 1 {
            Ok(())
        } else {
            Err(OwnedSdkCleanupError::MissingOwner)
        }
    }
    pub fn acknowledge_initial_for_shutdown(
        &mut self,
    ) -> Result<Arc<super::initial_panel::AcceptedShutdownTerminal>, ApiError> {
        self.closing = true;
        self.initial
            .as_mut()
            .ok_or_else(|| {
                ApiError::new(
                    kinekino_player::application::api::ErrorCode::InvalidArgument,
                    "Initial owner has already transferred",
                )
            })?
            .acknowledge_for_shutdown()
    }
    /// Exactly one current ownership branch is observed. No terminal is accepted here.
    pub fn poll_owned_sdk_cleanup(
        &mut self,
    ) -> Result<super::document_factory::ParentCloseProgress, OwnedSdkCleanupError> {
        self.validate_cleanup_ownership()?;
        self.closing = true;
        if let Some(initial) = &mut self.initial {
            let progress = initial
                .poll_close()
                .map_err(|error| OwnedSdkCleanupError::Sdk(Arc::new(error)))?;
            self.close_progress = Some(progress.clone());
            return sdk_cleanup_projection(progress);
        }
        if let Some(handoff) = &self.handoff {
            let progress = handoff
                .poll_shutdown()
                .map_err(|error| OwnedSdkCleanupError::Sdk(Arc::new(error)))?;
            self.close_progress = Some(progress.clone());
            return sdk_cleanup_projection(progress);
        }
        if let Some(failure) = &mut self.mount_failure {
            let result = failure.poll_cleanup();
            self.close_progress = failure
                .close_progress
                .clone()
                .or_else(|| failure.factory.parent_close_progress().cloned());
            return result.map_err(OwnedSdkCleanupError::Plugin);
        }
        if let Some(failure) = &mut self.bridge_failure {
            let result = failure.poll_cleanup();
            self.close_progress = failure.parent_close_progress().cloned();
            return result.map_err(OwnedSdkCleanupError::Plugin);
        }
        if let Some(runtime) = &self.runtime {
            let mut runtime = runtime.try_borrow_mut().map_err(|_| {
                OwnedSdkCleanupError::Host(
                    meari_native::infrastructure::native_host::HostAccessError::Busy,
                )
            })?;
            let result = runtime.host_mut().poll_owned_cleanup();
            self.close_progress = runtime.host().parent_close_progress().cloned();
            return result.map_err(OwnedSdkCleanupError::Bridge);
        }
        Err(OwnedSdkCleanupError::MissingOwner)
    }
    pub fn new(input: WindowStartupInput) -> Result<Self, Box<WindowStartupInput>> {
        if input.worker_exit_delay < input.limits.min_delay()
            || input.worker_exit_delay > input.limits.max_delay()
        {
            return Err(Box::new(input));
        }
        Ok(Self {
            initial: Some(input.initial),
            handoff: None,
            input: WindowStartupConfiguration {
                initial_labels: input.initial_labels,
                correction_labels: input.correction_labels,
                pins: Some(input.pins),
                options: input.options,
                limits: input.limits,
                worker_exit_delay: input.worker_exit_delay,
            },
            mount: Some(input.mount),
            labels: Some(input.labels),
            correction_labels: input.live_correction_labels,
            close_policy: Some(input.close_policy),
            commit: input.commit,
            observer: Some(input.observer),
            preparation_error: None,
            initial_failure: None,
            sdk_error: None,
            close_progress: None,
            mount_failure: None,
            bridge_failure: None,
            attach_failure: None,
            runtime: None,
            wake: None,
            prepared: None,
            closing: false,
            attachment_attempted: false,
            closed: false,
        })
    }
    /// SDK-IO uses the existing worker; document composition may be exercised headlessly before attach.
    pub fn prepare_document(
        &mut self,
    ) -> Result<Option<Rc<RefCell<ProposalRuntime>>>, WindowStartupError> {
        if let Some(runtime) = &self.runtime {
            return Ok(Some(runtime.clone()));
        }
        if self.closing {
            return Err(WindowStartupError::AlreadyTransferred);
        }
        if self.mount_failure.is_some() {
            return Ok(None);
        }
        if self.handoff.is_none() {
            let initial = self
                .initial
                .as_mut()
                .ok_or(WindowStartupError::AlreadyTransferred)?;
            match initial.prepare(
                (self.input.initial_labels)(),
                (self.input.correction_labels)(),
            ) {
                Ok(InitialPreparationState::Loading) => {
                    return Ok(None);
                }
                Ok(InitialPreparationState::Failed(snapshot)) => {
                    self.initial_failure = Some(snapshot);
                    return Ok(None);
                }
                Ok(InitialPreparationState::Prepared) => {}
                Err(error) => {
                    self.preparation_error = Some(error);
                    return Err(WindowStartupError::Preparation);
                }
            }
            if let Err(error) = initial.acknowledge_prepared() {
                self.sdk_error = Some(Arc::new(error));
                return Err(WindowStartupError::Acknowledgement);
            }
            let handoff = match self.initial.take().expect("checked initial").into_handoff() {
                Ok(handoff) => handoff,
                Err(initial) => {
                    self.initial = Some(initial);
                    return Err(WindowStartupError::Acknowledgement);
                }
            };
            self.handoff = Some(handoff);
        }
        let mount = self
            .mount
            .take()
            .ok_or(WindowStartupError::AlreadyTransferred)?;
        let mounted = match mount(self.handoff.take().expect("accepted preparation")) {
            Ok(mounted) => mounted,
            Err(failure) => {
                self.mount_failure = Some(Box::new(failure));
                return Ok(None);
            }
        };
        self.prepared = Some(mounted.prepared);
        let (document, mut bridge) = match ProposalWindowBridge::from_factory(
            mounted.panel,
            self.labels.take().expect("untransferred labels"),
            self.close_policy
                .take()
                .expect("untransferred close policy"),
            self.input.pins.take().expect("untransferred pins"),
        ) {
            Ok(parts) => parts,
            Err(failure) => {
                self.bridge_failure = Some(failure);
                return Err(WindowStartupError::Bridge);
            }
        };
        if let Some(labels) = self.correction_labels.take() {
            bridge = bridge.with_correction_labels(labels);
        }
        if let Some(commit) = self.commit.take() {
            bridge = bridge.with_commit_port(commit);
        }
        let bridge = bridge
            .with_worker_exit_retry(self.input.worker_exit_delay, self.input.limits)
            .unwrap_or_else(|_| unreachable!("constructor validated retry delay"));
        let runtime = Rc::new(RefCell::new(NativeHostRuntime::with_schedule_limits(
            document,
            bridge,
            self.input.limits,
        )));
        self.runtime = Some(runtime.clone());
        Ok(Some(runtime))
    }
    pub fn initial_mut(&mut self) -> Option<&mut InitialPanelStartup> {
        self.initial.as_mut()
    }
    /// Explicit retry only: unbind the failed actual scope and retain the same worker/preparation.
    pub fn recover_mount_for_retry(&mut self, mount: PanelMount) -> Result<(), PanelMount> {
        let Some(failure) = self.mount_failure.take() else {
            return Err(mount);
        };
        match (*failure).recover_for_retry() {
            Ok(handoff) => {
                self.handoff = Some(handoff);
                self.mount = Some(mount);
                Ok(())
            }
            Err(failure) => {
                self.mount_failure = Some(Box::new(failure));
                Err(mount)
            }
        }
    }
    /// Cancellation does not accept or acknowledge any retained terminal.
    pub fn request_initial_close(&mut self) -> Result<(), ApiError> {
        if let Some(initial) = &self.initial {
            initial.cancel()?;
        }
        self.closing = true;
        Ok(())
    }
    fn observe_initial_close(
        &mut self,
    ) -> Result<NativeStartupPoll<ProposalWindowBridge<InitialExecutionParent>>, WindowStartupError>
    {
        let initial = self
            .initial
            .as_mut()
            .ok_or(WindowStartupError::AlreadyTransferred)?;
        let progress = initial.poll_close();
        self.close_progress = progress.as_ref().ok().cloned();
        match progress {
            Ok(ExecutionCloseProgress::Closed(_)) => {
                self.closed = true;
                Ok(NativeStartupPoll::Stop)
            }
            Ok(ExecutionCloseProgress::WorkerExitPending { .. }) => Ok(NativeStartupPoll::Pending(
                HostRetry::After(self.input.worker_exit_delay),
            )),
            Ok(ExecutionCloseProgress::TerminalAckPending { .. }) => {
                Ok(NativeStartupPoll::Pending(HostRetry::OnWake))
            }
            Ok(ExecutionCloseProgress::Faulted { error, .. }) => {
                self.sdk_error = Some(error);
                Err(WindowStartupError::Close)
            }
            Err(error) => {
                self.sdk_error = Some(Arc::new(error));
                Err(WindowStartupError::Close)
            }
        }
    }
}
fn sdk_cleanup_projection(
    progress: ExecutionCloseProgress,
) -> Result<super::document_factory::ParentCloseProgress, OwnedSdkCleanupError> {
    use super::document_factory::ParentCloseProgress;
    match progress {
        ExecutionCloseProgress::Closed(_) => Ok(ParentCloseProgress::Complete),
        ExecutionCloseProgress::TerminalAckPending { .. } => {
            Ok(ParentCloseProgress::TerminalAckPending)
        }
        ExecutionCloseProgress::WorkerExitPending { .. } => {
            Ok(ParentCloseProgress::WorkerExitPending)
        }
        ExecutionCloseProgress::Faulted { error, .. } => Err(OwnedSdkCleanupError::Sdk(error)),
    }
}
impl NativeStartupController for ProposalWindowStartup {
    type Host = ProposalWindowBridge<InitialExecutionParent>;
    type Error = WindowStartupError;
    fn observe(
        &mut self,
        context: &mut NativeStartupContext<'_>,
    ) -> Result<NativeStartupPoll<Self::Host>, Self::Error> {
        if self.closing {
            return self.observe_initial_close();
        }
        let Some(runtime) = self.prepare_document()? else {
            if self.initial_failure.is_some() {
                return Err(WindowStartupError::InitialFailure);
            }
            return Ok(NativeStartupPoll::Pending(HostRetry::OnWake));
        };
        if self.attachment_attempted {
            return Err(WindowStartupError::AlreadyTransferred);
        }
        self.attachment_attempted = true;
        match context.attach_observed(
            runtime,
            self.input.options.clone(),
            self.observer.take().expect("untransferred observer"),
        ) {
            Ok(attachment) => Ok(NativeStartupPoll::Attach(attachment)),
            Err(failure) => {
                self.attach_failure = Some(failure);
                Err(WindowStartupError::Attach)
            }
        }
    }
    fn attached(
        &mut self,
        runtime: Rc<RefCell<ProposalRuntime>>,
        wake: NativeWindowWake,
    ) -> Result<(), Self::Error> {
        self.runtime = Some(runtime);
        self.wake = Some(wake);
        Ok(())
    }
}

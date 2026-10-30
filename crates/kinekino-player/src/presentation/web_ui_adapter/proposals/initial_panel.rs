//! Trusted startup owns initial loading and immutable selection before a document scope exists.
#[path = "initial_panel/ports.rs"]
mod ports;
use super::bootstrap::{
    BootstrapError, BootstrapLabels, BootstrapSelection, BootstrapTarget, project_initial,
};
use super::correction_panel::{
    CorrectionLabels, initial_correction_props, initial_correction_props_with_selection,
};
use super::document_factory::{
    FactoryCause, MountedProposalPanel, PanelFactoryFailure, PanelFactoryInput, PanelSelectionMode,
    ParentCloseProgress, ParentProposalLease, mount_inspected_panel,
};
use super::panel_model::PanelPreparationError;
use kinekino_player::{
    application::{
        api::{ApiError, RequestContext},
        proposals::InspectProposal,
        sdk_execution::{
            ExecutionCloseProgress, ExecutionHandle, ExecutionOwner, ExecutionTerminal,
            InitialProposalInspection,
        },
    },
    assembly::meari::async_proposals::AsyncProposalRegistration,
    domain::proposals::StoredProposal,
};
use kinekino_ui_plugins::infrastructure::typed_async_sdk_plugin::AsyncSdkPump;
use meari::{application::plugins::*, domain::Value};
pub use ports::InitialInspectionPort;
use std::{cell::RefCell, collections::BTreeMap, rc::Rc, sync::Arc};

pub struct InitialStartFailure {
    pub error: ApiError,
    pub owner: ExecutionOwner,
}
pub enum InitialPrepareError {
    Sdk(ApiError),
    Selection(BootstrapError),
    Panel(PanelPreparationError),
    MissingCorrectionLabels,
}
pub enum InitialPreparationState {
    Loading,
    Prepared,
    Failed(Arc<Result<StoredProposal, ApiError>>),
}
/// Private fields prevent mutation after initial ACK. Raw execution evidence remains retained.
pub struct PreparedInitialPanel {
    snapshot: Arc<Result<StoredProposal, ApiError>>,
    selection: Arc<BootstrapSelection>,
    raw: Arc<ExecutionTerminal>,
    props: BTreeMap<String, Value>,
}
pub struct AcceptedInitialFailure {
    snapshot: Arc<Result<StoredProposal, ApiError>>,
    raw: Arc<ExecutionTerminal>,
}
pub struct AcceptedShutdownTerminal {
    snapshot: Arc<Result<StoredProposal, ApiError>>,
    raw: Arc<ExecutionTerminal>,
}
impl AcceptedShutdownTerminal {
    pub fn snapshot(&self) -> &Arc<Result<StoredProposal, ApiError>> {
        &self.snapshot
    }
    pub fn raw_terminal(&self) -> &Arc<ExecutionTerminal> {
        &self.raw
    }
}
impl AcceptedInitialFailure {
    pub fn snapshot(&self) -> &Arc<Result<StoredProposal, ApiError>> {
        &self.snapshot
    }
    pub fn raw_terminal(&self) -> &Arc<ExecutionTerminal> {
        &self.raw
    }
}
impl PreparedInitialPanel {
    pub fn snapshot(&self) -> &Arc<Result<StoredProposal, ApiError>> {
        &self.snapshot
    }
    pub fn selection(&self) -> &BootstrapSelection {
        &self.selection
    }
    pub fn raw_terminal(&self) -> &Arc<ExecutionTerminal> {
        &self.raw
    }
    pub fn props(&self) -> &BTreeMap<String, Value> {
        &self.props
    }
}
impl InitialInspectionPort for InitialProposalInspection {
    fn peek_terminal(&mut self) -> Result<Option<Arc<Result<StoredProposal, ApiError>>>, ApiError> {
        InitialProposalInspection::peek_terminal(self)
    }
    fn retained_terminal(&self) -> Result<Option<Arc<ExecutionTerminal>>, ApiError> {
        InitialProposalInspection::retained_terminal(self)
    }
    fn acknowledge(&mut self) -> Result<(), ApiError> {
        InitialProposalInspection::acknowledge(self)?;
        Ok(())
    }
    fn cancel(&self) -> Result<(), ApiError> {
        InitialProposalInspection::cancel(self)?;
        Ok(())
    }
}
pub struct InitialPanelStartup<I = InitialProposalInspection> {
    initial: I,
    owner: ExecutionOwner,
    selection: Arc<BootstrapSelection>,
    prepared: Option<Arc<PreparedInitialPanel>>,
    accepted_failure: Option<Arc<AcceptedInitialFailure>>,
    accepted_shutdown: Option<Arc<AcceptedShutdownTerminal>>,
    acknowledged: bool,
    closing: bool,
}
impl<I: InitialInspectionPort> InitialPanelStartup<I> {
    /// Explicit reselection before accepting any preparation; the inspected proposal stays fixed.
    pub fn select_before_preparation(
        &mut self,
        selection: BootstrapSelection,
    ) -> Result<(), BootstrapError> {
        if self.prepared.is_some()
            || self.acknowledged
            || selection.proposal_id != self.selection.proposal_id
        {
            return Err(BootstrapError::StaleSelection);
        }
        self.selection = Arc::new(selection);
        Ok(())
    }
}
impl InitialPanelStartup {
    pub fn start(
        owner: ExecutionOwner,
        context: RequestContext,
        selection: BootstrapSelection,
        max_typed_wire_bytes: usize,
    ) -> Result<Self, InitialStartFailure> {
        let initial = match InitialProposalInspection::start(
            owner.handle(),
            context,
            InspectProposal {
                proposal_id: selection.proposal_id.clone(),
            },
            max_typed_wire_bytes,
        ) {
            Ok(initial) => initial,
            Err(error) => return Err(InitialStartFailure { error, owner }),
        };
        Ok(Self {
            initial,
            owner,
            selection: Arc::new(selection),
            prepared: None,
            accepted_failure: None,
            accepted_shutdown: None,
            acknowledged: false,
            closing: false,
        })
    }
}
impl<I: InitialInspectionPort> InitialPanelStartup<I> {
    pub fn prepare(
        &mut self,
        labels: BootstrapLabels,
        correction: Option<CorrectionLabels>,
    ) -> Result<InitialPreparationState, InitialPrepareError> {
        if self.prepared.is_some() {
            return Ok(InitialPreparationState::Prepared);
        }
        let Some(snapshot) = self
            .initial
            .peek_terminal()
            .map_err(InitialPrepareError::Sdk)?
        else {
            return Ok(InitialPreparationState::Loading);
        };
        let stored = match snapshot.as_ref() {
            Ok(stored) => stored,
            Err(_) => return Ok(InitialPreparationState::Failed(snapshot)),
        };
        let projected = project_initial(stored, &self.selection, labels)
            .map_err(InitialPrepareError::Selection)?;
        let props = match &self.selection.target {
            BootstrapTarget::CueCorrection { index, .. }
            | BootstrapTarget::DocumentCorrection { index, .. } => initial_correction_props(
                stored,
                Some(*index),
                correction.ok_or(InitialPrepareError::MissingCorrectionLabels)?,
            )
            .map_err(InitialPrepareError::Panel)?,
            BootstrapTarget::DocumentRepair { source_ref } => {
                initial_correction_props_with_selection(
                    stored,
                    None,
                    source_ref.as_ref(),
                    correction.ok_or(InitialPrepareError::MissingCorrectionLabels)?,
                )
                .map_err(InitialPrepareError::Panel)?
            }
            _ => projected.props,
        };
        let raw = self
            .initial
            .retained_terminal()
            .map_err(InitialPrepareError::Sdk)?
            .ok_or_else(|| {
                InitialPrepareError::Sdk(ApiError::new(
                    kinekino_player::application::api::ErrorCode::Internal,
                    "Initial terminal ownership disappeared",
                ))
            })?;
        self.prepared = Some(Arc::new(PreparedInitialPanel {
            snapshot,
            selection: self.selection.clone(),
            raw,
            props,
        }));
        Ok(InitialPreparationState::Prepared)
    }
    pub fn prepared(&self) -> Option<&Arc<PreparedInitialPanel>> {
        self.prepared.as_ref()
    }
    pub fn accepted_failure(&self) -> Option<&Arc<AcceptedInitialFailure>> {
        self.accepted_failure.as_ref()
    }
    /// Initial data acceptance only. ACK failure leaves the same startup, data and worker owned.
    pub fn acknowledge_prepared(&mut self) -> Result<(), ApiError> {
        if self.prepared.is_none() {
            return Err(ApiError::new(
                kinekino_player::application::api::ErrorCode::NeedsInput,
                "Explicit immutable preparation is required before initial ACK",
            ));
        }
        self.initial.acknowledge()?;
        self.acknowledged = true;
        Ok(())
    }
    /// Explicit acceptance of a canonical business error, including real queued cancellation.
    /// Invalid envelopes/oversize or successful-but-mismatched selections cannot enter here.
    pub fn acknowledge_failure(&mut self) -> Result<Arc<AcceptedInitialFailure>, ApiError> {
        if self.accepted_failure.is_none() {
            let snapshot = self.initial.peek_terminal()?.ok_or_else(|| {
                ApiError::new(
                    kinekino_player::application::api::ErrorCode::NeedsInput,
                    "Initial terminal is not ready",
                )
            })?;
            if snapshot.is_ok() {
                return Err(ApiError::new(
                    kinekino_player::application::api::ErrorCode::InvalidArgument,
                    "A successful initial result requires explicit selection preparation",
                ));
            }
            let raw = self.initial.retained_terminal()?.ok_or_else(|| {
                ApiError::new(
                    kinekino_player::application::api::ErrorCode::Internal,
                    "Initial failure ownership disappeared",
                )
            })?;
            self.accepted_failure = Some(Arc::new(AcceptedInitialFailure { snapshot, raw }));
        }
        self.initial.acknowledge()?;
        self.acknowledged = true;
        Ok(self
            .accepted_failure
            .as_ref()
            .expect("immutable failure retained before ACK")
            .clone())
    }
    pub fn into_handoff(self) -> Result<InitialPanelHandoff, Self> {
        if self.closing || !self.acknowledged || self.prepared.is_none() {
            return Err(self);
        }
        let Self {
            owner,
            prepared: Some(prepared),
            ..
        } = self
        else {
            unreachable!("checked immutable preparation")
        };
        Ok(InitialPanelHandoff { owner, prepared })
    }
    pub fn cancel(&self) -> Result<(), ApiError> {
        if self.acknowledged {
            return Ok(());
        }
        self.initial.cancel()?;
        Ok(())
    }
    /// Explicit quarantine/acceptance for shutdown only. It never authorizes mounting a bad selection.
    pub fn acknowledge_for_shutdown(&mut self) -> Result<Arc<AcceptedShutdownTerminal>, ApiError> {
        self.closing = true;
        if self.acknowledged
            && let Some(accepted) = &self.accepted_shutdown
        {
            return Ok(accepted.clone());
        }
        if self.accepted_shutdown.is_none() {
            let snapshot = self.initial.peek_terminal()?.ok_or_else(|| {
                ApiError::new(
                    kinekino_player::application::api::ErrorCode::NeedsInput,
                    "Initial terminal is not ready for explicit shutdown acceptance",
                )
            })?;
            if let Ok(stored) = snapshot.as_ref() {
                stored.validate().map_err(|_| {
                    ApiError::new(
                        kinekino_player::application::api::ErrorCode::InvalidArgument,
                        "Invalid stored proposal cannot be accepted for shutdown",
                    )
                })?;
            }
            let raw = self.initial.retained_terminal()?.ok_or_else(|| {
                ApiError::new(
                    kinekino_player::application::api::ErrorCode::Internal,
                    "Initial raw terminal ownership is missing",
                )
            })?;
            self.accepted_shutdown = Some(Arc::new(AcceptedShutdownTerminal { snapshot, raw }));
        }
        if !self.acknowledged {
            self.initial.acknowledge()?;
        }
        self.acknowledged = true;
        Ok(self
            .accepted_shutdown
            .as_ref()
            .expect("retained before ACK")
            .clone())
    }
    pub fn retained_terminal(&self) -> Result<Option<Arc<ExecutionTerminal>>, ApiError> {
        self.initial.retained_terminal()
    }
    /// Close does not ACK a result or discard late completion. Caller must retain and explicitly accept it.
    pub fn close(&mut self) -> Result<(), ApiError> {
        self.closing = true;
        self.owner.close()?;
        Ok(())
    }
    pub fn poll_close(&mut self) -> Result<ExecutionCloseProgress, ApiError> {
        self.closing = true;
        self.owner.poll_close()
    }
}
pub struct InitialPanelHandoff {
    owner: ExecutionOwner,
    prepared: Arc<PreparedInitialPanel>,
}
pub struct InitialExecutionParent {
    registration: AsyncProposalRegistration,
    execution_owner: ExecutionOwner,
    close_error: Option<Arc<ApiError>>,
    close_progress: Option<ExecutionCloseProgress>,
}
impl InitialExecutionParent {
    pub fn execution_owner(&self) -> &ExecutionOwner {
        &self.execution_owner
    }
    pub fn last_close_error(&self) -> Option<&Arc<ApiError>> {
        self.close_error.as_ref()
    }
    pub fn last_close_progress(&self) -> Option<&ExecutionCloseProgress> {
        self.close_progress.as_ref()
    }
}
impl ParentProposalLease for InitialExecutionParent {
    fn registration(&self) -> Result<&RegistrationHandle, PluginError> {
        self.registration.handle()
    }
    fn async_pump(&self) -> Option<AsyncSdkPump> {
        Some(self.registration.pump().clone())
    }
    fn close(&mut self) -> Result<(), PluginError> {
        self.registration.close()?;
        if let Err(error) = self.execution_owner.close() {
            self.close_error = Some(Arc::new(error));
            return Err(PluginError::new(
                PluginErrorCode::Busy,
                "SDK registration unbound; worker close/join requires retry; canonical error retained",
            ));
        }
        self.close_error = None;
        Ok(())
    }
    fn unbind_for_retry(&mut self) -> Result<(), PluginError> {
        self.registration.close()
    }
    fn poll_close(&mut self) -> Result<ParentCloseProgress, PluginError> {
        self.close_progress = None;
        self.registration.close()?;
        let progress = match self.execution_owner.poll_close() {
            Ok(progress) => progress,
            Err(error) => {
                self.close_error = Some(Arc::new(error));
                return Err(PluginError::new(
                    PluginErrorCode::Busy,
                    "SDK close observation failed; canonical error retained",
                ));
            }
        };
        self.close_progress = Some(progress.clone());
        match progress {
            ExecutionCloseProgress::Closed(_) => {
                self.close_error = None;
                Ok(ParentCloseProgress::Complete)
            }
            ExecutionCloseProgress::TerminalAckPending { .. } => {
                Ok(ParentCloseProgress::TerminalAckPending)
            }
            ExecutionCloseProgress::WorkerExitPending { .. } => {
                Ok(ParentCloseProgress::WorkerExitPending)
            }
            ExecutionCloseProgress::Faulted { error, .. } => {
                self.close_error = Some(error);
                Err(PluginError::new(
                    PluginErrorCode::Busy,
                    "SDK close fault requires explicit recovery; original diagnostic retained",
                ))
            }
        }
    }
    fn close_error(&self) -> Option<&ApiError> {
        self.close_error.as_deref()
    }
    fn close_progress(&self) -> Option<&ExecutionCloseProgress> {
        self.last_close_progress()
    }
}
pub struct InitialPanelMount {
    pub panel: MountedProposalPanel<InitialExecutionParent>,
    pub prepared: Arc<PreparedInitialPanel>,
}
pub struct InitialPanelMountFailure {
    pub factory: Box<PanelFactoryFailure<InitialExecutionParent>>,
    pub prepared: Arc<PreparedInitialPanel>,
    unregistered: Option<InitialPanelHandoff>,
    final_cleanup_started: bool,
    pub close_progress: Option<ExecutionCloseProgress>,
    pub close_error: Option<Arc<ApiError>>,
}
impl InitialPanelMountFailure {
    /// Quiescent unbind recovers the same worker; Busy preserves the original partial owner.
    pub fn recover_for_retry(mut self) -> Result<InitialPanelHandoff, Self> {
        if self.final_cleanup_started {
            return Err(self);
        }
        if let Some(startup) = self.unregistered.take() {
            if self.factory.retry_cleanup().is_err() {
                self.unregistered = Some(startup);
                return Err(self);
            }
            return Ok(startup);
        }
        if self.factory.retry_unbind().is_err() {
            return Err(self);
        }
        let Some(parent) = self.factory.take_unbound_parent() else {
            return Err(self);
        };
        Ok(InitialPanelHandoff {
            owner: parent.execution_owner,
            prepared: self.prepared,
        })
    }
    pub fn poll_cleanup(&mut self) -> Result<ParentCloseProgress, PluginError> {
        self.final_cleanup_started = true;
        let local = self.factory.poll_cleanup()?;
        if local != ParentCloseProgress::Complete {
            return Ok(local);
        }
        if let Some(handoff) = &self.unregistered {
            let progress = match handoff.owner.poll_close() {
                Ok(progress) => progress,
                Err(error) => {
                    self.close_error = Some(Arc::new(error));
                    return Err(PluginError::new(
                        PluginErrorCode::Adapter,
                        "SDK close observation error retained on mount failure",
                    ));
                }
            };
            self.close_progress = Some(progress.clone());
            return match progress {
                ExecutionCloseProgress::Closed(_) => Ok(ParentCloseProgress::Complete),
                ExecutionCloseProgress::TerminalAckPending { .. } => {
                    Ok(ParentCloseProgress::TerminalAckPending)
                }
                ExecutionCloseProgress::WorkerExitPending { .. } => {
                    Ok(ParentCloseProgress::WorkerExitPending)
                }
                ExecutionCloseProgress::Faulted { .. } => Err(PluginError::new(
                    PluginErrorCode::Adapter,
                    "Canonical SDK close fault retained on mount failure",
                )),
            };
        }
        Ok(local)
    }
}
impl InitialPanelHandoff {
    pub(super) fn poll_shutdown(&self) -> Result<ExecutionCloseProgress, ApiError> {
        self.owner.poll_close()
    }
    pub fn prepared(&self) -> &Arc<PreparedInitialPanel> {
        &self.prepared
    }
    pub fn mount(
        self,
        input: PanelFactoryInput,
        correction: Option<CorrectionLabels>,
        register: impl FnOnce(
            ExecutionHandle,
            Rc<RefCell<PluginRegistry>>,
            &PluginScope,
            Rc<RefCell<RequestContext>>,
        ) -> Result<AsyncProposalRegistration, PluginError>,
    ) -> Result<InitialPanelMount, InitialPanelMountFailure> {
        self.mount_composed(input, correction, None, register)
    }
    pub fn mount_in_frame(
        self,
        input: PanelFactoryInput,
        correction: Option<CorrectionLabels>,
        frame: super::frame_composition::FrameComposition,
        register: impl FnOnce(
            ExecutionHandle,
            Rc<RefCell<PluginRegistry>>,
            &PluginScope,
            Rc<RefCell<RequestContext>>,
        ) -> Result<AsyncProposalRegistration, PluginError>,
    ) -> Result<InitialPanelMount, InitialPanelMountFailure> {
        self.mount_composed(input, correction, Some(frame), register)
    }
    pub(super) fn mount_composed(
        self,
        mut input: PanelFactoryInput,
        correction: Option<CorrectionLabels>,
        frame: Option<super::frame_composition::FrameComposition>,
        register: impl FnOnce(
            ExecutionHandle,
            Rc<RefCell<PluginRegistry>>,
            &PluginScope,
            Rc<RefCell<RequestContext>>,
        ) -> Result<AsyncProposalRegistration, PluginError>,
    ) -> Result<InitialPanelMount, InitialPanelMountFailure> {
        let accepted = &self.prepared.selection;
        let index_matches = match &accepted.target {
            BootstrapTarget::Card { index }
            | BootstrapTarget::Highlight { index }
            | BootstrapTarget::CueCorrection { index, .. }
            | BootstrapTarget::DocumentCorrection { index, .. } => {
                input.selection.mode == PanelSelectionMode::Item
                    && Some(*index) == input.selection.item_index
            }
            BootstrapTarget::DocumentRepair { .. } => {
                input.selection.mode == PanelSelectionMode::DocumentRepair
                    && input.selection.item_index.is_none()
            }
            BootstrapTarget::ManualRepair { .. } => {
                input.selection.mode == PanelSelectionMode::ManualRepair
                    && input.selection.item_index.is_none()
            }
        };
        if !index_matches
            || accepted.proposal_id != input.selection.proposal.proposal_id
            || accepted.source_id != input.selection.source_id
            || accepted.source_revision != input.selection.source_revision
            || accepted.source_sha256 != input.selection.source_sha256
        {
            return Err(InitialPanelMountFailure {
                factory: PanelFactoryFailure::before_mount(FactoryCause::Input(
                    "Mount selection differs from accepted startup preparation",
                )),
                prepared: self.prepared.clone(),
                unregistered: Some(self),
                final_cleanup_started: false,
                close_progress: None,
                close_error: None,
            });
        }
        let prepared = self.prepared;
        // Owned data was validated before ACK; any differing caller input is rejected by factory.
        input.modules.initial_props = prepared.props.clone();
        let stored = match prepared.snapshot.as_ref() {
            Ok(stored) => stored.clone(),
            Err(_) => unreachable!("only typed success is prepared"),
        };
        let initial_repair = match &prepared.selection.target {
            BootstrapTarget::DocumentRepair { source_ref } => source_ref.clone(),
            _ => None,
        };
        let owner_slot = Rc::new(RefCell::new(Some(self.owner)));
        let transferred = owner_slot.clone();
        let result = mount_inspected_panel(
            input,
            stored,
            prepared.selection.document_revision,
            correction,
            initial_repair.as_ref(),
            frame,
            move |_, registry, scope, context| {
                let execution = transferred
                    .try_borrow()
                    .map_err(|_| {
                        PluginError::new(PluginErrorCode::Busy, "Startup owner is borrowed")
                    })?
                    .as_ref()
                    .ok_or_else(|| {
                        PluginError::new(
                            PluginErrorCode::Stale,
                            "Startup owner was already transferred",
                        )
                    })?
                    .handle();
                let registration = register(execution, registry, scope, context)?;
                let execution_owner = transferred
                    .try_borrow_mut()
                    .map_err(|_| {
                        PluginError::new(
                            PluginErrorCode::Busy,
                            "Startup owner transfer is borrowed",
                        )
                    })?
                    .take()
                    .ok_or_else(|| {
                        PluginError::new(
                            PluginErrorCode::Stale,
                            "Startup owner was already transferred",
                        )
                    })?;
                Ok(InitialExecutionParent {
                    registration,
                    execution_owner,
                    close_error: None,
                    close_progress: None,
                })
            },
        );
        match result {
            Ok(panel) => Ok(InitialPanelMount { panel, prepared }),
            Err(factory) => {
                let owner = owner_slot.borrow_mut().take();
                let unregistered = owner.map(|owner| InitialPanelHandoff {
                    owner,
                    prepared: prepared.clone(),
                });
                Err(InitialPanelMountFailure {
                    factory,
                    prepared,
                    unregistered,
                    final_cleanup_started: false,
                    close_progress: None,
                    close_error: None,
                })
            }
        }
    }
}

#[cfg(test)]
#[path = "initial_panel/tests.rs"]
mod tests;

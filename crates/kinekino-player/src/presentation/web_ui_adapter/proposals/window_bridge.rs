//! Unregistered product consumer of Meari's published Native host port.
use super::correction_panel::{
    CorrectionBudget, CorrectionLabels, CorrectionPreparationError, CorrectionSelection,
    prepare_correction,
};
use super::document_controller::InteractionError;
use super::{
    IntentPins, ProposalDocumentSession, ReprojectionError, ReviewSession, ViewError,
    document_factory::{
        CallbackCleanupEvidence, ExplicitPanelSelection, MountedProposalPanel, PanelHostParts,
        PanelSelectionMode, ParentCloseProgress, ParentProposalLease,
    },
    panel_model::{PanelBudget, PanelLabels, PanelPreparationError, PanelSelection, prepare_panel},
};
use kinekino_player::application::api::RequestContext;
use kinekino_player::domain::proposals::ProposalTask;
use meari::application::{
    host_callbacks::{
        CallbackBindings, LocalActionInvocation, LocalDispatchError, PendingCallback,
    },
    plugins::*,
};
use meari::domain::{Value, build::BuildPath};
use meari_native::infrastructure::native_host::*;
use std::{cell::RefCell, collections::BTreeMap, fmt, rc::Rc, time::Duration};

#[path = "window_bridge/navigation.rs"]
mod navigation;

#[cfg(all(windows, feature = "native-proposal-window"))]
#[path = "window_bridge/scoped.rs"]
mod scoped;
#[cfg(all(windows, feature = "native-proposal-window"))]
pub use scoped::{ScopedBridgeMount, ScopedBridgeMountFailure};

#[derive(Debug)]
pub enum PanelBridgeError {
    CompletionRegistryGuard(PluginError),
    Access(HostAccessError),
    Plugin(PluginError),
    View(ViewError),
    Panel(PanelPreparationError),
    Reprojection(ReprojectionError),
    Admission(&'static str),
    Commit(String),
}
impl fmt::Display for PanelBridgeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}
impl std::error::Error for PanelBridgeError {}
pub type PanelLabelProvider =
    Box<dyn Fn(&ReviewSession, Option<usize>, Option<ViewError>) -> PanelLabels>;
pub type CorrectionLabelProvider =
    Box<dyn Fn(&ReviewSession, Option<usize>, Option<ViewError>) -> CorrectionLabels>;
pub type PanelClosePolicy =
    Box<dyn FnMut(&ReviewSession, CloseCause) -> Result<bool, PanelBridgeError>>;
pub type PanelCommitPort = Box<
    dyn FnMut(
        &mut meari_native::infrastructure::document::InteractiveDocument,
        BTreeMap<String, Value>,
        CallbackBindings,
    ) -> Result<(), String>,
>;
/// Consumes a validated local invocation immediately; only queues presentation work.
pub type PanelLocalIntentPort = Box<
    dyn FnMut(&ProposalDocumentSession, LocalActionInvocation) -> IntentAdmission<PanelBridgeError>,
>;

/// Product owners retained separately from the single Native document handle.
pub struct PanelBridgeOwners<L: ParentProposalLease> {
    frame: Option<super::frame_composition::FrameProjection>,
    pub session: ProposalDocumentSession,
    pub registry: Rc<RefCell<PluginRegistry>>,
    registry_pending: usize,
    pub context: Rc<RefCell<RequestContext>>,
    pub selection: ExplicitPanelSelection,
    pub inventory: BTreeMap<BuildPath, String>,
    parent: L,
}
pub struct BridgeAssemblyFailure<L: ParentProposalLease> {
    pub error: HostAccessError,
    pub owners: PanelBridgeOwners<L>,
    pub session_closed: bool,
    pub parent_closed: bool,
}
impl<L: ParentProposalLease> BridgeAssemblyFailure<L> {
    pub fn poll_cleanup(&mut self) -> Result<ParentCloseProgress, PluginError> {
        if !self.session_closed {
            self.owners.session.close()?;
            self.session_closed = true;
        }
        if !self.parent_closed {
            let progress = self.owners.parent.poll_close()?;
            if progress != ParentCloseProgress::Complete {
                return Ok(progress);
            }
            self.parent_closed = true;
        }
        Ok(ParentCloseProgress::Complete)
    }
    pub fn parent_close_progress(
        &self,
    ) -> Option<&kinekino_player::application::sdk_execution::ExecutionCloseProgress> {
        self.owners.parent.close_progress()
    }
    /// A failed handle construction has no normal driver close receipt.
    pub fn retry_cleanup(&mut self) -> Result<(), PluginError> {
        if !self.session_closed {
            self.owners.session.close()?;
            self.session_closed = true;
        }
        if !self.parent_closed {
            self.owners.parent.close()?;
            self.parent_closed = true;
        }
        Ok(())
    }
}
/// Not serializable/cloneable; the runtime returns the same consumer-owned cleanup checkpoint.
pub struct PanelCloseCheckpoint {
    host_id: u64,
    owner: PluginScope,
    pub callbacks: CallbackCloseReceipt,
    pub session_closed: bool,
    pub parent_closed: bool,
}
#[derive(Default, Debug)]
pub struct PanelBridgeObservations {
    pub completion_turns: usize,
    pub completion_terminals: usize,
    pub projection_commits: usize,
}
pub struct ProposalWindowBridge<L: ParentProposalLease> {
    staged_frame: Option<super::frame_composition::FrameProjection>,
    handle: NativeDocumentHandle,
    owners: PanelBridgeOwners<L>,
    labels: PanelLabelProvider,
    correction_labels: Option<CorrectionLabelProvider>,
    close_policy: PanelClosePolicy,
    budget: PanelBudget,
    pins: IntentPins,
    needs_drain: bool,
    needs_projection: bool,
    retired_cleanup_pending: bool,
    last_view_error: Option<ViewError>,
    unexpected_ticket: Option<PluginTicket>,
    worker_exit_retry: Option<Duration>,
    close_retry: HostRetry,
    observations: PanelBridgeObservations,
    commit: PanelCommitPort,
    local_intents: Option<PanelLocalIntentPort>,
    owned_session_closed: bool,
    owned_parent_closed: bool,
}
impl<L: ParentProposalLease> ProposalWindowBridge<L> {
    pub fn from_factory(
        panel: MountedProposalPanel<L>,
        labels: PanelLabelProvider,
        close_policy: PanelClosePolicy,
        pins: IntentPins,
    ) -> Result<(NativeDocumentHandle, Self), Box<BridgeAssemblyFailure<L>>> {
        let PanelHostParts {
            frame,
            mut document,
            session,
            parent,
            registry,
            registry_pending,
            budget,
            context,
            selection,
            inventory,
            cleanup,
        } = panel.into_host_parts();
        let generation = session.review().scope().view.source_generation;
        let owners = PanelBridgeOwners {
            frame,
            session,
            parent,
            registry,
            registry_pending,
            context,
            selection,
            inventory,
        };
        if cleanup.callbacks != CallbackCleanupEvidence::Active
            || cleanup.session_closed
            || cleanup.parent_closed
            || document.callback_scope() != Some(&owners.session.review().scope().view)
        {
            document.close_callbacks();
            return Err(Box::new(BridgeAssemblyFailure {
                error: HostAccessError::Closed,
                owners,
                session_closed: cleanup.session_closed,
                parent_closed: cleanup.parent_closed,
            }));
        }
        let handle = match NativeDocumentHandle::new(document, generation) {
            Ok(handle) => handle,
            Err(error) => {
                return Err(Box::new(BridgeAssemblyFailure {
                    error,
                    owners,
                    session_closed: false,
                    parent_closed: false,
                }));
            }
        };
        Ok((
            handle.clone(),
            Self::from_owned_parts(handle, owners, labels, close_policy, budget, pins),
        ))
    }
    fn from_owned_parts(
        handle: NativeDocumentHandle,
        owners: PanelBridgeOwners<L>,
        labels: PanelLabelProvider,
        close_policy: PanelClosePolicy,
        budget: PanelBudget,
        pins: IntentPins,
    ) -> Self {
        Self {
            staged_frame: None,
            handle,
            owners,
            labels,
            correction_labels: None,
            close_policy,
            budget,
            pins,
            needs_drain: false,
            needs_projection: false,
            retired_cleanup_pending: false,
            last_view_error: None,
            unexpected_ticket: None,
            worker_exit_retry: None,
            close_retry: HostRetry::OnWake,
            observations: PanelBridgeObservations::default(),
            commit: Box::new(|document, props, export| {
                document.update_module_props_with_callbacks(props, export)
            }),
            local_intents: None,
            owned_session_closed: false,
            owned_parent_closed: false,
        }
    }
    pub fn review(&self) -> &ReviewSession {
        self.owners.session.review()
    }
    pub fn output_review_state(&self) -> Option<&super::output_review_state::OutputReviewState> {
        self.owners.session.output_review()
    }
    /// Actual SDK transport work; a retained domain token may instead await explicit refresh.
    pub fn operation_pending(&self) -> bool {
        self.owners.session.operation_pending()
    }
    pub fn frame_projection(&self) -> Option<&super::frame_composition::FrameProjection> {
        self.owners.frame.as_ref()
    }
    /// Explicit presentation replacement; the next checked commit validates the new slot key.
    pub fn replace_frame_workspace(&mut self, key: String) -> Result<(), HostAccessError> {
        self.owners
            .frame
            .as_mut()
            .ok_or_else(|| HostAccessError::Invalid("Document has no frame slot".into()))?
            .replace_key(key);
        self.needs_projection = true;
        Ok(())
    }
    pub fn observations(&self) -> &PanelBridgeObservations {
        &self.observations
    }
    /// Trusted, construction-time renderer boundary; never a serialized or global setter.
    pub fn with_commit_port(mut self, commit: PanelCommitPort) -> Self {
        self.commit = commit;
        self
    }
    pub fn with_local_intents(mut self, port: PanelLocalIntentPort) -> Self {
        self.local_intents = Some(port);
        self
    }
    pub(crate) fn with_caption_adapter(
        mut self,
        source: std::rc::Weak<
            std::cell::RefCell<
                crate::presentation::web_ui_adapter::player_shell::window_caption::CaptionState,
            >,
        >,
    ) -> Self {
        use crate::presentation::web_ui_adapter::player_shell::window_caption;
        if let Some(frame) = self.owners.frame.as_mut() {
            frame.set_caption_source(source.clone());
        }
        let mut previous = self.local_intents.take();
        self.local_intents = Some(Box::new(
            move |session, invocation| match window_caption::decode(&invocation, |name| {
                session.local_action_key(name).ok().cloned()
            }) {
                Ok(Some(intent)) => {
                    let Some(source) = source.upgrade() else {
                        return IntentAdmission::Rejected(PanelBridgeError::Admission(
                            "Original window caption source closed",
                        ));
                    };
                    let Ok(mut source) = source.try_borrow_mut() else {
                        return IntentAdmission::Capacity;
                    };
                    match source.request(intent) {
                        Ok(()) => IntentAdmission::Accepted,
                        Err(error)
                            if error.code == meari::application::plugins::PluginErrorCode::Busy =>
                        {
                            IntentAdmission::Capacity
                        }
                        Err(error) => IntentAdmission::Rejected(PanelBridgeError::Plugin(error)),
                    }
                }
                Ok(None) => match previous.as_mut() {
                    Some(port) => port(session, invocation),
                    None => IntentAdmission::Rejected(PanelBridgeError::Admission(
                        "Frame local action is not handled",
                    )),
                },
                Err(error) => IntentAdmission::Rejected(PanelBridgeError::Plugin(error)),
            },
        ));
        self.needs_projection = true;
        self
    }
    pub fn projection_pending(&self) -> bool {
        self.needs_projection
            || self.owners.session.projection_needed()
            || self
                .owners
                .frame
                .as_ref()
                .is_some_and(|frame| frame.caption_pending())
    }
    /// SDK/session cleanup after a failed/forced Native return; this is not an OS close receipt.
    pub fn poll_owned_cleanup(&mut self) -> Result<ParentCloseProgress, PanelBridgeError> {
        if !self.owned_session_closed {
            if let Err(error) = self.owners.session.close() {
                if self.owners.session.close_waiting() {
                    return Ok(ParentCloseProgress::TerminalAckPending);
                }
                return Err(PanelBridgeError::Plugin(error));
            }
            self.owned_session_closed = true;
        }
        if !self.owned_parent_closed {
            let progress = self
                .owners
                .parent
                .poll_close()
                .map_err(PanelBridgeError::Plugin)?;
            if progress != ParentCloseProgress::Complete {
                return Ok(progress);
            }
            self.owned_parent_closed = true;
        }
        Ok(ParentCloseProgress::Complete)
    }
    pub fn parent_close_error(&self) -> Option<&kinekino_media_contracts::errors::ApiError> {
        self.owners.parent.close_error()
    }
    pub fn parent_close_progress(
        &self,
    ) -> Option<&kinekino_player::application::sdk_execution::ExecutionCloseProgress> {
        self.owners.parent.close_progress()
    }
    pub fn with_correction_labels(mut self, labels: CorrectionLabelProvider) -> Self {
        self.correction_labels = Some(labels);
        self
    }
    pub fn observe_inspector_layout(
        &mut self,
        layout: super::inspector_state::InspectorLayout,
    ) -> Result<(), String> {
        self.owners
            .frame
            .as_ref()
            .ok_or("Frame composition is absent")?
            .observe_inspector_layout(layout)?;
        self.needs_projection = true;
        Ok(())
    }
    pub fn with_worker_exit_retry(
        mut self,
        delay: Duration,
        limits: HostScheduleLimits,
    ) -> Result<Self, Box<Self>> {
        if delay < limits.min_delay() || delay > limits.max_delay() {
            return Err(Box::new(self));
        }
        self.worker_exit_retry = Some(delay);
        Ok(self)
    }
    pub fn last_view_error(&self) -> Option<ViewError> {
        self.last_view_error
    }
    fn validate_stamp(&self, stamp: &HostStamp) -> Result<(), PanelBridgeError> {
        let current = self.handle.stamp().map_err(PanelBridgeError::Access)?;
        if &current != stamp || stamp.owner != self.review().scope().view {
            return Err(PanelBridgeError::Access(HostAccessError::Stale));
        }
        Ok(())
    }
    pub fn select_item(&mut self, index: usize) -> Result<(), ViewError> {
        self.ensure_selection_navigation()?;
        if self.review().stored().document.generated.task == ProposalTask::Correction {
            CorrectionSelection::at(self.review(), self.review().scope(), index)?;
        } else {
            PanelSelection::at(self.review(), self.review().scope(), index)?;
        }
        self.owners.selection.item_index = Some(index);
        self.owners.selection.mode = PanelSelectionMode::Item;
        self.needs_projection = true;
        Ok(())
    }
    fn copy_pins(&self) -> IntentPins {
        IntentPins {
            contract: self.pins.contract.clone(),
            provider: self.pins.provider.clone(),
            request: self.pins.request.clone(),
            reply: self.pins.reply.clone(),
        }
    }
    fn selected_index(&self) -> Result<Option<usize>, PanelBridgeError> {
        match self.owners.selection.mode {
            PanelSelectionMode::ManualRepair => Ok(None),
            PanelSelectionMode::Item => self
                .owners
                .selection
                .item_index
                .map(Some)
                .ok_or(PanelBridgeError::View(ViewError::InvalidSelection)),
            PanelSelectionMode::DocumentRepair => {
                if self.owners.selection.item_index.is_some() {
                    return Err(PanelBridgeError::View(ViewError::InvalidSelection));
                }
                match self.review().draft() {
                    None => Ok(None),
                    Some(
                        kinekino_player::domain::proposals::ProposalPayload::DocumentCorrection {
                            edits,
                        },
                    ) => {
                        let reference = self
                            .review()
                            .document_selection()
                            .ok_or(PanelBridgeError::View(ViewError::InvalidSelection))?;
                        edits
                            .iter()
                            .position(|edit| &edit.source_ref == reference)
                            .map(Some)
                            .ok_or(PanelBridgeError::View(ViewError::InvalidSelection))
                    }
                    _ => Err(PanelBridgeError::View(ViewError::InvalidSelection)),
                }
            }
        }
    }
}
impl<L: ParentProposalLease + 'static> NativeWindowHost for ProposalWindowBridge<L> {
    type Error = PanelBridgeError;
    type CloseCheckpoint = PanelCloseCheckpoint;
    fn admit_callback(
        &mut self,
        stamp: &HostStamp,
        callback: &PendingCallback,
    ) -> IntentAdmission<Self::Error> {
        if let Err(error) = self.validate_stamp(stamp) {
            return IntentAdmission::Rejected(error);
        }
        if self.unexpected_ticket.is_some() {
            return IntentAdmission::Rejected(PanelBridgeError::Admission(
                "Unexpected pending UI callback must reach terminal cleanup",
            ));
        }
        // This immutable limit is the exact one supplied to this factory's PluginRegistry::new.
        // A full pending budget is checked before callback dispatch can enqueue an intent.
        let registry = match self.owners.registry.try_borrow() {
            Ok(registry) => registry,
            Err(_) => {
                return IntentAdmission::Rejected(PanelBridgeError::Access(HostAccessError::Busy));
            }
        };
        if registry.scope() != &stamp.owner {
            return IntentAdmission::Rejected(PanelBridgeError::Access(HostAccessError::Stale));
        }
        let pending_budget_full = registry.pending_count() >= self.owners.registry_pending;
        drop(registry);
        match self.owners.session.dispatch_local_after_renderer_guard(
            &callback.handle,
            callback.invocation.arguments.clone(),
            &stamp.owner,
        ) {
            Ok(invocation) => {
                return match &mut self.local_intents {
                    Some(port) => port(&self.owners.session, invocation),
                    None => IntentAdmission::Rejected(PanelBridgeError::Admission(
                        "Frame local intent consumer is absent",
                    )),
                };
            }
            Err(LocalDispatchError::Invalid(error)) => {
                return IntentAdmission::Rejected(PanelBridgeError::Plugin(error));
            }
            Err(LocalDispatchError::NotLocal) => {}
        }
        let result = self.owners.session.dispatch_handle_after_renderer_guard(
            &callback.handle,
            callback.invocation.arguments.clone(),
        );
        match result {
            Ok(receipt) => match receipt.outcome {
                PluginDispatch::Immediate(Ok(_)) => {
                    self.needs_drain = true;
                    IntentAdmission::Accepted
                }
                PluginDispatch::Immediate(Err(error)) if error.code == PluginErrorCode::Busy => {
                    IntentAdmission::Capacity
                }
                PluginDispatch::Immediate(Err(error)) => {
                    IntentAdmission::Rejected(PanelBridgeError::Plugin(error))
                }
                PluginDispatch::Pending => {
                    self.unexpected_ticket = Some(receipt.ticket);
                    IntentAdmission::Rejected(PanelBridgeError::Admission(
                        "Presentation callback unexpectedly deferred; ticket retained",
                    ))
                }
            },
            // With this immutable full budget and no await/callback between checks, Core
            // still validates handle/signature/payload before returning pre-adapter Busy.
            Err(error) if pending_budget_full && error.code == PluginErrorCode::Busy => {
                IntentAdmission::Capacity
            }
            Err(error) => IntentAdmission::Rejected(PanelBridgeError::Plugin(error)),
        }
    }
    fn after_dispatch(&mut self, turn: HostTurn) -> Result<HostEffects, Self::Error> {
        self.validate_stamp(&turn.stamp)?;
        if turn.reason == HostTurnReason::Completion {
            self.observations.completion_turns += 1;
        }
        // Restore controlled props after rejected input; the typed error remains in the runtime.
        self.needs_projection |= turn.rejected_callbacks != 0;
        if self.retired_cleanup_pending {
            self.owners
                .session
                .retry_cleanup()
                .map_err(PanelBridgeError::Plugin)?;
            self.retired_cleanup_pending = false;
        }
        if self.needs_drain {
            let report = self
                .owners
                .session
                .drain_after_dispatch()
                .map_err(PanelBridgeError::Plugin)?;
            self.needs_drain = false;
            self.needs_projection |=
                report.processed != 0 || report.view_error.is_some() || report.sdk_error;
            self.last_view_error = report.view_error;
            if let Some(error) = report.transport_error {
                return Err(PanelBridgeError::Plugin(error));
            }
        }
        // No Native document guard is held while the registry accepts SDK terminals.
        let completion = self.owners.session.pump_after_dispatch();
        let completed = completion.map_err(|error| match error {
            InteractionError::View(error) => PanelBridgeError::View(error),
            InteractionError::Plugin(error) => {
                if self.owners.session.completion_guard_blocked() {
                    PanelBridgeError::CompletionRegistryGuard(error)
                } else {
                    PanelBridgeError::Plugin(error)
                }
            }
        })?;
        if completed && turn.reason == HostTurnReason::Completion {
            self.observations.completion_terminals += 1;
        }
        self.needs_projection |= completed;
        self.needs_projection |= self.owners.session.projection_needed();
        self.needs_projection |= self
            .owners
            .frame
            .as_ref()
            .is_some_and(|frame| frame.caption_pending());
        // Never replace an export while the runtime still owns an unadmitted suffix.
        self.needs_projection |= self
            .owners
            .frame
            .as_ref()
            .is_some_and(|frame| frame.inspector_pending());
        if let Some(frame) = self.staged_frame.as_ref().or(self.owners.frame.as_ref())
            && frame.inspector_pending()
        {
            self.staged_frame = Some(
                frame
                    .stage_inspector()
                    .map_err(|_| PanelBridgeError::Admission("Inspector queue is borrowed"))?,
            );
        }
        if turn.remaining_callbacks != 0 || !self.needs_projection {
            return Ok(HostEffects::default());
        }
        let selected_index = self.selected_index()?;
        let (prepared, feedback_labels) = if self.owners.selection.mode
            == PanelSelectionMode::ManualRepair
        {
            let labels = (self.labels)(self.review(), None, self.last_view_error);
            let feedback_labels = labels.clone();
            let prepared = super::repair_panel::prepare_repair_panel(
                self.review(),
                self.review().scope(),
                labels,
                PanelBudget {
                    entries: self.budget.entries,
                    text_bytes: self.budget.text_bytes,
                },
                self.copy_pins(),
            )
            .map_err(PanelBridgeError::Panel)?;
            (prepared, feedback_labels)
        } else if self.review().stored().document.generated.task == ProposalTask::Correction {
            let selected = if self.review().text_fields().is_empty() {
                CorrectionSelection::repair(self.review(), self.review().scope())
            } else {
                CorrectionSelection::at(
                    self.review(),
                    self.review().scope(),
                    selected_index.ok_or(PanelBridgeError::View(ViewError::InvalidSelection))?,
                )
            }
            .map_err(PanelBridgeError::View)?;
            let labels = self
                .correction_labels
                .as_ref()
                .ok_or(PanelBridgeError::Admission(
                    "Explicit correction labels are required",
                ))?;
            let labels = labels(self.review(), selected_index, self.last_view_error);
            let feedback_labels = labels.panel.clone();
            let prepared = prepare_correction(
                self.review(),
                self.review().scope(),
                &selected,
                labels,
                CorrectionBudget {
                    entries: self.budget.entries,
                    text_bytes: self.budget.text_bytes,
                },
                self.copy_pins(),
            )
            .map_err(|error| match error {
                CorrectionPreparationError::View(error) => PanelBridgeError::View(error),
                CorrectionPreparationError::Queue(error) => {
                    PanelBridgeError::Panel(PanelPreparationError::Queue(error))
                }
            })?;
            (
                super::panel_model::PreparedPanel {
                    props: prepared.props,
                    bindings: prepared.bindings,
                },
                feedback_labels,
            )
        } else {
            let selected = PanelSelection::at(
                self.review(),
                self.review().scope(),
                selected_index.ok_or(PanelBridgeError::View(ViewError::InvalidSelection))?,
            )
            .map_err(PanelBridgeError::View)?;
            let labels = (self.labels)(self.review(), selected_index, self.last_view_error);
            let feedback_labels = labels.clone();
            let prepared = prepare_panel(
                self.review(),
                self.review().scope(),
                &selected,
                labels,
                PanelBudget {
                    entries: self.budget.entries,
                    text_bytes: self.budget.text_bytes,
                },
                self.copy_pins(),
            )
            .map_err(PanelBridgeError::Panel)?;
            (prepared, feedback_labels)
        };
        let mut prepared = prepared.with_transport_pending(self.owners.session.operation_pending());
        let mut feedback = super::review_feedback::ReviewFeedback::from_review(self.review())
            .with_local_error(self.last_view_error);
        feedback.pending |= self.owners.session.operation_pending();
        if self.owners.selection.mode == PanelSelectionMode::ManualRepair {
            super::repair_panel::apply_feedback(&feedback, &mut prepared.props, &feedback_labels);
        } else {
            feedback.apply(&mut prepared.props, &feedback_labels);
        }
        super::output_review_panel::project_component_props(
            &mut prepared.props,
            self.owners.session.output_review(),
            self.owners.session.operation_pending(),
        );
        if prepared.props.contains_key("rawReviewPackage")
            || self
                .staged_frame
                .as_ref()
                .or(self.owners.frame.as_ref())
                .is_some_and(|frame| frame.raw_review_enabled())
        {
            super::raw_review_panel::project_component_props(
                &mut prepared.props,
                self.owners.session.raw_review(),
                self.owners.session.operation_pending(),
                self.owners.session.raw_review_retained(),
                self.owners.session.raw_recovery(),
            );
        }
        let staged_frame = self
            .staged_frame
            .take()
            .or_else(|| self.owners.frame.clone());
        if let Some(frame) = &staged_frame {
            frame
                .bind_inspector(&mut prepared.bindings)
                .map_err(|error| PanelBridgeError::Panel(PanelPreparationError::Queue(error)))?;
        }
        let session = &mut self.owners.session;
        let frame = staged_frame.clone();
        let commit = &mut self.commit;
        let (reprojection, receipt) = self
            .handle
            .try_reproject(&turn.stamp, |document| {
                session.reproject_with(prepared.bindings, |export| {
                    let props = match frame {
                        Some(frame) => {
                            let editor = frame
                                .compose_editor_slot(prepared.props, |key, reference, props| {
                                    document.node_slot(key, reference, props)
                                })?;
                            if let Some(reference) = frame.inspector_reference() {
                                let inspector_props =
                                    frame.inspector_props(editor).expect("inspector props");
                                frame.inspector_root_props(document.node_slot(
                                    "proposal-inspector",
                                    reference.clone(),
                                    inspector_props,
                                )?)
                            } else {
                                frame.root_props(editor)
                            }
                        }
                        None => prepared.props,
                    };
                    commit(document, props, export)
                })
            })
            .map_err(|error| match error {
                HostUpdateError::Access(error) => PanelBridgeError::Access(error),
                HostUpdateError::Apply(error) => PanelBridgeError::Reprojection(error),
            })?;
        self.retired_cleanup_pending = reprojection.cleanup_pending;
        self.owners.frame = staged_frame;
        if let Some(frame) = self.owners.frame.as_mut() {
            frame.caption_committed();
        }
        self.owners.session.projection_committed();
        self.needs_projection = false;
        self.observations.projection_commits += 1;
        Ok(HostEffects {
            committed: Some(receipt),
            redraw: true,
        })
    }
    fn can_close(&mut self, stamp: &HostStamp, cause: CloseCause) -> Result<bool, Self::Error> {
        self.validate_stamp(stamp)?;
        (self.close_policy)(self.owners.session.review(), cause)
    }
    fn next_close_observation(&self, _: &Self::CloseCheckpoint) -> HostRetry {
        self.close_retry
    }
    fn retry_after_dispatch(&self, error: &Self::Error) -> Option<HostRetry> {
        match error {
            PanelBridgeError::CompletionRegistryGuard(_)
            | PanelBridgeError::Access(HostAccessError::Busy) => Some(HostRetry::OnWake),
            _ => None,
        }
    }
    fn close(
        &mut self,
        request: CloseRequest<Self::CloseCheckpoint>,
    ) -> CloseDecision<Self::CloseCheckpoint, Self::Error> {
        let mut checkpoint = request.checkpoint.unwrap_or(PanelCloseCheckpoint {
            host_id: request.stamp.host_id,
            owner: request.stamp.owner.clone(),
            callbacks: request.callbacks,
            session_closed: false,
            parent_closed: false,
        });
        if checkpoint.host_id != request.stamp.host_id
            || checkpoint.owner != request.stamp.owner
            || request.stamp.owner != self.review().scope().view
        {
            return CloseDecision::RetryableFailure {
                checkpoint,
                error: PanelBridgeError::Access(HostAccessError::Stale),
            };
        }
        self.close_retry = HostRetry::OnWake;
        let cleanup = (|| -> Result<bool, PanelBridgeError> {
            if let Some(ticket) = &self.unexpected_ticket {
                let cancelled = self
                    .owners
                    .registry
                    .try_borrow_mut()
                    .map_err(|_| PanelBridgeError::Admission("Registry cleanup is borrowed"))?
                    .cancel(ticket)
                    .map_err(PanelBridgeError::Plugin)?;
                if cancelled == CancelReceipt::Requested {
                    return Ok(false);
                }
                self.unexpected_ticket = None;
            }
            if !checkpoint.session_closed {
                if let Err(error) = self.owners.session.close() {
                    if self.owners.session.close_waiting() {
                        return Ok(false);
                    }
                    return Err(PanelBridgeError::Plugin(error));
                }
                checkpoint.session_closed = true;
                self.needs_drain = false;
            }
            if !checkpoint.parent_closed {
                let progress = self
                    .owners
                    .parent
                    .poll_close()
                    .map_err(PanelBridgeError::Plugin)?;
                match progress {
                    ParentCloseProgress::Complete => {}
                    ParentCloseProgress::TerminalAckPending => return Ok(false),
                    ParentCloseProgress::WorkerExitPending => {
                        self.close_retry = self
                            .worker_exit_retry
                            .map_or(HostRetry::OnWake, HostRetry::After);
                        return Ok(false);
                    }
                }
                checkpoint.parent_closed = true;
            }
            Ok(true)
        })();
        if request.cause == CloseCause::ForcedOsDestroy {
            return CloseDecision::ForcedIncomplete {
                checkpoint: Some(checkpoint),
                error: cleanup.err(),
            };
        }
        match cleanup {
            Ok(true) => CloseDecision::HostCleanupComplete { checkpoint },
            Ok(false) => CloseDecision::Pending { checkpoint },
            Err(error) => CloseDecision::RetryableFailure { checkpoint, error },
        }
    }
}

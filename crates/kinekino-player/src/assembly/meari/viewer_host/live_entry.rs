//! Explicit live handoff on the same owner; mount, retained observation and input activation differ.
use super::*;
mod diagnostic;
use crate::{
    application::launch::native_proposal::{NativeProposalSelection, NativeProposalTarget},
    assembly::meari::card_current_inspection::{
        CardCurrentInspection, CardInspectionInput, CardInspectionStartFailure,
    },
    presentation::web_ui_adapter::{
        player_shell::{NavigationGuards, RouteCatalog, RouteTicket},
        proposals::{
            bootstrap::{BootstrapError, BootstrapSelection, project_initial},
            pending_host::PendingOutput,
            product_window::{ProductWindowLabels, product_codec_budget},
        },
    },
};
use meari::{
    application::plugins::{PluginError, PluginErrorCode, PluginRegistry},
    domain::build::TargetKind,
};
use std::{
    cell::Cell,
    num::{NonZeroU64, NonZeroUsize},
};

pub(crate) const LIVE_ENTRY_BINDING: &str = "onOpenLiveProposal";

/// Trusted local request/create results supplied by product assembly, never captured rows.
/// This is a bounded presentation reference table, not SDK mutation permission issuance.
pub struct TrustedLiveChoices {
    rows: Vec<(String, Arc<NativeProposalSelection>)>,
}
impl TrustedLiveChoices {
    pub fn from_trusted_local(
        selections: Vec<NativeProposalSelection>,
        generation: NonZeroU64,
        capacity: NonZeroUsize,
    ) -> Result<Self, PluginError> {
        if selections.is_empty() || selections.len() > capacity.get() {
            return Err(PluginError::new(
                PluginErrorCode::InvalidPayload,
                "Explicit bounded live choices required",
            ));
        }
        let mut rows = Vec::with_capacity(selections.len());
        for (index, selection) in selections.into_iter().enumerate() {
            if matches!(
                &selection.target,
                NativeProposalTarget::DocumentRepair { .. }
            ) {
                return Err(PluginError::new(
                    PluginErrorCode::MissingCapability,
                    "Live repair entry is not connected",
                ));
            }
            selection.target.checked_item_index().map_err(|error| {
                PluginError::new(PluginErrorCode::InvalidPayload, error.message)
            })?;
            rows.push((
                format!("live-proposal-{}-{index}", generation.get()),
                Arc::new(selection),
            ));
        }
        Ok(Self { rows })
    }
    pub(crate) fn into_rows(self) -> Vec<(String, Arc<NativeProposalSelection>)> {
        self.rows
    }
}

pub(crate) struct LiveEntryRequest {
    pub selected: Arc<NativeProposalSelection>,
    pub pending: PendingCallback,
}
impl Clone for LiveEntryRequest {
    fn clone(&self) -> Self {
        Self {
            selected: self.selected.clone(),
            pending: PendingCallback {
                handle: self.pending.handle.clone(),
                invocation: self.pending.invocation.clone(),
            },
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LiveEntryPhase {
    Draining,
    Starting,
    Inspecting,
    Inspected,
    AwaitingArm,
    AwaitingFrame,
    Activating,
    Active,
    Failed,
    Closing,
}
pub(super) struct LiveOpening {
    pub request: LiveEntryRequest,
    pub prepared: Option<crate::assembly::meari::viewer_callbacks::chooser::PreparedSelection>,
    pub ticket: RouteTicket,
    pub phase: LiveEntryPhase,
    pub start_failure: Option<Box<CardInspectionStartFailure>>,
    pub validation_error: Option<BootstrapError>,
    pub first_error: Option<ViewerHostError>,
    pub error: Option<ViewerHostError>,
    pub receipt: Option<HostCommitReceipt>,
    pub proof: Option<RetainedCommitProof>,
    pub proof_error: Option<Arc<HostAccessError>>,
    pub retained_inspection: Option<CardCurrentInspection>,
    pub mount_failure: Option<Box<super::mount_proposal::ProductProposalMountFailure>>,
    pub rejected_bridge: Option<Box<ActiveCardBridge>>,
    pub gate: Rc<Cell<bool>>,
    pub observed: bool,
    pub wake_error: Option<Arc<crate::application::api::ApiError>>,
}
impl LiveOpening {
    fn fail(&mut self, error: ViewerHostError) -> ViewerHostError {
        self.first_error.get_or_insert_with(|| error.clone());
        self.error = Some(error.clone());
        self.phase = LiveEntryPhase::Failed;
        diagnostic::opening(self, Some(&error));
        error
    }
}
fn plugin(error: PluginError) -> ViewerHostError {
    ViewerHostError::Callback(Arc::new(error))
}
fn blocked(message: &'static str) -> ViewerHostError {
    plugin(PluginError::new(PluginErrorCode::Closing, message))
}
fn route(selection: &NativeProposalSelection) -> &'static str {
    match &selection.target {
        NativeProposalTarget::Card { .. } => "studio.cards",
        NativeProposalTarget::Highlight { .. } => "studio.shorts.recommendations",
        NativeProposalTarget::ManualRepair { task } => match task {
            crate::domain::proposals::ProposalTask::Cards => "studio.cards",
            crate::domain::proposals::ProposalTask::Highlights => "studio.shorts.recommendations",
            crate::domain::proposals::ProposalTask::Correction => "studio.subtitles.correction",
        },
        _ => "studio.subtitles.correction",
    }
}
impl ViewerHost {
    #[cfg(test)]
    pub(crate) fn install_live_fixture_notifier(&mut self) {
        if let Some(probe) =
            super::super::viewer_native_probe::current().filter(|probe| probe.is_live())
        {
            match self.completion.take() {
                Some(original) => self.completion = Some(probe.live_notifier(original)),
                None => probe.live_fault("original startup route notifier missing".into()),
            }
        }
    }
    #[cfg(test)]
    fn live_fixture_turn(&mut self, turn: &HostTurn) -> Result<(), ViewerHostError> {
        let Some(probe) =
            super::super::viewer_native_probe::current().filter(|probe| probe.is_live())
        else {
            return Ok(());
        };
        let mut save_snapshot = None;
        {
            let cards = self
                .cards
                .try_borrow()
                .map_err(|_| ViewerHostError::UnsupportedCallback)?;
            if let Some(entry) = &cards.live_opening {
                let guards = cards
                    .active
                    .as_ref()
                    .map(|active| active.navigation_guards());
                let active = cards.active.as_ref();
                save_snapshot = Some(
                    super::super::viewer_native_probe::live_retained::SaveSnapshot {
                        stage: "before",
                        active: entry.phase == LiveEntryPhase::Active,
                        dirty: guards.map(|guard| guard.dirty),
                        pending: guards.map(|guard| guard.pending),
                        recovery: guards.map(|guard| guard.recovery_blocked),
                        transport: active.map(|active| active.operation_pending()),
                        token: active.map(|active| active.review().pending().is_some()),
                        confirmed: active
                            .map(|active| active.review().confirmed_mutation().is_some()),
                        projection: active.map(|active| active.projection_pending()),
                        revision: active
                            .map(|active| active.review().stored().document.draft_revision.0),
                        remaining: turn.remaining_callbacks,
                        reason: turn.reason,
                    },
                );
                probe.live_host_state(
                    entry.phase == LiveEntryPhase::Active,
                    guards.is_some_and(|guard| guard.dirty),
                    guards.is_some_and(|guard| guard.pending),
                    entry.observed,
                    entry.gate.get(),
                    entry.wake_error.as_deref(),
                );
                probe.live_error_state(entry.first_error.as_ref(), entry.error.as_ref());
            }
        }
        if let Some(snapshot) = save_snapshot {
            probe.live_saving_checkpoint(snapshot);
        }
        if probe.live_return_ready() && !self.shutdown_started && turn.remaining_callbacks == 0 {
            self.request_viewing_return(turn)?;
            probe.live_return_requested();
        }
        Ok(())
    }
    #[cfg(test)]
    fn live_fixture_entry_commit(&self, entry: &LiveOpening) {
        if let Some(probe) =
            super::super::viewer_native_probe::current().filter(|probe| probe.is_live())
        {
            if let (Some(document), Some(receipt)) = (&self.document, &entry.receipt) {
                probe.live_commit(
                    document,
                    receipt,
                    entry.proof.as_ref(),
                    entry.proof_error.as_deref(),
                    super::super::viewer_native_probe::live_retained::ProofStage::Entry,
                );
                self.live_fixture_cancel_model_route(
                    &entry.ticket,
                    route(&entry.request.selected),
                    super::super::viewer_native_probe::live_retained::ProofStage::Entry,
                );
            } else {
                probe.live_fault("original live commit carrier missing".into());
            }
        }
    }
    #[cfg(test)]
    pub(super) fn live_fixture_cancel_model_route(
        &self,
        ticket: &crate::presentation::web_ui_adapter::player_shell::RouteTicket,
        target: &str,
        stage: super::super::viewer_native_probe::live_retained::ProofStage,
    ) {
        let Some(probe) = super::super::viewer_native_probe::current()
            .filter(|probe| probe.take_live_model_cancel(stage))
        else {
            return;
        };
        let result = (|| {
            let mut observer = self
                .route_observation
                .as_ref()
                .ok_or(ViewerHostError::UnsupportedCallback)?
                .try_borrow_mut()
                .map_err(|_| ViewerHostError::UnsupportedCallback)?;
            if observer
                .state
                .route_target(ticket)
                .map_err(ViewerHostError::Navigation)?
                .as_str()
                != target
            {
                return Err(ViewerHostError::Navigation(crate::presentation::web_ui_adapter::player_shell::NavigationError::RouteMismatch));
            }
            observer
                .state
                .cancel_route(ticket)
                .map_err(ViewerHostError::Navigation)
        })();
        probe.live_model_cancelled(stage, result);
    }
    pub(super) fn route_continuation(&self) -> Option<Arc<crate::application::api::ApiError>> {
        self.completion.as_ref().map_or_else(
            || {
                Some(Arc::new(crate::application::api::ApiError::new(
                    crate::application::api::ErrorCode::Unsupported,
                    "Original Viewer continuation notifier is not configured",
                )))
            },
            |completion| completion.notify().err().map(Arc::new),
        )
    }
    pub fn configure_live_choices(
        &mut self,
        choices: TrustedLiveChoices,
    ) -> Result<(), ViewerHostError> {
        self.callbacks
            .try_borrow_mut()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?
            .configure_live_choices(choices)
            .map_err(plugin)
    }
    pub fn live_entry_phase(&self) -> Option<LiveEntryPhase> {
        self.cards
            .try_borrow()
            .ok()?
            .live_opening
            .as_ref()
            .map(|entry| entry.phase)
    }
    pub(super) fn admit_live_entry(
        &mut self,
        stamp: &HostStamp,
        pending: &PendingCallback,
    ) -> Option<IntentAdmission<ViewerHostError>> {
        if pending.invocation.prop != LIVE_ENTRY_BINDING {
            return None;
        }
        if self.shutdown_started
            || self.document.as_ref().is_none_or(|document| {
                document.lifecycle() != HostLifecycle::Open
                    || document.stamp().ok().as_ref() != Some(stamp)
            })
        {
            return Some(IntentAdmission::Rejected(
                ViewerHostError::ForeignCheckpoint,
            ));
        }
        let Ok(cards) = self.cards.try_borrow() else {
            return Some(IntentAdmission::Capacity);
        };
        if cards.live_opening.is_some()
            || cards.active.is_some()
            || cards.opening.is_some()
            || cards.inspection.is_some()
            || cards.capture.is_some()
            || cards.capture_entry.is_some()
            || cards.mount_failure.is_some()
            || cards.rejected_bridge.is_some()
            || cards.viewing_return.is_some()
        {
            return Some(IntentAdmission::Rejected(blocked(
                "An original proposal owner is retained",
            )));
        }
        drop(cards);
        let Ok(jobs) = self.jobs.try_borrow() else {
            return Some(IntentAdmission::Capacity);
        };
        if jobs.occupied() {
            return Some(IntentAdmission::Rejected(blocked(
                "Original JOB owner is retained",
            )));
        }
        drop(jobs);
        let Ok(mut owner) = self.callbacks.try_borrow_mut() else {
            return Some(IntentAdmission::Capacity);
        };
        Some(match owner.queue_live_entry(&stamp.owner, pending) {
            Ok(IntentAdmission::Accepted) => IntentAdmission::Accepted,
            Ok(IntentAdmission::Capacity) => IntentAdmission::Capacity,
            Ok(IntentAdmission::Rejected(error)) | Err(error) => {
                IntentAdmission::Rejected(plugin(error))
            }
        })
    }
    pub(super) fn advance_live_entry(
        &mut self,
        turn: &HostTurn,
    ) -> Result<Option<HostEffects>, ViewerHostError> {
        diagnostic::host(self, None);
        let result = self.advance_live_entry_inner(turn);
        diagnostic::host(self, result.as_ref().err());
        result
    }
    fn advance_live_entry_inner(
        &mut self,
        turn: &HostTurn,
    ) -> Result<Option<HostEffects>, ViewerHostError> {
        if turn.remaining_callbacks != 0 {
            return Ok(None);
        }
        if self.live_entry_phase().is_some_and(|phase| {
            matches!(
                phase,
                LiveEntryPhase::Inspected
                    | LiveEntryPhase::AwaitingArm
                    | LiveEntryPhase::AwaitingFrame
                    | LiveEntryPhase::Activating
                    | LiveEntryPhase::Active
                    | LiveEntryPhase::Failed
                    | LiveEntryPhase::Closing
            )
        }) {
            return self.advance_live_projection(turn);
        }
        if self
            .cards
            .try_borrow()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?
            .live_opening
            .is_none()
        {
            if !self
                .callbacks
                .try_borrow()
                .map_err(|_| ViewerHostError::UnsupportedCallback)?
                .has_live_entry()
            {
                return Ok(None);
            }
            if self.product_frame.is_none() || self.shutdown_started {
                return Err(ViewerHostError::ForeignCheckpoint);
            }
            let mut cards = self
                .cards
                .try_borrow_mut()
                .map_err(|_| ViewerHostError::UnsupportedCallback)?;
            if cards.active.is_some()
                || cards.opening.is_some()
                || cards.inspection.is_some()
                || cards.capture.is_some()
                || cards.capture_entry.is_some()
                || cards.viewing_return.is_some()
                || cards.mount_failure.is_some()
                || cards.rejected_bridge.is_some()
            {
                return Err(blocked("Original Card/capture owner is retained"));
            }
            let jobs = self
                .jobs
                .try_borrow()
                .map_err(|_| ViewerHostError::UnsupportedCallback)?;
            if jobs.occupied() {
                return Err(blocked("Original JOB owner is retained"));
            }
            let mut owner = self
                .callbacks
                .try_borrow_mut()
                .map_err(|_| ViewerHostError::UnsupportedCallback)?;
            if owner.has_jobs_entry() {
                return Err(blocked("An original JOB intent is retained"));
            }
            // Keep the original static-request preflight; prepared provenance is consumed
            // only through the atomic handoff below.
            if owner.chooser_prepared().is_none() {
                owner
                    .live_entry_request(&turn.stamp.owner)
                    .map_err(plugin)?
                    .ok_or(ViewerHostError::ForeignCheckpoint)?;
            }
            let mut observer = self
                .route_observation
                .as_ref()
                .ok_or(ViewerHostError::UnsupportedCallback)?
                .try_borrow_mut()
                .map_err(|_| ViewerHostError::UnsupportedCallback)?;
            if observer
                .state
                .observed_route()
                .is_none_or(|route| route.as_str() != "view.start")
            {
                return Err(blocked("Live entry requires observed Viewing"));
            }
            let catalog =
                RouteCatalog::bundled().map_err(|_| ViewerHostError::UnsupportedCallback)?;
            let gate = Rc::new(Cell::new(false));
            owner
                .commit_live_entry_handoff(&turn.stamp.owner, |selected, pending, prepared| {
                    let ticket = observer
                        .state
                        .request_route(&catalog, route(&selected), NavigationGuards::default())
                        .map_err(ViewerHostError::Navigation)?;
                    cards.live_opening = Some(LiveOpening {
                        request: LiveEntryRequest { selected, pending },
                        prepared,
                        ticket,
                        phase: LiveEntryPhase::Draining,
                        start_failure: None,
                        validation_error: None,
                        first_error: None,
                        error: None,
                        receipt: None,
                        proof: None,
                        proof_error: None,
                        retained_inspection: None,
                        mount_failure: None,
                        rejected_bridge: None,
                        gate,
                        observed: false,
                        wake_error: None,
                    });
                    if let Some(opening) = &cards.live_opening {
                        diagnostic::opening(opening, None);
                    }
                    Ok::<(), ViewerHostError>(())
                })
                .map_err(plugin)??;
        }
        let phase = self
            .live_entry_phase()
            .ok_or(ViewerHostError::ForeignCheckpoint)?;
        if matches!(
            phase,
            LiveEntryPhase::Inspected | LiveEntryPhase::Failed | LiveEntryPhase::Closing
        ) {
            return Ok(Some(HostEffects::default()));
        }
        if phase == LiveEntryPhase::Draining {
            let mut owner = self
                .callbacks
                .try_borrow_mut()
                .map_err(|_| ViewerHostError::UnsupportedCallback)?;
            owner.consume_reads().map_err(plugin)?;
            if owner
                .prepare_route_release(&turn.stamp.owner)
                .map_err(plugin)?
                .is_some()
            {
                if let Some(delay) = owner.next_consumer_retry() {
                    return Err(ViewerHostError::ReadConsumerRetry(delay));
                }
                return Ok(Some(HostEffects::default()));
            }
            drop(owner);
            // Callback release does not release the original native child. Keep
            // the same retirement ticket until its adapter reports cleanup complete.
            match self.native_route.retire(&turn.stamp.owner) {
                Ok(true) => (),
                Ok(false) => {
                    return Err(ViewerHostError::ReadConsumerRetry(Duration::from_millis(5)));
                }
                Err(error) => {
                    let error = ViewerHostError::Sdk(error);
                    let mut cards = self
                        .cards
                        .try_borrow_mut()
                        .map_err(|_| ViewerHostError::UnsupportedCallback)?;
                    return Err(cards
                        .live_opening
                        .as_mut()
                        .ok_or(ViewerHostError::ForeignCheckpoint)?
                        .fail(error));
                }
            }
            self.cards
                .try_borrow_mut()
                .map_err(|_| ViewerHostError::UnsupportedCallback)?
                .live_opening
                .as_mut()
                .unwrap()
                .phase = LiveEntryPhase::Starting;
            diagnostic::host(self, None);
        }
        if self.live_entry_phase() == Some(LiveEntryPhase::Starting) {
            let owner = self
                .callbacks
                .try_borrow()
                .map_err(|_| ViewerHostError::UnsupportedCallback)?;
            if !owner.is_released() {
                return Err(blocked("Original Viewing attachment still retained"));
            }
            let execution = owner.original_execution();
            let context = owner.original_context();
            drop(owner);
            let source = self
                .product_frame
                .as_ref()
                .ok_or(ViewerHostError::UnsupportedCallback)?
                .graph
                .fingerprint()
                .map_err(|message| plugin(PluginError::new(PluginErrorCode::Adapter, message)))?;
            let codec = product_codec_budget();
            let limit = codec.max_pending;
            let registry = Rc::new(RefCell::new(
                PluginRegistry::new(source, TargetKind::Native, turn.stamp.owner.clone(), limit)
                    .map_err(plugin)?,
            ));
            let mut cards = self
                .cards
                .try_borrow_mut()
                .map_err(|_| ViewerHostError::UnsupportedCallback)?;
            let opening = cards.live_opening.as_mut().unwrap();
            if opening.prepared.as_ref().is_some_and(|prepared| {
                prepared.scope != turn.stamp.owner
                    || !Arc::ptr_eq(&prepared.selection, &opening.request.selected)
            }) {
                let error = plugin(PluginError::new(
                    PluginErrorCode::Stale,
                    "Original prepared Open provenance changed",
                ));
                return Err(opening.fail(error));
            }
            let selection = BootstrapSelection::try_from(opening.request.selected.as_ref().clone())
                .map_err(|error| opening.fail(ViewerHostError::Sdk(Arc::new(error))))?;
            let mut inspection = match CardCurrentInspection::new(CardInspectionInput {
                execution,
                scope: turn.stamp.owner.clone(),
                registry,
                context: Rc::new(RefCell::new(context)),
                namespace: format!("live-proposal-inspect-{}", opening.ticket.generation()),
                limit,
                codec,
            }) {
                Ok(inspection) => inspection,
                Err(failure) => {
                    let error = plugin(failure.error.clone());
                    opening.start_failure = Some(failure);
                    return Err(opening.fail(error));
                }
            };
            let result = inspection.begin(selection);
            let error = result
                .err()
                .map(|error| opening.fail(ViewerHostError::Sdk(Arc::new(error))));
            if error.is_none() {
                opening.phase = LiveEntryPhase::Inspecting;
                diagnostic::opening(opening, None);
            }
            cards.inspection = Some(inspection);
            if let Some(error) = error {
                return Err(error);
            }
            return Ok(Some(HostEffects::default()));
        }
        let mut cards = self
            .cards
            .try_borrow_mut()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?;
        let cards = &mut *cards;
        let opening = cards
            .live_opening
            .as_mut()
            .ok_or(ViewerHostError::ForeignCheckpoint)?;
        let inspection = cards
            .inspection
            .as_mut()
            .ok_or(ViewerHostError::ForeignCheckpoint)?;
        if !inspection.poll().map_err(plugin)? {
            return Ok(Some(HostEffects::default()));
        }
        let terminal = inspection
            .terminal()
            .ok_or(ViewerHostError::ForeignCheckpoint)?;
        let stored = match &terminal.outcome {
            Ok(PendingOutput::Inspected(stored)) if !terminal.cancellation_requested => stored,
            _ => {
                return Err(opening.fail(blocked(
                    "Original inspect terminal failed/cancelled; original outcome retained",
                )));
            }
        };
        let selection = inspection
            .selection()
            .ok_or(ViewerHostError::ForeignCheckpoint)?;
        if let Err(error) =
            project_initial(stored, selection, ProductWindowLabels::korean().bootstrap())
        {
            opening.validation_error = Some(error);
            return Err(opening.fail(plugin(PluginError::new(
                PluginErrorCode::Stale,
                "Live inspection identity/target differs; original terminal retained",
            ))));
        }
        if let Err(error) = inspection.release() {
            return Err(opening.fail(plugin(error)));
        }
        opening.phase = LiveEntryPhase::Inspected;
        diagnostic::opening(opening, None);
        if let Some(error) = self.route_continuation() {
            opening.wake_error.get_or_insert(error);
        }
        Ok(Some(HostEffects {
            committed: None,
            redraw: true,
        }))
    }

    pub(super) fn advance_live_projection(
        &mut self,
        turn: &HostTurn,
    ) -> Result<Option<HostEffects>, ViewerHostError> {
        use super::mount_proposal::{ProductProposalMountInput, mount_current_proposal, task};
        use crate::presentation::web_ui_adapter::player_shell::ShellPhase;
        #[cfg(test)]
        self.live_fixture_turn(turn)?;
        if self.live_entry_phase() == Some(LiveEntryPhase::Active) {
            return Ok(None);
        }
        let mut entry = self
            .cards
            .try_borrow_mut()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?
            .live_opening
            .take()
            .ok_or(ViewerHostError::ForeignCheckpoint)?;
        let mut effects = HostEffects::default();
        let result = (|| {
            if self.shutdown_started
                || matches!(
                    entry.phase,
                    LiveEntryPhase::Failed | LiveEntryPhase::Closing
                )
                || turn.remaining_callbacks != 0
            {
                return Ok(());
            }
            let task = task(&entry.request.selected)
                .map_err(|message| plugin(PluginError::new(PluginErrorCode::Adapter, message)))?;
            if entry.phase == LiveEntryPhase::Inspected {
                let owner = self
                    .callbacks
                    .try_borrow()
                    .map_err(|_| ViewerHostError::UnsupportedCallback)?;
                if !owner.is_released() {
                    return Err(blocked("Original Viewing attachment is retained"));
                }
                let execution = owner.original_execution();
                let context = owner.original_context();
                drop(owner);
                let inspection = self
                    .cards
                    .try_borrow_mut()
                    .map_err(|_| ViewerHostError::UnsupportedCallback)?
                    .inspection
                    .take()
                    .ok_or(ViewerHostError::ForeignCheckpoint)?;
                entry.retained_inspection = Some(inspection);
                let terminal = entry
                    .retained_inspection
                    .as_ref()
                    .unwrap()
                    .terminal()
                    .ok_or(ViewerHostError::ForeignCheckpoint)?;
                let stored = match &terminal.outcome {
                    Ok(PendingOutput::Inspected(stored)) if !terminal.cancellation_requested => {
                        stored.clone()
                    }
                    _ => return Err(blocked("Original inspect terminal unavailable")),
                };
                let generation = NonZeroU64::new(entry.ticket.generation())
                    .ok_or(ViewerHostError::ForeignCheckpoint)?;
                let input = ProductProposalMountInput {
                    document: self
                        .document
                        .clone()
                        .ok_or(ViewerHostError::ForeignCheckpoint)?,
                    turn: turn.clone(),
                    frame: self
                        .product_frame
                        .clone()
                        .ok_or(ViewerHostError::ForeignCheckpoint)?,
                    selected: entry.request.selected.as_ref().clone(),
                    stored,
                    client: self.client.clone(),
                    execution,
                    context,
                    namespace: format!("product-live-editor-{generation}"),
                    generation,
                    labels: ProductWindowLabels::korean(),
                    queue: None,
                    navigation: Some(
                        self.cards
                            .try_borrow()
                            .map_err(|_| ViewerHostError::UnsupportedCallback)?
                            .navigation
                            .clone(),
                    ),
                };
                match mount_current_proposal(input, Some(entry.gate.clone())) {
                    Err(failure) => {
                        let error = plugin(PluginError::new(
                            PluginErrorCode::Adapter,
                            failure.diagnostic.clone(),
                        ));
                        entry.mount_failure = Some(failure);
                        return Err(error);
                    }
                    Ok(mounted) => {
                        // The common mount issued proof before any fallible active-host binding.
                        entry.receipt = Some(mounted.receipt.clone());
                        entry.proof = mounted.proof;
                        entry.proof_error = mounted.proof_error;
                        effects.committed = Some(mounted.receipt);
                        effects.redraw = true;
                        if let Err(bridge) = self.bind_active_card(mounted.bridge) {
                            entry.rejected_bridge = Some(bridge);
                            return Err(blocked(
                                "Committed original live bridge binding was rejected",
                            ));
                        }
                        self.cards
                            .try_borrow_mut()
                            .map_err(|_| ViewerHostError::UnsupportedCallback)?
                            .active_task = Some(task);
                        if entry.proof.is_none() {
                            #[cfg(test)]
                            self.live_fixture_entry_commit(&entry);
                            return Err(blocked(
                                "Original receipt retained; proof issuance failed, explicit close required",
                            ));
                        }
                        entry.phase = LiveEntryPhase::AwaitingArm;
                        diagnostic::opening(&entry, None);
                        #[cfg(test)]
                        self.live_fixture_entry_commit(&entry);
                    }
                }
            }
            if entry.phase == LiveEntryPhase::AwaitingArm {
                // Count only entry/return arm notifications; the original inspect continuation
                // was already sent through the same unwrapped startup notifier.
                #[cfg(test)]
                self.install_live_fixture_notifier();
                let document = self
                    .document
                    .clone()
                    .ok_or(ViewerHostError::ForeignCheckpoint)?;
                let proof = entry
                    .proof
                    .clone()
                    .ok_or(ViewerHostError::ForeignCheckpoint)?;
                if let Err(error) = document.validate_retained_commit(&proof) {
                    entry.proof_error.get_or_insert_with(|| Arc::new(error));
                    return Err(ViewerHostError::ForeignCheckpoint);
                }
                let armed = self
                    .route_observation
                    .as_ref()
                    .ok_or(ViewerHostError::UnsupportedCallback)
                    .and_then(|observer| {
                        #[cfg(test)]
                        let probe = super::super::viewer_native_probe::current()
                            .filter(|probe| probe.is_live());
                        #[cfg(test)]
                        let held = if probe
                            .as_ref()
                            .is_some_and(|probe| probe.take_live_arm_busy())
                        {
                            observer.try_borrow_mut().ok()
                        } else {
                            None
                        };
                        let result = observer
                            .try_borrow_mut()
                            .map_err(|_| ViewerHostError::UnsupportedCallback)
                            .and_then(|mut observer| {
                                observer
                                    .arm_retained_studio_route(
                                        entry.ticket.clone(),
                                        document,
                                        proof,
                                        task,
                                    )
                                    .map_err(ViewerHostError::Navigation)
                            });
                        #[cfg(test)]
                        if held.is_some()
                            && let Some(probe) = probe
                        {
                            probe.live_arm_busy(result.is_err());
                        }
                        result
                    });
                match armed {
                    Ok(()) => {
                        entry.error = None;
                        entry.phase = LiveEntryPhase::AwaitingFrame;
                        diagnostic::opening(&entry, None);
                        if let Some(error) = self.route_continuation() {
                            entry.wake_error.get_or_insert(error);
                        }
                        effects.redraw = true;
                    }
                    Err(error) => {
                        entry.first_error.get_or_insert_with(|| error.clone());
                        entry.error = Some(error);
                        return Ok(()); // Same proof may rearm after input/turn; never replay receipt.
                    }
                }
            }
            if entry.phase == LiveEntryPhase::AwaitingFrame {
                let observed = self
                    .route_observation
                    .as_ref()
                    .and_then(|observer| observer.try_borrow().ok())
                    .is_some_and(|observer| {
                        observer.state.phase() == ShellPhase::Observed
                            && observer
                                .state
                                .observed_route()
                                .is_some_and(|route| route.as_str() == task.route_id())
                    });
                if !observed {
                    return Ok(());
                }
                // Finalize the actual observation before the next projection invalidates proof.
                entry.observed = true;
                entry.phase = LiveEntryPhase::Activating;
                diagnostic::opening(&entry, None);
            }
            if entry.phase == LiveEntryPhase::Activating {
                let index = entry
                    .request
                    .selected
                    .target
                    .checked_item_index()
                    .map_err(|error| ViewerHostError::Sdk(Arc::new(error)))?
                    .ok_or(ViewerHostError::ForeignCheckpoint)?;
                let mut cards = self
                    .cards
                    .try_borrow_mut()
                    .map_err(|_| ViewerHostError::UnsupportedCallback)?;
                let active = cards
                    .active
                    .as_mut()
                    .ok_or(ViewerHostError::ForeignCheckpoint)?;
                if !active.projection_pending() {
                    // Reproject this same explicit selection; no SDK request or fallback item.
                    active.select_item(index).map_err(|error| {
                        plugin(PluginError::new(
                            PluginErrorCode::Adapter,
                            format!("{error:?}"),
                        ))
                    })?;
                }
                entry.gate.set(true);
                let activated = active
                    .after_dispatch(turn.clone())
                    .map_err(|error| ViewerHostError::Card(Arc::new(error)))?;
                if activated.committed.is_some() {
                    entry.phase = LiveEntryPhase::Active;
                    diagnostic::opening(&entry, None);
                    #[cfg(test)]
                    if let Some(probe) =
                        super::super::viewer_native_probe::current().filter(|probe| probe.is_live())
                    {
                        let guards = active.navigation_guards();
                        probe.chooser_active_identity(
                            &entry.request.selected,
                            active.review().stored(),
                            active.review().document_selection(),
                        );
                        probe.live_host_state(
                            true,
                            guards.dirty,
                            guards.pending,
                            entry.observed,
                            entry.gate.get(),
                            entry.wake_error.as_deref(),
                        );
                        probe.live_error_state(entry.first_error.as_ref(), entry.error.as_ref());
                    }
                }
                effects = activated;
            }
            Ok(())
        })();
        if let Err(error) = &result {
            entry.fail(error.clone());
        }
        // No fallible operation after putting the exact same original carrier back.
        self.cards.borrow_mut().live_opening = Some(entry);
        if effects.committed.is_some() {
            return Ok(Some(effects));
        }
        result.map(|()| Some(effects))
    }
}

#[cfg(test)]
mod tests;

use super::codec::{self, READS, ReadInput};
use crate::application::{
    api::RequestContext,
    sdk_execution::{ExecutionAttachment, ExecutionHandle},
};
use crate::assembly::meari::viewer_read_actions::{
    AckClaim, LocalDrain, ReadActions, ReadIntent, ReadKind,
};
use kinekino_ui_plugins::infrastructure::typed_async_sdk_plugin::{
    AsyncSdkPump, TerminalAcceptance,
};
use meari::application::host_callbacks::LocalDispatchError;
use meari::{
    application::{
        host_callbacks::{CallbackBindings, HostCallbacks, LocalActionKey, PendingCallback},
        plugins::*,
    },
    domain::{
        build::{ContentHash, TargetKind},
        host_callbacks::CallbackSignature,
    },
};
use std::time::{Duration, Instant};
use std::{cell::RefCell, rc::Rc, sync::Arc};
struct AckRetry {
    generation: u64,
    ticket: PluginTicket,
    failures: usize,
    eligible: Instant,
}
type LiveEntryHandoff = (
    crate::assembly::meari::viewer_host::LiveEntryRequest,
    Option<super::chooser::PreparedSelection>,
);

/// Stored outside the consuming document constructor, including after any late export failure.
pub struct ViewerCallbacks {
    chooser: Option<super::chooser::Chooser>,
    jobs_entry_key: Option<LocalActionKey>,
    jobs_entry_pending: Option<PendingCallback>,
    actions: Option<ReadActions>,
    selection: Option<super::selection::ViewerSelection>,
    live_selection: Option<super::selection::LiveSelection>,
    local_error: Option<PluginError>,
    ack_retries: Vec<AckRetry>,
    pub projection: RefCell<crate::assembly::meari::viewer_read_projection::ReadProjection>,
    execution: ExecutionHandle,
    context: RequestContext,
    scope: Option<PluginScope>,
    attachment: Option<Arc<ExecutionAttachment>>,
    registry: Option<Rc<RefCell<PluginRegistry>>>,
    registration: Option<RegistrationHandle>,
    callbacks: Option<HostCallbacks>,
    pump: Option<AsyncSdkPump>,
    stopping: bool,
    invalidated: bool,
    released: bool,
}
fn error(code: PluginErrorCode, message: &str) -> PluginError {
    PluginError::new(code, message)
}
fn busy() -> PluginError {
    error(
        PluginErrorCode::Busy,
        "Viewer callback owner or registry is borrowed",
    )
}
impl ViewerCallbacks {
    pub fn new(execution: ExecutionHandle, context: RequestContext) -> Self {
        Self {
            chooser: None,
            jobs_entry_key: None,
            jobs_entry_pending: None,
            actions: None,
            selection: None,
            live_selection: None,
            local_error: None,
            ack_retries: Vec::new(),
            projection: RefCell::new(Default::default()),
            execution,
            context,
            scope: None,
            attachment: None,
            registry: None,
            registration: None,
            callbacks: None,
            pump: None,
            stopping: false,
            invalidated: false,
            released: false,
        }
    }
    pub fn scope(&self) -> Option<&PluginScope> {
        self.scope.as_ref()
    }
    /// Trusted assembly opt-in, before the original owner exports its one read profile.
    pub fn configure_proposal_chooser(
        &mut self,
        query: crate::application::proposals::list::ListProposals,
    ) -> Result<(), PluginError> {
        if self.scope.is_some() || self.stopping || self.chooser.is_some() {
            return Err(error(
                PluginErrorCode::Stale,
                "Chooser requires the original unexported owner",
            ));
        }
        self.chooser = Some(super::chooser::Chooser::new(query)?);
        Ok(())
    }
    fn read_profile(&self) -> codec::ReadProfile {
        if self.chooser.is_some() {
            codec::ReadProfile::ProposalChooser
        } else {
            codec::ReadProfile::Default
        }
    }
    pub fn chooser_phase(&self) -> Option<super::chooser::ChooserPhase> {
        self.chooser.as_ref().map(super::chooser::Chooser::phase)
    }
    pub fn chooser_documents(&self) -> &[super::chooser::DocumentChoice] {
        self.chooser
            .as_ref()
            .map_or(&[], super::chooser::Chooser::documents)
    }
    pub fn chooser_items(&self) -> &[super::chooser::ItemChoice] {
        self.chooser
            .as_ref()
            .map_or(&[], super::chooser::Chooser::items)
    }
    pub fn chooser_prepared(&self) -> Option<&super::chooser::PreparedSelection> {
        self.chooser
            .as_ref()
            .and_then(super::chooser::Chooser::prepared)
    }
    pub fn chooser_matches_prepared(&self, candidate: &super::chooser::PreparedSelection) -> bool {
        !self.stopping
            && self
                .chooser
                .as_ref()
                .is_some_and(|chooser| chooser.matches_prepared(candidate))
    }
    pub fn chooser_failure(&self) -> Option<&PluginError> {
        self.chooser
            .as_ref()
            .and_then(super::chooser::Chooser::failure)
    }
    pub fn chooser_terminal(&self) -> Option<&TerminalAcceptance> {
        self.chooser
            .as_ref()
            .and_then(super::chooser::Chooser::retained)
    }
    /// Full typed values are available to the trusted host, never renderer input authority.
    pub fn chooser_page(&self) -> Option<&crate::application::proposals::list::ProposalListPage> {
        self.chooser
            .as_ref()
            .and_then(super::chooser::Chooser::page)
    }
    pub fn chooser_document(&self) -> Option<&crate::application::proposals::StoredProposal> {
        self.chooser
            .as_ref()
            .and_then(super::chooser::Chooser::document)
    }
    /// Register only this original owner's currently prepared value; renderer receives a key.
    pub fn register_prepared_open(&mut self, scope: &PluginScope) -> Result<String, PluginError> {
        if self.stopping || self.scope.as_ref() != Some(scope) {
            return Err(error(
                PluginErrorCode::Stale,
                "Prepared Open owner is foreign or closing",
            ));
        }
        if self.has_jobs_entry() || self.has_live_entry() {
            return Err(error(
                PluginErrorCode::Busy,
                "Prepared Open owner is occupied",
            ));
        }
        let prepared = self.chooser_prepared().cloned().ok_or_else(|| {
            error(
                PluginErrorCode::Stale,
                "No acknowledged current prepared selection",
            )
        })?;
        if prepared.scope != *scope || !self.chooser_matches_prepared(&prepared) {
            return Err(error(
                PluginErrorCode::Stale,
                "Prepared Open carrier changed",
            ));
        }
        self.live_selection
            .as_mut()
            .ok_or_else(|| {
                error(
                    PluginErrorCode::MissingCapability,
                    "Prepared Open binding is not exported",
                )
            })?
            .register_prepared(prepared)
    }
    fn retire_prepared_open(&mut self) {
        if let Some(choices) = &mut self.live_selection {
            choices.retire_prepared();
        }
    }
    pub fn set_chooser_query(
        &mut self,
        scope: &PluginScope,
        query: crate::application::proposals::list::ListProposals,
    ) -> Result<(), PluginError> {
        if self.stopping
            || self.scope.as_ref() != Some(scope)
            || self.has_jobs_entry()
            || self.has_live_entry()
        {
            return Err(error(
                PluginErrorCode::Stale,
                "Chooser query scope changed or closing",
            ));
        }
        self.chooser
            .as_mut()
            .ok_or_else(|| error(PluginErrorCode::MissingCapability, "Chooser is not enabled"))?
            .set_query(query)?;
        self.retire_prepared_open();
        Ok(())
    }
    /// Cancellation retains the same ticket until its real terminal and owner ACK.
    pub fn cancel_chooser(
        &mut self,
        scope: &PluginScope,
    ) -> Result<Option<CancelReceipt>, PluginError> {
        if self.scope.as_ref() != Some(scope) {
            return Err(error(
                PluginErrorCode::Stale,
                "Chooser cancellation scope changed",
            ));
        }
        let chooser = self
            .chooser
            .as_mut()
            .ok_or_else(|| error(PluginErrorCode::MissingCapability, "Chooser is not enabled"))?;
        let receipt = if let Some(ticket) = chooser.active_ticket() {
            Some(
                self.registry
                    .as_ref()
                    .ok_or_else(busy)?
                    .try_borrow_mut()
                    .map_err(|_| busy())?
                    .cancel(ticket)?,
            )
        } else {
            None
        };
        chooser.cancel_requested();
        self.retire_prepared_open();
        Ok(receipt)
    }
    pub fn queue_chooser(
        &mut self,
        scope: &PluginScope,
        pending: &PendingCallback,
    ) -> Result<(), LocalDispatchError> {
        if self.stopping
            || self.scope.as_ref() != Some(scope)
            || self.has_jobs_entry()
            || self.has_live_entry()
        {
            return Err(error(
                PluginErrorCode::Stale,
                "Chooser scope is closing, foreign or changing route",
            )
            .into());
        }
        let invocation = self.callbacks.as_ref().ok_or_else(busy)?.dispatch_local(
            &pending.handle,
            pending.invocation.arguments.clone(),
            scope,
        )?;
        self.chooser
            .as_mut()
            .ok_or_else(|| error(PluginErrorCode::MissingCapability, "Chooser is not enabled"))?
            .enqueue(&invocation, pending)?;
        self.retire_prepared_open();
        Ok(())
    }
    pub fn drain_chooser(&mut self, scope: &PluginScope) -> Result<(), PluginError> {
        if self.stopping
            || self.scope.as_ref() != Some(scope)
            || self.has_jobs_entry()
            || self.has_live_entry()
        {
            return Err(error(
                PluginErrorCode::Stale,
                "Chooser scope is closing, foreign or changing route",
            ));
        }
        let mut registry = self
            .registry
            .as_ref()
            .ok_or_else(busy)?
            .try_borrow_mut()
            .map_err(|_| busy())?;
        self.chooser
            .as_mut()
            .ok_or_else(|| error(PluginErrorCode::MissingCapability, "Chooser is not enabled"))?
            .drain(
                self.callbacks.as_ref().ok_or_else(busy)?,
                scope,
                &mut registry,
                self.registration.as_ref().ok_or_else(busy)?,
            )
    }
    pub(crate) fn original_execution(&self) -> ExecutionHandle {
        self.execution.clone()
    }
    pub(crate) fn original_context(&self) -> RequestContext {
        self.context.clone()
    }
    pub(crate) fn configure_live_choices(
        &mut self,
        choices: crate::assembly::meari::viewer_host::TrustedLiveChoices,
    ) -> Result<(), PluginError> {
        if self.scope.is_some()
            || self.stopping
            || self.selection.is_some()
            || self.live_selection.is_some()
        {
            return Err(error(
                PluginErrorCode::Stale,
                "Live choices require the original unexported owner",
            ));
        }
        self.live_selection = Some(super::selection::LiveSelection::new(choices));
        Ok(())
    }
    pub(crate) fn has_live_entry(&self) -> bool {
        self.live_selection
            .as_ref()
            .is_some_and(|choices| choices.request().is_some())
    }
    pub(crate) fn queue_live_entry(
        &mut self,
        scope: &PluginScope,
        pending: &PendingCallback,
    ) -> Result<meari_native::infrastructure::native_host::IntentAdmission<PluginError>, PluginError>
    {
        use meari_native::infrastructure::native_host::IntentAdmission;
        if self.stopping || self.scope.as_ref() != Some(scope) || self.has_jobs_entry() {
            return Ok(IntentAdmission::Rejected(error(
                PluginErrorCode::Closing,
                "Live source is closing or another route is pending",
            )));
        }
        let invocation = self
            .callbacks
            .as_ref()
            .ok_or_else(busy)?
            .dispatch_local(&pending.handle, pending.invocation.arguments.clone(), scope)
            .map_err(|error| match error {
                LocalDispatchError::Invalid(error) => error,
                LocalDispatchError::NotLocal => self::error(
                    PluginErrorCode::Stale,
                    "Live entry is not the original local callback",
                ),
            })?;
        // Revalidate the retained original callback as well; a new lease cannot revive it.
        if self.has_live_entry() {
            self.live_entry_handoff(scope)?;
        }
        let current = self.chooser_prepared().cloned();
        let choices = self
            .live_selection
            .as_mut()
            .ok_or_else(|| error(PluginErrorCode::Stale, "No trusted live choices configured"))?;
        Ok(choices.admit(&invocation, pending, current.as_ref()))
    }
    /// Inspect the retained original intent without consuming it before route admission.
    pub(crate) fn live_entry_request(
        &self,
        scope: &PluginScope,
    ) -> Result<Option<crate::assembly::meari::viewer_host::LiveEntryRequest>, PluginError> {
        let Some((request, prepared)) = self.live_entry_handoff(scope)? else {
            return Ok(None);
        };
        if prepared.is_some() {
            return Err(error(
                PluginErrorCode::MissingCapability,
                "Prepared Open awaits the original route-owner handoff integration",
            ));
        }
        Ok(Some(request))
    }
    pub(crate) fn live_entry_handoff(
        &self,
        scope: &PluginScope,
    ) -> Result<Option<LiveEntryHandoff>, PluginError> {
        if self.stopping || self.scope.as_ref() != Some(scope) {
            return Err(error(PluginErrorCode::Stale, "Live source scope changed"));
        }
        let Some(choices) = &self.live_selection else {
            return Ok(None);
        };
        let Some(request) = choices.request() else {
            return Ok(None);
        };
        let invocation = self
            .callbacks
            .as_ref()
            .ok_or_else(busy)?
            .dispatch_local(
                &request.pending.handle,
                request.pending.invocation.arguments.clone(),
                scope,
            )
            .map_err(|error| match error {
                LocalDispatchError::Invalid(error) => error,
                LocalDispatchError::NotLocal => {
                    self::error(PluginErrorCode::Stale, "Retained live input is not local")
                }
            })?;
        let prepared = choices.validate_pending(&invocation, self.chooser_prepared())?;
        Ok(Some((request.clone(), prepared)))
    }
    /// The future route owner must store its ticket/request/carrier before returning Ok.
    /// Err retains this same intent. No fallible work follows a successful ownership transfer.
    pub fn commit_live_entry_handoff<R, E>(
        &mut self,
        scope: &PluginScope,
        commit: impl FnOnce(
            Arc<crate::application::launch::native_proposal::NativeProposalSelection>,
            PendingCallback,
            Option<super::chooser::PreparedSelection>,
        ) -> Result<R, E>,
    ) -> Result<Result<R, E>, PluginError> {
        if self.has_jobs_entry() {
            return Err(error(
                PluginErrorCode::Busy,
                "Original Jobs intent is retained",
            ));
        }
        let (request, prepared) = self
            .live_entry_handoff(scope)?
            .ok_or_else(|| error(PluginErrorCode::Stale, "No original Open intent"))?;
        let result = commit(request.selected, request.pending, prepared);
        if result.is_ok() {
            self.stop_admission();
        }
        Ok(result)
    }
    pub(crate) fn consume_live_entry(&mut self) {
        if let Some(choices) = &mut self.live_selection {
            choices.retire();
        }
    }
    /// Original product assembly supplies a typed project before the first export.
    pub fn configure_card_choices(
        &mut self,
        project: crate::domain::projects::ProjectRevision,
        generation: std::num::NonZeroU64,
        capacity: std::num::NonZeroUsize,
    ) -> Result<(), PluginError> {
        if self.scope.is_some()
            || self.stopping
            || self.selection.is_some()
            || self.live_selection.is_some()
        {
            return Err(error(
                PluginErrorCode::Stale,
                "Viewing choices are already bound or closing",
            ));
        }
        self.selection = Some(super::selection::ViewerSelection::new(
            project, generation, capacity,
        )?);
        Ok(())
    }
    pub fn card_selection_queue(&self) -> Option<Rc<RefCell<crate::presentation::web_ui_adapter::proposals::card_selection_queue::CardSelectionQueue>>>{
        self.selection
            .as_ref()
            .and_then(super::selection::ViewerSelection::queue)
    }
    /// Separate explicit edit entry; legacy configure_card_choices keeps its readonly binding.
    pub fn configure_captured_edit_choices(
        &mut self,
        stored: crate::domain::projects::StoredProjectRevision,
        generation: std::num::NonZeroU64,
        capacity: std::num::NonZeroUsize,
    ) -> Result<(), PluginError> {
        if self.scope.is_some()
            || self.stopping
            || self.selection.is_some()
            || self.live_selection.is_some()
        {
            return Err(error(
                PluginErrorCode::Stale,
                "Captured edit choices are already bound or closing",
            ));
        }
        self.selection = Some(super::selection::ViewerSelection::from_stored(
            stored, generation, capacity,
        )?);
        Ok(())
    }
    pub fn queue_card_selection(
        &mut self,
        scope: &PluginScope,
        pending: &PendingCallback,
    ) -> Result<
        Option<meari_native::infrastructure::native_host::IntentAdmission<PluginError>>,
        LocalDispatchError,
    > {
        let Some(selection) = &mut self.selection else {
            return Ok(None);
        };
        if self.stopping || self.scope.as_ref() != Some(scope) {
            return Err(error(
                PluginErrorCode::Stale,
                "Viewing choices are closing or foreign",
            )
            .into());
        }
        let invocation = match self.callbacks.as_ref().ok_or_else(busy)?.dispatch_local(
            &pending.handle,
            pending.invocation.arguments.clone(),
            scope,
        ) {
            Ok(invocation) => invocation,
            Err(LocalDispatchError::NotLocal) => return Ok(None),
            Err(error) => return Err(error),
        };
        Ok(selection.admit(invocation))
    }
    pub fn ack_claim(&self, kind: ReadKind) -> Result<AckClaim, PluginError> {
        let tickets = self.tickets()?;
        let retry = self
            .ack_retries
            .iter()
            .find(|retry| {
                retry.ticket.capability() == kind.command()
                    && retry.failures >= 3
                    && retry.generation != u64::MAX
                    && tickets.contains(&retry.ticket)
            })
            .ok_or_else(|| error(PluginErrorCode::Stale, "No exhausted original read ticket"))?;
        Ok(AckClaim {
            ticket: retry.ticket.clone(),
            generation: retry.generation,
        })
    }
    fn resume_claim(&mut self, claim: &AckClaim) -> Result<(), PluginError> {
        let retry = self
            .ack_retries
            .iter()
            .find(|retry| {
                retry.ticket == claim.ticket
                    && retry.generation == claim.generation
                    && retry.failures >= 3
            })
            .ok_or_else(|| {
                error(
                    PluginErrorCode::Stale,
                    "Read retry claim is stale or already consumed",
                )
            })?;
        if retry.generation == u64::MAX {
            return Err(error(PluginErrorCode::Stale, "ACK generation exhausted"));
        }
        self.retry_read_acceptance(&claim.ticket)
    }
    fn native_ack_claim(&self) -> Result<AckClaim, PluginError> {
        let tickets = self.tickets()?;
        let retry = self
            .ack_retries
            .iter()
            .find(|retry| {
                crate::assembly::meari::viewer_read_projection::is_native_control(
                    retry.ticket.capability(),
                ) && retry.failures >= 3
                    && retry.generation != u64::MAX
                    && tickets.contains(&retry.ticket)
            })
            .ok_or_else(|| {
                error(
                    PluginErrorCode::Stale,
                    "No exhausted original native control ticket",
                )
            })?;
        Ok(AckClaim {
            ticket: retry.ticket.clone(),
            generation: retry.generation,
        })
    }
    fn catalog_ack_claim(&self) -> Result<AckClaim, PluginError> {
        let tickets = self.tickets()?;
        let retry = self
            .ack_retries
            .iter()
            .find(|retry| {
                super::catalog::is_command(retry.ticket.capability())
                    && retry.failures >= 3
                    && retry.generation != u64::MAX
                    && tickets.contains(&retry.ticket)
            })
            .ok_or_else(|| {
                error(
                    PluginErrorCode::Stale,
                    "No exhausted original catalog ticket",
                )
            })?;
        Ok(AckClaim {
            ticket: retry.ticket.clone(),
            generation: retry.generation,
        })
    }
    pub fn queue_local(
        &mut self,
        scope: &PluginScope,
        pending: &PendingCallback,
    ) -> Result<(), LocalDispatchError> {
        if self.stopping {
            return Err(error(PluginErrorCode::Stale, "Read owner is closing").into());
        }
        let invocation = self.callbacks.as_ref().ok_or_else(busy)?.dispatch_local(
            &pending.handle,
            pending.invocation.arguments.clone(),
            scope,
        )?;
        let intent = self
            .actions
            .as_ref()
            .and_then(|actions| actions.intent(&invocation))
            .ok_or_else(|| error(PluginErrorCode::Stale, "Foreign local read key"))?;
        let claim = match intent {
            ReadIntent::RetryAck(kind) => Some(self.ack_claim(kind)?),
            ReadIntent::RetryNativeAck => Some(self.native_ack_claim()?),
            ReadIntent::RetryCatalogAck => Some(self.catalog_ack_claim()?),
            ReadIntent::RefreshProjection
            | ReadIntent::ReleaseCatalog
            | ReadIntent::SetNativeSeekDraft
            | ReadIntent::SetNativeVolumeDraft
            | ReadIntent::SetNativeSeekRangeDraft
            | ReadIntent::SetNativeVolumeRangeDraft => None,
            ReadIntent::WindowMinimize
            | ReadIntent::WindowToggleMaximize
            | ReadIntent::WindowUserClose => None,
            ReadIntent::ToggleFullscreen
            | ReadIntent::ToggleEpisodes
            | ReadIntent::ToggleDiagnostics => None,
        };
        self.actions
            .as_mut()
            .ok_or_else(busy)?
            .enqueue(pending, intent, claim)?;
        Ok(())
    }
    /// One retained local route intent, never an SDK request or backend configuration.
    pub(crate) fn queue_jobs_entry(
        &mut self,
        scope: &PluginScope,
        pending: &PendingCallback,
    ) -> Result<bool, LocalDispatchError> {
        if pending.invocation.prop != "onOpenJobs" {
            return Ok(false);
        }
        if self.stopping || self.scope.as_ref() != Some(scope) {
            return Err(error(
                PluginErrorCode::Stale,
                "Jobs entry scope is closing or foreign",
            )
            .into());
        }
        let invocation = self.callbacks.as_ref().ok_or_else(busy)?.dispatch_local(
            &pending.handle,
            pending.invocation.arguments.clone(),
            scope,
        )?;
        if !self
            .jobs_entry_key
            .as_ref()
            .is_some_and(|key| invocation.matches_key(key))
        {
            return Err(error(PluginErrorCode::Stale, "Foreign Jobs entry key").into());
        }
        if self.jobs_entry_pending.is_some() {
            return Err(busy().into());
        }
        self.jobs_entry_pending = Some(PendingCallback {
            handle: pending.handle.clone(),
            invocation: pending.invocation.clone(),
        });
        Ok(true)
    }
    /// Call at the original zero-callback barrier; stale/revoked intent stays retained on error.
    pub(crate) fn take_jobs_entry(
        &mut self,
        scope: &PluginScope,
    ) -> Result<Option<PendingCallback>, PluginError> {
        let Some(pending) = self.jobs_entry_pending.as_ref() else {
            return Ok(None);
        };
        if self.stopping || self.scope.as_ref() != Some(scope) {
            return Err(error(
                PluginErrorCode::Stale,
                "Jobs entry scope is closing or foreign",
            ));
        }
        let invocation = self
            .callbacks
            .as_ref()
            .ok_or_else(busy)?
            .dispatch_local(&pending.handle, pending.invocation.arguments.clone(), scope)
            .map_err(|failure| match failure {
                LocalDispatchError::Invalid(error) => error,
                LocalDispatchError::NotLocal => {
                    error(PluginErrorCode::Stale, "Jobs entry changed target")
                }
            })?;
        if !self
            .jobs_entry_key
            .as_ref()
            .is_some_and(|key| invocation.matches_key(key))
        {
            return Err(error(PluginErrorCode::Stale, "Jobs entry key changed"));
        }
        Ok(self.jobs_entry_pending.take())
    }
    /// Revalidate original callbacks after document borrows are released. Process at most four.
    pub fn drain_local(&mut self, scope: &PluginScope) -> Result<LocalDrain, PluginError> {
        let mut report = LocalDrain::default();
        for _ in 0..4 {
            let Some(action) = self.actions.as_mut().and_then(ReadActions::pop) else {
                break;
            };
            if self.stopping {
                self.local_error = Some(error(PluginErrorCode::Stale, "Read owner is closing"));
                continue;
            }
            let invocation = self.callbacks.as_ref().ok_or_else(busy)?.dispatch_local(
                &action.callback.handle,
                action.callback.invocation.arguments.clone(),
                scope,
            );
            let intent = match invocation {
                Ok(invocation) => self
                    .actions
                    .as_ref()
                    .and_then(|actions| actions.intent(&invocation)),
                Err(LocalDispatchError::Invalid(error)) if error.code == PluginErrorCode::Busy => {
                    self.actions
                        .as_mut()
                        .ok_or_else(busy)?
                        .restore(action)
                        .map_err(|_| busy())?;
                    return Err(error);
                }
                Err(LocalDispatchError::Invalid(error)) => {
                    self.local_error = Some(error);
                    continue;
                }
                Err(LocalDispatchError::NotLocal) => {
                    self.local_error = Some(error(
                        PluginErrorCode::InvalidPayload,
                        "Queued read action changed target",
                    ));
                    continue;
                }
            };
            if intent != Some(action.intent) {
                self.local_error = Some(error(PluginErrorCode::Stale, "Local key changed"));
                continue;
            }
            let result = match action.intent {
                ReadIntent::WindowMinimize
                | ReadIntent::WindowToggleMaximize
                | ReadIntent::WindowUserClose => {
                    use crate::assembly::meari::viewer_host::window_ui::CaptionIntent;
                    let mut projection = self.projection.try_borrow_mut().map_err(|_| busy())?;
                    if projection.window_ui.caption_requested.is_some() {
                        Err(busy())
                    } else {
                        projection.window_ui.caption_requested = Some(match action.intent {
                            ReadIntent::WindowMinimize => CaptionIntent::Minimize,
                            ReadIntent::WindowToggleMaximize => CaptionIntent::ToggleMaximize,
                            ReadIntent::WindowUserClose => CaptionIntent::UserClose,
                            _ => unreachable!(),
                        });
                        projection.dirty = true;
                        Ok(())
                    }
                }
                ReadIntent::ToggleFullscreen
                | ReadIntent::ToggleEpisodes
                | ReadIntent::ToggleDiagnostics => {
                    let mut projection = self.projection.try_borrow_mut().map_err(|_| busy())?;
                    if action.intent == ReadIntent::ToggleDiagnostics {
                        projection.diagnostics_expanded = !projection.diagnostics_expanded;
                    } else if action.intent == ReadIntent::ToggleEpisodes {
                        projection.window_ui.episodes_open = !projection.window_ui.episodes_open;
                    } else {
                        projection.window_ui.toggle_requested = true;
                    }
                    projection.dirty = true;
                    Ok(())
                }
                ReadIntent::SetNativeSeekDraft
                | ReadIntent::SetNativeVolumeDraft
                | ReadIntent::SetNativeSeekRangeDraft
                | ReadIntent::SetNativeVolumeRangeDraft => {
                    match &action.callback.invocation.arguments {
                        meari::domain::host_callbacks::CallbackArguments::Text(value)
                            if value.len() <= 64 =>
                        {
                            let mut projection =
                                self.projection.try_borrow_mut().map_err(|_| busy())?;
                            let result = if matches!(
                                action.intent,
                                ReadIntent::SetNativeSeekRangeDraft
                                    | ReadIntent::SetNativeVolumeRangeDraft
                            ) {
                                projection.set_native_range_draft(
                                    action.intent == ReadIntent::SetNativeSeekRangeDraft,
                                    value,
                                )
                            } else {
                                projection.set_native_draft(
                                    action.intent == ReadIntent::SetNativeSeekDraft,
                                    value.clone(),
                                )
                            };
                            if result.is_ok()
                                && !matches!(
                                    action.intent,
                                    ReadIntent::SetNativeSeekRangeDraft
                                        | ReadIntent::SetNativeVolumeRangeDraft
                                )
                            {
                                report.refreshed = true;
                            }
                            result
                        }
                        _ => Err(error(
                            PluginErrorCode::InvalidPayload,
                            "Native control draft exceeds UI input budget",
                        )),
                    }
                }
                ReadIntent::RetryAck(_)
                | ReadIntent::RetryNativeAck
                | ReadIntent::RetryCatalogAck => match &action.claim {
                    Some(claim) => self.resume_claim(claim),
                    None => Err(error(PluginErrorCode::Stale, "Missing read retry claim")),
                },
                ReadIntent::ReleaseCatalog => {
                    let mut projection = self.projection.try_borrow_mut().map_err(|_| busy())?;
                    let result = projection
                        .catalog
                        .try_borrow_mut()
                        .map_err(|_| busy())?
                        .release_terminal()
                        .map_err(|error| {
                            PluginError::new(PluginErrorCode::InvalidPayload, error.message)
                        });
                    if result.is_ok() {
                        projection.dirty = true;
                        report.refreshed = true;
                    }
                    result
                }
                ReadIntent::RefreshProjection => match self.projection.try_borrow_mut() {
                    Ok(mut projection) => {
                        projection.retry_decode();
                        report.refreshed = true;
                        Ok(())
                    }
                    Err(_) => Err(busy()),
                },
            };
            if let Err(error) = result {
                if error.code == PluginErrorCode::Busy {
                    self.actions
                        .as_mut()
                        .ok_or_else(busy)?
                        .restore(action)
                        .map_err(|_| busy())?;
                    return Err(error);
                }
                self.local_error = Some(error);
            } else if matches!(
                action.intent,
                ReadIntent::RetryAck(_) | ReadIntent::RetryNativeAck | ReadIntent::RetryCatalogAck
            ) {
                report.resumed = true;
            }
            // Successfully handled intents are gone before any subsequent DOM commit.
        }
        Ok(report)
    }
    pub fn local_error(&self) -> Option<&PluginError> {
        self.local_error.as_ref()
    }
    pub fn local_queue_empty(&self) -> bool {
        self.actions.as_ref().is_none_or(ReadActions::is_empty)
    }
    pub fn local_props(&self) -> std::collections::BTreeMap<String, meari::domain::Value> {
        let mut props =
            crate::presentation::web_ui_adapter::player_shell::read_observation::local_action_props(
            );
        props.insert(
            "localInputErrorPresent".into(),
            meari::domain::Value::Bool(self.local_error.is_some()),
        );
        props.insert(
            "localInputError".into(),
            meari::domain::Value::String(
                self.local_error
                    .as_ref()
                    .map(|error| format!("최근 입력 오류 · {:?}: {}", error.code, error.message))
                    .unwrap_or_default(),
            ),
        );
        if let Some(selection) = &self.selection {
            props.extend(selection.props());
        }
        props.insert(
            "nativeAckRetryDisabled".into(),
            meari::domain::Value::Bool(self.stopping || self.native_ack_claim().is_err()),
        );
        props.insert(
            "catalogAckRetryDisabled".into(),
            meari::domain::Value::Bool(self.stopping || self.catalog_ack_claim().is_err()),
        );
        for (kind, name) in [
            (ReadKind::Capabilities, "capabilitiesRetryDisabled"),
            (ReadKind::Engine, "engineRetryDisabled"),
            (ReadKind::Status, "statusRetryDisabled"),
        ] {
            props.insert(
                name.into(),
                meari::domain::Value::Bool(self.stopping || self.ack_claim(kind).is_err()),
            );
        }
        props
    }
    pub(crate) fn configured_card_props(
        &self,
    ) -> std::collections::BTreeMap<String, meari::domain::Value> {
        self.selection
            .as_ref()
            .map(super::selection::ViewerSelection::props)
            .unwrap_or_default()
    }
    pub(crate) fn has_jobs_entry(&self) -> bool {
        self.jobs_entry_pending.is_some()
    }
    pub(crate) fn verify_jobs_entry(
        &self,
        scope: &PluginScope,
        pending: &PendingCallback,
    ) -> Result<(), PluginError> {
        if self.stopping || self.scope.as_ref() != Some(scope) {
            return Err(error(PluginErrorCode::Stale, "Viewing Jobs source retired"));
        }
        let invocation = self
            .callbacks
            .as_ref()
            .ok_or_else(busy)?
            .dispatch_local(&pending.handle, pending.invocation.arguments.clone(), scope)
            .map_err(|failure| match failure {
                LocalDispatchError::Invalid(error) => error,
                LocalDispatchError::NotLocal => {
                    error(PluginErrorCode::Stale, "Jobs intent changed target")
                }
            })?;
        if !self
            .jobs_entry_key
            .as_ref()
            .is_some_and(|key| invocation.matches_key(key))
        {
            return Err(error(PluginErrorCode::Stale, "Jobs source key changed"));
        }
        Ok(())
    }
    pub fn stop_admission(&mut self) {
        if let Some(chooser) = &mut self.chooser {
            chooser.close();
        }
        self.consume_live_entry();
        self.stopping = true;
        if self.jobs_entry_pending.take().is_some() {
            self.local_error.get_or_insert_with(|| {
                error(
                    PluginErrorCode::Stale,
                    "Local Jobs intent discarded while closing",
                )
            });
        }
        if let Some(actions) = &mut self.actions {
            if !actions.is_empty() {
                self.local_error = Some(error(
                    PluginErrorCode::Stale,
                    "Local read inputs discarded while closing",
                ));
            }
            actions.clear();
        }
    }
    pub fn export(&mut self, scope: PluginScope) -> Result<CallbackBindings, PluginError> {
        if self.stopping || self.scope.is_some() {
            return Err(error(
                PluginErrorCode::Stale,
                "Viewer root scope already exported or closing",
            ));
        }
        if self.chooser.is_some() && self.live_selection.is_none() {
            self.live_selection = Some(super::selection::LiveSelection::for_chooser());
        }
        self.scope = Some(scope.clone());
        // Retain attachment before building plugins or callback exports that can fail later.
        let attachment = Arc::new(self.execution.attach().map_err(|_| busy())?);
        self.attachment = Some(attachment.clone());
        let registry = Rc::new(RefCell::new(PluginRegistry::new(
            ContentHash::of(b"viewer-root-read-v1"),
            TargetKind::Native,
            scope.clone(),
            8,
        )?));
        self.registry = Some(registry.clone());
        let profile = self.read_profile();
        let manifest = codec::manifest_for(profile);
        let (plugin, pump) = match profile {
            codec::ReadProfile::Default => codec::adapter(attachment, self.context.clone())?,
            codec::ReadProfile::ProposalChooser => {
                codec::adapter_for(attachment, self.context.clone(), profile)?
            }
        };
        self.pump = Some(pump);
        let grants = manifest.capabilities.keys().cloned().collect();
        let registration = registry
            .try_borrow_mut()
            .map_err(|_| busy())?
            .register(Box::new(plugin), grants)?;
        self.registration = Some(registration.clone());
        self.callbacks = Some(HostCallbacks::new(scope));
        let callbacks = self.callbacks.as_mut().expect("retained callback owner");
        for (prop, command) in READS {
            let schema = manifest.capabilities[command].request_schema.clone();
            callbacks.bind(
                prop.into(),
                CallbackSignature::Action,
                registration.clone(),
                command.into(),
                move |_| PluginPayload::encode(schema.clone(), &ReadInput {}),
            )?;
        }
        {
            let projection = self.projection.try_borrow().map_err(|_| busy())?;
            super::native_playback::bind(
                callbacks,
                registration.clone(),
                &manifest,
                projection.native.clone(),
                projection.native_admission.clone(),
            )?;
        }
        self.actions = Some(ReadActions::bind(callbacks)?);
        {
            let projection = self.projection.try_borrow().map_err(|_| busy())?;
            super::catalog::bind(
                callbacks,
                registration.clone(),
                &manifest,
                projection.catalog.clone(),
                projection.native.clone(),
                projection.native_admission.clone(),
                projection.catalog_projection_fault.clone(),
            )?;
        }
        if let Some(chooser) = &mut self.chooser {
            chooser.bind(callbacks, self.scope.as_ref().ok_or_else(busy)?)?;
        }
        self.jobs_entry_key =
            Some(callbacks.bind_local("onOpenJobs".into(), CallbackSignature::Action)?);
        if let Some(choices) = &mut self.live_selection {
            choices.bind(callbacks)?;
        }
        if let Some(selection) = &mut self.selection {
            selection.bind(callbacks, self.scope.as_ref().ok_or_else(busy)?)?;
        }
        callbacks.export_bindings()
    }
    pub fn dispatch(
        &self,
        scope: &PluginScope,
        pending: &PendingCallback,
    ) -> Result<DispatchReceipt, PluginError> {
        if self.stopping || self.scope.as_ref() != Some(scope) {
            return Err(error(
                PluginErrorCode::Stale,
                "Viewer callback scope is stale or closing",
            ));
        }
        let registry = self
            .registry
            .as_ref()
            .ok_or_else(|| error(PluginErrorCode::Stale, "Viewer callback registry missing"))?;
        let callbacks = self
            .callbacks
            .as_ref()
            .ok_or_else(|| error(PluginErrorCode::Stale, "Viewer callback owner missing"))?;
        let mut registry = registry.try_borrow_mut().map_err(|_| busy())?;
        if let Some((_, command)) = READS
            .iter()
            .chain(super::native_playback::CONTROLS.iter())
            .find(|(prop, _)| *prop == pending.invocation.prop)
            && self
                .tickets()?
                .iter()
                .any(|ticket| ticket.capability() == *command)
        {
            return Err(busy());
        }
        let mut projection = self.projection.try_borrow_mut().map_err(|_| busy())?;
        let receipt = callbacks.dispatch(
            &pending.handle,
            pending.invocation.arguments.clone(),
            &mut registry,
        )?;
        projection.requested(&receipt.ticket);
        Ok(receipt)
    }
    pub(crate) fn next_automatic_step(
        &self,
        enabled: bool,
    ) -> Result<Option<super::progression::Continuation>, PluginError> {
        if !enabled || self.stopping || !self.tickets()?.is_empty() {
            return Ok(None);
        }
        let projection = self.projection.try_borrow().map_err(|_| busy())?;
        if projection.catalog_projection_fault.get() {
            return Ok(None);
        }
        if projection.panel.rows[0].phase == crate::presentation::web_ui_adapter::player_shell::read_observation::ReadPhase::NotRequested {
            return Ok(Some(super::progression::Continuation { action: super::progression::Action::ReadCapabilities, operation: "original-capabilities".into() }));
        }
        let catalog = projection.catalog.try_borrow().map_err(|_| busy())?;
        if catalog.transition_blocks_controls() {
            let native = projection.native.try_borrow().map_err(|_| busy())?;
            let Some(actual) = native.settled_observation() else {
                return Ok(None);
            };
            return Ok(catalog.automatic_transition_step(actual, enabled));
        }
        if let Some(step) = catalog.automatic_catalog_step(enabled) {
            return Ok(Some(step));
        }
        if projection.panel.rows[1].phase == crate::presentation::web_ui_adapter::player_shell::read_observation::ReadPhase::Failed { return Ok(None); }
        let native = projection.native.try_borrow().map_err(|_| busy())?;
        Ok(catalog.engine_observation_step(native.observation()))
    }
    /// Revalidate the retained operation before resolving an original revocable export.
    pub(crate) fn dispatch_automatic_step(
        &self,
        scope: &PluginScope,
        expected: &super::progression::Continuation,
        enabled: bool,
    ) -> Result<DispatchReceipt, PluginError> {
        if self.next_automatic_step(enabled)?.as_ref() != Some(expected) {
            return Err(error(
                PluginErrorCode::Stale,
                "Automatic UI operation changed",
            ));
        }
        let bindings = self.bindings()?;
        let prop = expected.action.binding();
        let handle = bindings.resolve(&meari::domain::host_callbacks::CallbackReference {
            binding: prop.into(),
            signature: meari::domain::host_callbacks::CallbackSignature::Action,
        })?;
        let pending = meari::application::host_callbacks::PendingCallback {
            handle,
            invocation: meari::domain::host_callbacks::CallbackInvocation {
                prop: prop.into(),
                arguments: meari::domain::host_callbacks::CallbackArguments::Action,
                span: meari::domain::Span::default(),
            },
        };
        // Keep this actual export live through dispatch; original SDK ticket outlives the export.
        self.dispatch(scope, &pending)
    }
    /// Bounded read-consumer transaction. Core retains the event before SDK ACK;
    /// this owner retains the returned envelope before typed projection.
    pub fn consume_reads(&mut self) -> Result<(), PluginError> {
        self.consume_reads_at(Instant::now())
    }
    pub(super) fn consume_reads_at(&mut self, now: Instant) -> Result<(), PluginError> {
        for ticket in self.tickets()?.into_iter().take(8) {
            if self.ack_retries.iter().any(|retry| {
                retry.ticket == ticket && (retry.failures >= 3 || now < retry.eligible)
            }) {
                continue;
            }
            let chooser_ticket = self
                .chooser
                .as_ref()
                .is_some_and(|chooser| chooser.owns(&ticket));
            let mut projection = if chooser_ticket {
                None
            } else {
                Some(self.projection.try_borrow_mut().map_err(|_| busy())?)
            };
            if let Some(projection) = &mut projection
                && !projection.contains(&ticket)
            {
                projection.requested(&ticket);
            }
            match self.accept_terminal(&ticket) {
                Ok(Some(acceptance)) => {
                    if acceptance.owner_ack.is_err() {
                        if let Some(retry) = self
                            .ack_retries
                            .iter_mut()
                            .find(|retry| retry.ticket == ticket)
                        {
                            retry.failures += 1;
                            retry.eligible = now + Duration::from_millis(25);
                        } else {
                            self.ack_retries.push(AckRetry {
                                generation: 0,
                                ticket: ticket.clone(),
                                failures: 1,
                                eligible: now + Duration::from_millis(25),
                            });
                        }
                    } else {
                        self.ack_retries.retain(|retry| retry.ticket != ticket);
                    }
                    if chooser_ticket {
                        if let Some(chooser) = &mut self.chooser {
                            chooser.retain(acceptance);
                        }
                    } else if let Some(projection) = &mut projection {
                        projection.retain(acceptance);
                    }
                }
                Ok(None) => {}
                Err(error) => {
                    if chooser_ticket {
                        if let Some(chooser) = &mut self.chooser {
                            chooser.fault(&error);
                        }
                    } else if let Some(projection) = &mut projection {
                        projection.fault(&ticket, &error);
                    }
                }
            }
        }
        Ok(())
    }
    pub fn next_consumer_retry(&self) -> Option<Duration> {
        let now = Instant::now();
        self.ack_retries
            .iter()
            .filter(|retry| retry.failures < 3)
            .map(|retry| {
                retry
                    .eligible
                    .saturating_duration_since(now)
                    .max(Duration::from_millis(25))
            })
            .min()
    }
    /// Explicit recovery of one original ticket after its bounded ACK attempts were exhausted.
    pub fn retry_read_acceptance(&mut self, ticket: &PluginTicket) -> Result<(), PluginError> {
        if !self.tickets()?.contains(ticket) {
            return Err(error(
                PluginErrorCode::Stale,
                "Unknown retained read ticket",
            ));
        }
        let retry = self
            .ack_retries
            .iter_mut()
            .find(|retry| retry.ticket == *ticket)
            .ok_or_else(busy)?;
        if retry.failures < 3 {
            return Err(error(PluginErrorCode::Stale, "ACK budget is not exhausted"));
        }
        retry.generation = retry
            .generation
            .checked_add(1)
            .ok_or_else(|| error(PluginErrorCode::Busy, "ACK generation exhausted"))?;
        retry.failures = 0;
        retry.eligible = Instant::now();
        Ok(())
    }
    #[cfg(test)]
    pub(crate) fn install_fault_adapter(
        &mut self,
        plugin: kinekino_ui_plugins::infrastructure::typed_async_sdk_plugin::TypedAsyncSdkPlugin,
        pump: AsyncSdkPump,
    ) -> Result<(), PluginError> {
        let registry = self.registry.as_ref().ok_or_else(busy)?;
        let mut registry = registry.try_borrow_mut().map_err(|_| busy())?;
        registry.unregister(self.registration.as_ref().ok_or_else(busy)?)?;
        let manifest = codec::manifest_for(self.read_profile());
        let registration = registry.register(
            Box::new(plugin),
            manifest.capabilities.keys().cloned().collect(),
        )?;
        let mut callbacks = HostCallbacks::new(self.scope.clone().ok_or_else(busy)?);
        for (prop, command) in READS {
            let schema = manifest.capabilities[command].request_schema.clone();
            callbacks.bind(
                prop.into(),
                CallbackSignature::Action,
                registration.clone(),
                command.into(),
                move |_| PluginPayload::encode(schema.clone(), &ReadInput {}),
            )?;
        }
        {
            let projection = self.projection.try_borrow().map_err(|_| busy())?;
            super::native_playback::bind(
                &mut callbacks,
                registration.clone(),
                &manifest,
                projection.native.clone(),
                projection.native_admission.clone(),
            )?;
        }
        self.actions = Some(ReadActions::bind(&mut callbacks)?);
        {
            let projection = self.projection.try_borrow().map_err(|_| busy())?;
            super::catalog::bind(
                &mut callbacks,
                registration.clone(),
                &manifest,
                projection.catalog.clone(),
                projection.native.clone(),
                projection.native_admission.clone(),
                projection.catalog_projection_fault.clone(),
            )?;
        }
        if let Some(chooser) = &mut self.chooser {
            chooser.bind(&mut callbacks, self.scope.as_ref().ok_or_else(busy)?)?;
        }
        self.jobs_entry_key =
            Some(callbacks.bind_local("onOpenJobs".into(), CallbackSignature::Action)?);
        if let Some(choices) = &mut self.live_selection {
            choices.bind(&mut callbacks)?;
        }
        if let Some(selection) = &mut self.selection {
            selection.bind(&mut callbacks, self.scope.as_ref().ok_or_else(busy)?)?;
        }
        self.callbacks = Some(callbacks);
        self.registration = Some(registration);
        self.pump = Some(pump);
        Ok(())
    }
    pub fn bindings(&self) -> Result<CallbackBindings, PluginError> {
        self.callbacks.as_ref().ok_or_else(busy)?.export_bindings()
    }
    pub fn tickets(&self) -> Result<Vec<PluginTicket>, PluginError> {
        self.pump
            .as_ref()
            .map_or_else(|| Ok(Vec::new()), AsyncSdkPump::tickets)
    }
    /// Core acceptance and SDK ACK. Automatic callers must use the bounded retaining consumer.
    pub fn accept_terminal(
        &self,
        ticket: &PluginTicket,
    ) -> Result<Option<TerminalAcceptance>, PluginError> {
        let registry = self
            .registry
            .as_ref()
            .ok_or_else(|| error(PluginErrorCode::Stale, "Viewer registry missing"))?;
        let mut registry = registry.try_borrow_mut().map_err(|_| busy())?;
        self.pump
            .as_ref()
            .ok_or_else(|| error(PluginErrorCode::Stale, "Viewer pump missing"))?
            .accept(&mut registry, ticket)
    }
    /// Called only after original endpoint workers have both joined. Pending tickets are retained.
    pub fn prepare_release(&mut self) -> Result<Option<Vec<PluginTicket>>, PluginError> {
        self.release_callbacks()
    }
    /// Detach this route's callbacks while retaining the original endpoint and SDK worker.
    /// The caller must stop admitting this route and retain this owner until every returned
    /// ticket has been accepted and acknowledged. A new route may attach only after `None`.
    pub fn prepare_route_release(
        &mut self,
        expected: &PluginScope,
    ) -> Result<Option<Vec<PluginTicket>>, PluginError> {
        if self.scope.as_ref() != Some(expected) {
            return Err(error(PluginErrorCode::Stale, "Viewer route scope changed"));
        }
        self.release_callbacks()
    }
    fn release_callbacks(&mut self) -> Result<Option<Vec<PluginTicket>>, PluginError> {
        if let Some(chooser) = &mut self.chooser {
            chooser.close();
        }
        self.stopping = true;
        self.consume_live_entry();
        if self.jobs_entry_pending.take().is_some() {
            self.local_error.get_or_insert_with(|| {
                error(
                    PluginErrorCode::Stale,
                    "Unaccepted Jobs intent retired with its source scope",
                )
            });
        }
        if self.released {
            return Ok(None);
        }
        if !self.invalidated {
            if let Some(callbacks) = &mut self.callbacks {
                callbacks.invalidate()?;
            }
            self.invalidated = true;
        }
        if let Some(pump) = &self.pump {
            pump.begin_close()?;
        }
        let tickets = self.tickets()?;
        if !tickets.is_empty() {
            return Ok(Some(tickets));
        }
        if let Some(registration) = &self.registration {
            self.registry
                .as_ref()
                .ok_or_else(busy)?
                .try_borrow_mut()
                .map_err(|_| busy())?
                .unregister(registration)?;
            self.registration = None;
        }
        if let Some(attachment) = &self.attachment {
            attachment.release().map_err(|_| busy())?;
        }
        self.released = true;
        Ok(None)
    }
    pub fn is_released(&self) -> bool {
        self.released
    }
    /// Reattach the original read owner after its route has fully released. The caller
    /// retains this owner on export/checked-document-commit failure and releases it again.
    /// Successful export alone is not a committed Viewing route or a rendered frame.
    pub fn resume_released_route(
        &mut self,
        expected: &PluginScope,
    ) -> Result<CallbackBindings, PluginError> {
        if self.scope.as_ref() != Some(expected) {
            return Err(error(PluginErrorCode::Stale, "Viewer resume scope changed"));
        }
        if !self.released || !self.invalidated || !self.tickets()?.is_empty() {
            return Err(busy());
        }
        // release_callbacks already invalidated the previous export, unregistered the
        // plugin and released its attachment. Keep projection and configured choices.
        self.actions = None;
        self.local_error = None;
        self.ack_retries.clear();
        self.attachment = None;
        self.registry = None;
        self.registration = None;
        self.callbacks = None;
        self.pump = None;
        self.scope = None;
        self.stopping = false;
        self.invalidated = false;
        self.released = false;
        self.export(expected.clone())
    }
}
pub fn export_callback_bindings(
    owner: Rc<RefCell<ViewerCallbacks>>,
) -> Box<dyn FnOnce(PluginScope) -> Result<CallbackBindings, String>> {
    Box::new(move |scope| {
        owner
            .try_borrow_mut()
            .map_err(|_| busy().message)?
            .export(scope)
            .map_err(|error| error.message)
    })
}

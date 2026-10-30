//! NoSelection host using the supplied SDK execution and original LocalHost shutdown owner.
mod automatic_progress;
mod automatic_wake;
mod capture_adapter;
mod capture_entry;
mod capture_evidence;
mod capture_surface;
mod cards;
mod close;
pub(crate) mod jobs;
mod live_entry;
mod route_media;
pub(crate) mod window_ui;
pub(crate) use live_entry::{LIVE_ENTRY_BINDING, LiveEntryRequest};
pub use live_entry::{LiveEntryPhase, TrustedLiveChoices};
mod mount_card;
mod mount_proposal;
mod open_card;
mod return_viewing;
mod source_navigation;
mod viewing_chooser;
use super::quarantine_surface::frame_delivery::{
    FrameOwnedResult, FrameReceiveError, FrameReceiver,
};
use crate::assembly::meari::viewer_callbacks::ViewerCallbacks;
use crate::{
    application::{api::ApplicationClientHandle, sdk_execution::ExecutionOwner},
    infrastructure::local_host::LocalHost,
};
pub use capture_adapter::CaptureAttachFailure;
pub use capture_evidence::CaptureEvidenceStatus;
pub use cards::ActiveCardBridge;
use close::{LocalEndpoint, OrderedClose, ViewerExecution};
pub use close::{ViewerCloseError, ViewerCloseProgress};
use meari::application::host_callbacks::PendingCallback;
use meari_native::infrastructure::native_host::*;
use std::{cell::RefCell, fmt, rc::Rc, sync::Arc, time::Duration};

pub struct ViewerHost {
    native_route: route_media::NativeRouteGate,
    window_ui: window_ui::WindowUiHost,
    automatic: automatic_progress::AutomaticProgress,
    registration_redraw: Rc<super::player_shell_host::registration_redraw::RegistrationRedraw>,
    transition_slot: Option<Arc<crate::application::playback_session::transition::TransitionSlot>>,
    frame: FrameReflection,
    viewing_chooser: viewing_chooser::Projection,
    document: Option<NativeDocumentHandle>,
    base_props: std::collections::BTreeMap<String, meari::domain::Value>,
    product_frame: Option<crate::presentation::web_ui_adapter::player_shell::document::product_frame::ProductFrameBlueprint>,
    read_retries: usize,
    read_admitted: bool,
    viewing_generation: std::num::NonZeroU64,
    shutdown_started: bool,
    client: ApplicationClientHandle,
    closing: OrderedClose<LocalEndpoint, ViewerExecution>,
    callbacks: Rc<RefCell<ViewerCallbacks>>,
    cards: Rc<RefCell<cards::CardRoute>>,
    jobs: Rc<RefCell<jobs::JobsLedger>>,
    route_observation: Option<Rc<RefCell<super::player_shell_host::StartRouteObservation>>>,
    identity: Rc<()>,
    last_error: Option<ViewerHostError>,
    completion: Option<Arc<dyn crate::application::sdk_execution::CompletionNotifier>>,
}
#[derive(Default)]
struct FrameReflection {
    receiver: Option<FrameReceiver>,
    result: Option<FrameOwnedResult>,
    error: Option<FrameReceiveError>,
    #[cfg(test)]
    applied: Option<FrameOwnedResult>,
}
impl FrameReflection {
    fn apply(&mut self, turn: &HostTurn, effects: &mut HostEffects) {
        #[cfg(test)]
        {
            self.applied = None;
        }
        // Keep the original callback batch barrier: do not consume a redraw/result early.
        if turn.remaining_callbacks != 0 {
            return;
        }
        let Some(receiver) = &self.receiver else {
            return;
        };
        match receiver.receive(turn) {
            Ok(update) => {
                if let Some(result) = update.result {
                    #[cfg(test)]
                    {
                        self.applied = Some(result.clone());
                    }
                    self.result = Some(result);
                    self.error = None;
                }
                effects.redraw |= update.redraw;
            }
            // Reflection failure never changes the existing SDK/ACK/commit receipt protocol.
            Err(error) => self.error = Some(error),
        }
    }
}
#[derive(Clone, Debug)]
pub enum ViewerHostError {
    ReadConsumerRetry(Duration),
    Callback(Arc<meari::application::plugins::PluginError>),
    UnsupportedCallback,
    ForeignCheckpoint,
    Navigation(crate::presentation::web_ui_adapter::player_shell::NavigationError),
    Sdk(Arc<crate::application::api::ApiError>),
    Close(ViewerCloseError),
    Card(Arc<crate::presentation::web_ui_adapter::proposals::window_bridge::PanelBridgeError>),
}
impl fmt::Display for ViewerHostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "viewer host: {self:?}")
    }
}
impl std::error::Error for ViewerHostError {}
pub struct ViewerCloseCheckpoint {
    owner: Rc<()>,
}
impl ViewerHost {
    pub(crate) fn new_with_completion(
        client: ApplicationClientHandle,
        execution: ExecutionOwner,
        endpoint: LocalHost,
        completion: Arc<dyn crate::application::sdk_execution::CompletionNotifier>,
    ) -> Self {
        let mut host = Self::new(client, execution, endpoint);
        host.completion = Some(completion.clone());
        host.jobs.borrow_mut().set_completion(completion);
        host
    }
    pub fn new(
        client: ApplicationClientHandle,
        execution: ExecutionOwner,
        endpoint: LocalHost,
    ) -> Self {
        let callbacks = Rc::new(RefCell::new(ViewerCallbacks::new(
            execution.handle(),
            Default::default(),
        )));
        let cards = Rc::new(RefCell::new(cards::CardRoute::default()));
        let jobs = Rc::new(RefCell::new(jobs::JobsLedger::default()));
        Self {
            window_ui: Default::default(),
            native_route: Default::default(),
            automatic: Default::default(),
            registration_redraw: Default::default(),
            transition_slot: None,
            frame: FrameReflection::default(),
            viewing_chooser: Default::default(),
            document: None,
            base_props: Default::default(),
            product_frame: None,
            read_retries: 3,
            read_admitted: false,
            viewing_generation: std::num::NonZeroU64::new(1).unwrap(),
            shutdown_started: false,
            client,
            closing: OrderedClose::new(
                LocalEndpoint::new(endpoint),
                ViewerExecution {
                    owner: execution,
                    callbacks: callbacks.clone(),
                    cards: cards.clone(),
                    jobs: jobs.clone(),
                },
            ),
            callbacks,
            cards,
            jobs,
            route_observation: None,
            identity: Rc::new(()),
            last_error: None,
            completion: None,
        }
    }
    pub fn client(&self) -> &ApplicationClientHandle {
        &self.client
    }
    pub(crate) fn bind_original_transition_slot(
        &mut self,
        slot: Arc<crate::application::playback_session::transition::TransitionSlot>,
    ) -> Result<(), Arc<crate::application::playback_session::transition::TransitionSlot>> {
        if self.transition_slot.is_some() {
            return Err(slot);
        }
        self.transition_slot = Some(slot);
        Ok(())
    }
    #[allow(
        dead_code,
        reason = "Opt-in owned-result installation awaits production viewer wiring"
    )]
    pub(crate) fn install_frame_receiver(
        &mut self,
        receiver: FrameReceiver,
    ) -> Result<(), FrameReceiver> {
        if self.frame.receiver.is_some() {
            return Err(receiver);
        }
        self.frame.receiver = Some(receiver);
        Ok(())
    }
    #[allow(
        dead_code,
        reason = "Opt-in owned-result installation awaits production viewer wiring"
    )]
    pub(crate) fn frame_result(&self) -> Option<&FrameOwnedResult> {
        self.frame.result.as_ref()
    }
    #[allow(
        dead_code,
        reason = "Opt-in owned-result installation awaits production viewer wiring"
    )]
    pub(crate) fn frame_receive_error(&self) -> Option<FrameReceiveError> {
        self.frame.error
    }
    pub fn bind_document(
        &mut self,
        document: NativeDocumentHandle,
        props: std::collections::BTreeMap<String, meari::domain::Value>,
    ) {
        self.seed_chooser_projection(&document, &props);
        self.document = Some(document);
        self.base_props = props;
    }
    pub fn callbacks(&self) -> Rc<RefCell<ViewerCallbacks>> {
        self.callbacks.clone()
    }
    pub(crate) fn registration_redraw(
        &self,
    ) -> Rc<super::player_shell_host::registration_redraw::RegistrationRedraw> {
        self.registration_redraw.clone()
    }
    pub(crate) fn bind_route_observation(
        &mut self,
        route: Rc<RefCell<super::player_shell_host::StartRouteObservation>>,
    ) {
        self.route_observation = Some(route);
    }
    pub fn card_opening_target(&self) -> Option<Arc<crate::presentation::web_ui_adapter::proposals::card_selection_queue::CardSelectionTarget>>{
        self.cards
            .try_borrow()
            .ok()?
            .opening
            .as_ref()
            .map(|opening| opening.request.target.clone())
    }
    pub fn card_opening_error(&self) -> Option<Arc<crate::application::api::ApiError>> {
        self.cards
            .try_borrow()
            .ok()?
            .opening
            .as_ref()?
            .api_error
            .clone()
    }
    pub fn card_mount_receipt(&self) -> Option<HostCommitReceipt> {
        self.cards
            .try_borrow()
            .ok()?
            .opening
            .as_ref()?
            .receipt
            .clone()
    }
    pub fn card_route_observation_error(
        &self,
    ) -> Option<crate::presentation::web_ui_adapter::player_shell::NavigationError> {
        self.cards.try_borrow().ok()?.opening.as_ref()?.route_error
    }
    pub fn card_mount_diagnostic(&self) -> Option<String> {
        let cards = self.cards.try_borrow().ok()?;
        let failure = cards.mount_failure.as_ref()?;
        Some(format!(
            "{}: {}",
            failure.input.selected.proposal_id, failure.diagnostic
        ))
    }
    /// Uses the actual startup observer's retained state; no second ShellState is created.
    pub fn request_selected_card_from_original_route(
        &mut self,
        guards: crate::presentation::web_ui_adapter::player_shell::NavigationGuards,
    ) -> Result<Option<crate::presentation::web_ui_adapter::proposals::card_selection_queue::CardSelectionRoute>, ViewerHostError>{
        let route = self
            .route_observation
            .clone()
            .ok_or(ViewerHostError::UnsupportedCallback)?;
        let mut route = route
            .try_borrow_mut()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?;
        let catalog = crate::presentation::web_ui_adapter::player_shell::RouteCatalog::bundled()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?;
        self.request_selected_card_route(&mut route.state, &catalog, guards)
    }
    pub fn configure_card_choices(
        &mut self,
        project: crate::domain::projects::ProjectRevision,
        generation: std::num::NonZeroU64,
        capacity: std::num::NonZeroUsize,
    ) -> Result<(), ViewerHostError> {
        self.callbacks
            .try_borrow_mut()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?
            .configure_card_choices(project, generation, capacity)
            .map_err(|error| ViewerHostError::Callback(Arc::new(error)))
    }
    pub fn card_selection_queue(&self) -> Option<Rc<RefCell<crate::presentation::web_ui_adapter::proposals::card_selection_queue::CardSelectionQueue>>>{
        self.callbacks.try_borrow().ok()?.card_selection_queue()
    }
    /// Captured rows cannot request an editable route. A separate explicit current-live
    /// entry must supply its own authority; inspecting a capture is not that entry.
    pub fn request_selected_card_route(
        &mut self,
        _state: &mut crate::presentation::web_ui_adapter::player_shell::ShellState,
        _catalog: &crate::presentation::web_ui_adapter::player_shell::RouteCatalog,
        _guards: crate::presentation::web_ui_adapter::player_shell::NavigationGuards,
    ) -> Result<Option<crate::presentation::web_ui_adapter::proposals::card_selection_queue::CardSelectionRoute>, ViewerHostError>{
        if self.shutdown_started
            || self
                .document
                .as_ref()
                .is_none_or(|document| document.lifecycle() != HostLifecycle::Open)
        {
            return Err(ViewerHostError::ForeignCheckpoint);
        }
        let queue = self
            .card_selection_queue()
            .ok_or(ViewerHostError::UnsupportedCallback)?;
        let queue = queue
            .try_borrow()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?;
        if queue.is_empty() {
            return Ok(None);
        }
        Err(open_card::captured_editor_unavailable())
    }
    /// The original checked document must already contain this mounted Card export.
    /// Rejection returns the complete bridge; the caller must retain it for cleanup/retry.
    pub fn bind_active_card(
        &mut self,
        bridge: ActiveCardBridge,
    ) -> Result<(), Box<ActiveCardBridge>> {
        let eligible = !self.shutdown_started
            && self.product_frame.is_some()
            && self
                .callbacks
                .try_borrow()
                .is_ok_and(|owner| owner.is_released())
            && self.document.as_ref().is_some_and(|document| {
                let Ok(stamp) = document.stamp() else {
                    return false;
                };
                document.id() == bridge.document_handle().id()
                    && document.lifecycle() == HostLifecycle::Open
                    && bridge
                        .document_handle()
                        .stamp()
                        .is_ok_and(|candidate| candidate == stamp)
            });
        if !eligible {
            return Err(Box::new(bridge));
        }
        let mut cards = match self.cards.try_borrow_mut() {
            Ok(cards) => cards,
            Err(_) => return Err(Box::new(bridge)),
        };
        if cards.active.is_some()
            || cards.inspection.is_some()
            || cards.capture.is_some()
            || cards.capture_entry.is_some()
        {
            return Err(Box::new(bridge));
        }
        cards.active = Some(bridge.with_caption_adapter(Rc::downgrade(&self.window_ui.captions)));
        Ok(())
    }
    /// Retains the pending inspection in the same ledger observed by original execution close.
    pub fn bind_card_inspection(
        &mut self,
        inspection: crate::assembly::meari::card_current_inspection::CardCurrentInspection,
    ) -> Result<(), Box<crate::assembly::meari::card_current_inspection::CardCurrentInspection>>
    {
        let eligible = self.product_frame.is_some()
            && self
                .callbacks
                .try_borrow()
                .is_ok_and(|owner| owner.is_released())
            && self.document.as_ref().is_some_and(|document| {
                document.lifecycle() == HostLifecycle::Open
                    && document
                        .stamp()
                        .is_ok_and(|stamp| &stamp.owner == inspection.scope())
            });
        if !eligible {
            return Err(Box::new(inspection));
        }
        let mut cards = match self.cards.try_borrow_mut() {
            Ok(cards) => cards,
            Err(_) => return Err(Box::new(inspection)),
        };
        if cards.active.is_some()
            || cards.inspection.is_some()
            || cards.capture.is_some()
            || cards.capture_entry.is_some()
        {
            return Err(Box::new(inspection));
        }
        cards.inspection = Some(inspection);
        Ok(())
    }
    /// The caller accepts the retained result before opening the Card; pending owners stay here.
    pub fn take_released_card_inspection(
        &mut self,
    ) -> Option<crate::assembly::meari::card_current_inspection::CardCurrentInspection> {
        let mut cards = self.cards.try_borrow_mut().ok()?;
        if cards
            .inspection
            .as_ref()
            .is_some_and(|inspection| inspection.is_released())
        {
            cards.inspection.take()
        } else {
            None
        }
    }
    pub fn release_completed_card_inspection(
        &mut self,
    ) -> Result<(), meari::application::plugins::PluginError> {
        let mut cards = self.cards.try_borrow_mut().map_err(|_| {
            meari::application::plugins::PluginError::new(
                meari::application::plugins::PluginErrorCode::Busy,
                "Card route owner borrowed",
            )
        })?;
        if let Some(inspection) = &mut cards.inspection {
            inspection.release()?;
        }
        Ok(())
    }
    pub(crate) fn bind_product_document(
        &mut self,
        document: NativeDocumentHandle,
        props: std::collections::BTreeMap<String, meari::domain::Value>,
        frame: crate::presentation::web_ui_adapter::player_shell::document::product_frame::ProductFrameBlueprint,
    ) {
        self.product_frame = Some(frame);
        self.bind_document(document, props);
    }
    /// Trusted consumer recovery: same owner, no new read and no ACK on re-decode.
    pub fn retry_read_projection(&mut self) -> Result<(), ViewerHostError> {
        let owner = self
            .callbacks
            .try_borrow()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?;
        owner
            .projection
            .try_borrow_mut()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?
            .retry_decode();
        self.read_retries = 3;
        Ok(())
    }
    pub fn last_close_progress(&self) -> Option<&ViewerCloseProgress> {
        self.closing.last.as_ref()
    }
    pub fn last_close_error(&self) -> Option<&ViewerHostError> {
        self.last_error.as_ref()
    }
    /// Used for unrun owners only; returned runtimes must use the child-first runtime close path.
    pub fn poll_unrun_shutdown(&mut self) -> Result<ViewerCloseProgress, ViewerHostError> {
        self.shutdown_started = true;
        self.automatic.stop();
        self.window_ui.stop();
        let navigation = self
            .cards
            .try_borrow()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?
            .navigation
            .clone();
        navigation
            .try_borrow_mut()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?
            .close();
        self.callbacks
            .try_borrow_mut()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?
            .consume_reads()
            .map_err(|error| ViewerHostError::Callback(Arc::new(error)))?;
        self.callbacks
            .try_borrow_mut()
            .map_err(|_| {
                ViewerHostError::Callback(Arc::new(meari::application::plugins::PluginError::new(
                    meari::application::plugins::PluginErrorCode::Busy,
                    "Viewer callbacks borrowed",
                )))
            })?
            .stop_admission();
        match self.closing.poll() {
            Ok(progress) => {
                self.last_error = None;
                Ok(progress)
            }
            Err(error) => {
                let error = ViewerHostError::Close(error);
                self.last_error = Some(error.clone());
                Err(error)
            }
        }
    }
}
impl NativeWindowHost for ViewerHost {
    type Error = ViewerHostError;
    type CloseCheckpoint = ViewerCloseCheckpoint;
    fn admit_callback(
        &mut self,
        stamp: &HostStamp,
        callback: &PendingCallback,
    ) -> IntentAdmission<Self::Error> {
        use meari::application::plugins::{PluginDispatch, PluginErrorCode};
        if self.shutdown_started {
            return IntentAdmission::Rejected(ViewerHostError::ForeignCheckpoint);
        }
        if let Some(admission) = self.admit_live_entry(stamp, callback) {
            return admission;
        }
        if let Some(admission) = self.admit_jobs(stamp, callback) {
            return admission;
        }
        if let Some(admission) = self.admit_chooser(stamp, callback) {
            return admission;
        }
        {
            let mut cards = match self.cards.try_borrow_mut() {
                Ok(cards) => cards,
                Err(_) => return IntentAdmission::Capacity,
            };
            if cards
                .live_opening
                .as_ref()
                .is_some_and(|entry| entry.phase != LiveEntryPhase::Active)
            {
                return IntentAdmission::Rejected(ViewerHostError::Navigation(
                    crate::presentation::web_ui_adapter::player_shell::NavigationError::Busy,
                ));
            }
            if cards
                .viewing_return
                .as_ref()
                .is_some_and(|request| request.blocks_input())
            {
                return IntentAdmission::Rejected(ViewerHostError::Navigation(
                    crate::presentation::web_ui_adapter::player_shell::NavigationError::Busy,
                ));
            }
            if let Some(card) = &mut cards.active {
                return match card.admit_callback(stamp, callback) {
                    IntentAdmission::Accepted => IntentAdmission::Accepted,
                    IntentAdmission::Capacity => IntentAdmission::Capacity,
                    IntentAdmission::Rejected(error) => {
                        IntentAdmission::Rejected(ViewerHostError::Card(Arc::new(error)))
                    }
                };
            }
            if let Some(surface) = &mut cards.capture_surface {
                if let Some(document) = &self.document
                    && (document.lifecycle() != HostLifecycle::Open
                        || document.stamp().ok().as_ref() != Some(stamp))
                {
                    return IntentAdmission::Rejected(ViewerHostError::ForeignCheckpoint);
                }
                return surface.admit(stamp, callback);
            }
            if cards.capture.is_some() || cards.capture_entry.is_some() {
                // Trusted capture composition has no renderer callback bindings yet.
                // Never fall through to the released read consumer or guess a mutation.
                return IntentAdmission::Rejected(ViewerHostError::UnsupportedCallback);
            }
        }
        let mut callbacks = match self.callbacks.try_borrow_mut() {
            Ok(owner) => owner,
            Err(_) => return IntentAdmission::Capacity,
        };
        use meari::application::host_callbacks::LocalDispatchError;
        if let Some(document) = &self.document
            && (document.lifecycle() != HostLifecycle::Open
                || document.stamp().ok().as_ref() != Some(stamp))
        {
            return IntentAdmission::Rejected(ViewerHostError::ForeignCheckpoint);
        }
        match callbacks.queue_card_selection(&stamp.owner, callback) {
            Ok(Some(admission)) => {
                return match admission {
                    IntentAdmission::Accepted => IntentAdmission::Accepted,
                    IntentAdmission::Capacity => IntentAdmission::Capacity,
                    IntentAdmission::Rejected(error) => {
                        IntentAdmission::Rejected(ViewerHostError::Callback(Arc::new(error)))
                    }
                };
            }
            Ok(None) | Err(LocalDispatchError::NotLocal) => {}
            Err(LocalDispatchError::Invalid(error)) if error.code == PluginErrorCode::Busy => {
                return IntentAdmission::Capacity;
            }
            Err(LocalDispatchError::Invalid(error)) => {
                return IntentAdmission::Rejected(ViewerHostError::Callback(Arc::new(error)));
            }
        }
        match callbacks.queue_local(&stamp.owner, callback) {
            Ok(()) => return IntentAdmission::Accepted,
            Err(LocalDispatchError::NotLocal) => {}
            Err(LocalDispatchError::Invalid(error)) if error.code == PluginErrorCode::Busy => {
                return IntentAdmission::Capacity;
            }
            Err(LocalDispatchError::Invalid(error)) => {
                return IntentAdmission::Rejected(ViewerHostError::Callback(Arc::new(error)));
            }
        }
        match callbacks.dispatch(&stamp.owner, callback) {
            Ok(receipt) => match receipt.outcome {
                PluginDispatch::Pending | PluginDispatch::Immediate(Ok(_)) => {
                    self.read_admitted = true;
                    self.read_retries = 3;
                    IntentAdmission::Accepted
                }
                PluginDispatch::Immediate(Err(error)) => {
                    IntentAdmission::Rejected(ViewerHostError::Callback(Arc::new(error)))
                }
            },
            Err(error) if error.code == PluginErrorCode::Busy => IntentAdmission::Capacity,
            Err(error) => IntentAdmission::Rejected(ViewerHostError::Callback(Arc::new(error))),
        }
    }
    fn after_dispatch(&mut self, turn: HostTurn) -> Result<HostEffects, Self::Error> {
        #[cfg(test)]
        if let Some(probe) = super::viewer_native_probe::current() {
            probe.input_diagnostic_once(
                6,
                "first-host-turn",
                &format!(
                    "reason={:?} revision={} accepted={} rejected={} remaining={}",
                    turn.reason,
                    turn.stamp.document_revision,
                    turn.accepted_callbacks,
                    turn.rejected_callbacks,
                    turn.remaining_callbacks
                ),
            );
        }
        self.prepare_automatic_progress();
        if !self.observed_viewing_for_chooser()
            && let Ok(owner) = self.callbacks.try_borrow()
        {
            self.window_ui
                .advance(&owner, false, &turn, self.document.clone());
        }
        let reflection_turn = turn.clone();
        let outcome: Result<HostEffects, ViewerHostError> = (|| {
            if let Some(document) = &self.document
                && (document.lifecycle() != HostLifecycle::Open
                    || document.stamp().ok().as_ref() != Some(&turn.stamp))
            {
                return Err(ViewerHostError::ForeignCheckpoint);
            }
            let mut inspected_card_ready = false;
            if let Some(effects) = self.advance_live_entry(&turn)? {
                return Ok(effects);
            }
            self.advance_source_navigation(&turn)?;
            if let Some(effects) = self.advance_jobs(&turn)? {
                return Ok(effects);
            }
            if turn.remaining_callbacks == 0 {
                self.request_queued_captured_entry()?;
            }
            if turn.remaining_callbacks == 0 && self.advance_explicit_captured_entry()? {
                return Ok(HostEffects::default());
            }
            if let Some(effects) = self.advance_capture_surface(&turn)? {
                return Ok(effects);
            }
            if let Some(effects) = self.advance_viewing_return(&turn)? {
                return Ok(effects);
            }
            {
                let mut cards = self
                    .cards
                    .try_borrow_mut()
                    .map_err(|_| ViewerHostError::UnsupportedCallback)?;
                #[cfg(test)]
                let live_active = cards
                    .live_opening
                    .as_ref()
                    .is_some_and(|entry| entry.phase == LiveEntryPhase::Active);
                if let Some(card) = &mut cards.active {
                    #[cfg(test)]
                    let turn_metadata = (turn.reason, turn.remaining_callbacks);
                    let outcome = card.after_dispatch(turn);
                    #[cfg(test)]
                    if let Some(probe) = super::viewer_native_probe::current() {
                        probe.input_diagnostic_once(
                            7,
                            "original-bridge-outcome",
                            &format!(
                                "ok={} redraw={:?} error_variant={:?}",
                                outcome.is_ok(),
                                outcome.as_ref().ok().map(|effects| effects.redraw),
                                outcome.as_ref().err().map(std::mem::discriminant)
                            ),
                        );
                    }
                    #[cfg(test)]
                    {
                        let snapshot = super::viewer_native_probe::current()
                            .filter(|probe| probe.is_live())
                            .map(|probe| {
                                let guards = card.navigation_guards();
                                (
                                    probe,
                                    super::viewer_native_probe::live_retained::SaveSnapshot {
                                        stage: "after",
                                        active: live_active,
                                        dirty: Some(guards.dirty),
                                        pending: Some(guards.pending),
                                        recovery: Some(guards.recovery_blocked),
                                        transport: Some(card.operation_pending()),
                                        token: Some(card.review().pending().is_some()),
                                        confirmed: Some(
                                            card.review().confirmed_mutation().is_some(),
                                        ),
                                        projection: Some(card.projection_pending()),
                                        revision: Some(
                                            card.review().stored().document.draft_revision.0,
                                        ),
                                        remaining: turn_metadata.1,
                                        reason: turn_metadata.0,
                                    },
                                )
                            });
                        drop(cards);
                        if let Some((probe, snapshot)) = snapshot {
                            probe.input_after_dispatch_guards(&snapshot);
                            probe.live_save_after_dispatch(&snapshot);
                            probe.live_saving_checkpoint(snapshot);
                        }
                    }
                    return outcome.map_err(|error| ViewerHostError::Card(Arc::new(error)));
                }
                if let Some(capture) = &mut cards.capture {
                    if turn.remaining_callbacks != 0 {
                        return Ok(HostEffects::default());
                    }
                    let before = capture.execution_terminal().cloned();
                    capture.poll_execution().map_err(ViewerHostError::Sdk)?;
                    let changed = match (before.as_ref(), capture.execution_terminal()) {
                        (None, Some(_)) => true,
                        (Some(before), Some(after)) => !Arc::ptr_eq(before, after),
                        _ => false,
                    };
                    return Ok(HostEffects {
                        committed: None,
                        redraw: changed,
                    });
                }
                if let Some(inspection) = &mut cards.inspection {
                    if turn.remaining_callbacks != 0 {
                        return Ok(HostEffects::default());
                    }
                    let was_pending = inspection.pending_ticket().is_some();
                    let completed = inspection
                        .poll()
                        .map_err(|error| ViewerHostError::Callback(Arc::new(error)))?;
                    inspected_card_ready = completed
                        && cards
                            .opening
                            .as_ref()
                            .is_some_and(|opening| !opening.panel_attempted);
                    if !inspected_card_ready {
                        return Ok(HostEffects {
                            committed: None,
                            redraw: was_pending && completed,
                        });
                    }
                }
            }
            if inspected_card_ready {
                return self.finish_inspected_card(turn);
            }
            if self.advance_selected_card_inspection(&turn)? {
                return Ok(HostEffects::default());
            }
            let chooser_ready = self.observed_viewing_for_chooser();
            let mut owner = self
                .callbacks
                .try_borrow_mut()
                .map_err(|_| ViewerHostError::UnsupportedCallback)?;
            let chooser_enabled = chooser_ready && owner.chooser_phase().is_some();
            if chooser_enabled
                && turn.remaining_callbacks == 0
                && let Err(error) = owner.drain_chooser(&turn.stamp.owner)
            {
                // Retain the original consumer failure while its existing pump still drains.
                self.last_error
                    .get_or_insert_with(|| ViewerHostError::Callback(Arc::new(error)));
            }
            let local = owner
                .drain_local(&turn.stamp.owner)
                .map_err(|error| ViewerHostError::Callback(Arc::new(error)))?;
            self.window_ui
                .advance(&owner, chooser_ready, &turn, self.document.clone());
            if local.refreshed {
                self.read_retries = 3;
                self.automatic.retry_on_user_refresh();
            }
            if self.read_admitted
                || local.resumed
                || matches!(turn.reason, HostTurnReason::Completion)
                || (chooser_enabled && turn.remaining_callbacks == 0)
            {
                owner
                    .consume_reads()
                    .map_err(|error| ViewerHostError::Callback(Arc::new(error)))?;
                self.read_admitted = false;
            }
            if turn.remaining_callbacks == 0 {
                let automatic_viewing = self.observed_viewing_for_chooser();
                self.automatic.advance(&mut owner, &turn, automatic_viewing);
            }
            let chooser_values = if chooser_enabled && turn.remaining_callbacks == 0 {
                match viewing_chooser::display(&mut owner, Some(&turn.stamp.owner)) {
                    Ok(values) => values,
                    Err(error) => {
                        self.last_error.get_or_insert_with(|| error.clone());
                        if let Ok(mut projection) = owner.projection.try_borrow_mut() {
                            projection.projection_failed(error.to_string());
                        }
                        return Err(error);
                    }
                }
            } else {
                None
            };
            let mut projection = owner
                .projection
                .try_borrow_mut()
                .map_err(|_| ViewerHostError::UnsupportedCallback)?;
            // Runtime forbids replacing callback exports while this admitted batch still has input.
            if turn.remaining_callbacks != 0 {
                return Ok(HostEffects::default());
            }
            if let Some(delay) = owner.next_consumer_retry() {
                return Err(ViewerHostError::ReadConsumerRetry(delay));
            }
            let chooser_changed = chooser_values
                .as_ref()
                .is_some_and(|values| self.chooser_changed(&turn.stamp.owner, values));
            if !projection.dirty && !chooser_changed {
                return Ok(HostEffects::default());
            }
            let Some(document) = &self.document else {
                return Ok(HostEffects::default());
            };
            let mut props = self.base_props.clone();
            props.extend(
                projection
                    .props()
                    .map_err(|error| ViewerHostError::Callback(Arc::new(error)))?,
            );
            props.extend(owner.local_props());
            if let Some(values) = &chooser_values
                && let Err(error) = crate::presentation::web_ui_adapter::player_shell::document::proposal_chooser::stage_root(&mut props, values.clone()) {
                    let error = viewing_chooser::projection_error(error);
                    self.last_error.get_or_insert_with(|| error.clone());
                    projection.projection_failed(error.to_string());
                    return Err(error);
            }
            let retained_range = projection
                .range_feedback
                .retained(&turn.stamp, &props)
                .map_err(|message| {
                    projection.projection_failed(message);
                    ViewerHostError::UnsupportedCallback
                })?;
            let feedback_props = props.clone();
            let bindings = if retained_range {
                None
            } else {
                Some(
                    owner
                        .bindings()
                        .map_err(|error| ViewerHostError::Callback(Arc::new(error)))?,
                )
            };
            match document.try_reproject(&turn.stamp, |document| {
                let props = match &self.product_frame {
                    Some(frame) => frame.workspace_props(document,
                        crate::presentation::web_ui_adapter::player_shell::document::product_frame::WorkspaceRoute::Viewing,
                        self.viewing_generation, props)?,
                    None => {
                        let mut props = props;
                        crate::presentation::web_ui_adapter::player_shell::start_screen::compose_slot(
                            &mut props,
                            |reference, props| document.node_slot("viewing-start", reference, props),
                        )?;
                        props
                    },
                };
                if retained_range {
                    let paths = crate::assembly::meari::viewer_read_projection::range_feedback::display_paths(
                        &document.module_projection()?.component.root)?;
                    document.update_module_range_props_with_text(props, &paths)
                } else {
                    document.update_module_props_with_callbacks(props, bindings.expect("Fresh projection owns its callback export"))
                }
            }) {
                Ok(((), receipt)) => {
                    projection.range_feedback.committed(receipt.after().clone(), feedback_props);
                    projection.committed();
                    if let Some(values) = chooser_values {
                        self.viewing_chooser.committed(turn.stamp.owner.clone(), values);
                    }
                    Ok(HostEffects {
                        committed: Some(receipt),
                        redraw: true,
                    })
                }
                Err(error) => {
                    projection.projection_failed(format!("{error:?}"));
                    if retained_range {
                        projection.range_feedback.failed(format!("{error:?}"));
                        return Err(ViewerHostError::UnsupportedCallback);
                    }
                    if self.read_retries > 0 {
                        self.read_retries -= 1;
                        return Err(ViewerHostError::ReadConsumerRetry(Duration::from_millis(
                            25,
                        )));
                    }
                    Ok(HostEffects::default())
                }
            }
        })();
        #[cfg(test)]
        if let Err(error) = &outcome
            && let Some(probe) = super::viewer_native_probe::current()
        {
            probe.input_diagnostic_once(
                8,
                "original-host-error-before-return",
                &format!("error_variant={:?}", std::mem::discriminant(error)),
            );
        }
        let mut effects = outcome?;
        let open = !self.shutdown_started
            && self.document.as_ref().is_some_and(|document| {
                document.lifecycle() == HostLifecycle::Open
                    && document.stamp().ok().is_some_and(|current| {
                        current.host_id == reflection_turn.stamp.host_id
                            && current.owner == reflection_turn.stamp.owner
                            && current.definition_revision
                                == reflection_turn.stamp.definition_revision
                    })
            });
        effects.redraw |= self.registration_redraw.take_for(&reflection_turn, open);
        #[cfg(test)]
        super::routed_owner_fixture::after_dispatch();
        #[cfg(test)]
        if let Some(probe) = super::viewer_native_probe::current() {
            effects.redraw |= probe.source_return_redraw();
        }
        self.frame.apply(&reflection_turn, &mut effects);
        #[cfg(test)]
        if let Some(applied) = self.frame.applied.take() {
            let stamp = self
                .document
                .as_ref()
                .and_then(|document| document.stamp().ok());
            effects.redraw |= super::viewer_native_probe::reflection(
                &reflection_turn,
                &effects,
                stamp.as_ref(),
                &applied,
            );
        }
        #[cfg(test)]
        if let Some(probe) = super::viewer_native_probe::current() {
            probe.input_diagnostic_once(
                8,
                "original-final-redraw",
                &format!("redraw={}", effects.redraw),
            );
        }
        Ok(effects)
    }
    fn retry_after_dispatch(&self, error: &Self::Error) -> Option<HostRetry> {
        match error {
            ViewerHostError::Card(error) => match self.cards.try_borrow() {
                Ok(cards) => cards
                    .active
                    .as_ref()
                    .and_then(|card| card.retry_after_dispatch(error)),
                Err(_) => Some(HostRetry::OnWake),
            },
            ViewerHostError::ReadConsumerRetry(delay) => Some(HostRetry::After(*delay)),
            ViewerHostError::Callback(error)
                if error.code == meari::application::plugins::PluginErrorCode::Busy =>
            {
                Some(HostRetry::OnWake)
            }
            _ => None,
        }
    }
    fn next_close_observation(&self, _: &Self::CloseCheckpoint) -> HostRetry {
        HostRetry::After(Duration::from_millis(5))
    }
    fn can_close(&mut self, stamp: &HostStamp, cause: CloseCause) -> Result<bool, Self::Error> {
        let mut cards = self
            .cards
            .try_borrow_mut()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?;
        if cards.capture_surface.is_some()
            && self
                .document
                .as_ref()
                .and_then(|document| document.stamp().ok())
                .as_ref()
                != Some(stamp)
        {
            return Err(ViewerHostError::ForeignCheckpoint);
        }
        if cause == CloseCause::User
            && cards
                .capture_surface
                .as_ref()
                .is_some_and(|surface| surface.user_close_blocked())
        {
            if let Ok(owner) = self.callbacks.try_borrow()
                && let Ok(mut projection) = owner.projection.try_borrow_mut()
            {
                projection.window_ui.user_close_vetoed();
                projection.dirty = true;
                if let Ok(mut captions) = self.window_ui.captions.try_borrow_mut() {
                    captions.view = projection.window_ui.caption_view();
                }
                if let Some(completion) = &self.completion {
                    let _ = completion.notify();
                }
            }
            return Ok(false);
        }
        let permitted = match &mut cards.active {
            Some(card) => card
                .can_close(stamp, cause)
                .map_err(|error| ViewerHostError::Card(Arc::new(error))),
            None => Ok(true),
        };
        if cause == CloseCause::User
            && matches!(permitted, Ok(false))
            && let Ok(owner) = self.callbacks.try_borrow()
            && let Ok(mut projection) = owner.projection.try_borrow_mut()
        {
            projection.window_ui.user_close_vetoed();
            projection.dirty = true;
            if let Ok(mut captions) = self.window_ui.captions.try_borrow_mut() {
                captions.view = projection.window_ui.caption_view();
            }
            if let Some(completion) = &self.completion {
                let _ = completion.notify();
            }
        }
        if matches!(permitted, Ok(true))
            && let Some(slot) = &self.transition_slot
        {
            slot.close_admission();
        }
        permitted
    }
    fn close(
        &mut self,
        request: CloseRequest<Self::CloseCheckpoint>,
    ) -> CloseDecision<Self::CloseCheckpoint, Self::Error> {
        #[cfg(test)]
        super::viewer_native_probe::inner_close();
        let checkpoint = request.checkpoint.unwrap_or_else(|| ViewerCloseCheckpoint {
            owner: self.identity.clone(),
        });
        if !Rc::ptr_eq(&checkpoint.owner, &self.identity) {
            return CloseDecision::RetryableFailure {
                checkpoint,
                error: ViewerHostError::ForeignCheckpoint,
            };
        }
        match self.poll_unrun_shutdown() {
            Ok(ViewerCloseProgress::Complete { .. }) => {
                CloseDecision::HostCleanupComplete { checkpoint }
            }
            Ok(_) => CloseDecision::Pending { checkpoint },
            Err(error) => CloseDecision::ObservationUnavailable { checkpoint, error },
        }
    }
}

#[cfg(test)]
mod tests;

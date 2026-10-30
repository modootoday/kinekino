//! Accept a return intent on the original route state before releasing any editor owner.
use super::*;
use crate::presentation::web_ui_adapter::player_shell::{NavigationError, RouteCatalog};

#[derive(Clone, Copy, PartialEq, Eq)]
enum ReturnPhase {
    Requested,
    Releasing,
    Exporting,
    Commit,
    Committed,
}
#[derive(Clone, Copy)]
enum ReturnOwner {
    Live,
    Capture,
}
pub(super) struct ViewingReturn {
    owner: ReturnOwner,
    ticket: crate::presentation::web_ui_adapter::player_shell::RouteTicket,
    phase: ReturnPhase,
    receipt: Option<HostCommitReceipt>,
    error: Option<ViewerHostError>,
    first_error: Option<ViewerHostError>,
    native_error: Option<Arc<crate::application::api::ApiError>>,
    retained: bool,
    proof: Option<RetainedCommitProof>,
    proof_error: Option<Arc<HostAccessError>>,
    wake_error: Option<Arc<crate::application::api::ApiError>>,
    redraw: bool,
}
impl ViewingReturn {
    pub(super) fn committed(&self) -> bool {
        self.phase == ReturnPhase::Committed
    }
    pub(super) fn blocks_input(&self) -> bool {
        !self.committed() || self.retained
    }
}

impl ViewerHost {
    pub(super) fn take_pending_route_registration_redraw(&self, turn: &HostTurn) -> bool {
        let open = !self.shutdown_started
            && self.document.as_ref().is_some_and(|document| {
                document.lifecycle() == HostLifecycle::Open
                    && document.stamp().ok().is_some_and(|current| {
                        current.host_id == turn.stamp.host_id
                            && current.owner == turn.stamp.owner
                            && current.definition_revision == turn.stamp.definition_revision
                    })
            });
        self.registration_redraw.take_for(turn, open)
    }
    /// This request stage has not released any owner, so cancel only the exact retained
    /// route intent. A later release/commit transaction must use its own recovery state.
    pub fn cancel_viewing_return_request(&mut self) -> Result<(), ViewerHostError> {
        let mut cards = self
            .cards
            .try_borrow_mut()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?;
        let request = cards
            .viewing_return
            .as_ref()
            .ok_or(ViewerHostError::UnsupportedCallback)?;
        if request.phase != ReturnPhase::Requested {
            return Err(ViewerHostError::Navigation(
                NavigationError::RecoveryBlocked,
            ));
        }
        self.route_observation
            .as_ref()
            .ok_or(ViewerHostError::UnsupportedCallback)?
            .try_borrow_mut()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?
            .state
            .cancel_route(&request.ticket)
            .map_err(ViewerHostError::Navigation)?;
        cards.viewing_return = None;
        Ok(())
    }
    /// A successful request retains an AwaitingHost ticket only. The active editor,
    /// callbacks and SDK attachment remain owned until the return transaction commits.
    pub fn request_viewing_return(&mut self, turn: &HostTurn) -> Result<(), ViewerHostError> {
        if turn.remaining_callbacks != 0 || self.shutdown_started {
            return Err(ViewerHostError::Navigation(NavigationError::Busy));
        }
        let document = self
            .document
            .as_ref()
            .ok_or(ViewerHostError::ForeignCheckpoint)?;
        if self.product_frame.is_none()
            || document.lifecycle() != HostLifecycle::Open
            || document.stamp().ok().as_ref() != Some(&turn.stamp)
        {
            return Err(ViewerHostError::ForeignCheckpoint);
        }
        let mut cards = self
            .cards
            .try_borrow_mut()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?;
        if cards.viewing_return.is_some() {
            return Err(ViewerHostError::Navigation(NavigationError::Busy));
        }
        if cards.mount_failure.is_some()
            || cards.rejected_bridge.is_some()
            || cards.inspection.is_some()
        {
            return Err(ViewerHostError::Navigation(
                NavigationError::RecoveryBlocked,
            ));
        }
        let (owner, guards) = if let Some(active) = &cards.active {
            if cards.capture.is_some()
                || cards.capture_surface.is_some()
                || active.document_handle().stamp().ok().as_ref() != Some(&turn.stamp)
            {
                return Err(ViewerHostError::ForeignCheckpoint);
            }
            (ReturnOwner::Live, active.navigation_guards())
        } else {
            let entry = cards
                .capture_entry
                .as_ref()
                .ok_or(ViewerHostError::UnsupportedCallback)?;
            if !entry.matches_return_scope(&turn.stamp.owner)
                || cards.capture.is_none()
                || cards.capture_close_receipt.is_some()
            {
                return Err(ViewerHostError::ForeignCheckpoint);
            }
            (ReturnOwner::Capture, cards.capture_return_guards()?)
        };
        let observer = self
            .route_observation
            .as_ref()
            .ok_or(ViewerHostError::UnsupportedCallback)?;
        let mut observer = observer
            .try_borrow_mut()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?;
        let expected_route = cards
            .active_task
            .map(|task| task.route_id())
            .unwrap_or("studio.cards");
        let retained = matches!(owner, ReturnOwner::Live) && cards.active_task.is_some();
        if observer
            .state
            .observed_route()
            .is_none_or(|route| route.as_str() != expected_route)
        {
            return Err(ViewerHostError::Navigation(
                NavigationError::RecoveryBlocked,
            ));
        }
        let catalog = RouteCatalog::bundled().map_err(|_| ViewerHostError::UnsupportedCallback)?;
        let ticket = observer
            .state
            .request_route(&catalog, "view.start", guards)
            .map_err(ViewerHostError::Navigation)?;
        // All fallible checks precede this assignment. Never lose an accepted ticket
        // after a transient borrow failure or clear an editing owner on intent alone.
        cards.viewing_return = Some(ViewingReturn {
            owner,
            ticket,
            phase: ReturnPhase::Requested,
            native_error: None,
            receipt: None,
            error: None,
            first_error: None,
            retained,
            proof: None,
            proof_error: None,
            wake_error: None,
            redraw: false,
        });
        Ok(())
    }

    pub fn viewing_return_error(&self) -> Option<ViewerHostError> {
        self.cards
            .try_borrow()
            .ok()?
            .viewing_return
            .as_ref()?
            .error
            .clone()
    }
    pub fn viewing_return_receipt(&self) -> Option<HostCommitReceipt> {
        self.cards
            .try_borrow()
            .ok()?
            .viewing_return
            .as_ref()?
            .receipt
            .clone()
    }
    /// Explicit retry preserves the failed phase. A partial read export is released
    /// before retrying attachment; a committed slot is never committed a second time.
    pub fn retry_viewing_return(&mut self) -> Result<(), ViewerHostError> {
        if self.shutdown_started {
            return Err(ViewerHostError::ForeignCheckpoint);
        }
        let mut cards = self
            .cards
            .try_borrow_mut()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?;
        let request = cards
            .viewing_return
            .as_mut()
            .ok_or(ViewerHostError::UnsupportedCallback)?;
        if request.committed() {
            if request.error.is_none() {
                return Err(ViewerHostError::Navigation(NavigationError::Busy));
            }
            let retry = (|| {
                let receipt = request
                    .receipt
                    .clone()
                    .ok_or(ViewerHostError::ForeignCheckpoint)?;
                let document = self
                    .document
                    .as_ref()
                    .ok_or(ViewerHostError::ForeignCheckpoint)?;
                let mut observer = self
                    .route_observation
                    .as_ref()
                    .ok_or(ViewerHostError::UnsupportedCallback)?
                    .try_borrow_mut()
                    .map_err(|_| ViewerHostError::UnsupportedCallback)?;
                if request.retained {
                    let proof = request.proof.clone().ok_or(ViewerHostError::Navigation(
                        NavigationError::RecoveryBlocked,
                    ))?;
                    if let Err(error) = document.validate_retained_commit(&proof) {
                        request.proof_error.get_or_insert_with(|| Arc::new(error));
                        return Err(ViewerHostError::ForeignCheckpoint);
                    }
                    observer
                        .arm_retained_viewing_route(request.ticket.clone(), document.clone(), proof)
                        .map_err(ViewerHostError::Navigation)?;
                    if let Some(error) = self.route_continuation() {
                        request.wake_error.get_or_insert(error);
                    }
                    request.redraw = true;
                } else {
                    let current = document
                        .stamp()
                        .map_err(|_| ViewerHostError::ForeignCheckpoint)?;
                    observer
                        .arm_viewing_route(request.ticket.clone(), &receipt, &current)
                        .map_err(ViewerHostError::Navigation)?;
                }
                Ok(())
            })();
            match &retry {
                Ok(()) => request.error = None,
                Err(error) => {
                    request.first_error.get_or_insert_with(|| error.clone());
                    request.error = Some(error.clone());
                }
            }
            return retry;
        }
        if request.phase == ReturnPhase::Exporting {
            let mut owner = self
                .callbacks
                .try_borrow_mut()
                .map_err(|_| ViewerHostError::UnsupportedCallback)?;
            let scope = self
                .document
                .as_ref()
                .ok_or(ViewerHostError::ForeignCheckpoint)?
                .stamp()
                .map_err(|_| ViewerHostError::ForeignCheckpoint)?
                .owner;
            if owner
                .prepare_route_release(&scope)
                .map_err(|error| ViewerHostError::Callback(Arc::new(error)))?
                .is_some()
            {
                return Err(ViewerHostError::Navigation(NavigationError::Busy));
            }
        }
        request.error = None;
        Ok(())
    }

    pub(super) fn advance_viewing_return(
        &mut self,
        turn: &HostTurn,
    ) -> Result<Option<HostEffects>, ViewerHostError> {
        if self.shutdown_started {
            return Ok(None);
        }
        let cards_owner = self.cards.clone();
        let mut cards = cards_owner
            .try_borrow_mut()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?;
        let Some(mut request) = cards.viewing_return.take() else {
            return Ok(None);
        };
        // Restore the entire request on every outcome, including partially released owners.
        let mut result = (|| {
            if request.committed() {
                return Ok(request.retained.then(|| HostEffects {
                    committed: None,
                    redraw: std::mem::take(&mut request.redraw),
                }));
            }
            if request.error.is_some() || turn.remaining_callbacks != 0 {
                return Ok(Some(HostEffects::default()));
            }
            let observer = self
                .route_observation
                .as_ref()
                .ok_or(ViewerHostError::UnsupportedCallback)?;
            let observer = observer
                .try_borrow_mut()
                .map_err(|_| ViewerHostError::UnsupportedCallback)?;
            if observer
                .state
                .route_target(&request.ticket)
                .map_err(ViewerHostError::Navigation)?
                .as_str()
                != "view.start"
            {
                return Err(ViewerHostError::Navigation(NavigationError::RouteMismatch));
            }
            drop(observer);
            if request.phase == ReturnPhase::Requested {
                let guards = match request.owner {
                    ReturnOwner::Live => cards
                        .active
                        .as_ref()
                        .ok_or(ViewerHostError::UnsupportedCallback)?
                        .navigation_guards(),
                    ReturnOwner::Capture => {
                        let entry = cards
                            .capture_entry
                            .as_ref()
                            .ok_or(ViewerHostError::UnsupportedCallback)?;
                        if !entry.matches_return_scope(&turn.stamp.owner) {
                            return Err(ViewerHostError::ForeignCheckpoint);
                        }
                        cards.capture_return_guards()?
                    }
                };
                if guards.recovery_blocked {
                    return Err(ViewerHostError::Navigation(
                        NavigationError::RecoveryBlocked,
                    ));
                }
                if guards.pending {
                    return Err(ViewerHostError::Navigation(NavigationError::Busy));
                }
                if guards.dirty {
                    return Err(ViewerHostError::Navigation(NavigationError::UnsavedEdits));
                }
                request.phase = ReturnPhase::Releasing;
            }
            if request.phase == ReturnPhase::Releasing {
                match request.owner {
                    ReturnOwner::Live => {
                        let active = cards
                            .active
                            .as_mut()
                            .ok_or(ViewerHostError::UnsupportedCallback)?;
                        if let Err(error) = active.unbind_scope() {
                            if active.scope_close_waiting() {
                                return Ok(Some(HostEffects::default()));
                            }
                            return Err(ViewerHostError::Callback(Arc::new(error)));
                        }
                    }
                    ReturnOwner::Capture => {
                        if !cards
                            .poll_release()
                            .map_err(|error| ViewerHostError::Callback(Arc::new(error)))?
                        {
                            return Ok(Some(HostEffects::default()));
                        }
                    }
                }
                request.phase = ReturnPhase::Exporting;
            }
            let mut owner = self
                .callbacks
                .try_borrow_mut()
                .map_err(|_| ViewerHostError::UnsupportedCallback)?;
            if request.phase == ReturnPhase::Exporting {
                owner
                    .resume_released_route(&turn.stamp.owner)
                    .map_err(|error| ViewerHostError::Callback(Arc::new(error)))?;
                request.phase = ReturnPhase::Commit;
            }
            let chooser_values =
                super::viewing_chooser::display(&mut owner, Some(&turn.stamp.owner))?;
            let mut projection = owner
                .projection
                .try_borrow_mut()
                .map_err(|_| ViewerHostError::UnsupportedCallback)?;
            let mut props = self.base_props.clone();
            props.extend(
                projection
                    .props()
                    .map_err(|error| ViewerHostError::Callback(Arc::new(error)))?,
            );
            props.extend(owner.local_props());
            if let Some(values) = &chooser_values {
                crate::presentation::web_ui_adapter::player_shell::document::proposal_chooser::stage_root(&mut props, values.clone())
                    .map_err(super::viewing_chooser::projection_error)?;
            }
            let bindings = owner
                .bindings()
                .map_err(|error| ViewerHostError::Callback(Arc::new(error)))?;
            let generation = std::num::NonZeroU64::new(request.ticket.generation()).ok_or(
                ViewerHostError::Navigation(NavigationError::GenerationExhausted),
            )?;
            let document = self
                .document
                .as_ref()
                .ok_or(ViewerHostError::ForeignCheckpoint)?;
            let blueprint = self
                .product_frame
                .as_ref()
                .ok_or(ViewerHostError::UnsupportedCallback)?;
            let (_, receipt) = document.try_reproject(&turn.stamp, |document| {
                let props = blueprint.workspace_props(document,
                    crate::presentation::web_ui_adapter::player_shell::document::product_frame::WorkspaceRoute::Viewing,
                    generation, props)?;
                document.update_module_props_with_callbacks(props, bindings)
            }).map_err(|error| ViewerHostError::Callback(Arc::new(meari::application::plugins::PluginError::new(
                meari::application::plugins::PluginErrorCode::Adapter, format!("{error:?}")))))?;
            if let Some(values) = chooser_values {
                self.viewing_chooser
                    .committed(turn.stamp.owner.clone(), values);
            }
            if request.retained {
                let retained = {
                    #[cfg(test)]
                    {
                        super::super::viewer_native_probe::live_retained::retain(
                            document,
                            &receipt,
                            super::super::viewer_native_probe::live_retained::ProofStage::Return,
                        )
                    }
                    #[cfg(not(test))]
                    {
                        document.retain_commit(&receipt)
                    }
                };
                match retained {
                    Ok(proof) => request.proof = Some(proof),
                    Err(error) => request.proof_error = Some(Arc::new(error)),
                }
            }
            projection.committed();
            self.viewing_generation = generation;
            self.read_admitted = false;
            self.read_retries = 3;
            request.receipt = Some(receipt.clone());
            request.phase = ReturnPhase::Committed;
            #[cfg(test)]
            if request.retained
                && let Some(probe) =
                    super::super::viewer_native_probe::current().filter(|probe| probe.is_live())
            {
                probe.live_commit(
                    document,
                    &receipt,
                    request.proof.as_ref(),
                    request.proof_error.as_deref(),
                    super::super::viewer_native_probe::live_retained::ProofStage::Return,
                );
                self.live_fixture_cancel_model_route(
                    &request.ticket,
                    "view.start",
                    super::super::viewer_native_probe::live_retained::ProofStage::Return,
                );
            }
            // Props are committed before observing a frame. Keep the receipt even if arming
            // fails; callers must not repeat the slot commit or claim route observation.
            let arm = (|| {
                let mut observer = self
                    .route_observation
                    .as_ref()
                    .ok_or(ViewerHostError::UnsupportedCallback)?
                    .try_borrow_mut()
                    .map_err(|_| ViewerHostError::UnsupportedCallback)?;
                if request.retained {
                    let proof = request.proof.clone().ok_or(ViewerHostError::Navigation(
                        NavigationError::RecoveryBlocked,
                    ))?;
                    observer
                        .arm_retained_viewing_route(request.ticket.clone(), document.clone(), proof)
                        .map_err(ViewerHostError::Navigation)?;
                    if let Some(error) = self.route_continuation() {
                        request.wake_error.get_or_insert(error);
                    }
                } else {
                    let current = document
                        .stamp()
                        .map_err(|_| ViewerHostError::ForeignCheckpoint)?;
                    observer
                        .arm_viewing_route(request.ticket.clone(), &receipt, &current)
                        .map_err(ViewerHostError::Navigation)?;
                }
                Ok(())
            })();
            request.error = arm.err();
            if let Some(error) = &request.error {
                request.first_error.get_or_insert_with(|| error.clone());
            }
            match request.owner {
                ReturnOwner::Live => {
                    cards.active = None;
                    cards.opening = None;
                }
                ReturnOwner::Capture => {
                    // Only after release, read export and atomic workspace commit succeeded.
                    cards.capture_surface = None;
                    cards.capture = None;
                    cards.capture_entry = None;
                }
            }
            Ok(Some(HostEffects {
                committed: Some(receipt),
                redraw: true,
            }))
        })();
        if let Err(error) = &result {
            request.first_error.get_or_insert_with(|| error.clone());
            request.error = Some(error.clone());
        }
        // The exact first props receipt must be returned even if native rearm fails.
        // Poll fresh native registration only after that Viewing DOM commit exists.
        let has_receipt = matches!(&result, Ok(Some(effects)) if effects.committed.is_some());
        let mut native_ready = !request.retained;
        let mut native_pending = false;
        if request.committed()
            && request.retained
            && request.native_error.is_none()
            && request.error.is_none()
        {
            let scope = self
                .document
                .as_ref()
                .ok_or(ViewerHostError::ForeignCheckpoint)
                .and_then(|document| {
                    document
                        .stamp()
                        .map(|stamp| stamp.owner)
                        .map_err(|_| ViewerHostError::ForeignCheckpoint)
                });
            match scope {
                Ok(scope) => match self.native_route.restore_viewing(&scope) {
                    Ok(ready) => {
                        native_ready = ready;
                        native_pending = !ready;
                    }
                    Err(error) => {
                        request.native_error = Some(error.clone());
                        let error = ViewerHostError::Sdk(error);
                        request.first_error.get_or_insert_with(|| error.clone());
                        request.error = Some(error);
                    }
                },
                Err(error) => {
                    request.first_error.get_or_insert_with(|| error.clone());
                    request.error = Some(error);
                }
            }
        }
        if !has_receipt {
            if let Some(error) = &request.native_error {
                result = Err(ViewerHostError::Sdk(error.clone()));
            } else if native_pending && result.is_ok() {
                if self.take_pending_route_registration_redraw(turn) {
                    // Forward only the original one-shot queue value. The outer
                    // host sees it consumed and cannot manufacture a second redraw.
                    if let Ok(effects) = &mut result {
                        effects.get_or_insert_with(HostEffects::default).redraw = true;
                    }
                } else {
                    result = Err(ViewerHostError::ReadConsumerRetry(Duration::from_millis(5)));
                }
            }
        }
        let observed = native_ready
            && request.committed()
            && request.error.is_none()
            && self.route_observation.as_ref().is_some_and(|observer| {
                observer.try_borrow().is_ok_and(|observer| {
                    observer.state.phase()
                        == crate::presentation::web_ui_adapter::player_shell::ShellPhase::Observed
                        && observer
                            .state
                            .observed_route()
                            .is_some_and(|route| route.as_str() == "view.start")
                })
            });
        #[cfg(test)]
        let fixture_released = observed && request.retained;
        if !observed {
            cards.viewing_return = Some(request);
        } else if request.retained {
            cards.live_opening = None;
            cards.active_task = None;
        }
        #[cfg(test)]
        {
            drop(cards);
            if fixture_released
                && let Some(probe) =
                    super::super::viewer_native_probe::current().filter(|probe| probe.is_live())
            {
                probe.live_return_released();
            }
        }
        result
    }
}

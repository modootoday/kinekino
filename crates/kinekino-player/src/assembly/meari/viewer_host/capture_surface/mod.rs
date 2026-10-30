//! Product Card composition on the original document and execution owner.
#[cfg(test)]
pub(super) mod caption_contract_tests;
mod commands;
mod input;
mod navigation;
use super::*;
use crate::presentation::web_ui_adapter::player_shell::document::product_frame::WorkspaceRoute;
use crate::presentation::web_ui_adapter::proposals::{
    bootstrap::BootstrapPanel,
    capture_card::CaptureCardModel,
    capture_mutation::{
        CaptureExecutionAcknowledgement, CaptureExecutionProgress, RejectedCaptureIntent,
    },
    frame_composition::{FrameComposition, FrameProjection},
    product_window::product_modules_for_panel,
};
use std::num::NonZeroU64;

fn plugin(error: meari::application::plugins::PluginError) -> ViewerHostError {
    ViewerHostError::Callback(Arc::new(error))
}
fn preparation(message: impl Into<String>) -> ViewerHostError {
    plugin(meari::application::plugins::PluginError::new(
        meari::application::plugins::PluginErrorCode::Adapter,
        message,
    ))
}
pub(super) struct CapturedSurface {
    pub(super) return_requested: bool,
    model: CaptureCardModel,
    inputs: input::Inputs,
    projection: FrameProjection,
    acknowledged: Vec<CaptureExecutionAcknowledgement>,
    pub(super) evidence_blocked: bool,
    pub(super) evidence_retry: bool,
    rejected_intent: Option<Box<RejectedCaptureIntent>>,
    progress: Option<CaptureExecutionProgress>,
    pub(super) dirty: bool,
    pub(super) error: Option<ViewerHostError>,
    receipt: Option<HostCommitReceipt>,
    route_error: Option<crate::presentation::web_ui_adapter::player_shell::NavigationError>,
}
impl CapturedSurface {
    pub(super) fn take_acknowledged(&mut self) -> Vec<CaptureExecutionAcknowledgement> {
        if self.acknowledged.is_empty() {
            return Vec::new();
        }
        self.dirty = true;
        std::mem::take(&mut self.acknowledged)
    }
    fn new(host: &ViewerHost, turn: &HostTurn) -> Result<Self, ViewerHostError> {
        let frame = host
            .product_frame
            .as_ref()
            .ok_or(ViewerHostError::UnsupportedCallback)?;
        let cards = host
            .cards
            .try_borrow()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?;
        let entry = cards
            .capture_entry
            .as_ref()
            .ok_or(ViewerHostError::UnsupportedCallback)?;
        let capture = cards
            .capture
            .as_ref()
            .ok_or(ViewerHostError::UnsupportedCallback)?;
        let crate::application::launch::native_proposal::NativeProposalTarget::Card { index } =
            &entry.route.target.selection.target
        else {
            return Err(ViewerHostError::UnsupportedCallback);
        };
        let model = CaptureCardModel::new(capture, *index).map_err(preparation)?;
        let mut inputs = input::Inputs::new(turn.stamp.owner.clone())?;
        inputs.set_caption_source(Rc::downgrade(&host.window_ui.captions));
        let (_, mut inventory) = product_modules_for_panel(BootstrapPanel::Card);
        let entry = "studio/cards/captured-proposal-panel.tsx"
            .to_owned()
            .try_into()
            .map_err(|error| preparation(format!("{error:?}")))?;
        let provider = meari::parse_module_graph(&entry, &inventory)
            .map_err(|e| preparation(format!("{e:?}")))?;
        let (graph, mut projection) = FrameComposition::new(
            crate::domain::product::PRODUCT.display_name.into(),
            String::new(),
        )
        .prepare(provider, &mut inventory)
        .map_err(|e| preparation(format!("{e:?}")))?;
        projection.set_caption_source(Rc::downgrade(&host.window_ui.captions));
        if inventory
            .iter()
            .any(|(p, text)| frame.inventory.get(p) != Some(text))
            || graph
                .modules
                .iter()
                .any(|(p, module)| frame.graph.modules.get(p) != Some(module))
        {
            return Err(preparation(
                "Captured Card provider differs from the original closed product graph",
            ));
        }
        Ok(Self {
            return_requested: false,
            model,
            inputs,
            projection,
            acknowledged: Vec::new(),
            evidence_blocked: false,
            evidence_retry: false,
            rejected_intent: None,
            progress: None,
            dirty: true,
            error: None,
            receipt: None,
            route_error: None,
        })
    }
    pub(super) fn admit(
        &mut self,
        stamp: &HostStamp,
        pending: &PendingCallback,
    ) -> IntentAdmission<ViewerHostError> {
        if self.acknowledged.len() >= 8 || self.rejected_intent.is_some() {
            return IntentAdmission::Capacity;
        }
        let busy = self.progress != Some(CaptureExecutionProgress::Idle) || self.evidence_blocked;
        self.inputs.admit(stamp, pending, busy)
    }
    pub(super) fn stop(&mut self) -> Result<(), meari::application::plugins::PluginError> {
        self.inputs.stop()
    }
    pub(super) fn user_close_blocked(&self) -> bool {
        self.evidence_blocked
            || self.model.dirty()
            || !self.inputs.queue.is_empty()
            || self.rejected_intent.is_some()
            || !matches!(
                self.progress,
                Some(
                    CaptureExecutionProgress::Idle
                        | CaptureExecutionProgress::ConsumedAckPending
                        | CaptureExecutionProgress::DiscardedAckPending
                )
            )
    }
}
impl ViewerHost {
    pub fn captured_local_card_draft(&self) -> Option<crate::domain::proposals::ProposalPayload> {
        Some(
            self.cards
                .try_borrow()
                .ok()?
                .capture_surface
                .as_ref()?
                .model
                .payload(),
        )
    }
    pub(super) fn request_queued_captured_entry(&mut self) -> Result<(), ViewerHostError> {
        if self.product_frame.is_none() {
            return Ok(());
        }
        let Some(observer) = self.route_observation.clone() else {
            return Ok(());
        };
        let occupied = {
            let cards = self
                .cards
                .try_borrow()
                .map_err(|_| ViewerHostError::UnsupportedCallback)?;
            cards.capture_entry.is_some() || cards.capture.is_some()
        };
        if occupied {
            return Ok(());
        }
        let selected = self
            .card_selection_queue()
            .is_some_and(|queue| queue.try_borrow().is_ok_and(|queue| !queue.is_empty()));
        if !selected {
            return Ok(());
        }
        let catalog = crate::presentation::web_ui_adapter::player_shell::RouteCatalog::bundled()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?;
        let mut observer = observer
            .try_borrow_mut()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?;
        self.request_explicit_captured_edit(&mut observer.state, &catalog, Default::default())?;
        Ok(())
    }
    /// Original receipt is separate from frame observation and never claims a rendered route.
    pub fn captured_surface_receipt(&self) -> Option<HostCommitReceipt> {
        self.cards
            .try_borrow()
            .ok()?
            .capture_surface
            .as_ref()?
            .receipt
            .clone()
    }
    pub fn captured_surface_error(&self) -> Option<ViewerHostError> {
        self.cards
            .try_borrow()
            .ok()?
            .capture_surface
            .as_ref()?
            .error
            .clone()
    }
    pub fn captured_surface_route_error(
        &self,
    ) -> Option<crate::presentation::web_ui_adapter::player_shell::NavigationError> {
        self.cards
            .try_borrow()
            .ok()?
            .capture_surface
            .as_ref()?
            .route_error
    }
    /// Trusted evidence transfer. A GUI callback cannot drain another owner's evidence.
    pub fn take_captured_surface_acknowledgements(
        &mut self,
    ) -> Result<Vec<CaptureExecutionAcknowledgement>, ViewerHostError> {
        let mut cards = self
            .cards
            .try_borrow_mut()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?;
        let surface = cards
            .capture_surface
            .as_mut()
            .ok_or(ViewerHostError::UnsupportedCallback)?;
        Ok(surface.take_acknowledged())
    }
    pub(super) fn advance_capture_surface(
        &mut self,
        turn: &HostTurn,
    ) -> Result<Option<HostEffects>, ViewerHostError> {
        if turn.remaining_callbacks != 0
            || self.product_frame.is_none()
            || self.route_observation.is_none()
        {
            return Ok(None);
        }
        let needs_surface = {
            let cards = self
                .cards
                .try_borrow()
                .map_err(|_| ViewerHostError::UnsupportedCallback)?;
            if cards.viewing_return.is_some() {
                return Ok(None);
            }
            if cards.capture.is_none() || cards.capture_entry.is_none() {
                return Ok(None);
            }
            cards.capture_surface.is_none()
        };
        if needs_surface {
            let surface = CapturedSurface::new(self, turn)?;
            self.cards
                .try_borrow_mut()
                .map_err(|_| ViewerHostError::UnsupportedCallback)?
                .capture_surface = Some(surface);
        }
        let document = self
            .document
            .as_ref()
            .ok_or(ViewerHostError::ForeignCheckpoint)?
            .clone();
        let frame = self
            .product_frame
            .as_ref()
            .ok_or(ViewerHostError::UnsupportedCallback)?
            .clone();
        let context = self
            .callbacks
            .try_borrow()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?
            .original_context();
        let mut owned_cards = self
            .cards
            .try_borrow_mut()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?;
        let cards = &mut *owned_cards;
        let surface = cards
            .capture_surface
            .as_mut()
            .ok_or(ViewerHostError::UnsupportedCallback)?;
        let capture = cards
            .capture
            .as_mut()
            .ok_or(ViewerHostError::UnsupportedCallback)?;
        // Admission records presentation input only. SDK work begins after the batch barrier.
        // Errors remain owned and are projected; no implicit retry or result adoption.
        let _ = surface.drain(capture, context);
        let _ = cards.persist_capture_evidence(false);
        let returning = cards
            .capture_surface
            .as_mut()
            .is_some_and(|surface| std::mem::take(&mut surface.return_requested));
        if returning {
            drop(owned_cards);
            return self.capture_return_after_input(turn);
        }
        let surface = cards
            .capture_surface
            .as_mut()
            .ok_or(ViewerHostError::UnsupportedCallback)?;
        let capture = cards
            .capture
            .as_mut()
            .ok_or(ViewerHostError::UnsupportedCallback)?;
        let progress = capture.poll_execution().map_err(ViewerHostError::Sdk)?;
        surface.dirty |= surface.progress != Some(progress);
        surface.dirty |= surface.projection.caption_pending();
        surface.progress = Some(progress);
        if !surface.dirty {
            return Ok(Some(HostEffects::default()));
        }
        let mut props = surface
            .model
            .props(capture, progress)
            .map_err(preparation)?;
        if surface.evidence_blocked {
            for name in [
                "canEdit",
                "canSave",
                "canAccept",
                "captureCanConsume",
                "captureCanDiscard",
                "captureCanRevert",
            ] {
                props.insert(name.into(), meari::domain::Value::Bool(false));
            }
            props.insert("captureCanRetry".into(), meari::domain::Value::Bool(true));
            props.insert(
                "resultLabel".into(),
                meari::domain::Value::String(
                    "처리 기록을 저장하지 못했습니다. 다시 시도하세요.".into(),
                ),
            );
        } else if surface.acknowledged.len() >= 8 || surface.rejected_intent.is_some() {
            for name in ["canEdit", "canSave", "canAccept", "captureCanRetry"] {
                props.insert(name.into(), meari::domain::Value::Bool(false));
            }
            props.insert(
                "resultLabel".into(),
                meari::domain::Value::String(
                    "편집 처리 기록을 확인한 뒤 작업을 다시 여세요.".into(),
                ),
            );
        }
        let guards = surface.navigation_guards();
        props.insert(
            "captureCanReturn".into(),
            meari::domain::Value::Bool(
                !guards.dirty && !guards.pending && !guards.recovery_blocked,
            ),
        );
        props.extend(surface.inputs.props());
        let bindings = surface.inputs.bindings()?;
        let generation = NonZeroU64::new(
            cards
                .capture_entry
                .as_ref()
                .ok_or(ViewerHostError::UnsupportedCallback)?
                .route
                .target
                .selection
                .document_revision
                .0,
        )
        .ok_or_else(|| preparation("Captured Card generation is zero"))?;
        let projection = surface.projection.clone();
        let commit = document.try_reproject(&turn.stamp, |document| {
            let editor = document.node_slot(
                projection.workspace_key(),
                projection.editor().clone(),
                props,
            )?;
            let props = frame.workspace_props(
                document,
                WorkspaceRoute::Cards,
                generation,
                projection.root_props(editor),
            )?;
            document.update_module_props_with_callbacks(props, bindings)
        });
        let (_, receipt) = match commit {
            Ok(result) => result,
            Err(error) => {
                let error = preparation(format!("{error:?}"));
                surface.error.get_or_insert_with(|| error.clone());
                return Err(error);
            }
        };
        let first_mount = surface.receipt.is_none();
        surface.receipt = Some(receipt.clone());
        surface.dirty = false;
        surface.projection.caption_committed();
        if first_mount {
            let ticket = cards
                .capture_entry
                .as_ref()
                .ok_or(ViewerHostError::UnsupportedCallback)?
                .route
                .ticket
                .clone();
            let current = document.stamp().map_err(|e| preparation(e.to_string()))?;
            surface.route_error = self
                .route_observation
                .as_ref()
                .ok_or(ViewerHostError::UnsupportedCallback)?
                .try_borrow_mut()
                .map_err(|_| ViewerHostError::UnsupportedCallback)?
                .arm_card_route(ticket, &receipt, &current)
                .err();
        }
        Ok(Some(HostEffects {
            committed: Some(receipt),
            redraw: true,
        }))
    }
}

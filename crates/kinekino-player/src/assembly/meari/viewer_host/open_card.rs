//! Original Viewer turn → existing route guard → original read detach → current inspect.
use super::*;
use crate::{
    assembly::meari::card_current_inspection::{CardCurrentInspection, CardInspectionInput},
    presentation::web_ui_adapter::proposals::{
        bootstrap::BootstrapSelection, product_window::product_codec_budget,
    },
};
use meari::{application::plugins::*, domain::build::TargetKind};

pub(super) fn captured_editor_unavailable() -> ViewerHostError {
    ViewerHostError::Sdk(Arc::new(crate::application::api::ApiError::new(
        crate::application::api::ErrorCode::Unsupported,
        "Project captures are read-only; no atomic capture/source mutation guard is available",
    )))
}

impl ViewerHost {
    /// true means this turn belongs to the retained Card-opening path, not the old read view.
    pub(super) fn advance_selected_card_inspection(
        &mut self,
        turn: &HostTurn,
    ) -> Result<bool, ViewerHostError> {
        if self.product_frame.is_none() || self.route_observation.is_none() {
            return Ok(false);
        }
        if turn.remaining_callbacks != 0 {
            return Ok(false);
        }
        let opening = self
            .cards
            .try_borrow()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?
            .opening
            .is_some();
        if !opening {
            // Keep normal Viewing reads alive. Read-only capture selection is never
            // silently consumed into a new inspect/attachment/editor transaction.
            return Ok(false);
        }
        if self
            .cards
            .try_borrow()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?
            .opening
            .as_ref()
            .is_some_and(|opening| opening.attempted)
        {
            return Ok(true);
        }
        let mut owner = self
            .callbacks
            .try_borrow_mut()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?;
        owner
            .consume_reads()
            .map_err(|error| ViewerHostError::Callback(Arc::new(error)))?;
        if let Some(delay) = owner.next_consumer_retry() {
            return Err(ViewerHostError::ReadConsumerRetry(delay));
        }
        if owner
            .prepare_route_release(&turn.stamp.owner)
            .map_err(|error| ViewerHostError::Callback(Arc::new(error)))?
            .is_some()
        {
            return Ok(true);
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
            .map_err(|message| {
                ViewerHostError::Callback(Arc::new(PluginError::new(
                    PluginErrorCode::Adapter,
                    message,
                )))
            })?;
        let codec = product_codec_budget();
        let limit = codec.max_pending;
        let registry = Rc::new(RefCell::new(
            PluginRegistry::new(source, TargetKind::Native, turn.stamp.owner.clone(), limit)
                .map_err(|error| ViewerHostError::Callback(Arc::new(error)))?,
        ));
        let mut cards = self
            .cards
            .try_borrow_mut()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?;
        let opening = cards
            .opening
            .as_mut()
            .ok_or(ViewerHostError::UnsupportedCallback)?;
        let selection = BootstrapSelection::try_from(opening.request.target.selection.clone())
            .map_err(|error| ViewerHostError::Sdk(Arc::new(error)))?;
        opening.attempted = true;
        let mut inspection = match CardCurrentInspection::new(CardInspectionInput {
            execution,
            scope: turn.stamp.owner.clone(),
            registry,
            context: Rc::new(RefCell::new(context)),
            namespace: format!(
                "product-card-inspect-{}",
                opening.request.ticket.generation()
            ),
            limit,
            codec,
        }) {
            Ok(inspection) => inspection,
            Err(failure) => {
                let error = failure.error.clone();
                opening.start_failure = Some(failure);
                return Err(ViewerHostError::Callback(Arc::new(error)));
            }
        };
        let result = inspection.begin(selection);
        if let Err(error) = result {
            let error = Arc::new(error);
            opening.api_error = Some(error.clone());
            cards.inspection = Some(inspection);
            return Err(ViewerHostError::Sdk(error));
        }
        cards.inspection = Some(inspection);
        Ok(true)
    }
}

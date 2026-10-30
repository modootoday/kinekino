//! Card-only compatibility over the common original proposal mount.
#[cfg(test)]
use super::mount_proposal::ProductProposalMountPartial as ProductCardMountPartial;
pub(super) use super::mount_proposal::{
    ProductProposalMountFailure as ProductCardMountFailure,
    ProductProposalMountInput as ProductCardMountInput,
};
use super::*;
use crate::presentation::web_ui_adapter::proposals::product_window::ProductWindowLabels;
#[cfg(test)]
use crate::presentation::web_ui_adapter::{
    player_shell::document::product_frame::{ProductFrameBlueprint, WorkspaceRoute},
    proposals::{
        bootstrap::BootstrapPanel,
        card_selection_queue::{CARD_SELECTION_BINDING, CardSelectionQueue},
        frame_composition::FrameComposition,
        product_window::{product_codec_budget, product_modules_for_panel},
    },
};
#[cfg(test)]
use crate::{
    application::{api::RequestContext, launch::native_proposal::NativeProposalSelection},
    presentation::web_ui_adapter::proposals::bootstrap::BootstrapSelection,
};
use meari::application::plugins::{PluginError, PluginErrorCode};
#[cfg(test)]
use meari::application::plugins::{PluginRegistry, PluginScope};
#[cfg(test)]
use meari::domain::build::TargetKind;
#[cfg(test)]
use meari::domain::host_callbacks::CallbackSignature;
use std::num::NonZeroU64;
impl ViewerHost {
    pub(super) fn finish_inspected_card(
        &mut self,
        turn: HostTurn,
    ) -> Result<HostEffects, ViewerHostError> {
        use crate::application::api::{ApiError, ErrorCode};
        use crate::presentation::web_ui_adapter::proposals::pending_host::PendingOutput;
        {
            let mut cards = self
                .cards
                .try_borrow_mut()
                .map_err(|_| ViewerHostError::UnsupportedCallback)?;
            if let Some(opening) = &mut cards.opening {
                // CardOpening's target comes exclusively from the captured project queue.
                // Retain any prior terminal/registration for original close, never bind it
                // as an editable Card merely because current inspection happened to match.
                let error = open_card::captured_editor_unavailable();
                if let ViewerHostError::Sdk(error) = &error {
                    opening.api_error = Some(error.clone());
                }
                opening.panel_attempted = true;
                return Err(error);
            }
        }
        let owner = self
            .callbacks
            .try_borrow()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?;
        if !owner.is_released() {
            return Err(ViewerHostError::UnsupportedCallback);
        }
        let execution = owner.original_execution();
        let context = owner.original_context();
        drop(owner);
        let queue = self.card_selection_queue();
        let input = {
            let mut cards = self
                .cards
                .try_borrow_mut()
                .map_err(|_| ViewerHostError::UnsupportedCallback)?;
            let cards = &mut *cards;
            let opening = cards
                .opening
                .as_mut()
                .ok_or(ViewerHostError::UnsupportedCallback)?;
            let inspection = cards
                .inspection
                .as_mut()
                .ok_or(ViewerHostError::UnsupportedCallback)?;
            let terminal = inspection
                .terminal()
                .ok_or(ViewerHostError::UnsupportedCallback)?;
            let stored = match &terminal.outcome {
                Ok(PendingOutput::Inspected(stored)) if !terminal.cancellation_requested => {
                    stored.clone()
                }
                outcome => {
                    let error = match outcome {
                        Err(error) => ApiError {
                            code: error.code,
                            message: error.message.clone(),
                            retryable: error.retryable,
                            details: error.details.clone(),
                            remediation: error.remediation.clone(),
                            request_id: error.request_id.clone(),
                        },
                        _ => ApiError::new(
                            ErrorCode::NeedsInput,
                            "Current Card result is cancelled or not an inspected proposal",
                        ),
                    };
                    let error = Arc::new(error);
                    opening.api_error = Some(error.clone());
                    opening.panel_attempted = true;
                    return Err(ViewerHostError::Sdk(error));
                }
            };
            inspection
                .release()
                .map_err(|error| ViewerHostError::Callback(Arc::new(error)))?;
            let generation = NonZeroU64::new(opening.request.ticket.generation()).ok_or(ViewerHostError::Navigation(
                crate::presentation::web_ui_adapter::player_shell::NavigationError::GenerationExhausted,
            ))?;
            opening.retained_inspection = cards.inspection.take();
            opening.panel_attempted = true;
            ProductCardMountInput {
                document: self
                    .document
                    .clone()
                    .ok_or(ViewerHostError::UnsupportedCallback)?,
                turn,
                frame: self
                    .product_frame
                    .clone()
                    .ok_or(ViewerHostError::UnsupportedCallback)?,
                selected: opening.request.target.selection.clone(),
                stored,
                client: self.client.clone(),
                execution,
                context,
                namespace: format!("product-card-editor-{}", generation.get()),
                generation,
                labels: ProductWindowLabels::korean(),
                queue,
                navigation: None,
            }
        };
        match mount_current_card(input) {
            Err(failure) => {
                let diagnostic = failure.diagnostic.clone();
                self.cards
                    .try_borrow_mut()
                    .map_err(|_| ViewerHostError::UnsupportedCallback)?
                    .mount_failure = Some(failure);
                Err(ViewerHostError::Callback(Arc::new(PluginError::new(
                    PluginErrorCode::Adapter,
                    diagnostic,
                ))))
            }
            Ok((bridge, receipt)) => {
                if let Err(bridge) = self.bind_active_card(bridge) {
                    let mut cards = self
                        .cards
                        .try_borrow_mut()
                        .map_err(|_| ViewerHostError::UnsupportedCallback)?;
                    cards.rejected_bridge = Some(bridge);
                    if let Some(opening) = &mut cards.opening {
                        opening.api_error = Some(Arc::new(ApiError::new(
                            ErrorCode::NeedsInput,
                            "Card props committed; active host binding rejected and original owner retained",
                        )));
                    }
                } else {
                    use crate::presentation::web_ui_adapter::player_shell::NavigationError;
                    let ticket = {
                        let mut cards = self
                            .cards
                            .try_borrow_mut()
                            .map_err(|_| ViewerHostError::UnsupportedCallback)?;
                        cards.opening.as_mut().map(|opening| {
                            opening.receipt = Some(receipt.clone());
                            opening.request.ticket.clone()
                        })
                    };
                    let current = self
                        .document
                        .as_ref()
                        .and_then(|document| document.stamp().ok());
                    let armed = match (ticket, current, &self.route_observation) {
                        (Some(ticket), Some(current), Some(observer)) => match observer
                            .try_borrow_mut()
                        {
                            Ok(mut observer) => observer.arm_card_route(ticket, &receipt, &current),
                            Err(_) => Err(NavigationError::RecoveryBlocked),
                        },
                        _ => Err(NavigationError::ForeignObservation),
                    };
                    if let Err(error) = armed
                        && let Some(opening) = &mut self
                            .cards
                            .try_borrow_mut()
                            .map_err(|_| ViewerHostError::UnsupportedCallback)?
                            .opening
                    {
                        opening.route_error = Some(error);
                    }
                }
                // A successful document commit always returns its exact receipt, even if
                // the subsequent host bind must be recovered with its retained owner.
                Ok(HostEffects {
                    committed: Some(receipt),
                    redraw: true,
                })
            }
        }
    }
}
pub(super) fn mount_current_card(
    input: ProductCardMountInput,
) -> Result<(ActiveCardBridge, HostCommitReceipt), Box<ProductCardMountFailure>> {
    if !matches!(
        &input.selected.target,
        crate::application::launch::native_proposal::NativeProposalTarget::Card { .. }
    ) {
        return Err(Box::new(ProductCardMountFailure {
            input,
            diagnostic: "Card compatibility mount requires an explicit Card".into(),
            partial: None,
        }));
    }
    super::mount_proposal::mount_current_proposal(input, None)
        .map(|mounted| (mounted.bridge, mounted.receipt))
}

#[cfg(test)]
#[path = "mount_card/tests.rs"]
mod tests;

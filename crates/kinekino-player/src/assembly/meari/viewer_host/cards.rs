//! The original Viewer and execution shutdown share the same active Card owner.
mod capture;
use crate::{
    assembly::meari::scoped_card_session::ScopedProposalLease,
    presentation::web_ui_adapter::proposals::window_bridge::ProposalWindowBridge,
};
use meari::application::plugins::PluginError;

pub type ActiveCardBridge = ProposalWindowBridge<ScopedProposalLease>;
pub(super) struct CardOpening {
    pub(super) request:
        crate::presentation::web_ui_adapter::proposals::card_selection_queue::CardSelectionRoute,
    pub(super) attempted: bool,
    pub(super) panel_attempted: bool,
    pub(super) retained_inspection:
        Option<crate::assembly::meari::card_current_inspection::CardCurrentInspection>,
    pub(super) receipt: Option<meari_native::infrastructure::native_host::HostCommitReceipt>,
    pub(super) route_error:
        Option<crate::presentation::web_ui_adapter::player_shell::NavigationError>,
    pub(super) start_failure:
        Option<Box<crate::assembly::meari::card_current_inspection::CardInspectionStartFailure>>,
    pub(super) api_error: Option<std::sync::Arc<crate::application::api::ApiError>>,
}

#[derive(Default)]
pub(super) struct CardRoute {
    pub(super) navigation: std::rc::Rc<std::cell::RefCell<super::source_navigation::NavigationIntents>>,
    pub(super) active_task: Option<crate::presentation::web_ui_adapter::player_shell::document::product_frame::StudioTask>,
    pub(super) live_opening: Option<super::live_entry::LiveOpening>,
    pub(super) capture_evidence: Option<super::capture_evidence::CaptureEvidenceOwner>,
    pub(super) capture_surface: Option<super::capture_surface::CapturedSurface>,
    pub(super) capture_entry: Option<super::capture_entry::CapturedEntry>,
    pub(super) capture: Option<crate::presentation::web_ui_adapter::proposals::capture_mutation::CaptureMutationAdapter>,
    pub(super) capture_close_receipt: Option<crate::presentation::web_ui_adapter::proposals::capture_mutation::CaptureExecutionAcknowledgement>,
    pub(super) capture_close_error: Option<std::sync::Arc<crate::application::api::ApiError>>,
    pub(super) viewing_return: Option<super::return_viewing::ViewingReturn>,
    pub(super) mount_failure: Option<Box<super::mount_card::ProductCardMountFailure>>,
    pub(super) rejected_bridge: Option<Box<ActiveCardBridge>>,
    pub(super) opening: Option<CardOpening>,
    pub(super) active: Option<ActiveCardBridge>,
    pub(super) inspection:
        Option<crate::assembly::meari::card_current_inspection::CardCurrentInspection>,
}
impl CardRoute {
    pub(super) fn prepare_live_close(&mut self) {
        if let Some(entry) = &mut self.live_opening {
            entry.phase = super::live_entry::LiveEntryPhase::Closing;
        }
    }
    pub(super) fn poll_release(&mut self) -> Result<bool, PluginError> {
        if let Some(entry) = &mut self.live_opening
            && let Some(failure) = &mut entry.start_failure
        {
            failure.retry_unbind()?;
        }
        if let Some(entry) = &mut self.live_opening
            && let Some(inspection) = &mut entry.retained_inspection
            && !inspection.poll_close()?
        {
            return Ok(false);
        }
        if let Some(entry) = &mut self.live_opening
            && let Some(failure) = &mut entry.mount_failure
            && !failure.poll_release()?
        {
            return Ok(false);
        }
        if let Some(entry) = &mut self.live_opening
            && let Some(bridge) = &mut entry.rejected_bridge
            && let Err(error) = bridge.unbind_scope()
        {
            if bridge.scope_close_waiting() {
                return Ok(false);
            }
            return Err(error);
        }
        if !self.poll_capture_release()? {
            return Ok(false);
        }
        self.persist_capture_evidence(false).map_err(|error| {
            PluginError::new(
                meari::application::plugins::PluginErrorCode::Adapter,
                error.message.clone(),
            )
        })?;
        if let Some(opening) = &mut self.opening
            && let Some(failure) = &mut opening.start_failure
        {
            failure.retry_unbind()?;
        }
        if let Some(opening) = &mut self.opening
            && let Some(inspection) = &mut opening.retained_inspection
            && !inspection.poll_close()?
        {
            return Ok(false);
        }
        if let Some(failure) = &mut self.mount_failure
            && !failure.poll_release()?
        {
            return Ok(false);
        }
        if let Some(bridge) = &mut self.rejected_bridge
            && let Err(error) = bridge.unbind_scope()
        {
            if bridge.scope_close_waiting() {
                return Ok(false);
            }
            return Err(error);
        }
        if let Some(inspection) = &mut self.inspection
            && !inspection.poll_close()?
        {
            return Ok(false);
        }
        let Some(active) = &mut self.active else {
            return Ok(true);
        };
        match active.unbind_scope() {
            Ok(()) => Ok(true),
            Err(_) if active.scope_close_waiting() => Ok(false),
            Err(error) => Err(error),
        }
    }
}

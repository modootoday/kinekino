//! Explicit trusted composition only. Captured-selection callbacks remain read-only.
use super::ViewerHost;
use crate::{
    application::api::{ApiError, ErrorCode},
    presentation::web_ui_adapter::proposals::capture_mutation::CaptureMutationAdapter,
};
use std::cell::RefMut;

#[must_use = "Retain the original adapter and pending intent on attach failure"]
pub struct CaptureAttachFailure {
    pub error: ApiError,
    pub adapter: CaptureMutationAdapter,
}
fn unavailable() -> ApiError {
    ApiError::new(
        ErrorCode::NeedsInput,
        "Original Viewer capture scope is borrowed, occupied or closing",
    )
}
impl ViewerHost {
    /// Does not release existing read/Card attachments or change the captured route guard.
    pub fn attach_capture_adapter(
        &mut self,
        mut adapter: CaptureMutationAdapter,
    ) -> Result<(), Box<CaptureAttachFailure>> {
        if self.shutdown_started {
            return Err(Box::new(CaptureAttachFailure {
                error: unavailable(),
                adapter,
            }));
        }
        let mut cards = match self.cards.try_borrow_mut() {
            Ok(cards) => cards,
            Err(_) => {
                return Err(Box::new(CaptureAttachFailure {
                    error: unavailable(),
                    adapter,
                }));
            }
        };
        if cards.capture.is_some()
            || cards
                .capture_entry
                .as_ref()
                .is_some_and(|entry| entry.adapter.is_some())
            || cards.active.is_some()
            || cards.opening.is_some()
            || cards.inspection.is_some()
            || cards.mount_failure.is_some()
            || cards.rejected_bridge.is_some()
            || cards.viewing_return.is_some()
        {
            return Err(Box::new(CaptureAttachFailure {
                error: unavailable(),
                adapter,
            }));
        }
        if let Err(error) = adapter.attach_execution(&self.closing.original_capture_execution()) {
            return Err(Box::new(CaptureAttachFailure { error, adapter }));
        }
        cards.capture = Some(adapter);
        Ok(())
    }
    /// The existing route owner retains the adapter during dispatch, terminal decisions and close.
    pub fn capture_adapter_mut(&self) -> Result<RefMut<'_, CaptureMutationAdapter>, ApiError> {
        let cards = self.cards.try_borrow_mut().map_err(|_| unavailable())?;
        RefMut::filter_map(cards, |cards| cards.capture.as_mut()).map_err(|_| unavailable())
    }
}

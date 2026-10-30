use super::*;
use crate::{
    application::api::ApiError,
    presentation::web_ui_adapter::proposals::capture_mutation::CaptureExecutionProgress,
};
use meari::application::plugins::PluginErrorCode;
use std::sync::Arc;

impl CardRoute {
    fn capture_error(&mut self, error: Arc<ApiError>) -> PluginError {
        self.capture_close_error
            .get_or_insert_with(|| error.clone());
        PluginError::new(PluginErrorCode::Adapter, error.message.clone())
    }
    pub(super) fn poll_capture_release(&mut self) -> Result<bool, PluginError> {
        if let Some(surface) = &mut self.capture_surface {
            surface.stop()?;
        }
        let Some(capture) = self.capture.as_mut() else {
            return Ok(true);
        };
        if !capture.execution_attached() {
            return Ok(true);
        }
        let progress = match capture.request_execution_close() {
            Ok(progress) => progress,
            Err(error) => return Err(self.capture_error(error)),
        };
        match progress {
            CaptureExecutionProgress::ConsumedAckPending
            | CaptureExecutionProgress::DiscardedAckPending => {
                // The original view already made the explicit consume/discard decision.
                // Parent close may retry ACK, but must not consume or discard on its behalf.
                let receipt = match capture.acknowledge_execution_terminal() {
                    Ok(receipt) => receipt,
                    Err(error) => return Err(self.capture_error(error)),
                };
                self.capture_close_receipt = Some(receipt);
            }
            CaptureExecutionProgress::Idle if capture.display().pending.is_none() => {}
            _ => return Ok(false),
        }
        match capture.release_execution() {
            Ok(()) => Ok(true),
            Err(error) => Err(self.capture_error(error)),
        }
    }
}

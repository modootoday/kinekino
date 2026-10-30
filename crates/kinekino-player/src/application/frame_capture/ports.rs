//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

pub trait FrameCapturePort {
    fn capture_recoverable(
        &self,
        request: &CaptureFrame,
        timeout_ms: u64,
        context: FrameExecutionContext<'_>,
    ) -> Result<CapturedFrame, ApiError> {
        match context.control {
            Some(control) => self.capture_controlled(request, timeout_ms, control),
            None => self.capture(request, timeout_ms),
        }
    }
    fn capability(&self) -> FrameCaptureCapability;
    fn capture(&self, request: &CaptureFrame, timeout_ms: u64) -> Result<CapturedFrame, ApiError>;
    fn cooperative_cancel_supported(&self) -> bool {
        false
    }
    /// Legacy adapters retain their actual response; this default only checks admission.
    fn capture_controlled(
        &self,
        request: &CaptureFrame,
        timeout_ms: u64,
        control: &dyn crate::application::review_source::MutationControl,
    ) -> Result<CapturedFrame, ApiError> {
        control.checkpoint()?;
        self.capture(request, timeout_ms)
    }
}

use crate::application::{
    api::{ApiError, ErrorCode},
    sdk_execution::CompletionNotifier,
};
use meari_native::infrastructure::window::{NativeStartupError, NativeStartupWake};

/// The Native startup owner already handles dirty/coalesced routing and attach forwarding.
pub struct NativeStartupCompletionNotifier {
    wake: NativeStartupWake,
}
impl NativeStartupCompletionNotifier {
    pub fn new(wake: NativeStartupWake) -> Self {
        Self { wake }
    }
}
impl CompletionNotifier for NativeStartupCompletionNotifier {
    fn notify(&self) -> Result<(), ApiError> {
        self.wake.notify().map(|_| ()).map_err(notification_error)
    }
}
fn notification_error(error: NativeStartupError) -> ApiError {
    let (code, kind) = match &error {
        NativeStartupError::Busy => (ErrorCode::NeedsInput, "busy"),
        NativeStartupError::Closed => (ErrorCode::SourceExpired, "closed"),
        NativeStartupError::Stale => (ErrorCode::SourceExpired, "stale"),
        NativeStartupError::AlreadyIssued => (ErrorCode::InvalidArgument, "already_issued"),
        NativeStartupError::CounterExhausted => (ErrorCode::ResourceExhausted, "counter_exhausted"),
        NativeStartupError::Invalid(_) => (ErrorCode::InvalidArgument, "invalid"),
        NativeStartupError::Attach(_) => (ErrorCode::EngineFailed, "attach"),
        NativeStartupError::Loop(_) => (ErrorCode::EngineFailed, "loop"),
    };
    let mut result = ApiError::new(code, "Native startup completion notification failed");
    result.details = serde_json::json!({"adapter":"native_startup_completion","kind":kind,"native_error":error.to_string()});
    result
}

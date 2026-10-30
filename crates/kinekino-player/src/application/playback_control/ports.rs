//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

pub trait PlaybackControlPort: Send + Sync {
    fn audio_handoff_available(&self) -> bool {
        false
    }
    fn audio_handoff_status(&self) -> Result<Value, ApiError> {
        Err(ApiError::new(
            ErrorCode::Unsupported,
            "Native audio handoff observations are unavailable",
        ))
    }
    fn available(&self) -> bool {
        true
    }
    fn submit(&self, control: PlaybackControl) -> Result<ControlReceipt, ApiError>;
}

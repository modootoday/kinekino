//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

/// Persists original transport facts in addition to original process receipts.
pub trait SourceReceiptSink: ParentReceiptSink + Send {
    fn record_prepared_transport<G: Send>(
        &mut self,
        _facts: &LaunchCloseFacts,
        _original: &ParentLaunch<G>,
    ) -> Result<(), ApiError> {
        Err(ApiError::new(
            ErrorCode::Unsupported,
            "Original prepared transport receipt unavailable",
        ))
    }
    fn record_transport<G: Send>(
        &mut self,
        facts: &LaunchCloseFacts,
        original: &ParentTransfer<G>,
    ) -> Result<(), ApiError>;
}

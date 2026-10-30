//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

/// Implementations bind this journal to the same trusted Jobs store/host identity.
/// No execution is automatically resumed on open or after an error.
pub trait BatchStore: Send + Sync {
    fn scope_sha256(&self) -> &str;
    /// Check validate_initial(document, this scope) before the first create-only record.
    fn create(&self, document: &BatchDocument) -> Result<(), ApiError>;
    fn acquire(&self, batch_id: &str) -> Result<Box<dyn BatchLease + '_>, ApiError>;
}

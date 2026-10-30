//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

/// Implementations must check CAS and prepare_publication under the same writer lock.
pub trait CardProjectStore: Send + Sync {
    fn load(
        &self,
        document_id: &str,
        revision: Option<DecimalU64>,
    ) -> Result<StoredCardProject, ApiError>;
    fn publish(
        &self,
        expected: Option<DecimalU64>,
        document: &CardDocument,
    ) -> Result<StoredCardProject, ApiError>;
}

//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

pub trait ClipProjectStore: Send + Sync {
    fn load(&self, id: &str, revision: Option<DecimalU64>) -> Result<StoredClipProject, ApiError>;
    /// Same native writer lease must cover current verified head, CAS and publication.
    fn publish(
        &self,
        expected: Option<DecimalU64>,
        document: &ClipDocument,
    ) -> Result<StoredClipProject, ApiError>;
}

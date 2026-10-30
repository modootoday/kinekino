//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

pub trait ProjectStore: Send + Sync {
    fn available(&self) -> bool {
        true
    }
    fn load(
        &self,
        project_id: &str,
        revision: Option<u64>,
    ) -> Result<StoredProjectRevision, ApiError>;
    fn publish(
        &self,
        expected: Option<u64>,
        project: &ProjectRevision,
    ) -> Result<StoredProjectRevision, ApiError>;
}

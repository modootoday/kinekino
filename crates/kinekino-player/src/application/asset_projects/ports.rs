//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

pub trait AssetProjectStore: Send + Sync {
    fn load(&self, id: &str, revision: Option<DecimalU64>) -> Result<StoredAssetProject, ApiError>;
    /// Re-read the verified head and perform prepare_publication under one writer lock.
    fn publish(
        &self,
        expected: Option<DecimalU64>,
        document: &AssetDocument,
    ) -> Result<StoredAssetProject, ApiError>;
}

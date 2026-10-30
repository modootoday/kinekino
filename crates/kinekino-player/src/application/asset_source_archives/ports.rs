//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

pub trait AssetSourceArchivePort: Send + Sync {
    fn export(
        &self,
        input: &ExportAssetSources,
        control: &dyn MutationControl,
    ) -> Result<AssetSourcesObservation, ApiError>;
    fn inspect(
        &self,
        input: &InspectAssetSources,
        control: &dyn MutationControl,
    ) -> Result<AssetSourcesObservation, ApiError>;
}

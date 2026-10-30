//! Transport-neutral original source storage; no Jobs or review authority.
#[path = "asset_source_archives/ports.rs"]
mod ports;
pub use ports::AssetSourceArchivePort;
mod import;
use crate::application::{
    api::{ApiError, ErrorCode},
    asset_bundle::{AssetSourceManifest, MissingAssetFramePolicy, SavedAssetProjectPin},
    asset_projects::AssetPublication,
    review_source::MutationControl,
};
use crate::domain::{
    api_values::DecimalU64,
    identifiers::{valid_id, valid_sha256},
};
pub use import::{AssetSourceImportPlan, ImportAssetSources, prepare_import};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportAssetSources {
    pub archive_id: String,
    pub project: SavedAssetProjectPin,
    pub missing_frames: MissingAssetFramePolicy,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InspectAssetSources {
    pub archive_id: String,
    pub manifest_sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetSourcesObservation {
    pub archive_id: String,
    pub manifest_sha256: String,
    pub manifest_bytes: DecimalU64,
    pub manifest: AssetSourceManifest,
    pub project: AssetPublication,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetSourceImportReceipt {
    pub schema_version: String,
    pub source: InspectAssetSources,
    pub original_project: SavedAssetProjectPin,
    pub imported_project: SavedAssetProjectPin,
    pub plan_sha256: String,
    pub quality_approved: bool,
    pub review_admitted: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportedAssetSources {
    pub receipt: AssetSourceImportReceipt,
    pub commit_sha256: String,
    pub project: crate::application::asset_projects::StoredAssetProject,
}
impl ExportAssetSources {
    pub fn validate(&self) -> Result<(), ApiError> {
        identity(&self.archive_id)?;
        self.project.validate()
    }
}
impl InspectAssetSources {
    pub fn validate(&self) -> Result<(), ApiError> {
        identity(&self.archive_id)?;
        if !valid_sha256(&self.manifest_sha256) {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Exact source manifest SHA required",
            ));
        }
        Ok(())
    }
}
fn identity(id: &str) -> Result<(), ApiError> {
    if valid_id(id) {
        Ok(())
    } else {
        Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Source archive identity required",
        ))
    }
}

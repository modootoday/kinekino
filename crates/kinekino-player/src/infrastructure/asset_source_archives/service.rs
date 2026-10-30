use super::*;
use crate::application::{
    asset_bundle::capture_source_project, asset_projects::publication_sha256,
    asset_source_archives::*,
};
use crate::infrastructure::{asset_inputs::FixedAssetInputs, asset_projects::AssetProjectFiles};
use std::sync::Arc;
pub struct NativeAssetSourceArchiveService {
    pub(crate) _host_guards: Arc<Vec<File>>,
    pub projects: Arc<AssetProjectFiles>,
    pub inputs: Arc<FixedAssetInputs>,
    pub archives: AssetSourceArchiveFiles,
}
pub(super) fn observation(
    id: &str,
    sha: &str,
    archive: &RetainedAssetSourceArchive,
) -> Result<AssetSourcesObservation, ApiError> {
    let bytes = serde_json::to_vec(archive.manifest()).map_err(|_| changed())?;
    // Persisted sources.json is canonical application JSON; inspect must pin its exact bytes.
    use sha2::{Digest, Sha256};
    if format!("{:x}", Sha256::digest(&bytes)) != sha
        || publication_sha256(archive.project())? != archive.manifest().project.publication_sha256
    {
        return Err(changed());
    }
    Ok(AssetSourcesObservation {
        archive_id: id.into(),
        manifest_sha256: sha.into(),
        manifest_bytes: crate::domain::api_values::DecimalU64(bytes.len() as u64),
        manifest: archive.manifest().clone(),
        project: archive.project().clone(),
    })
}
impl AssetSourceArchivePort for NativeAssetSourceArchiveService {
    fn export(
        &self,
        input: &ExportAssetSources,
        control: &dyn MutationControl,
    ) -> Result<AssetSourcesObservation, ApiError> {
        input.validate()?;
        let captured = capture_source_project(
            self.projects.as_ref(),
            &input.project,
            input.missing_frames,
            control,
        )?;
        let sources = captured.capture_sources(self.inputs.as_ref(), control)?;
        let published = self
            .archives
            .publish(&input.archive_id, &sources, control)?;
        observation(
            &input.archive_id,
            &published.pin.manifest_sha256,
            &published.archive,
        )
    }
    fn inspect(
        &self,
        input: &InspectAssetSources,
        control: &dyn MutationControl,
    ) -> Result<AssetSourcesObservation, ApiError> {
        input.validate()?;
        let archive = self
            .archives
            .inspect(&input.archive_id, &input.manifest_sha256, control)?;
        observation(&input.archive_id, &input.manifest_sha256, &archive)
    }
}

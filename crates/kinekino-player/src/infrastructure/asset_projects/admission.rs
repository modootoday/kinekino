use super::*;
use crate::application::{
    asset_bundle::{AssetProjectAdmission, AssetProjectLease, SavedAssetProjectPin},
    asset_projects::AssetPublication,
    review_source::MutationControl,
};
pub struct NativeAssetProjectLease {
    history: records::RetainedAssetHistory,
    _root_guards: Vec<File>,
}
impl AssetProjectLease for NativeAssetProjectLease {
    fn publication(&self) -> &AssetPublication {
        &self.history.project.publication
    }
    fn publication_sha256(&self) -> &str {
        &self.history.project.publication_sha256
    }
}
impl AssetProjectAdmission for AssetProjectFiles {
    type Project = NativeAssetProjectLease;
    fn open_project(
        &self,
        pin: &SavedAssetProjectPin,
        control: &dyn MutationControl,
    ) -> Result<Self::Project, ApiError> {
        pin.validate()?;
        control.checkpoint()?;
        let _writer = paths::lock(&self.root.join("asset-projects-writer.lock"))?;
        let history = self.scan_retained(
            &pin.document_id,
            &self.directory(&pin.document_id)?,
            Some(pin.publication_revision),
        )?;
        if history.project.publication_sha256 != pin.publication_sha256 {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Saved asset project differs from exact selected publication",
            ));
        }
        let guards = self
            ._guards
            .iter()
            .map(File::try_clone)
            .collect::<Result<Vec<_>, _>>()
            .map_err(io)?;
        control.checkpoint()?;
        Ok(NativeAssetProjectLease {
            history,
            _root_guards: guards,
        })
    }
}

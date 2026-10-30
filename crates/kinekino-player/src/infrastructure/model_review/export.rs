//! Guarded immutable package export for trusted local derived-package publishers.
use super::*;
use crate::domain::model_review::EvidenceAsset;
use crate::domain::review_source::ReviewCase;
pub(crate) struct PackageExport {
    pub package: ReviewPackage,
    pub package_file: File,
    pub objects: Vec<(EvidenceAsset, File)>,
    pub _session: Box<dyn ReviewSession>,
}
impl ReviewFiles {
    pub(crate) fn canonical_export_root(&self) -> &Path {
        &self.root.path
    }
    pub(crate) fn export_case(&self, case: &ReviewCase) -> Result<PackageExport, ApiError> {
        case.validate()
            .map_err(|_| ApiError::new(ErrorCode::InvalidArgument, "Invalid export case"))?;
        let session = ReviewStore::open(self, &case.package_sha256)?;
        let package = session.package().clone();
        let selected = package
            .items
            .iter()
            .find(|item| item.item_id == case.item_id)
            .ok_or_else(|| ApiError::new(ErrorCode::NotFound, "Export item missing"))?;
        if selected.source.sha256 != case.source_request_sha256 {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Export source request differs",
            ));
        }
        let package_file = paths::file(
            &self
                .root
                .path
                .join("packages")
                .join(format!("{}.json", case.package_sha256)),
        )?;
        let mut objects = Vec::new();
        let mut seen = BTreeSet::new();
        for asset in package.assets() {
            if seen.insert(asset.sha256.clone()) {
                objects.push((
                    asset.clone(),
                    paths::file(&self.root.path.join("objects").join(&asset.sha256))?,
                ));
            }
        }
        Ok(PackageExport {
            package,
            package_file,
            objects,
            _session: session,
        })
    }
}

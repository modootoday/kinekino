//! Import planning is data only. The host must retain/readmit the original archive.
use super::*;
use crate::domain::asset_generation::{AssetDocument, AssetFrameEntry, AssetMaskReview};
use sha2::{Digest, Sha256};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportAssetSources {
    pub archive_id: String,
    pub manifest_sha256: String,
    pub new_document_id: String,
    pub acknowledge_unreviewed_masks: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetSourceImportPlan {
    pub source: InspectAssetSources,
    pub original_project: SavedAssetProjectPin,
    pub document: AssetDocument,
    pub quality_approved: bool,
    pub review_admitted: bool,
}
/// Re-identifies an editable draft, never an admitted source or persisted import receipt.
pub fn prepare_import(
    observed: &AssetSourcesObservation,
    input: &ImportAssetSources,
) -> Result<AssetSourceImportPlan, ApiError> {
    InspectAssetSources {
        archive_id: input.archive_id.clone(),
        manifest_sha256: input.manifest_sha256.clone(),
    }
    .validate()?;
    identity(&input.new_document_id)?;
    if !input.acknowledge_unreviewed_masks {
        return Err(ApiError::new(
            ErrorCode::NeedsInput,
            "Explicit unreviewed mask acknowledgement required for imported draft",
        ));
    }
    if input.archive_id != observed.archive_id
        || input.manifest_sha256 != observed.manifest_sha256
        || input.new_document_id == observed.project.document.document_id
    {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Import must select exact archive and a new project identity",
        ));
    }
    let bytes = serde_json::to_vec(&observed.manifest).map_err(|_| {
        ApiError::new(
            ErrorCode::Internal,
            "Source import manifest encoding failed",
        )
    })?;
    if format!("{:x}", Sha256::digest(&bytes)) != observed.manifest_sha256
        || bytes.len() as u64 != observed.manifest_bytes.0
    {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Import metadata differs from pinned archive",
        ));
    }
    let project_bytes = serde_json::to_vec(&observed.project)
        .map_err(|_| ApiError::new(ErrorCode::Internal, "Source import project encoding failed"))?;
    struct Observe;
    impl MutationControl for Observe {
        fn checkpoint(&self) -> Result<(), ApiError> {
            Ok(())
        }
    }
    observed
        .manifest
        .validate_project(&project_bytes, observed.manifest_bytes.0, &Observe)?;
    let mut document = observed.project.document.clone();
    document.document_id = input.new_document_id.clone();
    document.revision = DecimalU64(1);
    for frame in &mut document.frames {
        if let AssetFrameEntry::Available { input, .. } = frame {
            input.review = AssetMaskReview::UnreviewedAcknowledged {
                mask: input.mask.blob.clone(),
            };
        }
    }
    document
        .validate()
        .map_err(|message| ApiError::new(ErrorCode::InvalidArgument, message))?;
    Ok(AssetSourceImportPlan {
        source: InspectAssetSources {
            archive_id: observed.archive_id.clone(),
            manifest_sha256: observed.manifest_sha256.clone(),
        },
        original_project: observed.manifest.project.clone(),
        document,
        quality_approved: false,
        review_admitted: false,
    })
}

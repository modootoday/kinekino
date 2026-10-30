//! Immutable asset project revisions. Saving a plan never admits its source or review.
#[path = "ports.rs"]
mod ports;
use crate::{
    application::api::{ApiError, ErrorCode},
    domain::{
        api_values::DecimalU64,
        asset_generation::AssetDocument,
        identifiers::{valid_id, valid_sha256},
    },
};
pub use ports::AssetProjectStore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetPublication {
    pub schema_version: String,
    pub publication_revision: DecimalU64,
    pub parent_publication_sha256: Option<String>,
    pub document: AssetDocument,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredAssetProject {
    pub publication_sha256: String,
    pub latest_publication_revision: DecimalU64,
    pub pending_publication_count: DecimalU64,
    pub history_bytes: DecimalU64,
    pub publication: AssetPublication,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetProjectInput {
    pub document: AssetDocument,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InspectAssetProject {
    pub document_id: String,
    pub publication_revision: Option<DecimalU64>,
}
fn invalid(message: &str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
pub fn create(
    store: &dyn AssetProjectStore,
    document: &AssetDocument,
) -> Result<StoredAssetProject, ApiError> {
    prepare_publication(None, document)?;
    store.publish(None, document)
}
pub fn save(
    store: &dyn AssetProjectStore,
    expected: DecimalU64,
    document: &AssetDocument,
) -> Result<StoredAssetProject, ApiError> {
    if expected.0 == 0 {
        return Err(invalid(
            "Positive expected asset publication revision required",
        ));
    }
    document.validate().map_err(invalid)?;
    store.publish(Some(expected), document)
}
pub fn inspect(
    store: &dyn AssetProjectStore,
    id: &str,
    revision: Option<DecimalU64>,
) -> Result<StoredAssetProject, ApiError> {
    if !valid_id(id) || revision.is_some_and(|revision| revision.0 == 0) {
        return Err(invalid(
            "Invalid asset project identity or publication revision",
        ));
    }
    store.load(id, revision)
}
pub fn publication_sha256(publication: &AssetPublication) -> Result<String, ApiError> {
    Ok(format!(
        "{:x}",
        Sha256::digest(
            serde_json::to_vec(publication)
                .map_err(|_| invalid("Invalid asset publication encoding"))?
        )
    ))
}
pub fn prepare_publication(
    previous: Option<&StoredAssetProject>,
    document: &AssetDocument,
) -> Result<AssetPublication, ApiError> {
    document.validate().map_err(invalid)?;
    let (revision, parent) = if let Some(previous) = previous {
        let prior = &previous.publication;
        prior.document.validate().map_err(invalid)?;
        if prior.schema_version != "asset-project-publication/1"
            || prior.publication_revision.0 == 0
            || previous.latest_publication_revision != prior.publication_revision
            || !valid_sha256(&previous.publication_sha256)
            || publication_sha256(prior)? != previous.publication_sha256
        {
            return Err(invalid(
                "Previous asset publication must be the original verified head",
            ));
        }
        if prior.document.document_id != document.document_id {
            return Err(invalid("Asset project identity is immutable"));
        }
        if document.revision.0 <= prior.document.revision.0 {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Asset working revision must advance",
            ));
        }
        for (id, source) in &prior.document.sources {
            let next = document
                .sources
                .get(id)
                .ok_or_else(|| invalid("Retain source slot history when removing asset frames"))?;
            if source.position != next.position
                || (source.selected_image.is_some() && source.selected_image != next.selected_image)
            {
                return Err(ApiError::new(
                    ErrorCode::SourceExpired,
                    "Asset source slots cannot be rebound or forget a selected image; register a new frame identity",
                ));
            }
        }
        (
            prior.publication_revision.0.checked_add(1).ok_or_else(|| {
                ApiError::new(
                    ErrorCode::ResourceExhausted,
                    "Asset publication revision exhausted",
                )
            })?,
            Some(previous.publication_sha256.clone()),
        )
    } else {
        if document.revision.0 != 1 {
            return Err(invalid(
                "New asset project must start at working revision one",
            ));
        }
        (1, None)
    };
    Ok(AssetPublication {
        schema_version: "asset-project-publication/1".into(),
        publication_revision: DecimalU64(revision),
        parent_publication_sha256: parent,
        document: document.clone(),
    })
}

//! Immutable clip project publication policy; storage verifies chain and CAS under its writer lease.
#[path = "clip_projects/ports.rs"]
mod ports;
pub use ports::ClipProjectStore;
#[cfg(test)]
mod tests;
use crate::{
    application::api::{ApiError, ErrorCode},
    domain::{
        api_values::DecimalU64,
        clip_document::ClipDocument,
        identifiers::{valid_id, valid_sha256},
    },
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClipProjectInput {
    pub document: ClipDocument,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InspectClipProject {
    pub document_id: String,
    pub publication_revision: Option<DecimalU64>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClipPublication {
    pub schema_version: String,
    pub publication_revision: DecimalU64,
    pub parent_publication_sha256: Option<String>,
    pub document: ClipDocument,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredClipProject {
    pub publication_sha256: String,
    pub latest_publication_revision: DecimalU64,
    pub publication: ClipPublication,
}
pub fn create(
    store: &dyn ClipProjectStore,
    document: &ClipDocument,
) -> Result<StoredClipProject, ApiError> {
    prepare_publication(None, document)?;
    store.publish(None, document)
}
pub fn save(
    store: &dyn ClipProjectStore,
    expected: DecimalU64,
    document: &ClipDocument,
) -> Result<StoredClipProject, ApiError> {
    if expected.0 == 0 {
        return Err(invalid(
            "Expected clip publication revision must be positive",
        ));
    }
    document.validate().map_err(invalid)?;
    store.publish(Some(expected), document)
}
pub fn inspect(
    store: &dyn ClipProjectStore,
    id: &str,
    revision: Option<DecimalU64>,
) -> Result<StoredClipProject, ApiError> {
    if !valid_id(id) || revision.is_some_and(|revision| revision.0 == 0) {
        return Err(invalid(
            "Invalid clip project identity or publication revision",
        ));
    }
    store.load(id, revision)
}
/// Storage calls this only after verifying the original full chain and current head.
/// Checks here validate values; they do not confer filesystem origin/custody authority.
pub fn prepare_publication(
    previous: Option<&StoredClipProject>,
    document: &ClipDocument,
) -> Result<ClipPublication, ApiError> {
    document.validate().map_err(invalid)?;
    let (revision, parent) = match previous {
        None => {
            if document.revision.0 != 1 {
                return Err(invalid(
                    "A new clip project must start at edit revision one",
                ));
            }
            (1, None)
        }
        Some(previous) => {
            let publication = &previous.publication;
            let old = &publication.document;
            let valid_parent = match publication.publication_revision.0 {
                0 => false,
                1 => publication.parent_publication_sha256.is_none(),
                _ => publication
                    .parent_publication_sha256
                    .as_deref()
                    .is_some_and(valid_sha256),
            };
            if publication.schema_version != "1"
                || !valid_parent
                || old.validate().is_err()
                || previous.latest_publication_revision != publication.publication_revision
                || publication_sha256(publication)? != previous.publication_sha256
            {
                return Err(ApiError::new(
                    ErrorCode::NeedsInput,
                    "Invalid or non-head clip publication; preserve original journal",
                ));
            }
            if old.document_id != document.document_id || old.sources != document.sources {
                return Err(ApiError::new(
                    ErrorCode::RevisionConflict,
                    "Clip project identity and original source declarations are immutable",
                ));
            }
            if document.revision.0 <= old.revision.0 {
                return Err(ApiError::new(
                    ErrorCode::RevisionConflict,
                    "Clip working revision must advance",
                ));
            }
            let next = publication
                .publication_revision
                .0
                .checked_add(1)
                .ok_or_else(|| {
                    ApiError::new(
                        ErrorCode::ResourceExhausted,
                        "Clip publication revision exhausted",
                    )
                })?;
            (next, Some(previous.publication_sha256.clone()))
        }
    };
    Ok(ClipPublication {
        schema_version: "1".into(),
        publication_revision: DecimalU64(revision),
        parent_publication_sha256: parent,
        document: document.clone(),
    })
}
pub fn publication_sha256(value: &ClipPublication) -> Result<String, ApiError> {
    Ok(format!(
        "{:x}",
        Sha256::digest(
            serde_json::to_vec(value)
                .map_err(|_| invalid("Invalid clip publication serialization"))?
        )
    ))
}
fn invalid(message: &str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}

//! Immutable saved card projects. Publication revision and working edit revision are distinct.
#[path = "ports.rs"]
mod ports;
use crate::{
    application::api::{ApiError, ErrorCode},
    domain::{api_values::DecimalU64, card_document::CardDocument},
};
pub use ports::CardProjectStore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CardProjectInput {
    pub document: CardDocument,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InspectCardProject {
    pub document_id: String,
    pub publication_revision: Option<DecimalU64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CardPublication {
    pub schema_version: String,
    pub publication_revision: DecimalU64,
    pub parent_publication_sha256: Option<String>,
    pub document: CardDocument,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredCardProject {
    pub publication_sha256: String,
    pub latest_publication_revision: DecimalU64,
    pub publication: CardPublication,
}

pub fn create(
    store: &dyn CardProjectStore,
    document: &CardDocument,
) -> Result<StoredCardProject, ApiError> {
    prepare_publication(None, document)?;
    store.publish(None, document)
}

pub fn save(
    store: &dyn CardProjectStore,
    expected: DecimalU64,
    document: &CardDocument,
) -> Result<StoredCardProject, ApiError> {
    if expected.0 == 0 {
        return Err(invalid("Expected publication revision must be positive"));
    }
    document
        .validate()
        .map_err(|_| invalid("Invalid card document"))?;
    store.publish(Some(expected), document)
}

pub fn inspect(
    store: &dyn CardProjectStore,
    document_id: &str,
    revision: Option<DecimalU64>,
) -> Result<StoredCardProject, ApiError> {
    if !crate::domain::identifiers::valid_id(document_id) || revision.is_some_and(|r| r.0 == 0) {
        return Err(invalid(
            "Invalid card project identity or publication revision",
        ));
    }
    store.load(document_id, revision)
}

/// Called by storage only after reading the current, verified head while holding its lock.
pub fn prepare_publication(
    previous: Option<&StoredCardProject>,
    document: &CardDocument,
) -> Result<CardPublication, ApiError> {
    document
        .validate()
        .map_err(|_| invalid("Invalid card document"))?;
    let (revision, parent) = if let Some(previous) = previous {
        let old = &previous.publication.document;
        if old.document_id != document.document_id
            || serde_json::to_vec(&old.source).map_err(|_| invalid("Invalid source"))?
                != serde_json::to_vec(&document.source).map_err(|_| invalid("Invalid source"))?
        {
            return Err(invalid(
                "Saved card project source and identity are immutable",
            ));
        }
        if document.revision.0 <= old.revision.0 {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Working edit revision must advance",
            ));
        }
        let next = previous
            .publication
            .publication_revision
            .0
            .checked_add(1)
            .ok_or_else(|| {
                ApiError::new(
                    ErrorCode::ResourceExhausted,
                    "Card publication revision exhausted",
                )
            })?;
        (next, Some(previous.publication_sha256.clone()))
    } else {
        if document.revision.0 != 1 {
            return Err(invalid(
                "A new card project must start at edit revision one",
            ));
        }
        (1, None)
    };
    Ok(CardPublication {
        schema_version: "1".into(),
        publication_revision: DecimalU64(revision),
        parent_publication_sha256: parent,
        document: document.clone(),
    })
}

pub fn publication_sha256(publication: &CardPublication) -> Result<String, ApiError> {
    Ok(format!(
        "{:x}",
        Sha256::digest(
            serde_json::to_vec(publication).map_err(|_| invalid("Invalid card publication"))?
        )
    ))
}

fn invalid(message: &str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}

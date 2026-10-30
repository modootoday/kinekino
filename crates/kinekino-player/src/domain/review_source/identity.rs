use super::ValidationError;
use crate::domain::identifiers::valid_sha256;
use serde::{Deserialize, Serialize};

/// Scope-specific IDs from the sealed checkpoint; not the SDK's Unicode request IDs.
pub fn valid_review_object_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewCase {
    pub package_sha256: String,
    pub item_id: String,
    pub source_request_sha256: String,
}
impl ReviewCase {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if valid_sha256(&self.package_sha256)
            && valid_sha256(&self.source_request_sha256)
            && valid_review_object_id(&self.item_id)
        {
            Ok(())
        } else {
            Err(ValidationError::InvalidIdentity)
        }
    }
}
/// Identity in existing asset custody, never a caller-supplied path or URL.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MediaReference {
    pub asset_id: String,
    pub sha256: String,
}
impl MediaReference {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if valid_review_object_id(&self.asset_id) && valid_sha256(&self.sha256) {
            Ok(())
        } else {
            Err(ValidationError::InvalidIdentity)
        }
    }
}

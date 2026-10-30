use crate::domain::identifiers::{valid_id, valid_sha256};
use serde::{Deserialize, Serialize};
pub const HISTORICAL_DOCUMENT_VERSION: &str = "3";
pub const HISTORICAL_BINDING_VERSION: &str = "1";
pub const LEGACY_DOCUMENT_VERSIONS: [&str; 2] = ["1", "2"];
pub const DOCUMENT_VERSIONS: [&str; 3] = ["1", "2", HISTORICAL_DOCUMENT_VERSION];

pub fn proposal_document_version(
    source: &super::SourceSnapshot,
    historical: Option<&HistoricalBinding>,
) -> &'static str {
    if historical.is_some() {
        HISTORICAL_DOCUMENT_VERSION
    } else {
        source.schema_version()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompanionReference {
    pub sha256: String,
    pub bytes: u64,
    pub item_id: String,
}
impl CompanionReference {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !valid_sha256(&self.sha256)
            || !valid_id(&self.item_id)
            || !(1..=262_144).contains(&self.bytes)
        {
            return Err("Invalid historical companion reference");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoricalBinding {
    pub schema_version: String,
    pub reference: CompanionReference,
}
impl HistoricalBinding {
    pub fn new(reference: CompanionReference) -> Self {
        Self {
            schema_version: HISTORICAL_BINDING_VERSION.into(),
            reference,
        }
    }
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != HISTORICAL_BINDING_VERSION {
            return Err("Unsupported historical binding version");
        }
        self.reference.validate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{projects::canonical_sha256, proposals::*};
    #[test]
    fn legacy_bytes_and_capture_hash_survive_while_binding_changes_identity() {
        let mut stored = crate::domain::projects::tests::proposal();
        #[derive(Serialize)]
        struct Legacy<'a> {
            document: &'a ProposalDocument,
            acceptance: &'a Option<AcceptanceReceipt>,
        }
        let legacy = Legacy {
            document: &stored.document,
            acceptance: &stored.acceptance,
        };
        let bytes = serde_json::to_vec(&legacy).unwrap();
        let before_hash = canonical_sha256(&legacy).unwrap();
        assert_eq!(serde_json::to_vec(&stored).unwrap(), bytes);
        let reopened: StoredProposal = serde_json::from_slice(&bytes).unwrap();
        assert!(reopened.historical.is_none());
        assert_eq!(canonical_sha256(&reopened).unwrap(), before_hash);
        stored.historical = Some(HistoricalBinding {
            schema_version: "1".into(),
            reference: CompanionReference {
                sha256: "a".repeat(64),
                bytes: 20,
                item_id: "technical-fixture".into(),
            },
        });
        stored.document.schema_version = "3".into();
        stored.validate().unwrap();
        assert_ne!(canonical_sha256(&stored).unwrap(), before_hash);
        let mut value = serde_json::to_value(&stored).unwrap();
        value["historical"]
            .as_object_mut()
            .unwrap()
            .remove("reference");
        assert!(serde_json::from_value::<StoredProposal>(value).is_err());
        let mut removed = serde_json::to_value(&stored).unwrap();
        removed.as_object_mut().unwrap().remove("historical");
        assert!(
            serde_json::from_value::<StoredProposal>(removed)
                .unwrap()
                .validate()
                .is_err()
        );
        stored.historical.as_mut().unwrap().schema_version = "2".into();
        assert!(stored.validate().is_err());
    }
}

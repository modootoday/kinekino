use super::{MediaReference, ReviewCase, ValidationError, next_revision, valid_review_object_id};
use crate::domain::api_values::DecimalU64;
use serde::{Deserialize, Serialize};

/// Archived evidence metadata is never a live playback capability.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimelineStatus {
    Unverified,
    Verified(Box<super::timeline_mapping::TimelineMappingRecord>),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceReference {
    pub asset_id: String,
    pub sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssociationRevision {
    pub association_id: String,
    pub case: ReviewCase,
    pub media: MediaReference,
    pub revision: DecimalU64,
    pub timeline_status: TimelineStatus,
    pub evidence_refs: Vec<EvidenceReference>,
    pub note: String,
}
impl AssociationRevision {
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.case.validate()?;
        self.media.validate()?;
        if self.note.len() > 8192 {
            return Err(ValidationError::NoteTooLarge);
        }
        if !valid_review_object_id(&self.association_id) {
            return Err(ValidationError::InvalidIdentity);
        }
        if self.revision.0 == 0 {
            return Err(ValidationError::InvalidRevision);
        }
        match &self.timeline_status {
            TimelineStatus::Unverified if !self.evidence_refs.is_empty() => {
                return Err(ValidationError::UnsupportedTimelineEvidence);
            }
            TimelineStatus::Verified(record) => {
                record.validate()?;
                if record.case != self.case
                    || record.media != self.media
                    || self.evidence_refs.as_slice() != [record.evidence.clone()]
                {
                    return Err(ValidationError::BindingChanged);
                }
            }
            _ => {}
        }
        Ok(())
    }
    pub fn validate_successor(&self, previous: &Self) -> Result<(), ValidationError> {
        previous.validate()?;
        self.validate()?;
        if self.association_id != previous.association_id || self.case != previous.case {
            return Err(ValidationError::BindingChanged);
        }
        if self.revision != next_revision(Some(previous.revision))? {
            return Err(ValidationError::RevisionConflict);
        }
        Ok(())
    }
    pub fn playback_ready(&self) -> bool {
        false
    }
}

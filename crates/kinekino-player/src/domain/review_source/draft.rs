use super::{AssociationRevision, ValidationError, next_revision, valid_review_object_id};
use crate::domain::api_values::DecimalU64;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DraftAuthor {
    User,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserRange {
    pub start_us: DecimalU64,
    pub end_us: DecimalU64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RangeDraft {
    pub draft_id: String,
    pub association_id: String,
    pub association_revision: DecimalU64,
    pub draft_revision: DecimalU64,
    pub authored_by: DraftAuthor,
    pub ranges: Vec<UserRange>,
    pub note: String,
}
impl RangeDraft {
    pub fn validate_shape(&self) -> Result<(), ValidationError> {
        if !valid_review_object_id(&self.draft_id) || !valid_review_object_id(&self.association_id)
        {
            return Err(ValidationError::InvalidIdentity);
        }
        if self.draft_revision.0 == 0 || self.association_revision.0 == 0 {
            return Err(ValidationError::InvalidRevision);
        }
        if self.ranges.len() > 64 {
            return Err(ValidationError::TooManyRanges);
        }
        if self.note.len() > 8192 {
            return Err(ValidationError::NoteTooLarge);
        }
        if self.ranges.iter().any(|r| r.start_us.0 >= r.end_us.0) {
            return Err(ValidationError::InvalidRange);
        }
        Ok(())
    }
    /// Duration, when provided, must come from a retained verified media lease;
    /// it is not accepted as request data and does not confer timeline readiness.
    pub fn validate(
        &self,
        association: &AssociationRevision,
        verified_duration_us: Option<u64>,
    ) -> Result<(), ValidationError> {
        association.validate()?;
        self.validate_shape()?;
        if self.association_id != association.association_id
            || self.association_revision != association.revision
        {
            return Err(ValidationError::BindingChanged);
        }
        for range in &self.ranges {
            if verified_duration_us.is_some_and(|duration| range.end_us.0 > duration) {
                return Err(ValidationError::RangeExceedsVerifiedDuration);
            }
        }
        Ok(())
    }
    pub fn validate_successor(
        &self,
        previous: &Self,
        association: &AssociationRevision,
        verified_duration_us: Option<u64>,
    ) -> Result<(), ValidationError> {
        previous.validate(association, None)?;
        self.validate(association, verified_duration_us)?;
        if self.draft_id != previous.draft_id
            || self.association_id != previous.association_id
            || self.association_revision != previous.association_revision
        {
            return Err(ValidationError::BindingChanged);
        }
        if self.draft_revision != next_revision(Some(previous.draft_revision))? {
            return Err(ValidationError::RevisionConflict);
        }
        Ok(())
    }
}

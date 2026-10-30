//! Archivable mapping metadata. Deserializing this record never grants live authority.
use super::{
    EvidenceReference, MediaReference, ReviewCase, ValidationError, valid_review_object_id,
};
use crate::domain::{api_values::DecimalU64, identifiers::valid_sha256, media_timeline::TimeBase};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimelineRatioRecord {
    pub numerator: String,
    pub denominator: String,
}
impl TimelineRatioRecord {
    pub fn from_time_base(base: TimeBase) -> Self {
        let (n, d) = base.ratio();
        Self {
            numerator: n.to_string(),
            denominator: d.to_string(),
        }
    }
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.numerator.len() > 39 || self.denominator.len() > 39 {
            return Err(ValidationError::UnsupportedTimelineEvidence);
        }
        let base = TimeBase::parse(&format!("{}/{}", self.numerator, self.denominator))
            .map_err(|_| ValidationError::UnsupportedTimelineEvidence)?;
        if Self::from_time_base(base) != *self {
            return Err(ValidationError::UnsupportedTimelineEvidence);
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimelineMappingRecord {
    pub case: ReviewCase,
    pub media: MediaReference,
    pub evidence: EvidenceReference,
    pub mapping_revision: DecimalU64,
    pub source_time_base: TimelineRatioRecord,
    pub origin_pts: String,
    pub origin_time_base: TimelineRatioRecord,
    pub origin_observation_sha256: String,
    pub source_start_us: DecimalU64,
    pub source_end_us: DecimalU64,
    pub start_frame_pts: String,
    pub end_frame_pts: String,
}
fn tick(value: &str) -> Result<i64, ValidationError> {
    if value.len() > 20 {
        return Err(ValidationError::UnsupportedTimelineEvidence);
    }
    let number: i64 = value
        .parse()
        .map_err(|_| ValidationError::UnsupportedTimelineEvidence)?;
    if number.to_string() != value {
        return Err(ValidationError::UnsupportedTimelineEvidence);
    }
    Ok(number)
}
impl TimelineMappingRecord {
    /// Structural archival validation only; fresh retained evidence is still mandatory.
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.case.validate()?;
        self.media.validate()?;
        self.source_time_base.validate()?;
        self.origin_time_base.validate()?;
        if self.source_time_base != self.origin_time_base {
            return Err(ValidationError::UnsupportedTimelineEvidence);
        }
        tick(&self.origin_pts)?;
        let start = tick(&self.start_frame_pts)?;
        let end = tick(&self.end_frame_pts)?;
        if !valid_review_object_id(&self.evidence.asset_id)
            || !valid_sha256(&self.evidence.sha256)
            || !valid_sha256(&self.origin_observation_sha256)
            || self.mapping_revision.0 == 0
            || self.source_start_us.0 >= self.source_end_us.0
            || start > end
        {
            return Err(ValidationError::UnsupportedTimelineEvidence);
        }
        Ok(())
    }
}

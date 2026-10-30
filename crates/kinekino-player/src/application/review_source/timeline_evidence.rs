//! Server-owned custody admission. Observations alone never create Verified wire state.
use super::{CaseLease, CaseResolver, MediaLease, MediaResolver};
mod mapping;
use crate::{
    application::api::{ApiError, ErrorCode},
    domain::review_source::{
        EvidenceReference, MediaReference, ReviewCase,
        timeline_evidence::{MappingWindow, TimelineObservations},
    },
};
pub use mapping::{RetainedMappingRange, TimelineRangePlan};
/// Implementations must verify actual source/tool/observation hashes under retained custody.
/// A deserialized caller summary is not an implementation of this authority contract.
pub trait TimelineEvidenceLease {
    fn observations(&self) -> &TimelineObservations;
    fn check_current(&self) -> Result<(), ApiError>;
}
pub trait TimelineEvidenceResolver: Send + Sync {
    fn retain(
        &self,
        case: &ReviewCase,
        media: &MediaReference,
        reference: &EvidenceReference,
    ) -> Result<Box<dyn TimelineEvidenceLease + '_>, ApiError>;
}
/// Share the original trusted authority across storage and playback; cloning the
/// Arc does not reconstruct authority from persisted records or request data.
impl<T: TimelineEvidenceResolver + ?Sized> TimelineEvidenceResolver for std::sync::Arc<T> {
    fn retain(
        &self,
        case: &ReviewCase,
        media: &MediaReference,
        reference: &EvidenceReference,
    ) -> Result<Box<dyn TimelineEvidenceLease + '_>, ApiError> {
        (**self).retain(case, media, reference)
    }
}
/// No public fields or Deserialize implementation; keep the original leases until use ends.
/// ```compile_fail
/// use kinekino_player::application::review_source::timeline_evidence::RetainedTimelineEvidence;
/// let forged: RetainedTimelineEvidence<'_> = serde_json::from_str("{}").unwrap();
/// ```
pub struct RetainedTimelineEvidence<'a> {
    case: Box<dyn CaseLease + 'a>,
    media: Box<dyn MediaLease + 'a>,
    evidence: Box<dyn TimelineEvidenceLease + 'a>,
    snapshot: TimelineObservations,
    mapping: MappingWindow,
}
impl RetainedTimelineEvidence<'_> {
    pub fn mapping(&self) -> MappingWindow {
        self.mapping
    }
    pub fn playback_ready(&self) -> bool {
        false
    }
    pub fn check_current(&self) -> Result<(), ApiError> {
        self.case.check_current()?;
        self.media.check_current()?;
        self.evidence.check_current()?;
        if self.case.case() != &self.snapshot.case
            || self.media.identity() != &self.snapshot.media
            || self.evidence.observations() != &self.snapshot
        {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Retained timeline evidence binding changed",
            ));
        }
        Ok(())
    }
}
pub fn retain<'a>(
    cases: &'a dyn CaseResolver,
    media: &'a dyn MediaResolver,
    resolver: &'a dyn TimelineEvidenceResolver,
    case: &ReviewCase,
    reference_media: &MediaReference,
    reference: &EvidenceReference,
) -> Result<RetainedTimelineEvidence<'a>, ApiError> {
    case.validate().map_err(|_| invalid())?;
    reference_media.validate().map_err(|_| invalid())?;
    if !crate::domain::review_source::valid_review_object_id(&reference.asset_id)
        || !crate::domain::identifiers::valid_sha256(&reference.sha256)
    {
        return Err(invalid());
    }
    let case_lease = cases.retain(case)?;
    if case_lease.case() != case {
        return Err(binding());
    }
    case_lease.check_current()?;
    let media_lease = media.retain(case, reference_media)?;
    if media_lease.identity() != reference_media {
        return Err(binding());
    }
    media_lease.check_current()?;
    let evidence = resolver.retain(case, reference_media, reference)?;
    evidence.check_current()?;
    let facts = evidence.observations();
    if &facts.case != case || &facts.media != reference_media || &facts.reference != reference {
        return Err(binding());
    }
    let mapping = facts.validate().map_err(|_| invalid())?;
    let snapshot = facts.clone();
    let retained = RetainedTimelineEvidence {
        case: case_lease,
        media: media_lease,
        evidence,
        snapshot,
        mapping,
    };
    retained.check_current()?;
    Ok(retained)
}
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::InvalidArgument,
        "Invalid or inconsistent timeline evidence observations",
    )
}
fn binding() -> ApiError {
    ApiError::new(
        ErrorCode::RevisionConflict,
        "Foreign timeline evidence binding",
    )
}
#[cfg(test)]
pub(super) mod tests;

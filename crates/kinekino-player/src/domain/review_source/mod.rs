//! User source bindings and range drafts; no storage, model verdict or playback policy.
mod association;
mod draft;
mod identity;
mod playback_window;
pub use playback_window::PlaybackWindow;
pub mod timeline_evidence;
pub mod timeline_mapping;
pub mod video_cadence;
use crate::domain::api_values::DecimalU64;
pub use association::{AssociationRevision, EvidenceReference, TimelineStatus};
pub use draft::{DraftAuthor, RangeDraft, UserRange};
pub use identity::{MediaReference, ReviewCase, valid_review_object_id};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValidationError {
    InvalidIdentity,
    InvalidRevision,
    RevisionConflict,
    BindingChanged,
    InvalidRange,
    RangeExceedsVerifiedDuration,
    TooManyRanges,
    NoteTooLarge,
    UnsupportedTimelineEvidence,
}

pub fn next_revision(expected: Option<DecimalU64>) -> Result<DecimalU64, ValidationError> {
    match expected {
        None => Ok(DecimalU64(1)),
        Some(DecimalU64(0)) => Err(ValidationError::InvalidRevision),
        Some(revision) => revision
            .0
            .checked_add(1)
            .map(DecimalU64)
            .ok_or(ValidationError::InvalidRevision),
    }
}
#[cfg(test)]
mod tests;

//! Exact accepted-output decisions, independent of candidate model sample reviews.
//! Criteria values are supplied explicitly; no task criteria are approved here.
mod criteria;
mod decision;
mod hash;
mod target;

pub use criteria::{CriteriaDocument, CriteriaIdentity, OutputCriterion};
pub use decision::{
    DeclaredReviewer, ObservedRecordTime, OutputDecision, OutputDecisionInput, OutputDecisionScope,
    OutputVerdict, RecordClock,
};
pub use hash::{OUTPUT_HASH_FORMAT, accepted_output_sha256};
pub use target::{OutputReviewTarget, OutputTargetKind};

pub(super) fn text(value: &str, limit: usize) -> bool {
    !value.trim().is_empty()
        && value.len() <= limit
        && !value
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\t')
}

#[cfg(test)]
mod tests;

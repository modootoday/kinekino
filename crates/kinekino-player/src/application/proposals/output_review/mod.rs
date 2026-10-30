//! Common accepted-output review use cases and optional ownership contracts.
mod catalog;
mod clock;
mod ports;
pub mod schema;
mod types;
mod usecases;
use super::ProposalStore;
use crate::{
    application::api::{ApiError, ErrorCode},
    domain::identifiers::valid_id,
};
pub use catalog::{criteria_for, require_current_criteria};
pub use clock::OutputReviewClock;
pub use ports::{
    CurrentEvidenceStatus, OutputDecisionHistory, OutputReviewSession, PreparedOutputDecision,
    StoredOutputDecision,
};
pub use types::{
    EvidenceObservation, InspectOutputReviewTarget, ListOutputDecisions, OutputReviewInspection,
    RecordOutputDecision,
};
pub use usecases::{execute, inspect_target, is_command, output_decisions, record_decision};

pub(super) fn unavailable() -> ApiError {
    ApiError::new(
        ErrorCode::Unsupported,
        "Accepted output review session or reader unavailable",
    )
}

pub fn open_session<'a>(
    store: &'a dyn ProposalStore,
    proposal_id: &str,
) -> Result<Box<dyn OutputReviewSession + 'a>, ApiError> {
    if !valid_id(proposal_id) {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Invalid proposal identity",
        ));
    }
    if !store.available() {
        return Err(unavailable());
    }
    store
        .open_output_review(proposal_id)?
        .ok_or_else(unavailable)
}

#[cfg(test)]
mod tests;

pub(super) mod clock;
mod paths;
mod publication;
mod records;
mod session;
#[cfg(test)]
mod tests;
use super::{ApiError, ErrorCode, ProposalFiles, reservation::WriterReservation};
use crate::application::proposals::output_review::{
    CurrentEvidenceStatus, OutputDecisionHistory, OutputReviewSession,
};

const MAX_HISTORY: usize = 256;
const MAX_RECORD: u64 = 1_048_576;
fn incomplete() -> ApiError {
    let mut error = ApiError::new(
        ErrorCode::NeedsInput,
        "Output decision publication is incomplete or corrupt; preserve it for manual recovery",
    );
    error.remediation = "recover_output_decision_publication".into();
    error
}
pub(super) fn open<'a>(
    store: &'a ProposalFiles,
    id: &str,
) -> Result<Box<dyn OutputReviewSession + 'a>, ApiError> {
    session::open(store, id)
}
pub(super) fn history(store: &ProposalFiles, id: &str) -> Result<OutputDecisionHistory, ApiError> {
    store.identity(id)?;
    let writer = WriterReservation::acquire(store)?;
    Ok(OutputDecisionHistory {
        records: records::history(store, &writer, id)?,
        current_evidence: CurrentEvidenceStatus::NotChecked,
    })
}

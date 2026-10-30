use super::{
    ApiError, ErrorCode, MAX_HISTORY, ProposalFiles, WriterReservation, publication, records,
};
use crate::{
    application::{
        ai_provenance,
        proposals::{
            ProposalStore,
            output_review::{
                OutputReviewSession, PreparedOutputDecision, StoredOutputDecision,
                require_current_criteria,
            },
        },
    },
    domain::proposals::{StoredProposal, output_review::OutputDecision},
};

struct Session<'a> {
    store: &'a ProposalFiles,
    writer: WriterReservation<'a>,
    snapshot: StoredProposal,
}
pub(super) fn open<'a>(
    store: &'a ProposalFiles,
    id: &str,
) -> Result<Box<dyn OutputReviewSession + 'a>, ApiError> {
    store.identity(id)?;
    let writer = WriterReservation::acquire(store)?;
    let snapshot = store.read_current_locked(&writer, id)?;
    ai_provenance::verify_bound(store.historical_reader(), &snapshot)?;
    Ok(Box::new(Session {
        store,
        writer,
        snapshot,
    }))
}
impl OutputReviewSession for Session<'_> {
    fn current(&self) -> &StoredProposal {
        &self.snapshot
    }
    fn verify_current_evidence(&self) -> Result<(), ApiError> {
        ai_provenance::verify_bound(self.store.historical_reader(), &self.snapshot)
    }
    fn append(
        &mut self,
        prepared: &PreparedOutputDecision,
    ) -> Result<StoredOutputDecision, ApiError> {
        let current = self
            .store
            .read_current_locked(&self.writer, &self.snapshot.document.proposal_id)?;
        prepared
            .input()
            .validate(&current, prepared.criteria())
            .map_err(|message| ApiError::new(ErrorCode::RevisionConflict, message))?;
        let criteria = require_current_criteria(&prepared.input().criteria)?;
        if criteria != *prepared.criteria() {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Prepared criteria differ from trusted catalog",
            ));
        }
        ai_provenance::verify_bound(self.store.historical_reader(), &current)?;
        let history = records::history(self.store, &self.writer, &current.document.proposal_id)?;
        if let Some(existing) = history
            .iter()
            .find(|record| record.decision.input.decision_id == prepared.input().decision_id)
        {
            if existing.decision.matches_request(prepared.input()) {
                return Ok(existing.clone());
            }
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Output decision identity already names another request",
            ));
        }
        if history.len() >= MAX_HISTORY {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Output decision history is full",
            ));
        }
        let time = self.store.record_clock.observe_record_time()?;
        let decision = OutputDecision::prepare(prepared.input().clone(), &current, criteria, time)
            .map_err(|message| ApiError::new(ErrorCode::InvalidArgument, message))?;
        // External evidence can change independently of this proposal root reservation.
        ai_provenance::verify_bound(self.store.historical_reader(), &current)?;
        publication::publish(self.store, &self.writer, &decision)
    }
}

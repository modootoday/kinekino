use super::{
    ProposalFiles, ProposalStore, StoredProposal, immutable_equal, origin,
    reservation::WriterReservation,
};
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        proposals::guarded::CaptureMutationSession,
    },
    domain::proposals::ProposalStoreOrigin,
};

struct Session<'a> {
    store: &'a ProposalFiles,
    writer: WriterReservation<'a>,
    origin: ProposalStoreOrigin,
    state: StoredProposal,
}

pub(super) fn open<'a>(
    store: &'a ProposalFiles,
    id: &str,
) -> Result<Box<dyn CaptureMutationSession + 'a>, ApiError> {
    store.identity(id)?;
    let writer = WriterReservation::acquire(store)?;
    let origin = origin::existing_origin(store, &writer)?;
    let state = store.read_current_locked(&writer, id)?;
    crate::application::ai_provenance::verify_bound(store.historical_reader(), &state)?;
    Ok(Box::new(Session {
        store,
        writer,
        origin,
        state,
    }))
}

impl CaptureMutationSession for Session<'_> {
    fn origin(&self) -> &ProposalStoreOrigin {
        &self.origin
    }
    fn current(&self) -> &StoredProposal {
        &self.state
    }
    fn revision(&self, revision: u64) -> Result<StoredProposal, ApiError> {
        self.writer.check(self.store)?;
        if revision == 0 || revision > self.state.document.draft_revision.0 {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Expected immutable revision is unavailable",
            ));
        }
        let dir = self.store.identity(&self.state.document.proposal_id)?;
        let state = self
            .store
            .read(&self.state.document.proposal_id, &dir, revision)?;
        if !immutable_equal(&state, &self.state) {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Expected immutable source changed",
            ));
        }
        crate::application::ai_provenance::verify_bound(self.store.historical_reader(), &state)?;
        Ok(state)
    }
    fn publish(&mut self, expected: u64, state: &StoredProposal) -> Result<(), ApiError> {
        if state.document.proposal_id != self.state.document.proposal_id
            || self.state.document.draft_revision.0 != expected
            || origin::existing_origin(self.store, &self.writer)? != self.origin
        {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Owned capture mutation state changed",
            ));
        }
        self.store
            .publish_locked(&self.writer, Some(expected), state)?;
        self.state = self
            .store
            .read_current_locked(&self.writer, &state.document.proposal_id)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;

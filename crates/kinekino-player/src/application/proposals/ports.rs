//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

pub trait ProposalStore: Send + Sync {
    fn source_context_available(&self) -> bool {
        false
    }
    fn inspect_source_context(
        &self,
        _proposal_id: &str,
    ) -> Result<source_context::SourceContextInspection, ApiError> {
        Err(source_context::unavailable())
    }
    fn list_available(&self) -> bool {
        false
    }
    fn list(&self, _input: &list::ListProposals) -> Result<list::ProposalListPage, ApiError> {
        Err(list::unavailable())
    }
    fn capture_mutation_available(&self) -> bool {
        false
    }
    /// Optional short-lived writer session. Never emulate with pre-inspect/load.
    fn open_capture_mutation(
        &self,
        _proposal_id: &str,
    ) -> Result<Option<Box<dyn guarded::CaptureMutationSession + '_>>, ApiError> {
        Ok(None)
    }
    fn historical_reader(
        &self,
    ) -> Option<&dyn crate::application::ai_provenance::HistoricalEvidenceReader> {
        None
    }
    fn available(&self) -> bool {
        true
    }
    fn load(&self, proposal_id: &str) -> Result<StoredProposal, ApiError>;
    /// Legacy providers remain readable without inventing trusted provenance.
    fn capture_current(
        &self,
        proposal_id: &str,
        expected_revision: u64,
    ) -> Result<CapturedProposal, ApiError> {
        let state = self.load(proposal_id)?;
        if state.document.proposal_id != proposal_id
            || state.document.draft_revision.0 != expected_revision
        {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Live proposal revision changed",
            ));
        }
        Ok(CapturedProposal {
            state,
            origin: None,
        })
    }
    /// Compare latest revision and publish one immutable state atomically; None means new identity.
    fn publish(&self, expected: Option<u64>, state: &StoredProposal) -> Result<(), ApiError>;
    fn output_review_available(&self) -> bool {
        false
    }
    /// Optional ownership session; existing adapters remain unsupported by default.
    fn open_output_review(
        &self,
        _proposal_id: &str,
    ) -> Result<Option<Box<dyn output_review::OutputReviewSession + '_>>, ApiError> {
        Ok(None)
    }
    /// Reads stored records independently of current proposal/evidence availability.
    fn output_decisions(
        &self,
        _proposal_id: &str,
    ) -> Result<output_review::OutputDecisionHistory, ApiError> {
        Err(output_review::unavailable())
    }
}

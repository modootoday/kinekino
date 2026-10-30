//! Deterministic proposal policy shared by ordinary and guarded use cases.
use super::{AcceptanceReceipt, DraftState, ProposalPayload, StoredProposal};
use crate::domain::api_values::DecimalU64;

#[derive(Debug, PartialEq, Eq)]
pub enum ProposalMutationError {
    InvalidArgument(&'static str),
    RevisionConflict(&'static str),
    NeedsRepair,
}

#[derive(Debug)]
pub struct AcceptedProposal {
    pub state: StoredProposal,
    pub receipt: AcceptanceReceipt,
    pub publication_required: bool,
}

fn require_revision(state: &StoredProposal, expected: u64) -> Result<(), ProposalMutationError> {
    if state.document.draft_revision.0 != expected {
        return Err(ProposalMutationError::RevisionConflict(
            "Proposal revision changed; inspect before editing or accepting",
        ));
    }
    Ok(())
}

fn next(expected: u64) -> Result<DecimalU64, ProposalMutationError> {
    expected
        .checked_add(1)
        .map(DecimalU64)
        .ok_or(ProposalMutationError::InvalidArgument("Revision exhausted"))
}

pub fn edit_state(
    mut state: StoredProposal,
    draft: ProposalPayload,
    expected: u64,
) -> Result<StoredProposal, ProposalMutationError> {
    require_revision(&state, expected)?;
    draft
        .validate(&state.document.source, state.document.generated.task)
        .map_err(ProposalMutationError::InvalidArgument)?;
    state.document.draft = Some(draft);
    state.document.state = DraftState::Editable;
    state.document.draft_revision = next(expected)?;
    state.acceptance = None;
    Ok(state)
}

/// Callers establish ownership/origin before this policy, including retries.
pub fn accept_state(
    mut state: StoredProposal,
    target_source_revision: DecimalU64,
    expected: u64,
) -> Result<AcceptedProposal, ProposalMutationError> {
    if let Some(receipt) = state
        .acceptance
        .as_ref()
        .filter(|receipt| {
            receipt.draft_revision.0 == expected
                && receipt.source_revision == target_source_revision
        })
        .cloned()
    {
        return Ok(AcceptedProposal {
            state,
            receipt,
            publication_required: false,
        });
    }
    require_revision(&state, expected)?;
    let document = &state.document;
    if document.source.source_revision() != target_source_revision {
        return Err(ProposalMutationError::RevisionConflict(
            "Target source revision differs from immutable snapshot",
        ));
    }
    let output = document
        .draft
        .clone()
        .ok_or(ProposalMutationError::NeedsRepair)?;
    output
        .validate(&document.source, document.generated.task)
        .map_err(ProposalMutationError::InvalidArgument)?;
    let receipt = AcceptanceReceipt {
        proposal_id: document.proposal_id.clone(),
        draft_revision: document.draft_revision,
        source_id: document.source.source_id().into(),
        source_revision: document.source.source_revision(),
        source_sha256: document.source.source_sha256().into(),
        output_revision: next(expected)?,
        output,
        automatic_application_enabled: false,
    };
    state.document.draft_revision = receipt.output_revision;
    state.acceptance = Some(receipt.clone());
    Ok(AcceptedProposal {
        state,
        receipt,
        publication_required: true,
    })
}

#[cfg(test)]
#[path = "transitions_tests.rs"]
mod tests;

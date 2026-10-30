use super::*;
use crate::{
    application::proposals::{self, ProposalStore},
    domain::{
        api_values::DecimalU64,
        proposals::{GeneratedResult, HistoricalBinding},
    },
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportHistoricalProposal {
    pub proposal_id: String,
    pub source_id: String,
    pub source_revision: DecimalU64,
    pub reference: CompanionReference,
}

pub fn import(
    store: &dyn ProposalStore,
    input: ImportHistoricalProposal,
) -> Result<StoredProposal, ApiError> {
    if !valid_id(&input.proposal_id) || !valid_id(&input.source_id) || input.source_revision.0 == 0
    {
        return Err(invalid(
            "Invalid historical import identity or source revision",
        ));
    }
    validate_reference(&input.reference)?;
    let reader = store.historical_reader().ok_or_else(|| {
        ApiError::new(
            ErrorCode::Unsupported,
            "An explicit pinned historical verifier is required",
        )
    })?;
    let sample = reader.verify(&input.reference)?;
    let source = proposals::source_import::from_request_asset(
        input.source_id,
        input.source_revision,
        sample.source_sha256,
        sample.source_request_utf8,
        sample.task,
    )?;
    let prepared = prepare(
        reader,
        input.reference,
        CreateProposal {
            proposal_id: input.proposal_id,
            source,
            generated: GeneratedResult {
                task: sample.task,
                raw_text: sample.candidate_raw_utf8,
            },
        },
    )?;
    let (proposal, reference) = prepared.into_parts();
    let id = proposal.proposal_id.clone();
    proposals::create_bound(store, proposal, Some(HistoricalBinding::new(reference)))?;
    proposals::inspect(store, proposals::InspectProposal { proposal_id: id })
}

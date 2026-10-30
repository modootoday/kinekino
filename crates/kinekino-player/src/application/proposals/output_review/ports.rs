use crate::{
    application::api::ApiError,
    domain::proposals::{
        StoredProposal,
        output_review::{CriteriaDocument, OutputDecision, OutputDecisionInput},
    },
};
use serde::{Deserialize, Serialize};

/// Domain-validated request, before any adapter clock observation or publication.
pub struct PreparedOutputDecision {
    input: OutputDecisionInput,
    criteria: CriteriaDocument,
}
impl PreparedOutputDecision {
    /// Structural validation only; not external evidence verification or current-state CAS.
    pub fn new(
        input: OutputDecisionInput,
        state: &StoredProposal,
        criteria: CriteriaDocument,
    ) -> Result<Self, &'static str> {
        input.validate(state, &criteria)?;
        Ok(Self { input, criteria })
    }
    pub fn input(&self) -> &OutputDecisionInput {
        &self.input
    }
    pub fn criteria(&self) -> &CriteriaDocument {
        &self.criteria
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredOutputDecision {
    pub decision_sha256: String,
    pub decision: OutputDecision,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CurrentEvidenceStatus {
    NotChecked,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputDecisionHistory {
    pub records: Vec<StoredOutputDecision>,
    pub current_evidence: CurrentEvidenceStatus,
}

/// An adapter must retain the proposal writer reservation through append and readback.
/// Implementations must not reacquire that lock using public load/publish. This trait
/// declaration provides no filesystem guarantee and owns no OS/window authority.
pub trait OutputReviewSession {
    fn current(&self) -> &StoredProposal;
    fn verify_current_evidence(&self) -> Result<(), ApiError>;
    /// Recheck target/criteria/evidence against the owned snapshot; only the adapter
    /// supplies the first observed time. Identical retry returns the original record.
    fn append(
        &mut self,
        decision: &PreparedOutputDecision,
    ) -> Result<StoredOutputDecision, ApiError>;
}

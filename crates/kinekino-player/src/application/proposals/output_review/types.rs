use crate::domain::proposals::{
    StoredProposal,
    output_review::{CriteriaDocument, CriteriaIdentity, OutputDecisionInput, OutputReviewTarget},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InspectOutputReviewTarget {
    pub proposal_id: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordOutputDecision {
    pub proposal_id: String,
    pub decision: OutputDecisionInput,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListOutputDecisions {
    pub proposal_id: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceObservation {
    NotHistoricallyBound,
    HistoricalBindingReverifiedInSession,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputReviewInspection {
    pub schema_version: String,
    pub state: StoredProposal,
    pub target: OutputReviewTarget,
    pub criteria_document: CriteriaDocument,
    pub criteria_identity: CriteriaIdentity,
    pub evidence_observation: EvidenceObservation,
}

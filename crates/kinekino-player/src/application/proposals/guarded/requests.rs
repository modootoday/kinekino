use super::{CaptureAnchor, CaptureExpectation};
use crate::domain::{api_values::DecimalU64, proposals::ProposalPayload};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditCapturedProposal {
    pub anchor: CaptureAnchor,
    pub expected: Option<CaptureExpectation>,
    pub draft: ProposalPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcceptCapturedProposal {
    pub anchor: CaptureAnchor,
    pub expected: Option<CaptureExpectation>,
    pub target_source_revision: DecimalU64,
}

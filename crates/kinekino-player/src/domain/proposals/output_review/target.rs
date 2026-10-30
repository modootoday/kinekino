use super::super::{HistoricalBinding, ProposalTask, StoredProposal};
use super::{OUTPUT_HASH_FORMAT, accepted_output_sha256, hash::raw_sha256};
use crate::domain::{
    api_values::DecimalU64,
    identifiers::{valid_id, valid_sha256},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputTargetKind {
    AcceptedOutput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputReviewTarget {
    pub schema_version: String,
    pub kind: OutputTargetKind,
    pub proposal_id: String,
    pub task: ProposalTask,
    pub draft_revision: DecimalU64,
    pub output_revision: DecimalU64,
    pub hash_format: String,
    pub output_sha256: String,
    pub source_id: String,
    pub source_revision: DecimalU64,
    pub source_sha256: String,
    pub generated_raw_sha256: String,
    pub generated_raw_bytes: DecimalU64,
    pub historical: Option<HistoricalBinding>,
}
impl OutputReviewTarget {
    /// Checks stored identity syntax only; it cannot recompute absent output/source bytes.
    pub fn validate_stored(&self) -> Result<(), &'static str> {
        if self.schema_version != "1"
            || self.hash_format != OUTPUT_HASH_FORMAT
            || !valid_id(&self.proposal_id)
            || !valid_id(&self.source_id)
            || self.draft_revision.0 == 0
            || self.source_revision.0 == 0
            || self.draft_revision.0.checked_add(1) != Some(self.output_revision.0)
            || !valid_sha256(&self.output_sha256)
            || !valid_sha256(&self.source_sha256)
            || !valid_sha256(&self.generated_raw_sha256)
            || self.generated_raw_bytes.0 > 262_144
        {
            return Err("Invalid stored accepted output target");
        }
        if let Some(historical) = &self.historical {
            historical.validate()?;
        }
        Ok(())
    }
    /// Structural binding only. External evidence and current-state CAS belong to application.
    pub fn from_stored(state: &StoredProposal) -> Result<Self, &'static str> {
        state.validate()?;
        let receipt = state
            .acceptance
            .as_ref()
            .ok_or("Current proposal has no accepted output")?;
        let target = Self {
            schema_version: "1".into(),
            kind: OutputTargetKind::AcceptedOutput,
            proposal_id: receipt.proposal_id.clone(),
            task: state.document.generated.task,
            draft_revision: receipt.draft_revision,
            output_revision: receipt.output_revision,
            hash_format: OUTPUT_HASH_FORMAT.into(),
            output_sha256: accepted_output_sha256(&receipt.output)?,
            source_id: receipt.source_id.clone(),
            source_revision: receipt.source_revision,
            source_sha256: receipt.source_sha256.clone(),
            generated_raw_sha256: raw_sha256(state.document.generated.raw_text.as_bytes()),
            generated_raw_bytes: DecimalU64(
                u64::try_from(state.document.generated.raw_text.len())
                    .map_err(|_| "Generated text size exceeds u64")?,
            ),
            historical: state.historical.clone(),
        };
        target.validate_stored()?;
        Ok(target)
    }
    pub fn validate_against(&self, state: &StoredProposal) -> Result<(), &'static str> {
        self.validate_stored()?;
        if *self != Self::from_stored(state)? {
            return Err("Accepted output target changed");
        }
        Ok(())
    }
}

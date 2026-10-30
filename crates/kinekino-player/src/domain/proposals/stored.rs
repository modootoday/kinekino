use super::{AcceptanceReceipt, DraftState, ProposalDocument};
use crate::domain::model_review::valid_id;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredProposal {
    pub document: ProposalDocument,
    pub acceptance: Option<AcceptanceReceipt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub historical: Option<super::HistoricalBinding>,
}
impl StoredProposal {
    pub fn validate(&self) -> Result<(), &'static str> {
        if let Some(binding) = &self.historical {
            binding.validate()?;
        }
        let d = &self.document;
        let schema_matches = d.schema_version
            == super::proposal_document_version(&d.source, self.historical.as_ref());
        if !schema_matches
            || !valid_id(&d.proposal_id)
            || d.draft_revision.0 == 0
            || d.generated.raw_text.len() > 262_144
            || (d.state == DraftState::Editable) != d.draft.is_some()
        {
            return Err("Invalid proposal document state");
        }
        d.source.validate()?;
        if !d.source.supports_task(d.generated.task) {
            return Err("Source kind does not support this proposal task");
        }
        if let Some(p) = &d.draft {
            p.validate(&d.source, d.generated.task)?;
        }
        if let Some(r) = &self.acceptance {
            if r.proposal_id != d.proposal_id
                || r.source_id != d.source.source_id()
                || r.source_revision != d.source.source_revision()
                || r.source_sha256 != d.source.source_sha256()
                || r.output_revision != d.draft_revision
                || r.draft_revision.0.checked_add(1) != Some(r.output_revision.0)
                || r.automatic_application_enabled
                || serde_json::to_value(&r.output).ok() != serde_json::to_value(&d.draft).ok()
            {
                return Err("Acceptance receipt does not match its immutable publication");
            }
            r.output.validate(&d.source, d.generated.task)?;
        }
        Ok(())
    }
}

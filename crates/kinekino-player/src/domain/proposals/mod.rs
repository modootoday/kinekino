pub mod accepted_subtitle_projection;
mod document_source;
mod historical;
mod origin;
pub use origin::ProposalStoreOrigin;
pub mod output_review;
mod request_source;
mod stored;
mod transitions;
pub use historical::{
    CompanionReference, DOCUMENT_VERSIONS, HISTORICAL_BINDING_VERSION, HISTORICAL_DOCUMENT_VERSION,
    HistoricalBinding, LEGACY_DOCUMENT_VERSIONS, proposal_document_version,
};
pub use stored::StoredProposal;
pub use transitions::{AcceptedProposal, ProposalMutationError, accept_state, edit_state};
#[cfg(test)]
mod tests;
mod validation;
use crate::domain::api_values::DecimalU64;
pub use document_source::*;
pub use request_source::{MAX_DOCUMENT_BYTES, request_source_payload};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceCue {
    pub cue_id: String,
    pub text: String,
    pub start_us: Option<DecimalU64>,
    pub end_us: Option<DecimalU64>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CueSourceSnapshot {
    pub source_id: String,
    pub source_revision: DecimalU64,
    pub source_sha256: String,
    pub cues: Vec<SourceCue>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SourceSnapshot {
    Cue(CueSourceSnapshot),
    Document(DocumentSourceSnapshot),
}
impl SourceSnapshot {
    pub fn source_id(&self) -> &str {
        match self {
            Self::Cue(s) => &s.source_id,
            Self::Document(s) => &s.source_id,
        }
    }
    pub fn source_revision(&self) -> DecimalU64 {
        match self {
            Self::Cue(s) => s.source_revision,
            Self::Document(s) => s.source_revision,
        }
    }
    pub fn source_sha256(&self) -> &str {
        match self {
            Self::Cue(s) => &s.source_sha256,
            Self::Document(s) => &s.source_sha256,
        }
    }
    pub fn schema_version(&self) -> &'static str {
        match self {
            Self::Cue(_) => "1",
            Self::Document(_) => "2",
        }
    }
    pub fn cues(&self) -> &[SourceCue] {
        match self {
            Self::Cue(s) => &s.cues,
            Self::Document(_) => &[],
        }
    }
    pub fn as_document(&self) -> Option<&DocumentSourceSnapshot> {
        match self {
            Self::Document(s) => Some(s),
            Self::Cue(_) => None,
        }
    }
    pub fn validate(&self) -> Result<(), &'static str> {
        match self {
            Self::Cue(s) => s.validate(),
            Self::Document(s) => s.validate(),
        }
    }
    pub fn supports_task(&self, task: ProposalTask) -> bool {
        match self {
            Self::Cue(_) => true,
            Self::Document(_) => task == ProposalTask::Correction,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalTask {
    Correction,
    Cards,
    Highlights,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeneratedResult {
    pub task: ProposalTask,
    pub raw_text: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Correction {
    pub cue_id: String,
    pub text: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Card {
    pub front: String,
    pub back: String,
    pub source_refs: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Highlight {
    pub title: String,
    pub source_refs: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "task", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProposalPayload {
    Correction { edits: Vec<Correction> },
    DocumentCorrection { edits: Vec<DocumentCorrection> },
    Cards { cards: Vec<Card> },
    Highlights { highlights: Vec<Highlight> },
}
impl ProposalPayload {
    pub fn task(&self) -> ProposalTask {
        match self {
            Self::Correction { .. } | Self::DocumentCorrection { .. } => ProposalTask::Correction,
            Self::Cards { .. } => ProposalTask::Cards,
            Self::Highlights { .. } => ProposalTask::Highlights,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DraftState {
    Editable,
    NeedsRepair,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProposalDocument {
    pub schema_version: String,
    pub proposal_id: String,
    pub source: SourceSnapshot,
    pub generated: GeneratedResult,
    pub draft_revision: DecimalU64,
    pub state: DraftState,
    pub draft: Option<ProposalPayload>,
    pub import_error: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcceptanceReceipt {
    pub proposal_id: String,
    pub draft_revision: DecimalU64,
    pub source_id: String,
    pub source_revision: DecimalU64,
    pub source_sha256: String,
    pub output_revision: DecimalU64,
    pub output: ProposalPayload,
    pub automatic_application_enabled: bool,
}

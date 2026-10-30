#[cfg(test)]
mod origin_tests;
pub mod reconnect;
#[cfg(test)]
pub(crate) mod tests;
mod validation;

use crate::domain::{
    api_values::DecimalU64,
    proposals::{ProposalPayload, StoredProposal},
};
use serde::{Deserialize, Serialize};

pub const MAX_RECORD_BYTES: usize = 3 * 1024 * 1024;
pub const MAX_WORKING_BYTES: usize = 512 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum WatchingSource {
    LocalVideo {
        source_id: String,
        path: String,
    },
    EpisodePage {
        source_id: String,
        episode_url: String,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WatchingSelection {
    pub label: String,
    pub source: WatchingSource,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkingState {
    Absent,
    Valid,
    NeedsInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectProposalSnapshot {
    pub capture_sha256: String,
    pub captured: StoredProposal,
    pub working_draft: Option<ProposalPayload>,
    pub working_state: WorkingState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposal_origin: Option<crate::domain::proposals::ProposalStoreOrigin>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectWorkState {
    pub needs_repair: u32,
    pub editable: u32,
    pub accepted_output: u32,
    pub working_needs_input: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectRevision {
    pub schema_version: String,
    pub project_id: String,
    pub revision: DecimalU64,
    pub parent_revision: Option<DecimalU64>,
    pub title: String,
    pub watching: Option<WatchingSelection>,
    pub proposals: Vec<ProjectProposalSnapshot>,
    pub work_state: ProjectWorkState,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredProjectRevision {
    pub revision_sha256: String,
    pub latest_revision: DecimalU64,
    pub project: ProjectRevision,
}

pub fn canonical_sha256(value: &impl Serialize) -> Result<String, &'static str> {
    use sha2::{Digest, Sha256};
    let bytes = serde_json::to_vec(value).map_err(|_| "Project serialization failed")?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

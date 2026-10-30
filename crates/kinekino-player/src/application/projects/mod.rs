#[path = "ports.rs"]
mod ports;
pub use ports::ProjectStore;
mod capture;
mod reconnect;
pub use reconnect::{ReconnectProjectSource, ReconnectedProjectSource, reconnect_source};
#[cfg(test)]
mod tests;
use crate::{
    application::proposals::ProposalStore,
    domain::{api_values::DecimalU64, projects::*, proposals::ProposalPayload},
};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use serde::{Deserialize, Serialize};

pub struct UnavailableProjectStore;
impl ProjectStore for UnavailableProjectStore {
    fn available(&self) -> bool {
        false
    }
    fn load(&self, _: &str, _: Option<u64>) -> Result<StoredProjectRevision, ApiError> {
        Err(unavailable())
    }
    fn publish(
        &self,
        _: Option<u64>,
        _: &ProjectRevision,
    ) -> Result<StoredProjectRevision, ApiError> {
        Err(unavailable())
    }
}
pub fn unavailable() -> ApiError {
    ApiError::new(
        ErrorCode::Unsupported,
        "Project storage adapter unavailable",
    )
}
fn invalid(message: impl Into<String>) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
fn conflict(message: &str) -> ApiError {
    ApiError::new(ErrorCode::RevisionConflict, message)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "origin", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProposalRevisionRef {
    Live {
        proposal_id: String,
        draft_revision: DecimalU64,
        working_draft: Option<ProposalPayload>,
    },
    Saved {
        proposal_id: String,
        capture_sha256: String,
        working_draft: Option<ProposalPayload>,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectContentInput {
    pub watching: Option<WatchingSelection>,
    pub proposals: Vec<ProposalRevisionRef>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateProject {
    pub project_id: String,
    pub title: String,
    pub content: ProjectContentInput,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveProjectRevision {
    pub project_id: String,
    pub title: String,
    pub content: ProjectContentInput,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InspectProject {
    pub project_id: String,
    pub revision: Option<DecimalU64>,
}

pub fn create(
    store: &dyn ProjectStore,
    proposals: &dyn ProposalStore,
    input: CreateProject,
) -> Result<StoredProjectRevision, ApiError> {
    if !store.available() {
        return Err(unavailable());
    }
    let project = capture::revision(
        proposals,
        input.project_id,
        input.title,
        input.content,
        None,
    )?;
    store.publish(None, &project)
}
pub fn inspect(
    store: &dyn ProjectStore,
    input: InspectProject,
) -> Result<StoredProjectRevision, ApiError> {
    inspect_with_proposals(
        store,
        &crate::application::proposals::UnavailableProposalStore,
        input,
    )
}
pub fn inspect_with_proposals(
    store: &dyn ProjectStore,
    proposals: &dyn ProposalStore,
    input: InspectProject,
) -> Result<StoredProjectRevision, ApiError> {
    if !store.available() {
        return Err(unavailable());
    }
    if !crate::domain::identifiers::valid_id(&input.project_id)
        || input.revision.is_some_and(|n| n.0 == 0)
    {
        return Err(invalid("Invalid project identity or revision"));
    }
    let state = store.load(&input.project_id, input.revision.map(|n| n.0))?;
    for snapshot in &state.project.proposals {
        crate::application::ai_provenance::verify_bound(
            proposals.historical_reader(),
            &snapshot.captured,
        )?;
    }
    Ok(state)
}
pub fn save(
    store: &dyn ProjectStore,
    proposals: &dyn ProposalStore,
    expected: u64,
    input: SaveProjectRevision,
) -> Result<StoredProjectRevision, ApiError> {
    if !store.available() {
        return Err(unavailable());
    }
    let previous = store.load(&input.project_id, None)?;
    if previous.latest_revision.0 != expected {
        return Err(conflict("Project revision changed; inspect before saving"));
    }
    for snapshot in &previous.project.proposals {
        crate::application::ai_provenance::verify_bound(
            proposals.historical_reader(),
            &snapshot.captured,
        )?;
    }
    let project = capture::revision(
        proposals,
        input.project_id,
        input.title,
        input.content,
        Some(&previous.project),
    )?;
    store.publish(Some(expected), &project)
}

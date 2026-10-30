use super::*;
use crate::domain::projects::reconnect::{self, ReconnectError};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReconnectProjectSource {
    pub project_id: String,
    pub expected_source: WatchingSource,
    pub replacement: WatchingSelection,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReconnectedProjectSource {
    pub state: StoredProjectRevision,
    pub scope: String,
    pub content_identity_verified: bool,
    pub source_opened: bool,
    pub playback_rebound: bool,
}
pub fn reconnect_source(
    store: &dyn ProjectStore,
    proposals: &dyn ProposalStore,
    expected: u64,
    input: ReconnectProjectSource,
) -> Result<ReconnectedProjectSource, ApiError> {
    if expected == 0 {
        return Err(invalid("Positive project revision required"));
    }
    let previous = inspect_with_proposals(
        store,
        proposals,
        InspectProject {
            project_id: input.project_id,
            revision: None,
        },
    )?;
    if previous.latest_revision.0 != expected || previous.project.revision.0 != expected {
        return Err(conflict(
            "Project revision changed; inspect before reconnecting",
        ));
    }
    let next =
        reconnect::watching_locator(&previous.project, &input.expected_source, input.replacement)
            .map_err(|error| match error {
            ReconnectError::SelectionChanged => conflict("Watching selection changed"),
            ReconnectError::NoWatchingSelection => ApiError::new(
                ErrorCode::NeedsInput,
                "Project has no watching source selection",
            ),
            ReconnectError::InvalidProject(message) => {
                ApiError::new(ErrorCode::NeedsInput, message)
            }
            ReconnectError::InvalidSelection(message) => invalid(message),
            ReconnectError::KindOrIdChanged => {
                invalid("Locator reconnection must retain source kind and source_id")
            }
            ReconnectError::RevisionExhausted => {
                ApiError::new(ErrorCode::ResourceExhausted, "Project revision exhausted")
            }
        })?;
    Ok(ReconnectedProjectSource {
        state: store.publish(Some(expected), &next)?,
        scope: "watching_locator_only".into(),
        content_identity_verified: false,
        source_opened: false,
        playback_rebound: false,
    })
}

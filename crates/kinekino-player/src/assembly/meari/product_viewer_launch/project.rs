//! Resolve the explicit selection before creating any LocalHost, Native loop or worker owner.
use crate::application::api::ProjectClient;
use crate::{
    application::{
        api::{ApiError, ApplicationClient, ErrorCode, RequestContext},
        catalog::CatalogArtifacts,
        projects::InspectProject,
    },
    domain::projects::StoredProjectRevision,
};
use std::{path::Path, sync::Arc};
pub struct SelectedViewerProject {
    pub stored: StoredProjectRevision,
    pub artifacts: Arc<dyn CatalogArtifacts>,
}
pub fn prepare_project(
    selection: Option<InspectProject>,
    client: &dyn ApplicationClient,
    project_root: Option<&Path>,
) -> Result<Option<SelectedViewerProject>, ApiError> {
    let Some(selection) = selection else {
        return Ok(None);
    };
    let root = project_root.ok_or_else(|| {
        ApiError::new(
            ErrorCode::NeedsInput,
            "Selected Viewer project requires configured project storage",
        )
    })?;
    let stored = client.inspect_project(RequestContext::default(), selection)?;
    let artifacts = crate::infrastructure::meari::capture_evidence::open(root)?;
    Ok(Some(SelectedViewerProject { stored, artifacts }))
}

//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

/// Projects.
pub trait ProjectClient: ApplicationClient {
    fn reconnect_project_source(
        &self,
        context: RequestContext,
        input: crate::application::projects::ReconnectProjectSource,
    ) -> Result<crate::application::projects::ReconnectedProjectSource, ApiError> {
        review_call(self, context, "projects.reconnect_source", input)
    }
    fn create_project(
        &self,
        context: RequestContext,
        payload: crate::application::projects::CreateProject,
    ) -> Result<crate::domain::projects::StoredProjectRevision, ApiError> {
        review_call(self, context, "projects.create", payload)
    }
    fn inspect_project(
        &self,
        context: RequestContext,
        payload: crate::application::projects::InspectProject,
    ) -> Result<crate::domain::projects::StoredProjectRevision, ApiError> {
        review_call(self, context, "projects.inspect", payload)
    }
    fn save_project_revision(
        &self,
        context: RequestContext,
        payload: crate::application::projects::SaveProjectRevision,
    ) -> Result<crate::domain::projects::StoredProjectRevision, ApiError> {
        review_call(self, context, "projects.save_revision", payload)
    }
}

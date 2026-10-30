//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

/// Clip projects, presets, renders and sources.
pub trait ClipClient: ApplicationClient {
    /// Project persistence only; does not validate media custody or render a clip.
    fn create_clip_project(
        &self,
        context: RequestContext,
        document: crate::domain::clip_document::ClipDocument,
    ) -> Result<crate::application::clip_projects::StoredClipProject, ApiError> {
        review_call(
            self,
            context,
            "clip_projects.create",
            crate::application::clip_projects::ClipProjectInput { document },
        )
    }
    /// The context carries the expected publication head, distinct from edit revision.
    fn save_clip_project(
        &self,
        context: RequestContext,
        document: crate::domain::clip_document::ClipDocument,
    ) -> Result<crate::application::clip_projects::StoredClipProject, ApiError> {
        review_call(
            self,
            context,
            "clip_projects.save_revision",
            crate::application::clip_projects::ClipProjectInput { document },
        )
    }
    fn inspect_clip_project(
        &self,
        context: RequestContext,
        input: crate::application::clip_projects::InspectClipProject,
    ) -> Result<crate::application::clip_projects::StoredClipProject, ApiError> {
        review_call(self, context, "clip_projects.inspect", input)
    }
    fn clip_preset_catalog(
        &self,
        context: RequestContext,
    ) -> Result<crate::application::clip_render::presets::ClipPresetCatalog, ApiError> {
        let capabilities: serde_json::Value =
            review_call(self, context, "capabilities", serde_json::json!({}))?;
        serde_json::from_value(capabilities["clip_presets"].clone()).map_err(|_| invalid_response())
    }
    fn submit_clip_render(
        &self,
        context: RequestContext,
        input: crate::application::clip_render::ClipRenderRequest,
        execution_timeout_ms: Option<crate::domain::api_values::DecimalU64>,
    ) -> Result<crate::application::jobs::SubmittedJob, ApiError> {
        self.submit_job(
            context,
            crate::application::jobs::SubmitJob {
                operation: crate::application::clip_render::RENDER_OPERATION.into(),
                parameters: serde_json::to_value(input).map_err(|_| invalid_response())?,
                expected_source_sha256: None,
                execution_timeout_ms: execution_timeout_ms
                    .unwrap_or(crate::domain::api_values::DecimalU64(120_000)),
            },
        )
    }
    fn inspect_clip_source(
        &self,
        context: RequestContext,
        input: crate::application::clip_render::InspectClipSource,
    ) -> Result<crate::application::clip_render::ClipSourceObservation, ApiError> {
        review_call(self, context, "clip_sources.inspect", input)
    }
}

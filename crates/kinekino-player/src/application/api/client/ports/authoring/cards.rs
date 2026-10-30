//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

/// Card projects, templates, bundles and PNG review export.
pub trait CardClient: ApplicationClient {
    fn card_template_catalog(
        &self,
        context: RequestContext,
    ) -> Result<crate::application::card_bundle::CardTemplateCatalog, ApiError> {
        let capabilities: serde_json::Value =
            review_call(self, context, "capabilities", serde_json::json!({}))?;
        serde_json::from_value(capabilities["card_templates"].clone())
            .map_err(|_| invalid_response())
    }
    fn submit_card_bundle(
        &self,
        context: RequestContext,
        input: crate::application::card_bundle::CardBundleRequest,
        execution_timeout_ms: Option<crate::domain::api_values::DecimalU64>,
    ) -> Result<crate::application::jobs::SubmittedJob, ApiError> {
        self.submit_job(
            context,
            crate::application::jobs::SubmitJob {
                operation: crate::application::card_bundle::EXPORT_OPERATION.into(),
                parameters: serde_json::to_value(input).map_err(|_| invalid_response())?,
                expected_source_sha256: None,
                execution_timeout_ms: execution_timeout_ms
                    .unwrap_or(crate::domain::api_values::DecimalU64(30_000)),
            },
        )
    }
    fn create_card_project(
        &self,
        context: RequestContext,
        document: crate::domain::card_document::CardDocument,
    ) -> Result<crate::application::card_projects::StoredCardProject, ApiError> {
        review_call(
            self,
            context,
            "card_projects.create",
            crate::application::card_projects::CardProjectInput { document },
        )
    }
    fn save_card_project(
        &self,
        context: RequestContext,
        document: crate::domain::card_document::CardDocument,
    ) -> Result<crate::application::card_projects::StoredCardProject, ApiError> {
        review_call(
            self,
            context,
            "card_projects.save_revision",
            crate::application::card_projects::CardProjectInput { document },
        )
    }
    fn inspect_card_project(
        &self,
        context: RequestContext,
        input: crate::application::card_projects::InspectCardProject,
    ) -> Result<crate::application::card_projects::StoredCardProject, ApiError> {
        review_call(self, context, "card_projects.inspect", input)
    }
    fn export_card_review_png(
        &self,
        context: RequestContext,
        input: crate::application::proposals::card_export::ExportCardReviewPng,
    ) -> Result<crate::application::proposals::card_export::CardReviewArtifact, ApiError> {
        review_call(
            self,
            context,
            crate::application::proposals::card_export::COMMAND,
            input,
        )
    }
}

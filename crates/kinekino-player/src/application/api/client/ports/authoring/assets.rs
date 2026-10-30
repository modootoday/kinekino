//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

/// Asset sources, artifacts, projects and bundles.
pub trait AssetClient: ApplicationClient {
    fn export_asset_sources(
        &self,
        context: RequestContext,
        input: crate::application::asset_source_archives::ExportAssetSources,
    ) -> Result<crate::application::asset_source_archives::AssetSourcesObservation, ApiError> {
        review_call(self, context, "asset_sources.export", input)
    }
    fn inspect_asset_sources(
        &self,
        context: RequestContext,
        input: crate::application::asset_source_archives::InspectAssetSources,
    ) -> Result<crate::application::asset_source_archives::AssetSourcesObservation, ApiError> {
        review_call(self, context, "asset_sources.inspect", input)
    }
    fn inspect_asset_artifact(
        &self,
        context: RequestContext,
        input: crate::application::asset_artifacts::InspectAssetArtifact,
    ) -> Result<crate::application::asset_artifacts::AssetArtifactInspection, ApiError> {
        review_call(self, context, "asset_artifacts.inspect", input)
    }
    /// Project persistence only; does not validate media custody or render an asset.
    fn create_asset_project(
        &self,
        context: RequestContext,
        document: crate::domain::asset_generation::AssetDocument,
    ) -> Result<crate::application::asset_projects::StoredAssetProject, ApiError> {
        review_call(
            self,
            context,
            "asset_projects.create",
            crate::application::asset_projects::AssetProjectInput { document },
        )
    }
    /// The context carries the expected publication head, distinct from edit revision.
    fn save_asset_project(
        &self,
        context: RequestContext,
        document: crate::domain::asset_generation::AssetDocument,
    ) -> Result<crate::application::asset_projects::StoredAssetProject, ApiError> {
        review_call(
            self,
            context,
            "asset_projects.save_revision",
            crate::application::asset_projects::AssetProjectInput { document },
        )
    }
    fn inspect_asset_project(
        &self,
        context: RequestContext,
        input: crate::application::asset_projects::InspectAssetProject,
    ) -> Result<crate::application::asset_projects::StoredAssetProject, ApiError> {
        review_call(self, context, "asset_projects.inspect", input)
    }
    fn submit_asset_bundle(
        &self,
        context: RequestContext,
        input: crate::application::asset_bundle::AssetBundleRequest,
        execution_timeout_ms: Option<crate::domain::api_values::DecimalU64>,
    ) -> Result<crate::application::jobs::SubmittedJob, ApiError> {
        self.submit_job(
            context,
            crate::application::jobs::SubmitJob {
                operation: crate::application::asset_bundle::BUILD_OPERATION.into(),
                parameters: serde_json::to_value(input).map_err(|_| invalid_response())?,
                expected_source_sha256: None,
                execution_timeout_ms: execution_timeout_ms
                    .unwrap_or(crate::domain::api_values::DecimalU64(120_000)),
            },
        )
    }
    fn read_review_text_asset(
        &self,
        context: RequestContext,
        payload: crate::application::model_review::ReadReviewTextAsset,
    ) -> Result<crate::application::model_review::ReviewTextAsset, ApiError> {
        review_call(self, context, "reviews.read_text_asset", payload)
    }
}

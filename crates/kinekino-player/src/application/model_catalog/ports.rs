//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

pub trait ModelCatalogPort: Send + Sync {
    fn snapshot(&self) -> Result<(ModelCatalog, String), ApiError>;
    fn inspect_files(&self, model: &ModelManifest) -> Result<Vec<ModelFileObservation>, ApiError>;
    fn verification_available(&self) -> bool {
        false
    }
    fn verify_files(
        &self,
        _: &VerifyModel,
        _: &dyn crate::application::review_source::MutationControl,
    ) -> Result<ModelVerification, ApiError> {
        Err(ApiError::new(
            ErrorCode::Unsupported,
            "Model file verification is not configured",
        ))
    }
}

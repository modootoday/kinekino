#[path = "ports.rs"]
mod ports;
use super::BundleTemplatePin;
use crate::application::api::ApiError;
pub use ports::CardTemplateCatalogPort;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CardTemplateCatalog {
    pub schema_version: String,
    pub backend_available: bool,
    pub templates: Vec<BundleTemplatePin>,
    pub review_artifact_only: bool,
    pub quality_approved: bool,
    pub platform_requirements_verified: bool,
}
impl CardTemplateCatalog {
    pub fn unavailable() -> Self {
        Self {
            schema_version: "1".into(),
            backend_available: false,
            templates: Vec::new(),
            review_artifact_only: true,
            quality_approved: false,
            platform_requirements_verified: false,
        }
    }
}

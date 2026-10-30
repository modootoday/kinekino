//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

/// Product assembly supplies the actual renderer and licensed asset admission.
pub trait CardTemplateCatalogPort: Send + Sync {
    fn inspect_templates(&self) -> Result<CardTemplateCatalog, ApiError>;
}

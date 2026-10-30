//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

pub trait ClipPresetCatalogPort: ClipPresetAdmission {
    fn inspect_presets(&self) -> Result<ClipPresetCatalog, ApiError>;
}

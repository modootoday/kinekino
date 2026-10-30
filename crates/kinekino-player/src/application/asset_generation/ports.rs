//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

pub trait AssetRasterPort {
    fn render(
        &self,
        input: AssetRasterInput<'_>,
        transform: AssetTransform,
        layer: LayerKind,
        control: &dyn MutationControl,
    ) -> Result<AssetRasterOutput, ApiError>;
}

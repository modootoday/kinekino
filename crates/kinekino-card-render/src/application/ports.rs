use super::RenderError;
use crate::domain::{CardTemplate, CardText, RenderLimits, Size, SlotObservation};
use meari::application::assets::VerifiedAssets;
use std::collections::BTreeMap;

/// Host adapter MUST reuse the existing complete static PNG parser: CRC, Adler,
/// no APNG/trailing bytes, decoded dimensions and shared byte/pixel limits.
/// This is not satisfied by reading only the PNG signature or IHDR.
pub trait PngInspection {
    type Error;

    fn inspect_static_png(&self, bytes: &[u8], limits: RenderLimits) -> Result<Size, Self::Error>;
}

pub trait PngEncoding {
    fn encode_rgba(&self, size: Size, rgba: &[u8], limit: usize) -> Result<Vec<u8>, RenderError>;
}

pub struct RenderPorts<'a, E> {
    pub png: &'a dyn PngInspection<Error = E>,
    pub encoder: &'a dyn PngEncoding,
    pub rasterizer: &'a mut dyn CardRasterizer,
}

pub struct CardScene<'a> {
    pub template: &'a CardTemplate,
    pub selection: CardText<'a>,
    /// Empty only for an explicitly frame-free request; no image element or request.
    pub frame_url: &'a str,
    pub font_urls: &'a [String],
}

#[derive(Clone, Debug, Default)]
pub struct AssetDelivery {
    pub requests: u64,
    /// Per-path handler delivery counts, not delivered byte totals or shaping proof.
    pub delivered: BTreeMap<String, u64>,
    pub rejected: u64,
    pub failures: Vec<String>,
}

pub struct RasterizedCard {
    pub rgba: Vec<u8>,
    pub front: SlotObservation,
    pub back: SlotObservation,
    pub assets: AssetDelivery,
}
pub struct PagedRasterizedCard {
    pub card: RasterizedCard,
    pub page: Option<SlotObservation>,
}

/// Observations must be obtained after the same rasterization's final layout.
pub trait CardRasterizer {
    /// Existing adapters retain their original behavior for requests without a page.
    fn rasterize_with_page(
        &mut self,
        scene: CardScene<'_>,
        assets: VerifiedAssets,
        size: Size,
        page: Option<&crate::domain::PageFooter>,
    ) -> Result<PagedRasterizedCard, String> {
        if page.is_some() {
            return Err("This adapter does not support a measured page slot".into());
        }
        self.rasterize(scene, assets, size)
            .map(|card| PagedRasterizedCard { card, page: None })
    }
    fn rasterize(
        &mut self,
        scene: CardScene<'_>,
        assets: VerifiedAssets,
        size: Size,
    ) -> Result<RasterizedCard, String>;
}

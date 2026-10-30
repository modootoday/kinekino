mod assets;
pub mod ports;
mod render;
pub use render::{render, render_optional_frame, render_text, render_with_page};

use crate::domain::{
    CardTemplate, CardText, Limitation, RenderLimits, SelectedCard, Size, Slot, SlotObservation,
};
use meari::{
    application::assets::VerifiedAssets,
    domain::build::{BuildPath, ContentHash},
};

#[derive(Clone, Debug)]
pub struct FramePin {
    pub asset: BuildPath,
    pub sha256: ContentHash,
}

/// References to the existing common font manifest; no OS path or download input.
#[derive(Clone, Debug)]
pub struct FontSelection {
    pub regular: BuildPath,
    pub semibold: BuildPath,
    pub fallback: Option<BuildPath>,
}

pub struct RenderRequest<'a> {
    pub selection: SelectedCard<'a>,
    pub frame: FramePin,
    pub fonts: FontSelection,
    pub template: CardTemplate,
    pub limits: RenderLimits,
    pub assets: VerifiedAssets,
}

/// Shared raster input for manually edited projects and other explicit text consumers.
/// Authentication, publication and quality decisions remain in the product SDK.
pub struct TextRenderRequest<'a> {
    pub selection: CardText<'a>,
    pub frame: FramePin,
    pub fonts: FontSelection,
    pub template: CardTemplate,
    pub limits: RenderLimits,
    pub assets: VerifiedAssets,
}

pub struct RenderedTextCard<'a> {
    pub png: Vec<u8>,
    pub png_sha256: ContentHash,
    pub size: Size,
    pub selection: CardText<'a>,
    pub frame_sha256: ContentHash,
    pub frame_size: Size,
    pub template_sha256: ContentHash,
    pub template_version: String,
    pub asset_manifest_sha256: ContentHash,
    pub font_sha256: Vec<ContentHash>,
    pub font_delivery_observed: Vec<bool>,
    pub front: SlotObservation,
    pub back: SlotObservation,
    pub asset_delivery: ports::AssetDelivery,
    pub limitations: Vec<Limitation>,
}

/// Text-only templates carry no invented image identity or placeholder PNG.
pub struct OptionalFrameRenderRequest<'a> {
    pub selection: CardText<'a>,
    pub frame: Option<FramePin>,
    pub fonts: FontSelection,
    pub template: CardTemplate,
    pub limits: RenderLimits,
    pub assets: VerifiedAssets,
}
impl<'a> From<TextRenderRequest<'a>> for OptionalFrameRenderRequest<'a> {
    fn from(request: TextRenderRequest<'a>) -> Self {
        Self {
            selection: request.selection,
            frame: Some(request.frame),
            fonts: request.fonts,
            template: request.template,
            limits: request.limits,
            assets: request.assets,
        }
    }
}
pub struct RenderedOptionalFrameCard<'a> {
    pub png: Vec<u8>,
    pub png_sha256: ContentHash,
    pub size: Size,
    pub selection: CardText<'a>,
    pub frame_sha256: Option<ContentHash>,
    pub frame_size: Option<Size>,
    pub template_sha256: ContentHash,
    pub template_version: String,
    pub asset_manifest_sha256: ContentHash,
    pub font_sha256: Vec<ContentHash>,
    pub font_delivery_observed: Vec<bool>,
    pub front: SlotObservation,
    pub back: SlotObservation,
    pub asset_delivery: ports::AssetDelivery,
    pub limitations: Vec<Limitation>,
}
pub struct RenderedPagedCard<'a> {
    pub card: RenderedOptionalFrameCard<'a>,
    pub page: SlotObservation,
}

pub struct RenderedCard<'a> {
    pub png: Vec<u8>,
    pub png_sha256: ContentHash,
    pub size: Size,
    pub selection: SelectedCard<'a>,
    pub frame_sha256: ContentHash,
    pub frame_size: Size,
    pub template_sha256: ContentHash,
    pub template_version: String,
    pub asset_manifest_sha256: ContentHash,
    pub font_sha256: Vec<ContentHash>,
    pub font_delivery_observed: Vec<bool>,
    pub front: SlotObservation,
    pub back: SlotObservation,
    pub asset_delivery: ports::AssetDelivery,
    pub limitations: Vec<Limitation>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RenderError {
    InvalidInput(&'static str),
    ResourceLimit,
    Asset(String),
    FramePinMismatch,
    FramePng(String),
    Raster(String),
    LayoutUnavailable(Slot),
    LayoutOverflow(Slot),
    RgbaLengthMismatch,
    PngEncoding(String),
    OutputPng(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PngInspectionStage {
    Frame,
    Output,
}

/// Preserve the inspector's original owned cause; stage is separate metadata.
/// No Clone, Display, Error, serialization or Send bound is imposed on E.
#[derive(Debug)]
pub enum RenderFailure<E> {
    PngInspection { stage: PngInspectionStage, cause: E },
    Rendering(RenderError),
}

impl<E> From<RenderError> for RenderFailure<E> {
    fn from(error: RenderError) -> Self {
        Self::Rendering(error)
    }
}

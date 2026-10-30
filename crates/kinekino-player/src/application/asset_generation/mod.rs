//! Source/review admission and retained raster output shared by SDK, CLI and GUI.
#[path = "ports.rs"]
mod ports;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        review_source::MutationControl,
    },
    domain::asset_generation::*,
};
pub use ports::AssetRasterPort;

pub trait AssetImageLease: Send {
    fn pin(&self) -> &AssetImagePin;
    fn rgba(&self) -> &[u8];
    /// Exact original encoded bytes, when supported; never a re-encoded substitute.
    fn original_png(&self) -> Option<&[u8]> {
        None
    }
}
pub trait AssetMaskLease: Send {
    fn pin(&self) -> &AssetMaskPin;
    fn coverage(&self) -> &[u8];
    fn original_png(&self) -> Option<&[u8]> {
        None
    }
}
/// Trusted resolver owns authenticated user evidence; JSON cannot mint this lease.
pub trait AssetReviewLease: Send {
    fn record(&self) -> &AssetBlobPin;
    fn mask(&self) -> &AssetMaskPin;
    fn usable(&self) -> bool;
}
pub trait AssetInputAdmission {
    type Image: AssetImageLease;
    type Mask: AssetMaskLease;
    type Review: AssetReviewLease;
    fn open_image(
        &self,
        pin: &AssetImagePin,
        control: &dyn MutationControl,
    ) -> Result<Self::Image, ApiError>;
    fn open_mask(
        &self,
        pin: &AssetMaskPin,
        control: &dyn MutationControl,
    ) -> Result<Self::Mask, ApiError>;
    fn open_review(
        &self,
        record: &AssetBlobPin,
        mask: &AssetMaskPin,
        control: &dyn MutationControl,
    ) -> Result<Self::Review, ApiError>;
}
pub struct AssetRasterInput<'a> {
    pub width: u32,
    pub height: u32,
    pub rgba: &'a [u8],
    pub mask: &'a [u8],
}
pub struct AssetRasterOutput {
    pub geometry: AssetGeometry,
    pub rgba: Vec<u8>,
    pub mask: Vec<u8>,
    pub alpha_mode: AlphaMode,
    pub color: AssetColor,
}
pub struct PreparedAssetFrame<I: AssetImageLease, M: AssetMaskLease, R: AssetReviewLease> {
    request: AssetFrameRequest,
    image: I,
    mask: M,
    review: Option<R>,
}
pub struct RenderedAssetFrame<I: AssetImageLease, M: AssetMaskLease, R: AssetReviewLease> {
    prepared: PreparedAssetFrame<I, M, R>,
    pixels: AssetRasterOutput,
}
fn changed() -> ApiError {
    ApiError::new(
        ErrorCode::SourceExpired,
        "Original asset image/mask/review binding differs",
    )
}
fn invalid(message: &str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
pub type AdmittedAssetFrame<P> = PreparedAssetFrame<
    <P as AssetInputAdmission>::Image,
    <P as AssetInputAdmission>::Mask,
    <P as AssetInputAdmission>::Review,
>;
pub fn prepare<P: AssetInputAdmission>(
    request: &AssetFrameRequest,
    ports: &P,
    control: &dyn MutationControl,
) -> Result<AdmittedAssetFrame<P>, ApiError> {
    request.validate().map_err(invalid)?;
    control.checkpoint()?;
    let image = ports.open_image(&request.image, control)?;
    let count = pixel_count(request.image.width, request.image.height).map_err(invalid)?;
    if image.pin() != &request.image || image.rgba().len() != count * 4 {
        return Err(changed());
    }
    control.checkpoint()?;
    let mask = ports.open_mask(&request.mask, control)?;
    if mask.pin() != &request.mask || mask.coverage().len() != count {
        return Err(changed());
    }
    control.checkpoint()?;
    let review = match &request.review {
        AssetMaskReview::Reference { record } => {
            let review = ports.open_review(record, &request.mask, control)?;
            if review.record() != record || review.mask() != &request.mask {
                return Err(changed());
            }
            if !review.usable() {
                return Err(ApiError::new(
                    ErrorCode::NeedsInput,
                    "Mask user review is not usable",
                ));
            }
            Some(review)
        }
        AssetMaskReview::UnreviewedAcknowledged { .. } => None,
    };
    control.checkpoint()?;
    Ok(PreparedAssetFrame {
        request: request.clone(),
        image,
        mask,
        review,
    })
}
impl<I: AssetImageLease, M: AssetMaskLease, R: AssetReviewLease> PreparedAssetFrame<I, M, R> {
    pub fn request(&self) -> &AssetFrameRequest {
        &self.request
    }
    pub fn unreviewed_mask_warning(&self) -> bool {
        self.review.is_none()
    }
    pub fn render(
        self,
        raster: &dyn AssetRasterPort,
        control: &dyn MutationControl,
    ) -> Result<RenderedAssetFrame<I, M, R>, ApiError> {
        control.checkpoint()?;
        let pixels = raster.render(
            AssetRasterInput {
                width: self.request.image.width,
                height: self.request.image.height,
                rgba: self.image.rgba(),
                mask: self.mask.coverage(),
            },
            self.request.transform,
            self.request.layer,
            control,
        )?;
        let expected = geometry(
            self.request.transform,
            self.request.image.width,
            self.request.image.height,
            pixels.geometry.source_bounds,
        )
        .map_err(invalid)?;
        let count = pixel_count(expected.output_width, expected.output_height).map_err(invalid)?;
        if pixels.geometry != expected
            || pixels.rgba.len() != count * 4
            || pixels.mask.len() != count
            || pixels.alpha_mode != self.request.transform.alpha_mode
            || pixels.color != self.request.transform.color
        {
            return Err(changed());
        }
        for (row, mask) in pixels
            .rgba
            .chunks_exact(expected.output_width as usize * 4)
            .zip(pixels.mask.chunks_exact(expected.output_width as usize))
        {
            control.checkpoint()?;
            if row
                .as_chunks::<4>()
                .0
                .iter()
                .zip(mask)
                .any(|(pixel, mask)| {
                    pixel[3] != *mask
                        || (pixels.alpha_mode == AlphaMode::Premultiplied
                            && pixel[..3].iter().any(|channel| *channel > pixel[3]))
                })
            {
                return Err(changed());
            }
        }
        control.checkpoint()?;
        Ok(RenderedAssetFrame {
            prepared: self,
            pixels,
        })
    }
}
impl<I: AssetImageLease, M: AssetMaskLease, R: AssetReviewLease> RenderedAssetFrame<I, M, R> {
    pub fn retained_original_png_bytes(&self) -> u64 {
        self.prepared
            .image
            .original_png()
            .map_or(0, |b| b.len() as u64)
            + self
                .prepared
                .mask
                .original_png()
                .map_or(0, |b| b.len() as u64)
    }
    pub fn request(&self) -> &AssetFrameRequest {
        self.prepared.request()
    }
    pub fn pixels(&self) -> &AssetRasterOutput {
        &self.pixels
    }
    pub fn unreviewed_mask_warning(&self) -> bool {
        self.prepared.unreviewed_mask_warning()
    }
}

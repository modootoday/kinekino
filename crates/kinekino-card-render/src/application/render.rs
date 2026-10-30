use super::{
    OptionalFrameRenderRequest, PngInspectionStage, RenderError, RenderFailure, RenderRequest,
    RenderedCard, RenderedOptionalFrameCard, RenderedTextCard, TextRenderRequest, assets,
    ports::{CardScene, RenderPorts},
};
use crate::domain::{BoxContainment, RENDER_LIMITATIONS, Slot, SlotObservation};
use meari::domain::build::ContentHash;

/// Computes an artifact only. No store access, acceptance, filesystem or publishing.
pub fn render<'a, E>(
    request: RenderRequest<'a>,
    ports: RenderPorts<'_, E>,
) -> Result<RenderedCard<'a>, RenderFailure<E>> {
    let selection = request.selection;
    let pixels = render_text(
        TextRenderRequest {
            selection: selection.into(),
            frame: request.frame,
            fonts: request.fonts,
            template: request.template,
            limits: request.limits,
            assets: request.assets,
        },
        ports,
    )?;
    Ok(RenderedCard {
        png: pixels.png,
        png_sha256: pixels.png_sha256,
        size: pixels.size,
        selection,
        frame_sha256: pixels.frame_sha256,
        frame_size: pixels.frame_size,
        template_sha256: pixels.template_sha256,
        template_version: pixels.template_version,
        asset_manifest_sha256: pixels.asset_manifest_sha256,
        font_sha256: pixels.font_sha256,
        font_delivery_observed: pixels.font_delivery_observed,
        front: pixels.front,
        back: pixels.back,
        asset_delivery: pixels.asset_delivery,
        limitations: pixels.limitations,
    })
}

/// Does not construct an accepted-output identity for a project or manual draft.
pub fn render_text<'a, E>(
    request: TextRenderRequest<'a>,
    ports: RenderPorts<'_, E>,
) -> Result<RenderedTextCard<'a>, RenderFailure<E>> {
    let rendered = render_optional_frame(request.into(), ports)?;
    Ok(RenderedTextCard {
        png: rendered.png,
        png_sha256: rendered.png_sha256,
        size: rendered.size,
        selection: rendered.selection,
        frame_sha256: rendered
            .frame_sha256
            .ok_or(RenderError::InvalidInput("Required frame identity absent"))?,
        frame_size: rendered.frame_size.ok_or(RenderError::InvalidInput(
            "Required frame dimensions absent",
        ))?,
        template_sha256: rendered.template_sha256,
        template_version: rendered.template_version,
        asset_manifest_sha256: rendered.asset_manifest_sha256,
        font_sha256: rendered.font_sha256,
        font_delivery_observed: rendered.font_delivery_observed,
        front: rendered.front,
        back: rendered.back,
        asset_delivery: rendered.asset_delivery,
        limitations: rendered.limitations,
    })
}

pub fn render_optional_frame<'a, E>(
    request: OptionalFrameRenderRequest<'a>,
    ports: RenderPorts<'_, E>,
) -> Result<RenderedOptionalFrameCard<'a>, RenderFailure<E>> {
    render_inner(request, ports, None).map(|(card, _)| card)
}

pub fn render_with_page<'a, E>(
    request: OptionalFrameRenderRequest<'a>,
    page: crate::domain::PageFooter,
    ports: RenderPorts<'_, E>,
) -> Result<super::RenderedPagedCard<'a>, RenderFailure<E>> {
    let (card, observation) = render_inner(request, ports, Some(page))?;
    Ok(super::RenderedPagedCard {
        card,
        page: observation.ok_or(RenderError::LayoutUnavailable(Slot::Page))?,
    })
}

fn render_inner<'a, E>(
    request: OptionalFrameRenderRequest<'a>,
    ports: RenderPorts<'_, E>,
    page: Option<crate::domain::PageFooter>,
) -> Result<(RenderedOptionalFrameCard<'a>, Option<SlotObservation>), RenderFailure<E>> {
    validate(&request)?;
    if let Some(page) = page {
        page.validate(&request.template, request.frame.is_some())
            .map_err(RenderError::InvalidInput)?;
        if request
            .selection
            .front
            .len()
            .saturating_add(request.selection.back.len())
            .saturating_add(page.text().len())
            > request.limits.max_text_bytes
        {
            return Err(RenderError::ResourceLimit.into());
        }
    }
    let prepared = assets::prepare(&request)?;
    let frame_size = if let Some(pin) = &request.frame {
        let frame = assets::descriptor(&request.assets, &pin.asset)?;
        let frame_bytes = assets::resource(&request.assets, frame)?;
        if frame_bytes.len() > request.limits.max_png_bytes {
            return Err(RenderError::ResourceLimit.into());
        }
        let frame_size = ports
            .png
            .inspect_static_png(frame_bytes, request.limits)
            .map_err(|cause| RenderFailure::PngInspection {
                stage: PngInspectionStage::Frame,
                cause,
            })?;
        request
            .limits
            .rgba_bytes(frame_size)
            .ok_or(RenderError::ResourceLimit)?;
        Some(frame_size)
    } else {
        None
    };
    let manifest_sha256 = request
        .assets
        .manifest()
        .fingerprint()
        .map_err(RenderError::Asset)?;
    let template_sha256 = ContentHash::of(
        &serde_json::to_vec(&request.template).map_err(|e| RenderError::Asset(e.to_string()))?,
    );
    let size = request.template.canvas;
    let scene = CardScene {
        template: &request.template,
        selection: request.selection,
        frame_url: &prepared.frame_url,
        font_urls: &prepared.font_urls,
    };
    let painted = ports
        .rasterizer
        .rasterize_with_page(scene, request.assets, size, page.as_ref())
        .map_err(RenderError::Raster)?;
    let raster = painted.card;
    if page.is_some() {
        require_box(
            Slot::Page,
            painted
                .page
                .ok_or(RenderError::LayoutUnavailable(Slot::Page))?,
        )?;
    } else if painted.page.is_some() {
        return Err(RenderError::InvalidInput("Unexpected page observation").into());
    }
    if raster.assets.rejected != 0 || !raster.assets.failures.is_empty() {
        return Err(RenderError::Asset("renderer rejected a declared resource".into()).into());
    }
    if request.frame.is_some()
        && !raster
            .assets
            .delivered
            .get(&prepared.frame_url)
            .is_some_and(|count| *count > 0)
    {
        return Err(RenderError::Asset("frame resource delivery was not observed".into()).into());
    }
    require_box(Slot::Front, raster.front)?;
    require_box(Slot::Back, raster.back)?;
    let expected_bytes = request
        .limits
        .rgba_bytes(size)
        .ok_or(RenderError::ResourceLimit)?;
    if raster.rgba.len() != expected_bytes {
        return Err(RenderError::RgbaLengthMismatch.into());
    }
    let bytes = ports
        .encoder
        .encode_rgba(size, &raster.rgba, request.limits.max_png_bytes)?;
    if bytes.len() > request.limits.max_png_bytes {
        return Err(RenderError::ResourceLimit.into());
    }
    let decoded = ports
        .png
        .inspect_static_png(&bytes, request.limits)
        .map_err(|cause| RenderFailure::PngInspection {
            stage: PngInspectionStage::Output,
            cause,
        })?;
    if decoded != size {
        return Err(RenderError::OutputPng("encoded dimensions differ from canvas".into()).into());
    }
    Ok((
        RenderedOptionalFrameCard {
            png_sha256: ContentHash::of(&bytes),
            png: bytes,
            size,
            selection: request.selection,
            frame_sha256: request.frame.map(|pin| pin.sha256),
            frame_size,
            template_sha256,
            template_version: request.template.version,
            asset_manifest_sha256: manifest_sha256,
            font_sha256: prepared.font_sha256,
            font_delivery_observed: prepared
                .font_urls
                .iter()
                .map(|path| {
                    raster
                        .assets
                        .delivered
                        .get(path)
                        .is_some_and(|count| *count > 0)
                })
                .collect(),
            front: raster.front,
            back: raster.back,
            asset_delivery: raster.assets,
            limitations: RENDER_LIMITATIONS.to_vec(),
        },
        painted.page,
    ))
}

fn validate(request: &OptionalFrameRenderRequest<'_>) -> Result<(), RenderError> {
    request
        .template
        .validate()
        .map_err(RenderError::InvalidInput)?;
    let size = request.template.canvas;
    if size.width > 8192 || size.height > 8192 {
        return Err(RenderError::InvalidInput(
            "canvas exceeds current renderer viewport support",
        ));
    }
    request
        .limits
        .rgba_bytes(size)
        .ok_or(RenderError::ResourceLimit)?;
    let text_bytes = request
        .selection
        .front
        .len()
        .checked_add(request.selection.back.len())
        .ok_or(RenderError::ResourceLimit)?;
    if text_bytes > request.limits.max_text_bytes || request.limits.max_png_bytes == 0 {
        return Err(RenderError::ResourceLimit);
    }
    if request
        .selection
        .front
        .chars()
        .chain(request.selection.back.chars())
        .any(|ch| ch.is_control() && !matches!(ch, '\n' | '\r' | '\t'))
    {
        return Err(RenderError::InvalidInput(
            "text contains unsupported rendering controls",
        ));
    }
    Ok(())
}

fn require_box(slot: Slot, observation: SlotObservation) -> Result<(), RenderError> {
    match observation.containment() {
        BoxContainment::InsideLayoutBox => Ok(()),
        BoxContainment::OutsideLayoutBox => Err(RenderError::LayoutOverflow(slot)),
        BoxContainment::Unavailable => Err(RenderError::LayoutUnavailable(slot)),
    }
}

//! Headless CPU text image using existing verified common assets. No window or video worker.
mod markup;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        clip_render::subtitles::{layout::*, *},
        review_source::MutationControl,
    },
    infrastructure::clip_subtitle_assets::{NativeSubtitleDocument, NativeSubtitleStyle},
};
use meari::domain::build::BuildPath;
use meari_native::{
    application::Renderer,
    domain::{LayoutBox, Viewport},
    infrastructure::engine::NativeRenderer,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
type Admitted = PreparedClipSubtitles<NativeSubtitleDocument, NativeSubtitleStyle>;
#[derive(Debug, Serialize)]
pub struct ClipSubtitleImageFacts {
    pub recipe: String,
    pub cue_ids: Vec<String>,
    pub cue_text_sha256: Vec<String>,
    pub cue_separator: String,
    pub text_sha256: String,
    pub document: crate::domain::clip_document::SourceBoundArtifact,
    pub style: crate::domain::clip_document::ArtifactPin,
    pub fonts: Vec<ClipSubtitleFontPin>,
    pub width: u32,
    pub height: u32,
    pub png_sha256: String,
    pub alpha_mode: String,
    pub layout: SubtitleLayoutObservation,
    pub asset_delivery: serde_json::Value,
    pub nominal_cmap_checked: bool,
    pub backend_glyph_choice_observed: bool,
    pub ink_overflow_measured: bool,
    pub cooperative_checkpoints_only: bool,
    pub quality_approved: bool,
    pub text_source_provenance_verified: bool,
}
pub struct RenderedClipSubtitleImage<'a> {
    png: Vec<u8>,
    rgba: Vec<u8>,
    facts: ClipSubtitleImageFacts,
    html: String,
    _owner: &'a Admitted,
}
impl RenderedClipSubtitleImage<'_> {
    pub fn png(&self) -> &[u8] {
        &self.png
    }
    pub fn rgba(&self) -> &[u8] {
        &self.rgba
    }
    pub fn facts(&self) -> &ClipSubtitleImageFacts {
        &self.facts
    }
    pub fn markup_sha256(&self) -> String {
        hash(self.html.as_bytes())
    }
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn error(message: &str) -> ApiError {
    ApiError::new(ErrorCode::EngineFailed, message)
}
fn measured(b: LayoutBox) -> SubtitleMeasuredBox {
    SubtitleMeasuredBox {
        x: b.x,
        y: b.y,
        width: b.width,
        height: b.height,
    }
}
pub fn render_cue<'a>(
    owner: &'a Admitted,
    cue_index: usize,
    width: u32,
    height: u32,
    control: &dyn MutationControl,
) -> Result<RenderedClipSubtitleImage<'a>, ApiError> {
    render_cues(owner, &[cue_index], width, height, control)
}
/// Active cues share one box, in document order. Overflow rejects the entire set.
pub fn render_cues<'a>(
    owner: &'a Admitted,
    cue_indices: &[usize],
    width: u32,
    height: u32,
    control: &dyn MutationControl,
) -> Result<RenderedClipSubtitleImage<'a>, ApiError> {
    control.checkpoint()?;
    if cue_indices.is_empty()
        || cue_indices.len() > owner.cues().len()
        || cue_indices.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Nonempty ordered active cue indices required",
        ));
    }
    let cues: Vec<_> = cue_indices
        .iter()
        .map(|index| {
            owner.cues().get(*index).ok_or_else(|| {
                ApiError::new(
                    ErrorCode::InvalidArgument,
                    "Admitted subtitle cue index required",
                )
            })
        })
        .collect::<Result<_, _>>()?;
    let text = cues
        .iter()
        .map(|cue| cue.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let style = owner.style().style();
    let expected = pixel_box(style, width, height)?;
    let assets = owner.style().assets();
    let mut urls = Vec::new();
    let mut faces = Vec::new();
    for font in &style.fonts {
        let key = BuildPath::try_from(font.asset_path.clone())
            .map_err(|_| error("Invalid admitted font asset ID"))?;
        let descriptor = assets
            .manifest()
            .files
            .get(&key)
            .ok_or_else(|| error("Admitted common font missing"))?;
        let output = descriptor
            .output_path()
            .map_err(|_| error("Invalid common font output ID"))?;
        let (_, bytes) = assets
            .resource(&output)
            .ok_or_else(|| error("Common font data missing"))?;
        faces.push(
            ttf_parser::Face::parse(bytes, 0)
                .map_err(|_| error("Pinned font cmap cannot be parsed"))?,
        );
        urls.push(output.as_str().to_owned());
    }
    for ch in text.chars().filter(|ch| !ch.is_whitespace()) {
        control.checkpoint()?;
        if !faces
            .iter()
            .any(|face| face.glyph_index(ch).is_some_and(|glyph| glyph.0 != 0))
        {
            return Err(ApiError::new(
                ErrorCode::NeedsInput,
                format!(
                    "Selected common fonts have no nominal glyph for U+{:04X}",
                    u32::from(ch)
                ),
            ));
        }
    }
    let html = markup::document(style, &text, expected, width, height, &urls);
    let viewport = Viewport {
        width,
        height,
        dark: false,
    };
    let mut renderer = NativeRenderer::from_html_with_assets(&html, assets.clone())
        .map_err(|_| error("Subtitle CPU renderer initialization failed"))?;
    control.checkpoint()?;
    renderer
        .layout(&viewport, 0.0)
        .map_err(|_| error("Subtitle CPU layout failed"))?;
    control.checkpoint()?;
    let layout = SubtitleLayoutObservation {
        expected,
        slot: measured(
            renderer
                .bounds("subtitle-slot")
                .map_err(|_| error("Subtitle slot layout unavailable"))?,
        ),
        content: measured(
            renderer
                .bounds("subtitle-text")
                .map_err(|_| error("Subtitle text layout unavailable"))?,
        ),
    };
    require_layout(style, &layout)?;
    let mut rgba = renderer
        .rasterize(&viewport)
        .map_err(|_| error("Subtitle CPU paint failed"))?;
    control.checkpoint()?;
    if rgba.len() != width as usize * height as usize * 4 {
        return Err(error(
            "Subtitle renderer returned an unexpected RGBA length",
        ));
    }
    // The pinned CPU renderer's PixmapMut bytes are premultiplied. PNG needs straight alpha.
    for pixel in rgba.as_chunks_mut::<4>().0 {
        let alpha = u32::from(pixel[3]);
        for component in &mut pixel[..3] {
            if u32::from(*component) > alpha {
                return Err(error("Subtitle CPU premultiplied-alpha invariant failed"));
            }
            *component = (u32::from(*component) * 255 + alpha / 2)
                .checked_div(alpha)
                .unwrap_or(0)
                .min(255) as u8;
        }
    }
    let report = renderer
        .asset_report()
        .map_err(|_| error("Subtitle font delivery report unavailable"))?
        .ok_or_else(|| error("Subtitle renderer has no declared asset provider"))?;
    if report.rejected != 0
        || !report.failures.is_empty()
        || urls
            .iter()
            .any(|url| report.delivered.get(url).is_none_or(|count| *count == 0))
    {
        return Err(error("Subtitle selected font delivery was not observed"));
    }
    let mut png = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut png, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(|_| error("Subtitle PNG header failed"))?;
        writer
            .write_image_data(&rgba)
            .map_err(|_| error("Subtitle PNG data failed"))?;
        writer
            .finish()
            .map_err(|_| error("Subtitle PNG finish failed"))?;
    }
    control.checkpoint()?;
    if png.len() as u64 > crate::domain::frame_capture::MAX_FRAME_PNG_BYTES {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Subtitle PNG exceeds byte budget",
        ));
    }
    if crate::infrastructure::snapshot_export::inspect_static_png(&png)? != (width, height) {
        return Err(error("Subtitle PNG decoded dimensions differ"));
    }
    let facts = ClipSubtitleImageFacts {
        recipe: "clip-subtitle-rgba/2".into(),
        cue_ids: cues.iter().map(|cue| cue.cue_id.clone()).collect(),
        cue_text_sha256: cues.iter().map(|cue| hash(cue.text.as_bytes())).collect(),
        cue_separator: "newline".into(),
        text_sha256: hash(text.as_bytes()),
        document: owner.selection().document.clone(),
        style: owner.selection().style.clone(),
        fonts: owner.style().fonts().to_vec(),
        width,
        height,
        png_sha256: hash(&png),
        alpha_mode: "straight".into(),
        layout,
        asset_delivery: serde_json::to_value(report)
            .map_err(|_| error("Invalid subtitle asset report"))?,
        nominal_cmap_checked: true,
        backend_glyph_choice_observed: false,
        ink_overflow_measured: false,
        cooperative_checkpoints_only: true,
        quality_approved: false,
        text_source_provenance_verified: false,
    };
    Ok(RenderedClipSubtitleImage {
        png,
        rgba,
        facts,
        html,
        _owner: owner,
    })
}
#[cfg(test)]
mod tests;

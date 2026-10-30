//! Measured layout acceptance; it does not assert glyph ink or font-choice provenance.
use super::*;
#[derive(Debug, Clone, Copy, Serialize)]
pub struct SubtitlePixelBox {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
#[derive(Debug, Clone, Copy, Serialize)]
pub struct SubtitleMeasuredBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
#[derive(Debug, Clone, Serialize)]
pub struct SubtitleLayoutObservation {
    pub expected: SubtitlePixelBox,
    pub slot: SubtitleMeasuredBox,
    pub content: SubtitleMeasuredBox,
}
pub fn pixel_box(
    style: &ClipSubtitleStyle,
    width: u32,
    height: u32,
) -> Result<SubtitlePixelBox, ApiError> {
    validate_subtitle_style(style)?;
    if width == 0
        || height == 0
        || width > 8192
        || height > 8192
        || u64::from(width) * u64::from(height) > crate::domain::frame_capture::MAX_FRAME_PIXELS
    {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Subtitle viewport exceeds bounded output pixels",
        ));
    }
    let pixel = |value: u32, size: u32| {
        ((u64::from(value) * u64::from(size)) / u64::from(crate::domain::clip_document::CROP_SCALE))
            as u32
    };
    let value = SubtitlePixelBox {
        x: pixel(style.box_normalized.left, width),
        y: pixel(style.box_normalized.top, height),
        width: pixel(style.box_normalized.width, width),
        height: pixel(style.box_normalized.height, height),
    };
    if value.width == 0 || value.height < style.line_height_px {
        return Err(ApiError::new(
            ErrorCode::NeedsInput,
            "Subtitle box cannot contain one declared line; no automatic shrink",
        ));
    }
    Ok(value)
}
pub fn require_layout(
    style: &ClipSubtitleStyle,
    observed: &SubtitleLayoutObservation,
) -> Result<(), ApiError> {
    validate_subtitle_style(style)?;
    let fail = || {
        ApiError::new(
            ErrorCode::NeedsInput,
            "Subtitle text exceeds measured box/line limit; revise text or explicit style",
        )
    };
    for b in [observed.slot, observed.content] {
        if ![b.x, b.y, b.width, b.height]
            .iter()
            .all(|value| value.is_finite())
            || b.width < 0.0
            || b.height <= 0.0
        {
            return Err(fail());
        }
    }
    let expected = observed.expected;
    if [
        (observed.slot.x, expected.x),
        (observed.slot.y, expected.y),
        (observed.slot.width, expected.width),
        (observed.slot.height, expected.height),
    ]
    .iter()
    .any(|(a, b)| (*a - *b as f32).abs() > 0.25)
    {
        return Err(ApiError::new(
            ErrorCode::EngineFailed,
            "Subtitle renderer box differs from explicit style",
        ));
    }
    let s = observed.slot;
    let c = observed.content;
    if c.x < s.x - 0.25
        || c.y < s.y - 0.25
        || c.x + c.width > s.x + s.width + 0.25
        || c.y + c.height > s.y + s.height + 0.25
        || c.height > (style.max_lines * style.line_height_px) as f32 + 0.25
    {
        return Err(fail());
    }
    Ok(())
}

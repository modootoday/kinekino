use super::*;
use crate::{
    application::api::{ApiError, ErrorCode},
    domain::{
        clip_document::{AudioSelection, CROP_SCALE, FitMode},
        frame_capture::MAX_FRAME_PIXELS,
    },
};
use serde::Serialize;
use sha2::{Digest, Sha256};
#[derive(Debug, Clone, Serialize)]
pub struct ClipPixelCrop {
    pub left: u32,
    pub top: u32,
    pub width: u32,
    pub height: u32,
}
#[derive(Debug, Clone, Serialize)]
pub struct ManualClipAudio {
    pub stream_index: u32,
    pub volume_percent: u8,
    pub muted: bool,
    pub absolute_start_us: i128,
    pub absolute_end_us: i128,
}
#[derive(Debug, Clone, Serialize)]
pub struct ManualClipPlan {
    pub document_sha256: String,
    pub source_media_id: String,
    pub source_sha256: String,
    pub source_observation_sha256: String,
    pub segment_id: String,
    pub video_stream_index: u32,
    pub start_pts: i64,
    pub end_pts: i64,
    pub crop_sections: Vec<ManualClipCropSection>,
    pub width: u32,
    pub height: u32,
    pub fps_numerator: u32,
    pub fps_denominator: u32,
    pub duration_us: u64,
    pub input_seek_start_us: i128,
    pub audio: Option<ManualClipAudio>,
}
#[derive(Debug, Clone, Serialize)]
pub struct ManualClipCropSection {
    pub source_us: crate::domain::api_values::DecimalU64,
    pub threshold_pts: String,
    pub frame_offset: crate::domain::api_values::DecimalU64,
    pub crop: ClipPixelCrop,
    pub fit: FitMode,
}
pub fn manual_clip_plan<L: ClipSourceLease, E: ClipEncoderLease>(
    media: &PreparedClipMedia<L>,
    encoder: &PreparedClipEncoder<E>,
    boundary: &ClipFrameBoundaries,
) -> Result<ManualClipPlan, ApiError> {
    manual_clip_plan_inner(media, encoder, boundary, false)
}
pub fn manual_clip_plan_with_subtitles<L, E, D, S>(
    media: &PreparedClipMedia<L>,
    encoder: &PreparedClipEncoder<E>,
    boundary: &ClipFrameBoundaries,
    subtitles: &subtitles::PreparedClipSubtitles<D, S>,
) -> Result<ManualClipPlan, ApiError>
where
    L: ClipSourceLease,
    E: ClipEncoderLease,
    D: subtitles::ClipSubtitleDocumentLease,
    S: subtitles::ClipSubtitleStyleLease,
{
    let document = &media.publication().document;
    let segment = document.segments.first().ok_or_else(|| {
        ApiError::new(ErrorCode::InvalidArgument, "Subtitle clip segment required")
    })?;
    let source = document
        .sources
        .iter()
        .find(|source| source.media_id == segment.media_id)
        .ok_or_else(|| ApiError::new(ErrorCode::SourceExpired, "Subtitle source pin missing"))?;
    subtitles.require_binding(source, segment)?;
    manual_clip_plan_inner(media, encoder, boundary, true)
}
fn manual_clip_plan_inner<L: ClipSourceLease, E: ClipEncoderLease>(
    media: &PreparedClipMedia<L>,
    encoder: &PreparedClipEncoder<E>,
    boundary: &ClipFrameBoundaries,
    has_subtitle_adapter: bool,
) -> Result<ManualClipPlan, ApiError> {
    let document = &media.publication().document;
    let digest = format!(
        "{:x}",
        Sha256::digest(
            serde_json::to_vec(document)
                .map_err(|_| ApiError::new(ErrorCode::Internal, "Invalid manual clip document"))?
        )
    );
    if digest != encoder.document_sha256() {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Encoder admission belongs to a different clip document",
        ));
    }
    if document.segments.len() != 1 {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Initial manual renderer requires one clip segment",
        ));
    }
    let segment = &document.segments[0];
    if (segment.subtitles.is_some() && !has_subtitle_adapter) || segment.subject_ref.is_some() {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Selected subtitles and subject composition require their dedicated render adapters",
        ));
    }
    let source = document
        .sources
        .iter()
        .find(|source| source.media_id == segment.media_id)
        .ok_or_else(|| {
            ApiError::new(
                ErrorCode::SourceExpired,
                "Manual clip source pin is missing",
            )
        })?;
    let (start, end) = frame_thresholds(source, segment)?;
    if boundary.segment_id != segment.segment_id
        || boundary.stream_index != segment.video_stream_index
        || boundary.requested_range != segment.range
        || boundary.start_threshold_pts != start.to_string()
        || boundary.end_threshold_pts != end.to_string()
        || boundary.included_frames.0 == 0
    {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Decoded boundaries belong to a different clip segment",
        ));
    }
    if start.unsigned_abs() > (1u64 << 53)
        || end.unsigned_abs() > (1u64 << 53)
        || segment.range.end_us.0 - segment.range.start_us.0 > 60_000_000
        || u64::from(document.preset.width) * u64::from(document.preset.height) > MAX_FRAME_PIXELS
        || u64::from(document.preset.frame_rate.numerator)
            > 120 * u64::from(document.preset.frame_rate.denominator)
    {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Manual clip timestamp, duration or output pixel budget exceeded",
        ));
    }
    if boundary.crop_switches.len() != segment.crop_keys.len() {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Crop switches differ from the saved document",
        ));
    }
    // Logical branch buffer admission only, not an observed process RSS guarantee.
    let branch_bytes = u64::from(document.preset.width)
        * u64::from(document.preset.height)
        * 4
        * (3 * segment.crop_keys.len() as u64 + 4);
    if branch_bytes > 512 * 1024 * 1024 {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Section crop logical buffer budget exceeded",
        ));
    }
    let pixels = |value: u32, size: u32| -> u32 {
        ((u64::from(value) * u64::from(size)) / u64::from(CROP_SCALE)) as u32
    };
    let timeline = source
        .clock
        .transform()
        .map_err(|_| ApiError::new(ErrorCode::NeedsInput, "Invalid crop clock"))?;
    let mut crop_sections = Vec::with_capacity(segment.crop_keys.len());
    for (key, switch) in segment.crop_keys.iter().zip(&boundary.crop_switches) {
        let threshold = timeline
            .us_to_pts(key.source_us.0, crate::domain::media_timeline::Rounding::Up)
            .map_err(|_| ApiError::new(ErrorCode::NeedsInput, "Crop timestamp overflow"))?
            .value;
        if switch.source_us != key.source_us
            || switch.threshold_pts != threshold.to_string()
            || switch.frame_offset.0 > boundary.included_frames.0
        {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Decoded crop switch belongs to a different key",
            ));
        }
        let crop = ClipPixelCrop {
            left: pixels(key.crop.left, source.width),
            top: pixels(key.crop.top, source.height),
            width: pixels(key.crop.width, source.width),
            height: pixels(key.crop.height, source.height),
        };
        if crop.width == 0 || crop.height == 0 {
            return Err(ApiError::new(
                ErrorCode::NeedsInput,
                "Normalized crop has no complete source pixel",
            ));
        }
        crop_sections.push(ManualClipCropSection {
            source_us: key.source_us,
            threshold_pts: switch.threshold_pts.clone(),
            frame_offset: switch.frame_offset,
            crop,
            fit: key.fit,
        });
    }
    let origin = source
        .clock
        .origin_pts
        .parse::<i64>()
        .map_err(|_| ApiError::new(ErrorCode::NeedsInput, "Invalid source clock origin"))?;
    let origin_us = (i128::from(origin) * i128::from(source.clock.time_base_numerator) * 1_000_000)
        .div_euclid(i128::from(source.clock.time_base_denominator));
    let input_seek_start_us =
        origin_us + i128::from(segment.range.start_us.0.saturating_sub(2_000_000));
    let audio = match segment.audio {
        AudioSelection::Disabled => None,
        AudioSelection::Selected {
            stream_index,
            volume_percent,
            muted,
        } => {
            let a = origin_us + i128::from(segment.range.start_us.0);
            let b = origin_us + i128::from(segment.range.end_us.0);
            if a.unsigned_abs() > 86_400_000_000 || b.unsigned_abs() > 86_400_000_000 {
                return Err(ApiError::new(
                    ErrorCode::Unsupported,
                    "Audio trim seconds exceed the verified expression range",
                ));
            }
            Some(ManualClipAudio {
                stream_index,
                volume_percent,
                muted,
                absolute_start_us: a,
                absolute_end_us: b,
            })
        }
    };
    Ok(ManualClipPlan {
        document_sha256: digest,
        source_media_id: source.media_id.clone(),
        source_sha256: source.sha256.clone(),
        source_observation_sha256: source.clock.observation_sha256.clone(),
        segment_id: segment.segment_id.clone(),
        video_stream_index: segment.video_stream_index,
        start_pts: start,
        end_pts: end,
        crop_sections,
        width: document.preset.width,
        height: document.preset.height,
        fps_numerator: document.preset.frame_rate.numerator,
        fps_denominator: document.preset.frame_rate.denominator,
        duration_us: segment.range.end_us.0 - segment.range.start_us.0,
        input_seek_start_us,
        audio,
    })
}

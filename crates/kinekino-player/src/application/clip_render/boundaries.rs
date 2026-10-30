//! Integer frame membership policy. Values alone confer no decoder/source authority.
use crate::{
    application::api::{ApiError, ErrorCode},
    domain::{
        api_values::DecimalU64,
        clip_document::{ClipSegment, ClipSourcePin, SourceRange, SourceTrackKind},
        media_timeline::Rounding,
    },
};
use serde::Serialize;
pub const MAX_CLIP_BOUNDARY_FRAMES: usize = 8192;
#[derive(Debug, Serialize)]
pub struct ClipCropSwitch {
    pub source_us: DecimalU64,
    pub threshold_pts: String,
    /// Zero-based first decoded frame using this crop; equal offsets favor later keys.
    pub frame_offset: DecimalU64,
}
#[derive(Debug, Serialize)]
pub struct ClipFrameBoundaries {
    pub segment_id: String,
    pub stream_index: u32,
    pub requested_range: SourceRange,
    pub start_threshold_pts: String,
    pub end_threshold_pts: String,
    pub first_included_pts: String,
    pub last_included_pts: String,
    pub first_excluded_pts: String,
    pub included_frames: DecimalU64,
    pub crop_switches: Vec<ClipCropSwitch>,
}
pub fn frame_thresholds(
    source: &ClipSourcePin,
    segment: &ClipSegment,
) -> Result<(i64, i64), ApiError> {
    if segment.media_id != source.media_id
        || segment.range.start_us.0 >= segment.range.end_us.0
        || segment.range.end_us.0 > source.duration_us.0
        || !source.tracks.iter().any(|track| {
            track.stream_index == segment.video_stream_index && track.kind == SourceTrackKind::Video
        })
    {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Invalid clip source, video selection or interval",
        ));
    }
    let timeline = source
        .clock
        .transform()
        .map_err(|message| ApiError::new(ErrorCode::InvalidArgument, message))?;
    let start = timeline
        .us_to_pts(segment.range.start_us.0, Rounding::Up)
        .map_err(|_| ApiError::new(ErrorCode::NeedsInput, "Clip start timestamp overflow"))?
        .value;
    let end = timeline
        .us_to_pts(segment.range.end_us.0, Rounding::Up)
        .map_err(|_| ApiError::new(ErrorCode::NeedsInput, "Clip end timestamp overflow"))?
        .value;
    Ok((start, end))
}
pub fn evaluate_frame_boundaries(
    source: &ClipSourcePin,
    segment: &ClipSegment,
    decoded_pts: &[i64],
) -> Result<ClipFrameBoundaries, ApiError> {
    let (start, end) = frame_thresholds(source, segment)?;
    if decoded_pts.len() > MAX_CLIP_BOUNDARY_FRAMES {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Decoded clip frame window exceeds its bound",
        ));
    }
    if decoded_pts.len() < 2
        || decoded_pts.windows(2).any(|pair| pair[0] >= pair[1])
        || decoded_pts[0] > start
    {
        return Err(ApiError::new(
            ErrorCode::NeedsInput,
            "Frame window is empty, unordered or does not cover the requested start",
        ));
    }
    let first = decoded_pts
        .iter()
        .position(|pts| *pts >= start)
        .ok_or_else(|| {
            ApiError::new(
                ErrorCode::NeedsInput,
                "No decoded frame reaches the clip start",
            )
        })?;
    let excluded = decoded_pts
        .iter()
        .position(|pts| *pts >= end)
        .ok_or_else(|| {
            ApiError::new(
                ErrorCode::NeedsInput,
                "No observed excluded-end frame; EOF requires a separate verified path",
            )
        })?;
    if first >= excluded {
        return Err(ApiError::new(
            ErrorCode::NeedsInput,
            "Requested clip contains no decoded video frame",
        ));
    }
    let timeline = source
        .clock
        .transform()
        .map_err(|_| ApiError::new(ErrorCode::NeedsInput, "Invalid crop clock"))?;
    let mut crop_switches = Vec::with_capacity(segment.crop_keys.len());
    for key in &segment.crop_keys {
        let threshold = timeline
            .us_to_pts(key.source_us.0, Rounding::Up)
            .map_err(|_| ApiError::new(ErrorCode::NeedsInput, "Crop timestamp overflow"))?
            .value;
        let count = decoded_pts[first..excluded].partition_point(|pts| *pts < threshold);
        crop_switches.push(ClipCropSwitch {
            source_us: key.source_us,
            threshold_pts: threshold.to_string(),
            frame_offset: DecimalU64(count as u64),
        });
    }
    Ok(ClipFrameBoundaries {
        segment_id: segment.segment_id.clone(),
        stream_index: segment.video_stream_index,
        requested_range: segment.range,
        start_threshold_pts: start.to_string(),
        end_threshold_pts: end.to_string(),
        first_included_pts: decoded_pts[first].to_string(),
        last_included_pts: decoded_pts[excluded - 1].to_string(),
        first_excluded_pts: decoded_pts[excluded].to_string(),
        included_frames: DecimalU64((excluded - first) as u64),
        crop_switches,
    })
}
#[cfg(test)]
mod tests;

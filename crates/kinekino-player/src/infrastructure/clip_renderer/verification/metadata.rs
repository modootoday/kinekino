use crate::{
    application::{
        api::{ApiError, ErrorCode},
        clip_render::ManualClipPlan,
    },
    domain::api_values::DecimalU64,
};
use serde::Serialize;
use serde_json::Value;
#[derive(Debug, Serialize)]
pub struct ClipOutputMetadata {
    pub width: u32,
    pub height: u32,
    pub video_frames: DecimalU64,
    pub requested_duration_us: DecimalU64,
    pub video_duration_us: DecimalU64,
    pub audio_duration_us: Option<DecimalU64>,
    pub container_duration_us: DecimalU64,
    pub movie_timescale: u32,
    pub movie_duration_ticks: DecimalU64,
    pub declared_color_tags_match: bool,
}
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::NeedsInput,
        "Decoded clip output metadata differs from the render plan",
    )
}
fn ratio(value: &Value) -> Result<(u32, u32), ApiError> {
    let (a, b) = value
        .as_str()
        .and_then(|s| s.split_once('/'))
        .ok_or_else(invalid)?;
    let a = a.parse::<u32>().map_err(|_| invalid())?;
    let b = b.parse::<u32>().map_err(|_| invalid())?;
    if a == 0 || b == 0 {
        return Err(invalid());
    }
    Ok((a, b))
}
fn micros(value: &Value) -> Result<u64, ApiError> {
    let text = value.as_str().ok_or_else(invalid)?;
    let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
    if whole.is_empty()
        || !whole.bytes().all(|b| b.is_ascii_digit())
        || fraction.len() > 6
        || !fraction.bytes().all(|b| b.is_ascii_digit())
    {
        return Err(invalid());
    }
    let whole = whole.parse::<u64>().map_err(|_| invalid())?;
    let sub = if fraction.is_empty() {
        0
    } else {
        fraction.parse::<u64>().map_err(|_| invalid())?
    };
    whole
        .checked_mul(1_000_000)
        .and_then(|v| v.checked_add(sub * 10u64.pow(6 - fraction.len() as u32)))
        .ok_or_else(invalid)
}
fn stream_duration(stream: &Value) -> Result<(u64, u64), ApiError> {
    if stream["start_pts"].as_i64() != Some(0) {
        return Err(invalid());
    }
    let (n, d) = ratio(&stream["time_base"])?;
    let ticks = stream["duration_ts"]
        .as_u64()
        .filter(|ticks| *ticks > 0)
        .ok_or_else(invalid)?;
    let duration = u128::from(ticks) * u128::from(n) * 1_000_000 / u128::from(d);
    let quantum = (u64::from(n) * 1_000_000).div_ceil(u64::from(d));
    Ok((u64::try_from(duration).map_err(|_| invalid())?, quantum))
}
pub(super) fn validate(
    plan: &ManualClipPlan,
    json: &Value,
    scale: u32,
    movie_ticks: u64,
) -> Result<ClipOutputMetadata, ApiError> {
    if plan.fps_numerator == 0 || plan.fps_denominator == 0 || scale == 0 || plan.duration_us == 0 {
        return Err(invalid());
    }
    let streams = json["streams"].as_array().ok_or_else(invalid)?;
    if streams.len() != 1 + usize::from(plan.audio.is_some()) {
        return Err(invalid());
    }
    let video = streams
        .iter()
        .find(|s| s["codec_type"] == "video")
        .ok_or_else(invalid)?;
    let (n, d) = ratio(&video["r_frame_rate"])?;
    if video["codec_name"] != "h264"
        || video["width"].as_u64() != Some(u64::from(plan.width))
        || video["height"].as_u64() != Some(u64::from(plan.height))
        || u64::from(n) * u64::from(plan.fps_denominator)
            != u64::from(d) * u64::from(plan.fps_numerator)
        || video["pix_fmt"] != "yuv420p"
        || video["sample_aspect_ratio"] != "1:1"
        || video["color_range"] != "tv"
        || video["color_space"] != "bt709"
        || video["color_transfer"] != "bt709"
        || video["color_primaries"] != "bt709"
    {
        return Err(invalid());
    }
    let frames = video["nb_read_frames"]
        .as_str()
        .and_then(|n| n.parse::<u64>().ok())
        .ok_or_else(invalid)?;
    let expected = (u128::from(plan.duration_us) * u128::from(plan.fps_numerator))
        .div_ceil(u128::from(plan.fps_denominator) * 1_000_000);
    if u128::from(frames) != expected {
        return Err(invalid());
    }
    let video_end = (u128::from(frames) * u128::from(plan.fps_denominator) * 1_000_000
        / u128::from(plan.fps_numerator)) as u64;
    let (video_us, video_quantum) = stream_duration(video)?;
    if video_us.abs_diff(video_end) > video_quantum {
        return Err(invalid());
    }
    let audio_us = if plan.audio.is_some() {
        let audio = streams
            .iter()
            .find(|s| s["codec_type"] == "audio")
            .ok_or_else(invalid)?;
        let rate = audio["sample_rate"]
            .as_str()
            .and_then(|s| s.parse::<u32>().ok())
            .filter(|r| *r > 0 && *r <= 192000)
            .ok_or_else(invalid)?;
        if audio["codec_name"] != "aac"
            || audio["channels"]
                .as_u64()
                .is_none_or(|channels| channels == 0 || channels > 8)
        {
            return Err(invalid());
        }
        let (duration, _) = stream_duration(audio)?;
        if duration.abs_diff(plan.duration_us) > 1_000_000u64.div_ceil(u64::from(rate)) {
            return Err(invalid());
        }
        Some(duration)
    } else {
        None
    };
    if micros(&json["format"]["start_time"])? != 0 {
        return Err(invalid());
    }
    let container_us = micros(&json["format"]["duration"])?;
    let movie_us = u64::try_from(u128::from(movie_ticks) * 1_000_000 / u128::from(scale))
        .map_err(|_| invalid())?;
    let quantum = 1_000_000u64.div_ceil(u64::from(scale));
    let expected_end = video_end.max(audio_us.unwrap_or(0));
    if container_us.abs_diff(movie_us) > quantum
        || container_us.abs_diff(expected_end) > quantum + 1
    {
        return Err(invalid());
    }
    Ok(ClipOutputMetadata {
        width: plan.width,
        height: plan.height,
        video_frames: DecimalU64(frames),
        requested_duration_us: DecimalU64(plan.duration_us),
        video_duration_us: DecimalU64(video_us),
        audio_duration_us: audio_us.map(DecimalU64),
        container_duration_us: DecimalU64(container_us),
        movie_timescale: scale,
        movie_duration_ticks: DecimalU64(movie_ticks),
        declared_color_tags_match: true,
    })
}

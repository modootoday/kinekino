use super::raw::RetainedRaw;
use crate::{
    application::api::{ApiError, ErrorCode},
    domain::{
        media_timeline::TimeBase,
        review_source::timeline_evidence::{MAX_OBSERVED_FRAMES, VideoFrame},
    },
};
use serde::Deserialize;
use std::{collections::BTreeMap, path::Path};
/// Raw ffprobe video-frame window; no inferred timebase, origin or source binding.
pub struct RetainedVideoFrames {
    cadence: Option<crate::domain::review_source::video_cadence::ObservedCadence>,
    frames: Vec<VideoFrame>,
    raw: RetainedRaw,
    time_base: TimeBase,
    stream_index: u32,
}
impl RetainedVideoFrames {
    pub fn open(path: &Path, expected_sha256: &str) -> Result<Self, ApiError> {
        let mut raw = RetainedRaw::open(path, expected_sha256)?;
        let (frames, time_base, stream_index) = parse(&raw.read()?)?;
        Ok(Self {
            cadence: None,
            frames,
            raw,
            time_base,
            stream_index,
        })
    }
    pub fn frames(&self) -> &[VideoFrame] {
        &self.frames
    }
    pub fn open_live(
        path: &Path,
        expected_sha256: &str,
        class: &crate::application::review_playback::ReviewContentClass,
    ) -> Result<Self, ApiError> {
        let mut raw = RetainedRaw::open(path, expected_sha256)?;
        let (frames, time_base, stream_index, cadence) = parse_live(&raw.read()?, class)?;
        Ok(Self {
            frames,
            raw,
            time_base,
            stream_index,
            cadence,
        })
    }
    pub fn cadence(&self) -> Option<crate::domain::review_source::video_cadence::ObservedCadence> {
        self.cadence
    }
    pub fn sha256(&self) -> &str {
        self.raw.sha256()
    }
    pub fn time_base(&self) -> TimeBase {
        self.time_base
    }
    pub fn stream_index(&self) -> u32 {
        self.stream_index
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Window {
    frames: Vec<Frame>,
    streams: Vec<Stream>,
    #[serde(default)]
    programs: Vec<Empty>,
    #[serde(default)]
    stream_groups: Vec<Empty>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Empty {}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Stream {
    #[serde(default)]
    avg_frame_rate: Option<String>,
    #[serde(default)]
    r_frame_rate: Option<String>,
    index: u32,
    time_base: String,
    #[serde(default)]
    disposition: BTreeMap<String, u8>,
    #[serde(default)]
    tags: BTreeMap<String, String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Frame {
    pts: i64,
    best_effort_timestamp: i64,
    duration: u64,
    key_frame: u8,
}
pub(super) fn parse(raw: &str) -> Result<(Vec<VideoFrame>, TimeBase, u32), ApiError> {
    let (frames, base, index, _) = parse_profile(raw, false)?;
    Ok((frames, base, index))
}
type ParsedLive = (
    Vec<VideoFrame>,
    TimeBase,
    u32,
    Option<crate::domain::review_source::video_cadence::ObservedCadence>,
);
pub(super) fn parse_live(
    raw: &str,
    class: &crate::application::review_playback::ReviewContentClass,
) -> Result<ParsedLive, ApiError> {
    parse_profile(
        raw,
        matches!(
            class,
            crate::application::review_playback::ReviewContentClass::MatroskaHeader
        ),
    )
}
fn parse_profile(raw: &str, quantized: bool) -> Result<ParsedLive, ApiError> {
    let window: Window =
        serde_json::from_str(raw).map_err(|_| invalid("Invalid video observation fields"))?;
    if window.frames.is_empty() || window.frames.len() > MAX_OBSERVED_FRAMES {
        return Err(invalid("Video observation frame limit"));
    }
    if window.streams.len() != 1 || !window.programs.is_empty() || !window.stream_groups.is_empty()
    {
        return Err(invalid("Ambiguous video observation streams"));
    }
    let stream = &window.streams[0];
    if stream.disposition.values().any(|value| *value > 1)
        || stream
            .tags
            .iter()
            .any(|(key, value)| key.len() > 128 || value.len() > 4096)
    {
        return Err(invalid("Invalid video stream metadata"));
    }
    let time_base = TimeBase::parse(&stream.time_base)
        .map_err(|_| invalid("Invalid video observation timebase"))?;
    let cadence = if quantized {
        let avg = TimeBase::parse(
            stream
                .avg_frame_rate
                .as_deref()
                .ok_or_else(|| invalid("Missing observed average frame rate"))?,
        )
        .map_err(|_| invalid("Invalid observed average frame rate"))?;
        let nominal = TimeBase::parse(
            stream
                .r_frame_rate
                .as_deref()
                .ok_or_else(|| invalid("Missing observed base frame rate"))?,
        )
        .map_err(|_| invalid("Invalid observed base frame rate"))?;
        if avg != nominal {
            return Err(invalid("Observed frame rate candidates disagree"));
        }
        let (n, d) = avg.ratio();
        Some(
            crate::domain::review_source::video_cadence::ObservedCadence {
                rate_numerator: u64::try_from(n).map_err(|_| invalid("Rate overflow"))?,
                rate_denominator: u64::try_from(d).map_err(|_| invalid("Rate overflow"))?,
            },
        )
    } else {
        if stream.avg_frame_rate.is_some() || stream.r_frame_rate.is_some() {
            return Err(invalid("Legacy exact video contract rejects rate hints"));
        }
        None
    };
    let mut result = Vec::with_capacity(window.frames.len());
    for frame in window.frames {
        if frame.key_frame > 1 || frame.duration == 0 || frame.pts != frame.best_effort_timestamp {
            return Err(invalid("Video PTS ambiguity or discontinuity"));
        }
        result.push(VideoFrame {
            pts: frame.pts,
            duration: frame.duration,
        });
    }
    crate::domain::review_source::video_cadence::validate(&result, time_base, cadence).map_err(
        |_| invalid("Video PTS does not satisfy the selected observation clock contract"),
    )?;
    Ok((result, time_base, stream.index, cadence))
}
fn invalid(message: &str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
#[cfg(test)]
mod tests;

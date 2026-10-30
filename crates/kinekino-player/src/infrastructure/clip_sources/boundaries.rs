use super::*;
use crate::{
    application::clip_render::{
        ClipFrameBoundaries, MAX_CLIP_BOUNDARY_FRAMES, evaluate_frame_boundaries,
    },
    domain::clip_document::{ClipSegment, SourceClock},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
#[derive(Serialize)]
pub struct ClipBoundaryObservation {
    pub contract: String,
    pub source_sha256: String,
    pub source_observation_sha256: String,
    pub probe_sha256: String,
    pub raw_frame_window_sha256: String,
    pub interval: String,
    pub boundaries: ClipFrameBoundaries,
}
/// The borrowed original source/tool owner must survive until the decoder is closed.
pub struct NativeClipBoundaryLease<'a> {
    source: &'a NativeClipSourceLease,
    observation: ClipBoundaryObservation,
    raw: Vec<u8>,
    decoded_pts: Vec<i64>,
}
/// Original decoded data, with an opaque binding to its original source lease.
pub(crate) struct RetainedClipBoundary {
    source_owner: Arc<()>,
    observation: ClipBoundaryObservation,
    _raw: Vec<u8>,
    _decoded_pts: Vec<i64>,
}
impl RetainedClipBoundary {
    pub(crate) fn belongs_to(&self, source: &NativeClipSourceLease) -> bool {
        Arc::ptr_eq(&self.source_owner, &source.custody)
    }
    pub(crate) fn observation(&self) -> &ClipBoundaryObservation {
        &self.observation
    }
}
impl NativeClipBoundaryLease<'_> {
    pub(crate) fn into_retained(self) -> RetainedClipBoundary {
        RetainedClipBoundary {
            source_owner: self.source.custody.clone(),
            observation: self.observation,
            _raw: self.raw,
            _decoded_pts: self.decoded_pts,
        }
    }
    pub fn source(&self) -> &NativeClipSourceLease {
        self.source
    }
    pub fn observation(&self) -> &ClipBoundaryObservation {
        &self.observation
    }
    pub fn raw_frames(&self) -> &[u8] {
        &self.raw
    }
    pub fn subtitle_frames<D, S>(
        &self,
        segment: &ClipSegment,
        subtitles: &crate::application::clip_render::subtitles::PreparedClipSubtitles<D, S>,
        control: &dyn MutationControl,
    ) -> Result<
        Vec<crate::application::clip_render::subtitles::frames::ClipSubtitleFrameSpan>,
        ApiError,
    >
    where
        D: crate::application::clip_render::subtitles::ClipSubtitleDocumentLease,
        S: crate::application::clip_render::subtitles::ClipSubtitleStyleLease,
    {
        if segment.range.start_us != self.observation.boundaries.requested_range.start_us
            || segment.range.end_us != self.observation.boundaries.requested_range.end_us
            || segment.segment_id != self.observation.boundaries.segment_id
            || segment.video_stream_index != self.observation.boundaries.stream_index
        {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Decoded window belongs to a different segment",
            ));
        }
        crate::application::clip_render::subtitles::frames::map_subtitle_frames(
            &self.source.facts.source,
            segment,
            &self.decoded_pts,
            subtitles,
            control,
        )
    }
}
fn interval(clock: &SourceClock, segment: &ClipSegment) -> Result<String, ApiError> {
    fn absolute(clock: &SourceClock, relative: u64) -> Result<i128, ApiError> {
        clock
            .transform()
            .map_err(|_| ApiError::new(ErrorCode::InvalidArgument, "Invalid clip clock"))?;
        let origin = clock
            .origin_pts
            .parse::<i64>()
            .map_err(|_| ApiError::new(ErrorCode::InvalidArgument, "Invalid clip origin"))?;
        i128::from(origin)
            .checked_mul(i128::from(clock.time_base_numerator))
            .and_then(|value| value.checked_mul(1_000_000))
            .map(|value| value.div_euclid(i128::from(clock.time_base_denominator)))
            .and_then(|value| value.checked_add(i128::from(relative)))
            .ok_or_else(|| {
                ApiError::new(
                    ErrorCode::NeedsInput,
                    "Absolute clip probe interval overflow",
                )
            })
    }
    fn decimal(value: i128) -> String {
        let magnitude = value.unsigned_abs();
        format!(
            "{}{}.{:06}",
            if value < 0 { "-" } else { "" },
            magnitude / 1_000_000,
            magnitude % 1_000_000
        )
    }
    if segment
        .range
        .end_us
        .0
        .checked_sub(segment.range.start_us.0)
        .is_none_or(|duration| duration == 0 || duration > 60_000_000)
    {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "One decoded clip window must be positive and at most 60 seconds",
        ));
    }
    let start = absolute(clock, segment.range.start_us.0.saturating_sub(2_000_000))?;
    let end = absolute(
        clock,
        segment
            .range
            .end_us
            .0
            .checked_add(2_000_000)
            .ok_or_else(|| {
                ApiError::new(ErrorCode::NeedsInput, "Clip observation margin overflow")
            })?,
    )?;
    Ok(format!("{}%{}", decimal(start), decimal(end)))
}
#[derive(Deserialize)]
struct Window {
    frames: Vec<Frame>,
    streams: Vec<Stream>,
}
#[derive(Deserialize)]
struct Frame {
    stream_index: u32,
    pts: Option<i64>,
    best_effort_timestamp: Option<i64>,
}
#[derive(Deserialize)]
struct Stream {
    index: u32,
    time_base: String,
}
fn parse(
    raw: &[u8],
    source: &ClipSourcePin,
    segment: &ClipSegment,
) -> Result<(ClipFrameBoundaries, Vec<i64>), ApiError> {
    let window: Window = serde_json::from_slice(raw)
        .map_err(|_| ApiError::new(ErrorCode::NeedsInput, "Invalid decoded frame window JSON"))?;
    if window.frames.len() > MAX_CLIP_BOUNDARY_FRAMES {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Too many decoded clip frames",
        ));
    }
    if window.streams.len() != 1
        || window.streams[0].index != segment.video_stream_index
        || window.streams[0].time_base
            != format!(
                "{}/{}",
                source.clock.time_base_numerator, source.clock.time_base_denominator
            )
    {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Decoded frame stream or time base differs from the source pin",
        ));
    }
    let mut pts = Vec::with_capacity(window.frames.len());
    for frame in window.frames {
        let tick = frame.pts.ok_or_else(|| {
            ApiError::new(
                ErrorCode::NeedsInput,
                "Decoded source frame lacks original integer PTS",
            )
        })?;
        if frame.stream_index != segment.video_stream_index
            || frame.best_effort_timestamp.is_some_and(|best| best != tick)
        {
            return Err(ApiError::new(
                ErrorCode::NeedsInput,
                "Decoded frame stream or presentation timestamp is inconsistent",
            ));
        }
        pts.push(tick);
    }
    Ok((evaluate_frame_boundaries(source, segment, &pts)?, pts))
}
impl NativeClipSourceLease {
    pub fn observe_segment<'a>(
        &'a self,
        segment: &ClipSegment,
        timeout: Duration,
        control: &dyn MutationControl,
    ) -> Result<NativeClipBoundaryLease<'a>, ApiError> {
        if timeout.is_zero() || timeout > Duration::from_secs(120) {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Bounded decoded-frame timeout required",
            ));
        }
        crate::application::clip_render::frame_thresholds(&self.facts.source, segment)?;
        if segment.video_stream_index != self.facts.reference_stream_index {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Initial boundary decoder requires the admitted reference video stream",
            ));
        }
        let interval = interval(&self.facts.source.clock, segment)?;
        let deadline = Instant::now() + timeout;
        let control = ProbeControl {
            outer: control,
            deadline,
        };
        control.checkpoint()?;
        let mut command = Command::new(&self._tool.path);
        command.args([
            "-v",
            "error",
            "-threads",
            "1",
            "-max_alloc",
            "268435456",
            "-probesize",
            "16777216",
            "-analyzeduration",
            "10000000",
            "-protocol_whitelist",
            "file",
            "-f",
            self.container.demuxer(),
        ]);
        if matches!(self.container, ClipContainer::Mp4) {
            command.args(["-enable_drefs", "0"]);
        }
        command
            .args([
                "-select_streams",
                &segment.video_stream_index.to_string(),
                "-read_intervals",
                &interval,
                "-show_frames",
                "-show_streams",
                "-show_entries",
                "frame=stream_index,pts,best_effort_timestamp:stream=index,time_base",
                "-of",
                "json",
            ])
            .arg(&self.path);
        let output =
            process::run_with_recovery(&mut command, deadline, 2_097_152, Some(&control), None)?;
        control.checkpoint()?;
        let (boundaries, decoded_pts) = parse(&output.stdout, &self.facts.source, segment)?;
        control.checkpoint()?;
        let observation = ClipBoundaryObservation {
            contract: "clip-frame-boundary/1".into(),
            source_sha256: self.facts.source.sha256.clone(),
            source_observation_sha256: self.facts.source.clock.observation_sha256.clone(),
            probe_sha256: self._tool.sha256.clone(),
            raw_frame_window_sha256: format!("{:x}", Sha256::digest(&output.stdout)),
            interval,
            boundaries,
        };
        Ok(NativeClipBoundaryLease {
            source: self,
            observation,
            raw: output.stdout,
            decoded_pts,
        })
    }
}
#[cfg(test)]
mod tests;

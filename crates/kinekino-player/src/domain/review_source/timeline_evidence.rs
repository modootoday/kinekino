//! Observation consistency, not evidence authenticity or a caller-constructible Verified status.
use super::{EvidenceReference, MediaReference, ReviewCase, valid_review_object_id};
use crate::domain::{api_values::DecimalU64, identifiers::valid_sha256, media_timeline::TimeBase};
pub const MAX_OBSERVED_FRAMES: usize = 50_000;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioFrame {
    pub sequence: u64,
    pub pts: i64,
    pub samples: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VideoFrame {
    pub pts: i64,
    pub duration: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptureObservation {
    pub requested_source_us: u64,
    pub actual_pts: i64,
    pub time_base: TimeBase,
    pub source_sha256: String,
    pub png_sha256: String,
    pub engine_sha256: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedOrigin {
    pub pts: i64,
    pub time_base: TimeBase,
    pub source_sha256: String,
    pub observation_sha256: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PcmObservations {
    pub source_sha256: String,
    pub original_data_sha256: String,
    pub replay_data_sha256: String,
    pub replay_engine_sha256: String,
    pub sample_rate: u32,
    pub frames: Vec<AudioFrame>,
    pub clip_start_sample: u64,
    pub clip_samples: u64,
    pub original_slice_sha256: String,
    pub clip_data_sha256: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimelineObservations {
    pub video_cadence: Option<super::video_cadence::ObservedCadence>,
    pub case: ReviewCase,
    pub media: MediaReference,
    pub reference: EvidenceReference,
    pub mapping_revision: DecimalU64,
    pub time_base: TimeBase,
    pub origin: DecodedOrigin,
    pub video_frames: Vec<VideoFrame>,
    pub pcm: PcmObservations,
    pub start_capture: CaptureObservation,
    pub end_capture: CaptureObservation,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MappingWindow {
    pub source_start_us: u64,
    pub source_end_us: u64,
    pub start_frame_pts: i64,
    pub end_frame_pts: i64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvidenceError {
    InvalidIdentity,
    ContentMismatch,
    InvalidClock,
    InvalidVideoWindow,
    InvalidCapture,
    ResourceLimit,
    Overflow,
}
fn sample_time(origin: i64, sample: u64, rate: u32) -> Result<u64, EvidenceError> {
    let ticks = i128::from(origin) + i128::from(sample);
    let ticks = u128::try_from(ticks).map_err(|_| EvidenceError::InvalidClock)?;
    ticks
        .checked_mul(1_000_000)
        .map(|v| v.div_ceil(u128::from(rate)))
        .and_then(|v| u64::try_from(v).ok())
        .ok_or(EvidenceError::Overflow)
}
/// Consistency of source observations only; no case/asset custody or Verified authority.
pub struct TimelineContent<'a> {
    pub video_cadence: Option<super::video_cadence::ObservedCadence>,
    pub source_sha256: &'a str,
    pub time_base: TimeBase,
    pub origin: &'a DecodedOrigin,
    pub video_frames: &'a [VideoFrame],
    pub pcm: &'a PcmObservations,
    pub start_capture: &'a CaptureObservation,
    pub end_capture: &'a CaptureObservation,
}
impl TimelineObservations {
    pub fn validate(&self) -> Result<MappingWindow, EvidenceError> {
        if self.case.validate().is_err()
            || self.media.validate().is_err()
            || !valid_review_object_id(&self.reference.asset_id)
            || !valid_sha256(&self.reference.sha256)
            || self.mapping_revision.0 == 0
        {
            return Err(EvidenceError::InvalidIdentity);
        }
        self.content().validate()
    }
    pub fn content(&self) -> TimelineContent<'_> {
        TimelineContent {
            video_cadence: self.video_cadence,
            source_sha256: &self.media.sha256,
            time_base: self.time_base,
            origin: &self.origin,
            video_frames: &self.video_frames,
            pcm: &self.pcm,
            start_capture: &self.start_capture,
            end_capture: &self.end_capture,
        }
    }
}
impl TimelineContent<'_> {
    pub fn validate(&self) -> Result<MappingWindow, EvidenceError> {
        if !valid_sha256(self.source_sha256) {
            return Err(EvidenceError::InvalidIdentity);
        }
        let p = self.pcm;
        for hash in [
            &p.source_sha256,
            &p.original_data_sha256,
            &p.replay_data_sha256,
            &p.replay_engine_sha256,
            &p.original_slice_sha256,
            &p.clip_data_sha256,
            &self.origin.source_sha256,
            &self.origin.observation_sha256,
        ] {
            if !valid_sha256(hash) {
                return Err(EvidenceError::InvalidIdentity);
            }
        }
        if p.source_sha256 != self.source_sha256
            || self.origin.source_sha256 != self.source_sha256
            || p.original_data_sha256 != p.replay_data_sha256
            || p.original_slice_sha256 != p.clip_data_sha256
        {
            return Err(EvidenceError::ContentMismatch);
        }
        if self.video_frames.len() > MAX_OBSERVED_FRAMES || p.frames.len() > MAX_OBSERVED_FRAMES {
            return Err(EvidenceError::ResourceLimit);
        }
        if p.frames.is_empty() || p.sample_rate == 0 || p.clip_samples == 0 {
            return Err(EvidenceError::InvalidClock);
        }
        let first = p.frames[0].pts;
        let mut end = i128::from(first);
        let mut samples = 0u64;
        for (index, frame) in p.frames.iter().enumerate() {
            if frame.sequence != index as u64 || frame.samples == 0 || i128::from(frame.pts) != end
            {
                return Err(EvidenceError::InvalidClock);
            }
            end += i128::from(frame.samples);
            samples = samples
                .checked_add(u64::from(frame.samples))
                .ok_or(EvidenceError::Overflow)?;
        }
        let clip_end = p
            .clip_start_sample
            .checked_add(p.clip_samples)
            .ok_or(EvidenceError::Overflow)?;
        if clip_end > samples {
            return Err(EvidenceError::InvalidClock);
        }
        if self.video_frames.is_empty() || self.origin.time_base != self.time_base {
            return Err(EvidenceError::InvalidVideoWindow);
        }
        super::video_cadence::validate(self.video_frames, self.time_base, self.video_cadence)?;
        if self.origin.pts > self.video_frames[0].pts {
            return Err(EvidenceError::InvalidVideoWindow);
        }
        let source_start_us = sample_time(first, p.clip_start_sample, p.sample_rate)?;
        let source_end_us = sample_time(first, clip_end, p.sample_rate)?;
        if source_start_us >= source_end_us {
            return Err(EvidenceError::InvalidClock);
        }
        let start_frame_pts = self.capture(self.start_capture, source_start_us)?;
        let end_frame_pts = self.capture(self.end_capture, source_end_us)?;
        Ok(MappingWindow {
            source_start_us,
            source_end_us,
            start_frame_pts,
            end_frame_pts,
        })
    }
    fn capture(
        &self,
        capture: &CaptureObservation,
        expected_us: u64,
    ) -> Result<i64, EvidenceError> {
        if !valid_sha256(&capture.source_sha256)
            || !valid_sha256(&capture.png_sha256)
            || !valid_sha256(&capture.engine_sha256)
        {
            return Err(EvidenceError::InvalidIdentity);
        }
        if capture.source_sha256 != self.source_sha256 {
            return Err(EvidenceError::ContentMismatch);
        }
        if capture.time_base != self.time_base || capture.requested_source_us != expected_us {
            return Err(EvidenceError::InvalidCapture);
        }
        let threshold = self
            .time_base
            .ticks_at_or_after(expected_us)
            .map_err(|_| EvidenceError::Overflow)?;
        let threshold = i64::try_from(threshold).map_err(|_| EvidenceError::Overflow)?;
        if self.video_frames[0].pts > threshold {
            return Err(EvidenceError::InvalidVideoWindow);
        }
        let selected = self
            .video_frames
            .iter()
            .find(|frame| frame.pts >= threshold)
            .ok_or(EvidenceError::InvalidVideoWindow)?;
        if selected.pts != capture.actual_pts {
            return Err(EvidenceError::InvalidCapture);
        }
        Ok(selected.pts)
    }
}

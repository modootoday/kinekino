use crate::domain::{
    api_values::DecimalU64,
    media_timeline::{TimeBase, TimelineTransform},
};
use serde::{Deserialize, Serialize};
pub const CROP_SCALE: u32 = 1_000_000;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceClock {
    /// Canonical signed decimal ticks; negative origins must not be rounded through f64.
    pub origin_pts: String,
    pub time_base_numerator: u32,
    pub time_base_denominator: u32,
    pub observation_sha256: String,
}
impl SourceClock {
    /// A value transform only. Actual source/clock observation must be admitted separately.
    pub fn transform(&self) -> Result<TimelineTransform, &'static str> {
        let origin = self
            .origin_pts
            .parse::<i64>()
            .map_err(|_| "Invalid clip clock origin")?;
        if origin.to_string() != self.origin_pts {
            return Err("Noncanonical clip clock origin");
        }
        let base = TimeBase::new(
            self.time_base_numerator.into(),
            self.time_base_denominator.into(),
        )
        .map_err(|_| "Invalid clip clock time base")?;
        Ok(TimelineTransform::new(base, origin))
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceTrackKind {
    Video,
    Audio,
    Subtitle,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceTrack {
    pub stream_index: u32,
    pub kind: SourceTrackKind,
    pub codec_id: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRange {
    pub start_us: DecimalU64,
    pub end_us: DecimalU64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NormalizedCrop {
    pub left: u32,
    pub top: u32,
    pub width: u32,
    pub height: u32,
}
impl NormalizedCrop {
    pub fn validate(self) -> Result<(), &'static str> {
        if self.width == 0
            || self.height == 0
            || self
                .left
                .checked_add(self.width)
                .is_none_or(|end| end > CROP_SCALE)
            || self
                .top
                .checked_add(self.height)
                .is_none_or(|end| end > CROP_SCALE)
        {
            return Err("Invalid normalized clip crop");
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FitMode {
    Fit,
    Fill,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CropKey {
    pub source_us: DecimalU64,
    pub crop: NormalizedCrop,
    pub fit: FitMode,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum AudioSelection {
    Disabled,
    Selected {
        stream_index: u32,
        volume_percent: u8,
        muted: bool,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrameRate {
    pub numerator: u32,
    pub denominator: u32,
}
impl FrameRate {
    pub fn validate(self) -> Result<(), &'static str> {
        if self.numerator == 0 || self.denominator == 0 {
            return Err("Invalid clip frame rate");
        }
        let (mut a, mut b) = (self.numerator, self.denominator);
        while b != 0 {
            (a, b) = (b, a % b)
        }
        if a != 1 {
            return Err("Clip frame rate must be reduced");
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Container {
    Mp4,
    Webm,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VideoCodec {
    H264,
    Vp9,
    Av1,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioCodec {
    Aac,
    Opus,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputColor {
    SdrBt709,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CutMode {
    ExactReencode,
    FastKeyframe,
}

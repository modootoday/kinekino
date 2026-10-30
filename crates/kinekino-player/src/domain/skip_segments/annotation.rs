use crate::domain::{
    api_values::DecimalU64, identifiers::valid_id, playback_control::PlaybackControl,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SegmentKind {
    Opening,
    Ending,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SegmentOrigin {
    Manual,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SegmentTimebase {
    EnginePlayback,
}

/// Session-bound manual annotation; confirmation never implies model detection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManualSegment {
    pub id: String,
    pub revision: DecimalU64,
    pub kind: SegmentKind,
    pub in_us: DecimalU64,
    pub out_us: DecimalU64,
    pub origin: SegmentOrigin,
    pub timebase: SegmentTimebase,
    pub verified_revision: Option<DecimalU64>,
}
impl ManualSegment {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !valid_id(&self.id) || self.revision.0 == 0 || self.in_us.0 >= self.out_us.0 {
            return Err("Invalid manual segment identity, revision or interval");
        }
        if self
            .verified_revision
            .is_some_and(|revision| revision != self.revision)
        {
            return Err("Segment verification must name the current annotation revision");
        }
        PlaybackControl::Seek {
            position_us: self.out_us,
        }
        .validate()
    }
    pub fn overlaps(&self, other: &Self) -> bool {
        self.in_us < other.out_us && other.in_us < self.out_us
    }
}

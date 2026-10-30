use super::{ManualSegment, SegmentKind};
use crate::domain::api_values::DecimalU64;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkipMode {
    Off,
    #[default]
    Manual,
    Auto,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkipPolicy {
    pub opening: SkipMode,
    pub ending: SkipMode,
}
impl SkipPolicy {
    pub fn mode(&self, kind: SegmentKind) -> SkipMode {
        match kind {
            SegmentKind::Opening => self.opening,
            SegmentKind::Ending => self.ending,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Intent {
    Manual,
    Automatic,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Seekability {
    Unknown,
    Yes,
    No,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockReason {
    InvalidAnnotation,
    Unverified,
    Overlap,
    Off,
    ManualOnly,
    ObservationUnknown,
    SeekUnknown,
    NotSeekable,
    DurationUnknown,
    PositionUnknown,
    OutsideDuration,
    PausedOrUnknown,
    OutsideSegment,
    Overridden,
    AlreadyAttempted,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Decision {
    Blocked { reason: BlockReason },
    Notice { remaining_us: DecimalU64 },
    Eligible { target_us: DecimalU64 },
}
#[derive(Debug, Clone, Default)]
pub struct PassState {
    overridden: bool,
    suppressed: Vec<String>,
    attempted: Vec<String>,
    last_position_us: Option<DecimalU64>,
}
impl PassState {
    pub fn observe_position(&mut self, position: Option<DecimalU64>) {
        if let (Some(previous), Some(current)) = (self.last_position_us, position)
            && current < previous
        {
            self.overridden = true;
        }
        if position.is_some() {
            self.last_position_us = position;
        }
    }
    pub fn override_pass(&mut self) {
        self.overridden = true;
    }
    pub fn suppress(&mut self, id: &str) {
        if !self.suppressed.iter().any(|item| item == id) {
            self.suppressed.push(id.into());
        }
    }
    pub fn mark_attempted(&mut self, id: &str) {
        if !self.attempted.iter().any(|item| item == id) {
            self.attempted.push(id.into());
        }
    }
    pub fn was_attempted(&self, id: &str) -> bool {
        self.attempted.iter().any(|item| item == id)
    }
}
pub struct Evaluation {
    pub position_us: Option<DecimalU64>,
    pub duration_us: Option<DecimalU64>,
    pub playing: bool,
    /// Set only by a trusted host observation, never inferred from publication age.
    pub observation_authorized: bool,
    pub seekability: Seekability,
}
pub fn evaluate(
    segment: &ManualSegment,
    all: &[ManualSegment],
    policy: &SkipPolicy,
    pass: &PassState,
    observation: &Evaluation,
    intent: Intent,
) -> Decision {
    let block = |reason| Decision::Blocked { reason };
    if segment.validate().is_err() {
        return block(BlockReason::InvalidAnnotation);
    }
    if segment.verified_revision != Some(segment.revision) {
        return block(BlockReason::Unverified);
    }
    if all
        .iter()
        .any(|other| other.id != segment.id && segment.overlaps(other))
    {
        return block(BlockReason::Overlap);
    }
    let mode = policy.mode(segment.kind);
    if mode == SkipMode::Off {
        return block(BlockReason::Off);
    }
    if intent == Intent::Automatic {
        if mode != SkipMode::Auto {
            return block(BlockReason::ManualOnly);
        }
        if pass.overridden || pass.suppressed.iter().any(|id| id == &segment.id) {
            return block(BlockReason::Overridden);
        }
        if pass.was_attempted(&segment.id) {
            return block(BlockReason::AlreadyAttempted);
        }
    }
    if !observation.observation_authorized {
        return block(BlockReason::ObservationUnknown);
    }
    match observation.seekability {
        Seekability::Unknown => return block(BlockReason::SeekUnknown),
        Seekability::No => return block(BlockReason::NotSeekable),
        Seekability::Yes => {}
    }
    let Some(duration) = observation.duration_us else {
        return block(BlockReason::DurationUnknown);
    };
    if segment.out_us > duration {
        return block(BlockReason::OutsideDuration);
    }
    let Some(position) = observation.position_us else {
        return block(BlockReason::PositionUnknown);
    };
    if intent == Intent::Automatic && !observation.playing {
        return block(BlockReason::PausedOrUnknown);
    }
    if position < segment.in_us {
        let Some(remaining) = segment.in_us.0.checked_sub(position.0) else {
            return block(BlockReason::InvalidAnnotation);
        };
        if intent == Intent::Automatic && remaining <= 3_000_000 {
            return Decision::Notice {
                remaining_us: DecimalU64(remaining),
            };
        }
        return block(BlockReason::OutsideSegment);
    }
    if position >= segment.out_us {
        return block(BlockReason::OutsideSegment);
    }
    Decision::Eligible {
        target_us: segment.out_us,
    }
}

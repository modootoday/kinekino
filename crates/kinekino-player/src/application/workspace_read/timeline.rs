use super::ReadState;
use crate::domain::api_values::DecimalU64;
use serde::Serialize;
#[derive(Debug, Clone, Serialize)]
pub struct TimelineTrack {
    pub track_id: String,
    pub label: String,
    pub segments: Vec<TimelineSegment>,
}
#[derive(Debug, Clone, Serialize)]
pub struct TimelineSegment {
    pub segment_id: String,
    pub in_us: DecimalU64,
    pub out_us: DecimalU64,
    pub label: String,
    pub state: ReadState,
}

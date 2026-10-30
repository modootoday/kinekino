use crate::{
    application::playback_control::ControlReceipt,
    domain::{
        api_values::DecimalU64,
        skip_segments::{Decision, Intent, ManualSegment, SegmentKind, SkipPolicy},
    },
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkipBinding {
    pub engine_instance_id: DecimalU64,
    pub resolution_artifact_id: String,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EditSegment {
    pub expected_revision: DecimalU64,
    pub segment_id: String,
    pub kind: SegmentKind,
    pub in_us: DecimalU64,
    pub out_us: DecimalU64,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VerifySegment {
    pub expected_revision: DecimalU64,
    pub segment_id: String,
    pub annotation_revision: DecimalU64,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SetSkipPolicy {
    pub expected_revision: DecimalU64,
    pub policy: SkipPolicy,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SkipRequest {
    pub expected_revision: DecimalU64,
    pub segment_id: String,
    pub intent: Intent,
    pub cancelled: bool,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum SkipResult {
    NotSubmitted {
        decision: Decision,
    },
    Cancelled {
        session_revision: DecimalU64,
        status: CancelStatus,
    },
    Admitted {
        session_revision: DecimalU64,
        receipt: ControlReceipt,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CancelStatus {
    BeforeAdmission,
    TooLateOrUnobserved,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SkipSnapshot {
    pub binding: SkipBinding,
    pub revision: DecimalU64,
    pub pass_id: DecimalU64,
    pub policy: SkipPolicy,
    pub segments: Vec<ManualSegment>,
    pub durable: bool,
    pub detection_supported: bool,
}

use crate::{
    application::skip_segments::{CancelStatus, SkipBinding, SkipResult, SkipSnapshot},
    domain::{
        api_values::DecimalU64,
        skip_segments::{Decision, Intent, SegmentKind, SkipPolicy},
    },
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SessionRef {
    pub skip_session_id: String,
    pub binding: SkipBinding,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SessionRequest<T> {
    pub session: SessionRef,
    pub input: T,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OpenSkipSession {
    pub binding: SkipBinding,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EditInput {
    pub segment_id: String,
    pub kind: SegmentKind,
    pub in_us: DecimalU64,
    pub out_us: DecimalU64,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VerifyInput {
    pub segment_id: String,
    pub annotation_revision: DecimalU64,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyInput {
    pub policy: SkipPolicy,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluateInput {
    pub segment_id: String,
    pub intent: Intent,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SkipInput {
    pub segment_id: String,
    pub intent: Intent,
    #[serde(default)]
    pub cancelled: bool,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum OverrideInput {
    Cancel { segment_id: String },
    ManualSeek,
    BeginNewPass,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SessionView {
    pub session: SessionRef,
    pub snapshot: SkipSnapshot,
    pub reused: bool,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluationResult {
    pub snapshot: SkipSnapshot,
    pub decision: Decision,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ActionResult {
    pub snapshot: SkipSnapshot,
    pub result: SkipResult,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OverrideResult {
    pub snapshot: SkipSnapshot,
    pub cancel_status: Option<CancelStatus>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClosedSkipSession {
    pub skip_session_id: String,
    pub closed: bool,
    pub playback_closed: bool,
}
#[derive(Debug, Clone, Copy)]
pub struct SkipHostCapability {
    pub sessions: bool,
    pub seek_submission: bool,
}

use crate::application::{playback_control::ControlReceipt, playback_session::v1};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SkipExportRequest {
    pub issuer_scope: v1::NativeIssuerScopeV1,
    pub session: SessionRef,
    pub input: SkipInput,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SkipExportOrigin {
    pub session: SessionRef,
    pub pass_id: crate::domain::api_values::DecimalU64,
    pub request_revision: crate::domain::api_values::DecimalU64,
    pub segment_id: String,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ListSkipExports {
    pub issuer_scope: v1::NativeIssuerScopeV1,
    pub origin: SkipExportOrigin,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReadSkipExport {
    pub issuer_scope: v1::NativeIssuerScopeV1,
    pub reference: v1::NativeResultRefV1,
    pub origin: SkipExportOrigin,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SkipExportSessionState {
    pub session: SessionRef,
    pub revision: crate::domain::api_values::DecimalU64,
    pub pass_id: crate::domain::api_values::DecimalU64,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum SkipExportAction {
    NotSubmitted {
        decision: Decision,
    },
    Cancelled {
        status: CancelStatus,
    },
    Admitted {
        receipt: ControlReceipt,
        reference: v1::NativeResultRefV1,
        metadata: v1::NativeExportMetadataV1,
    },
    AdmittedWithError {
        receipt: ControlReceipt,
        reference: v1::NativeResultRefV1,
        metadata: v1::NativeExportMetadataV1,
        error: v1::NativeExportCoreErrorV1,
    },
    NoSubmissionReturned {
        error: v1::NativeExportCoreErrorV1,
        origin: SkipExportOrigin,
    },
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SkipExportResult {
    pub session_state: SkipExportSessionState,
    pub result: SkipExportAction,
}
#[derive(Debug, Clone, Serialize)]
pub struct SkipExportCapability {
    pub available: bool,
    pub issuer_scope: Option<v1::NativeIssuerScopeV1>,
}

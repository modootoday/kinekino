//! Owned GUI observations only. Serialized identities are never native/SDK authority.
use crate::{application::api::ApiError, domain::api_values::DecimalU64};
use serde::{Deserialize, Serialize};
pub mod encoding;

pub const NATIVE_READY_KIND: &str = "native-routed-frame-ready";
pub const VIEWER_OUTCOME_KIND: &str = "native-viewer-outcome";
pub const READY_MAX_BYTES: usize = 8 * 1024;
pub const FINAL_MAX_BYTES: usize = 128 * 1024;
pub const IDENTITY_TEXT_MAX_BYTES: usize = 128;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentObservation {
    pub host_id: DecimalU64,
    pub instance_document: String,
    pub instance_epoch: DecimalU64,
    pub instance_id: DecimalU64,
    pub source_generation: DecimalU64,
    pub definition_revision: DecimalU64,
    pub document_revision: DecimalU64,
    pub turn_sequence: DecimalU64,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WindowObservation {
    pub owner: DecimalU64,
    pub epoch: DecimalU64,
    pub surface_generation: DecimalU64,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrameObservation {
    pub context_id: String,
    pub context_generation: DecimalU64,
    pub scene_generation: DecimalU64,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadyStatus {
    Activated,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeRoutedFrameReadyV1 {
    pub schema_version: u32,
    pub document: DocumentObservation,
    pub window: WindowObservation,
    pub frame: FrameObservation,
    pub child_id: DecimalU64,
    pub route_generation: DecimalU64,
    pub status: ReadyStatus,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JoinObservation {
    Unobserved,
    Pending,
    Joined,
    Panicked,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DispatchObservation {
    Unobserved,
    Poisoned,
    Idle,
    Accepted,
    Running,
    Completed,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalHostObservation {
    pub pipe: JoinObservation,
    pub sdk: JoinObservation,
    pub dispatch: DispatchObservation,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionObservation {
    pub worker_stopped: Option<bool>,
    pub unacknowledged: Option<DecimalU64>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransitionObservation {
    pub configured: bool,
    pub admission_closed: Option<bool>,
    pub ended: Option<bool>,
    pub remaining_job_id: Option<String>,
    pub error: Option<ApiError>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderJobsObservation {
    pub completed_total: DecimalU64,
    pub cancelled_total: DecimalU64,
    pub interrupted_total: DecimalU64,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderObservation {
    pub configured: bool,
    pub completed: Option<bool>,
    pub pending: Option<bool>,
    pub jobs_receipt: Option<ProviderJobsObservation>,
    pub browser_worker_exited: Option<bool>,
    pub first_error: Option<ApiError>,
    pub secondary_errors: Vec<ApiError>,
    pub secondary_overflow: DecimalU64,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispatcherObservation {
    pub joined: Option<bool>,
    pub worker_panicked: Option<bool>,
    pub faults: Option<DecimalU64>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FailureObservation {
    pub phase: String,
    pub message: String,
    pub sdk: Option<ApiError>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeRoutedFinalV1 {
    pub schema_version: u32,
    pub native_closed: Option<bool>,
    pub viewer_workers_quiescent: Option<bool>,
    pub execution: ExecutionObservation,
    pub local_host: LocalHostObservation,
    pub transition: TransitionObservation,
    pub provider: ProviderObservation,
    pub dispatcher: DispatcherObservation,
    pub cleanup_complete: bool,
    pub first_error: Option<FailureObservation>,
    pub diagnostic_write_error: Option<String>,
}

/// Original bounded AttemptFailure fields; missing ApiError data is never reconstructed.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderShutdownDiagnostic {
    pub code: crate::application::api::ErrorCode,
    pub message: String,
    pub remediation: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderObservationV2 {
    pub configured: bool,
    pub completed: Option<bool>,
    pub pending: Option<bool>,
    pub jobs_receipt: Option<ProviderJobsObservation>,
    pub browser_worker_exited: Option<bool>,
    pub first_error: Option<ApiError>,
    pub secondary_errors: Vec<ProviderShutdownDiagnostic>,
    pub secondary_overflow: Option<bool>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeRoutedFinalV2 {
    pub schema_version: u32,
    pub native_closed: Option<bool>,
    pub viewer_workers_quiescent: Option<bool>,
    pub execution: ExecutionObservation,
    pub local_host: LocalHostObservation,
    pub transition: TransitionObservation,
    pub provider: ProviderObservationV2,
    pub dispatcher: DispatcherObservation,
    pub cleanup_complete: bool,
    pub first_error: Option<FailureObservation>,
    pub diagnostic_write_error: Option<String>,
}

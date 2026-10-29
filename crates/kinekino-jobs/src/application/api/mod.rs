#[path = "ports.rs"]
mod ports;
use super::{EventPage, JobListCursor, JobPage};
use crate::domain::{CancelDisposition, JobId, JobRecord};
use kinekino_media_contracts::{
    api_values::DecimalU64,
    errors::{ApiError, ErrorCode},
};
pub use ports::JobsApi;
use serde::{Deserialize, Serialize};
use serde_json::Value;
fn default_timeout() -> DecimalU64 {
    DecimalU64(30000)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubmitJob {
    pub operation: String,
    pub parameters: Value,
    pub expected_source_sha256: Option<String>,
    #[serde(default = "default_timeout")]
    pub execution_timeout_ms: DecimalU64,
}
/// Created only by the original service after admission-gated, explicit adapter capture.
/// No Deserialize/worker JSON constructor: this is not an admission or execution receipt.
#[derive(Debug, Clone)]
pub struct CapturedJobRequest {
    request: SubmitJob,
}
impl CapturedJobRequest {
    pub(crate) fn new(request: SubmitJob) -> Self {
        Self { request }
    }
    pub fn request(&self) -> &SubmitJob {
        &self.request
    }
    pub fn into_request(self) -> SubmitJob {
        self.request
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GetJob {
    pub job_id: JobId,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListJobs {
    pub cursor: Option<JobListCursor>,
    pub limit: Option<usize>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WatchJobs {
    pub after_sequence: DecimalU64,
    pub limit: Option<usize>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CancelJob {
    pub job_id: JobId,
    pub expected_attempt: DecimalU64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetryJob {
    pub job_id: JobId,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubmittedJob {
    pub job: JobRecord,
    pub created: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CancelledJob {
    pub job: JobRecord,
    pub disposition: CancelDisposition,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobsCapability {
    pub read_available: bool,
    pub execution_available: bool,
    pub operations: Vec<String>,
}
pub fn unavailable() -> ApiError {
    ApiError::new(
        ErrorCode::Unsupported,
        "Job backend or explicit execution host is unavailable",
    )
}
pub struct UnavailableJobs;
impl JobsApi for UnavailableJobs {}

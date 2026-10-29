use crate::domain::{JobId, JobRecord, PreparedJobInput, SourceBlobRef, VerifiedJobOutput};
use kinekino_media_contracts::{api_values::DecimalU64, errors::ApiError};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobEvent {
    pub sequence: DecimalU64,
    pub job: JobRecord,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobAdmission {
    pub event: JobEvent,
    pub created: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventPage {
    pub events: Vec<JobEvent>,
    pub next_sequence: DecimalU64,
    pub latest_sequence: DecimalU64,
    pub earliest_sequence: DecimalU64,
    pub gap: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobListCursor {
    pub through_sequence: DecimalU64,
    pub after_created_sequence: DecimalU64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobPage {
    pub jobs: Vec<JobRecord>,
    pub next_cursor: Option<JobListCursor>,
}
pub trait JobIdentitySource: Send + Sync {
    fn next(&self) -> Result<JobId, ApiError>;
}
/// A host-wide exclusive owner lease is separate from short record-publication locking.
pub trait ExecutionOwner: Send + Sync {}
pub trait JobStore: Send + Sync {
    fn find_submission(&self, key_sha256: &str) -> Result<Option<JobRecord>, ApiError>;
    fn admit(&self, job: &JobRecord) -> Result<JobAdmission, ApiError>;
    fn get(&self, job_id: &JobId) -> Result<JobRecord, ApiError>;
    fn publish(
        &self,
        expected_revision: Option<u64>,
        job: &JobRecord,
    ) -> Result<JobEvent, ApiError>;
    fn read_events(&self, after_sequence: u64, limit: usize) -> Result<EventPage, ApiError>;
    fn list(&self, cursor: Option<&JobListCursor>, limit: usize) -> Result<JobPage, ApiError>;
    fn acquire_execution_owner(&self) -> Result<Box<dyn ExecutionOwner>, ApiError>;
}
pub trait SourceBlobStore: Send + Sync {
    fn put_utf8(&self, source: &str) -> Result<SourceBlobRef, ApiError>;
    fn read_utf8(&self, source: &SourceBlobRef) -> Result<String, ApiError>;
}
/// Implemented by the coordinator; publication and cancellation share this gate.
pub trait ExecutionControl: Send + Sync {
    /// Original worker provenance only. Legacy/unbound controls need not supply it.
    /// This does not authorize publication or prove external process cleanup.
    fn execution_identity(&self) -> Option<&super::JobAttemptIdentityV1> {
        None
    }
    fn cancellation_requested(&self) -> bool;
    fn deadline(&self) -> std::time::Instant;
    /// Durable CAS is the linearization gate shared with request_cancel.
    fn reserve_publication(&self) -> Result<(), ApiError>;
    fn begin_verification(&self) -> Result<(), ApiError>;
}
/// Operation adapters own typed payload validation and reuse their feature use cases.
pub trait JobExecutor: Send + Sync {
    /// Opt in only when prepare validates safe persisted parameters, and its source SHA
    /// can be supplied as expected_source_sha256 on normalized request resubmission.
    /// Capture must not execute an engine, fetch a URL/model, or start another worker.
    fn supports_batch_capture(&self, _operation: &str) -> bool {
        false
    }
    /// Trusted static adapter selection, never a request/worker JSON mode switch.
    /// Legacy adapters remain None until the entire owned lifecycle is accepted.
    fn owned_executor(&self, _operation: &str) -> Option<&super::owned_attempt::OwnedExecutorPort> {
        None
    }
    fn operations(&self) -> Vec<String> {
        vec![]
    }
    fn supports(&self, operation: &str) -> bool;
    fn normalize_parameters(&self, operation: &str, parameters: Value) -> Result<Value, ApiError> {
        if !self.supports(operation) {
            return Err(ApiError::new(
                kinekino_media_contracts::errors::ErrorCode::Unsupported,
                "Job operation is unsupported",
            ));
        }
        Ok(parameters)
    }
    fn execution_timeout_limit_ms(&self, operation: &str) -> Option<u64> {
        self.supports(operation).then_some(86_400_000)
    }
    fn prepare(
        &self,
        operation: &str,
        parameters: Value,
        expected_source_sha256: Option<&str>,
        blobs: &dyn SourceBlobStore,
    ) -> Result<PreparedJobInput, ApiError>;
    fn execute(
        &self,
        input: &PreparedJobInput,
        blobs: &dyn SourceBlobStore,
        control: &dyn ExecutionControl,
    ) -> Result<VerifiedJobOutput, ApiError>;
}

use super::*;
use crate::domain::{JobRecord, MAX_PARAMETERS_BYTES, SubmissionRef, valid_sha256};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use sha2::{Digest, Sha256};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
pub(crate) trait JobWake: Send + Sync {
    fn wake(&self);
}
pub(crate) struct ExecutionResources {
    pub executor: Arc<dyn JobExecutor>,
    pub blobs: Arc<dyn SourceBlobStore>,
    pub identities: Arc<dyn JobIdentitySource>,
    pub wake: Arc<dyn JobWake>,
    pub running: Arc<AtomicBool>,
    pub admission: Arc<Mutex<()>>,
    pub retained_attempt: Arc<AtomicBool>,
}
pub struct JobService {
    store: Arc<dyn JobStore>,
    execution: Option<ExecutionResources>,
}
fn invalid(message: impl Into<String>) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn limit(value: Option<usize>) -> Result<usize, ApiError> {
    let value = value.unwrap_or(50);
    if !(1..=200).contains(&value) {
        Err(invalid("Job page limit must be1..200"))
    } else {
        Ok(value)
    }
}
/// Shared admission normalization. Capture must not maintain a second request policy.
pub(super) fn normalize_request(
    backend: &ExecutionResources,
    mut input: SubmitJob,
) -> Result<SubmitJob, ApiError> {
    if !input.parameters.is_object()
        || serde_json::to_vec(&input.parameters)
            .map_err(|_| invalid("Invalid job parameters"))?
            .len()
            > MAX_PARAMETERS_BYTES
    {
        return Err(invalid("Job parameters exceed their object/byte bound"));
    }
    let max = backend
        .executor
        .execution_timeout_limit_ms(&input.operation)
        .ok_or_else(|| {
            ApiError::new(ErrorCode::Unsupported, "No executor for this job operation")
        })?;
    if !(1..=max).contains(&input.execution_timeout_ms.0)
        || input
            .expected_source_sha256
            .as_deref()
            .is_some_and(|sha| !valid_sha256(sha))
    {
        return Err(invalid(
            "Invalid job execution budget or expected source SHA",
        ));
    }
    input.parameters = backend
        .executor
        .normalize_parameters(&input.operation, input.parameters)?;
    Ok(input)
}
impl JobService {
    pub fn read_only(store: Arc<dyn JobStore>) -> Self {
        Self {
            store,
            execution: None,
        }
    }
    pub(crate) fn host(store: Arc<dyn JobStore>, execution: ExecutionResources) -> Self {
        Self {
            store,
            execution: Some(execution),
        }
    }
    fn execution(&self) -> Result<&ExecutionResources, ApiError> {
        self.execution
            .as_ref()
            .filter(|backend| backend.running.load(Ordering::Acquire))
            .ok_or_else(unavailable)
    }
    fn existing(&self, key: &str, request: &str) -> Result<Option<JobRecord>, ApiError> {
        let Some(job) = self.store.find_submission(key)? else {
            return Ok(None);
        };
        if job
            .submission
            .as_ref()
            .is_none_or(|submission| submission.request_sha256 != request)
        {
            let mut error = ApiError::new(
                ErrorCode::RevisionConflict,
                "Idempotency key belongs to a different normalized request",
            );
            error.details =
                serde_json::json!({"conflict_kind":"idempotency_conflict","job_id":job.job_id});
            return Err(error);
        }
        Ok(Some(job))
    }
}
impl JobsApi for JobService {
    fn capture(&self, input: SubmitJob) -> Result<CapturedJobRequest, ApiError> {
        let backend = self.execution()?;
        if !backend.executor.supports_batch_capture(&input.operation) {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "This adapter has no admitted batch capture contract",
            ));
        }
        let _gate = backend.admission.lock().map_err(|_| {
            ApiError::new(ErrorCode::EngineFailed, "Job admission gate unavailable")
        })?;
        self.execution()?;
        let input = normalize_request(backend, input)?;
        super::capture::prepare(backend, input)
    }
    fn capability(&self) -> JobsCapability {
        let execution = self.execution().ok();
        JobsCapability {
            read_available: true,
            execution_available: execution.is_some(),
            operations: execution.map_or_else(Vec::new, |backend| backend.executor.operations()),
        }
    }
    fn get(&self, input: GetJob) -> Result<JobRecord, ApiError> {
        self.store.get(&input.job_id)
    }
    fn list(&self, input: ListJobs) -> Result<JobPage, ApiError> {
        self.store.list(input.cursor.as_ref(), limit(input.limit)?)
    }
    fn watch(&self, input: WatchJobs) -> Result<EventPage, ApiError> {
        self.store
            .read_events(input.after_sequence.0, limit(input.limit)?)
    }
    fn submit(&self, input: SubmitJob, key: &str) -> Result<SubmittedJob, ApiError> {
        let backend = self.execution()?;
        if key.is_empty() || key.chars().count() > 128 || key.contains('\0') {
            return Err(invalid(
                "Job submission requires an idempotency key of1..128 characters",
            ));
        }
        let input = normalize_request(backend, input)?;
        let request_sha = hash(
            &serde_json::to_vec(&input).map_err(|_| invalid("Invalid normalized job request"))?,
        );
        let key_sha = hash(key.as_bytes());
        let _gate = backend.admission.lock().map_err(|_| {
            ApiError::new(ErrorCode::EngineFailed, "Job admission gate unavailable")
        })?;
        self.execution()?;
        if let Some(job) = self.existing(&key_sha, &request_sha)? {
            return Ok(SubmittedJob {
                job,
                created: false,
            });
        }
        let mut prepared = backend.executor.prepare(
            &input.operation,
            input.parameters,
            input.expected_source_sha256.as_deref(),
            backend.blobs.as_ref(),
        )?;
        prepared.execution_timeout_ms = input.execution_timeout_ms;
        prepared.validate().map_err(invalid)?;
        let mut job = JobRecord::queued(backend.identities.next()?, prepared).map_err(invalid)?;
        job.submission = Some(SubmissionRef {
            key_sha256: key_sha,
            request_sha256: request_sha,
        });
        let admitted = self.store.admit(&job)?;
        if admitted.created {
            backend.wake.wake();
        }
        Ok(SubmittedJob {
            job: admitted.event.job,
            created: admitted.created,
        })
    }
    fn cancel(&self, input: CancelJob) -> Result<CancelledJob, ApiError> {
        let backend = self.execution()?;
        let _gate = backend.admission.lock().map_err(|_| {
            ApiError::new(ErrorCode::EngineFailed, "Job admission gate unavailable")
        })?;
        self.execution()?;
        let (job, disposition) =
            request_cancel(self.store.as_ref(), &input.job_id, input.expected_attempt.0)?;
        Ok(CancelledJob { job, disposition })
    }
    fn retry(&self, input: RetryJob, expected: u64) -> Result<JobRecord, ApiError> {
        let backend = self.execution()?;
        let _gate = backend.admission.lock().map_err(|_| {
            ApiError::new(ErrorCode::EngineFailed, "Job admission gate unavailable")
        })?;
        self.execution()?;
        if backend.retained_attempt.load(Ordering::Acquire) {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Original owned attempt remains retained; retry is unavailable",
            ));
        }
        let mut job = self.store.get(&input.job_id)?;
        if job.revision.0 != expected {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Job revision changed before retry",
            ));
        }
        job.retry().map_err(invalid)?;
        self.store.publish(Some(expected), &job)?;
        backend.wake.wake();
        Ok(job)
    }
}

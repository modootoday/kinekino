use super::{
    ExecutionControl, JobAttemptIdentityV1, JobExecutor, JobStore, SourceBlobStore,
    mutations::{change_attempt, conflict, read_attempt},
};
use crate::domain::{JobFailure, JobId, JobRecord, JobState};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
mod owned;
pub(crate) use owned::{OwnedRunOutcome, PendingOwnedAttempt, run_owned};

struct StoreExecutionControl<'a> {
    store: &'a dyn JobStore,
    job_id: &'a JobId,
    attempt: u64,
    deadline: std::time::Instant,
    identity: JobAttemptIdentityV1,
}
impl ExecutionControl for StoreExecutionControl<'_> {
    fn execution_identity(&self) -> Option<&JobAttemptIdentityV1> {
        Some(&self.identity)
    }
    fn deadline(&self) -> std::time::Instant {
        self.deadline
    }
    fn cancellation_requested(&self) -> bool {
        match read_attempt(self.store, self.job_id, self.attempt) {
            Ok(job) => matches!(
                job.state,
                JobState::Cancelling | JobState::Cancelled | JobState::Interrupted
            ),
            Err(_) => true,
        }
    }
    fn reserve_publication(&self) -> Result<(), ApiError> {
        if std::time::Instant::now() >= self.deadline {
            return Err(ApiError::new(
                ErrorCode::Timeout,
                "Job execution deadline expired before publication",
            ));
        }
        let current = read_attempt(self.store, self.job_id, self.attempt)?;
        if current.cancel_requested {
            return Err(ApiError::new(
                ErrorCode::Cancelled,
                "Cancellation won before publication",
            ));
        }
        change_attempt(
            self.store,
            self.job_id,
            self.attempt,
            JobRecord::begin_publication,
        )
        .map(|_| ())
        .map_err(|error| {
            if self.cancellation_requested() {
                ApiError::new(ErrorCode::Cancelled, "Cancellation won before publication")
            } else {
                error
            }
        })
    }
    fn begin_verification(&self) -> Result<(), ApiError> {
        change_attempt(
            self.store,
            self.job_id,
            self.attempt,
            JobRecord::begin_verification,
        )
        .map(|_| ())
    }
}
fn bounded_text(value: &str, maximum: usize) -> String {
    let end = value
        .char_indices()
        .map(|(index, ch)| index + ch.len_utf8())
        .take_while(|end| *end <= maximum)
        .last()
        .unwrap_or(0);
    value[..end].to_owned()
}
/// A host-owned worker calls this outside the UI/dispatch thread. This function never spawns a host.
pub fn run_queued(
    store: &dyn JobStore,
    executor: &dyn JobExecutor,
    blobs: &dyn SourceBlobStore,
    id: &JobId,
    attempt: u64,
) -> Result<JobRecord, ApiError> {
    let current = read_attempt(store, id, attempt)?;
    if executor.owned_executor(&current.input.operation).is_some() {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Owned execution requires its retaining host driver",
        ));
    }
    if current.state == JobState::Cancelling {
        return change_attempt(store, id, attempt, JobRecord::acknowledge_cancel);
    }
    if current.state != JobState::Queued {
        return Err(conflict(
            "Worker may only start an explicitly queued attempt",
        ));
    }
    let running = match change_attempt(store, id, attempt, JobRecord::start) {
        Ok(job) => job,
        Err(error) => {
            if read_attempt(store, id, attempt)?.state == JobState::Cancelling {
                return change_attempt(store, id, attempt, JobRecord::acknowledge_cancel);
            }
            return Err(error);
        }
    };
    let control = StoreExecutionControl {
        store,
        job_id: id,
        attempt,
        deadline: std::time::Instant::now()
            .checked_add(std::time::Duration::from_millis(
                running.input.execution_timeout_ms.0,
            ))
            .ok_or_else(|| ApiError::new(ErrorCode::InvalidArgument, "Invalid job deadline"))?,
        identity: JobAttemptIdentityV1::from_running_record(&running),
    };
    let result = executor
        .execute(&running.input, blobs, &control)
        .and_then(|output| {
            change_attempt(store, id, attempt, |job| job.complete(output.clone())).map_err(
                |error| {
                    ApiError::new(
                        ErrorCode::EngineFailed,
                        format!("Executor result could not be committed: {}", error.message),
                    )
                },
            )
        });
    match result {
        Ok(job) => Ok(job),
        Err(error) => {
            let current = read_attempt(store, id, attempt)?;
            if current.state == JobState::Cancelling && !current.publication_started {
                return change_attempt(store, id, attempt, JobRecord::acknowledge_cancel);
            }
            let needs_input = matches!(
                error.code,
                ErrorCode::NeedsInput
                    | ErrorCode::SourceExpired
                    | ErrorCode::NotFound
                    | ErrorCode::RevisionConflict
                    | ErrorCode::PermissionDenied
                    | ErrorCode::Unsupported
                    | ErrorCode::InvalidArgument
            );
            let details =
                if serde_json::to_vec(&error.details).is_ok_and(|bytes| bytes.len() <= 16384) {
                    error.details
                } else {
                    serde_json::json!({"details_omitted":true,"reason":"job_error_details_limit"})
                };
            let message = if error.message.trim().is_empty() {
                "Job execution failed"
            } else {
                &error.message
            };
            let failure = JobFailure {
                code: error.code,
                message: bounded_text(message, 4096),
                remediation: bounded_text(&error.remediation, 256),
                details,
                retryable: error.retryable,
            };
            change_attempt(store, id, attempt, |job| {
                job.fail(failure.clone(), needs_input)
            })
        }
    }
}

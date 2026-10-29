use super::JobStore;
use crate::domain::{CancelDisposition, JobId, JobRecord};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};

pub(super) fn invalid(message: impl Into<String>) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
pub(super) fn conflict(message: &str) -> ApiError {
    ApiError::new(ErrorCode::RevisionConflict, message)
}
pub(super) fn read_attempt(
    store: &dyn JobStore,
    id: &JobId,
    attempt: u64,
) -> Result<JobRecord, ApiError> {
    let job = store.get(id)?;
    job.validate()
        .map_err(|message| ApiError::new(ErrorCode::NeedsInput, message))?;
    if job.attempt.0 != attempt {
        return Err(conflict("Job attempt changed; inspect before acting"));
    }
    Ok(job)
}
pub fn request_cancel(
    store: &dyn JobStore,
    id: &JobId,
    attempt: u64,
) -> Result<(JobRecord, CancelDisposition), ApiError> {
    for _ in 0..8 {
        let mut job = read_attempt(store, id, attempt)?;
        let previous = job.revision.0;
        let disposition = job.request_cancel().map_err(invalid)?;
        if job.revision.0 == previous {
            return Ok((job, disposition));
        }
        match store.publish(Some(previous), &job) {
            Ok(_) => return Ok((job, disposition)),
            Err(error) if error.code == ErrorCode::RevisionConflict => {}
            Err(error) => return Err(error),
        }
    }
    Err(conflict("Job kept changing; inspect the current outcome"))
}
pub(super) fn change_attempt(
    store: &dyn JobStore,
    id: &JobId,
    attempt: u64,
    change: impl Fn(&mut JobRecord) -> Result<(), &'static str>,
) -> Result<JobRecord, ApiError> {
    for _ in 0..8 {
        let mut job = read_attempt(store, id, attempt)?;
        let previous = job.revision.0;
        change(&mut job).map_err(invalid)?;
        job.validate().map_err(invalid)?;
        match store.publish(Some(previous), &job) {
            Ok(_) => return Ok(job),
            Err(error) if error.code == ErrorCode::RevisionConflict => {}
            Err(error) => return Err(error),
        }
    }
    Err(conflict("Job kept changing; inspect the current outcome"))
}

//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

pub trait JobsApi: Send + Sync {
    fn capture(&self, _: SubmitJob) -> Result<CapturedJobRequest, ApiError> {
        Err(unavailable())
    }
    fn capability(&self) -> JobsCapability {
        JobsCapability {
            read_available: false,
            execution_available: false,
            operations: vec![],
        }
    }
    fn get(&self, _: GetJob) -> Result<JobRecord, ApiError> {
        Err(unavailable())
    }
    fn list(&self, _: ListJobs) -> Result<JobPage, ApiError> {
        Err(unavailable())
    }
    fn watch(&self, _: WatchJobs) -> Result<EventPage, ApiError> {
        Err(unavailable())
    }
    fn submit(&self, _: SubmitJob, _: &str) -> Result<SubmittedJob, ApiError> {
        Err(unavailable())
    }
    fn cancel(&self, _: CancelJob) -> Result<CancelledJob, ApiError> {
        Err(unavailable())
    }
    fn retry(&self, _: RetryJob, _: u64) -> Result<JobRecord, ApiError> {
        Err(unavailable())
    }
}

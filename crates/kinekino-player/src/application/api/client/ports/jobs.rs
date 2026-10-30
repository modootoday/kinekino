//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

/// Jobs.
pub trait JobClient: ApplicationClient {
    fn submit_job(
        &self,
        context: RequestContext,
        payload: crate::application::jobs::SubmitJob,
    ) -> Result<crate::application::jobs::SubmittedJob, ApiError> {
        review_call(self, context, "jobs.submit", payload)
    }
    fn get_job(
        &self,
        context: RequestContext,
        payload: crate::application::jobs::GetJob,
    ) -> Result<crate::application::jobs::JobRecord, ApiError> {
        review_call(self, context, "jobs.get", payload)
    }
    fn list_jobs(
        &self,
        context: RequestContext,
        payload: crate::application::jobs::ListJobs,
    ) -> Result<crate::application::jobs::JobPage, ApiError> {
        review_call(self, context, "jobs.list", payload)
    }
    fn watch_jobs(
        &self,
        context: RequestContext,
        payload: crate::application::jobs::WatchJobs,
    ) -> Result<crate::application::jobs::EventPage, ApiError> {
        review_call(self, context, "jobs.watch", payload)
    }
    fn cancel_job(
        &self,
        context: RequestContext,
        payload: crate::application::jobs::CancelJob,
    ) -> Result<crate::application::jobs::CancelledJob, ApiError> {
        review_call(self, context, "jobs.cancel", payload)
    }
    fn retry_job(
        &self,
        context: RequestContext,
        payload: crate::application::jobs::RetryJob,
    ) -> Result<crate::application::jobs::JobRecord, ApiError> {
        review_call(self, context, "jobs.retry", payload)
    }
}

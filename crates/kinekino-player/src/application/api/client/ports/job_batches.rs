//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

/// Job batches.
pub trait JobBatchClient: ApplicationClient {
    fn create_job_batch(
        &self,
        context: RequestContext,
        input: crate::application::job_batches::service::CreateBatch,
    ) -> Result<crate::application::job_batches::service::BatchReceipt, ApiError> {
        review_call(
            self,
            context,
            "job_batches.create",
            serde_json::to_value(input).map_err(|_| invalid_response())?,
        )
    }
    fn inspect_job_batch(
        &self,
        context: RequestContext,
        input: crate::application::job_batches::service::InspectBatch,
    ) -> Result<crate::application::job_batches::service::BatchPage, ApiError> {
        review_call(
            self,
            context,
            "job_batches.inspect",
            serde_json::to_value(input).map_err(|_| invalid_response())?,
        )
    }
    fn submit_job_batch(
        &self,
        context: RequestContext,
        input: crate::application::job_batches::service::BatchIdentity,
    ) -> Result<crate::application::job_batches::service::BatchReceipt, ApiError> {
        review_call(
            self,
            context,
            "job_batches.submit",
            serde_json::to_value(input).map_err(|_| invalid_response())?,
        )
    }
    fn refresh_job_batch(
        &self,
        context: RequestContext,
        input: crate::application::job_batches::service::BatchIdentity,
    ) -> Result<crate::application::job_batches::service::BatchReceipt, ApiError> {
        review_call(
            self,
            context,
            "job_batches.refresh",
            serde_json::to_value(input).map_err(|_| invalid_response())?,
        )
    }
    fn retry_failed_job_batch(
        &self,
        context: RequestContext,
        input: crate::application::job_batches::service::BatchIdentity,
    ) -> Result<crate::application::job_batches::service::BatchReceipt, ApiError> {
        review_call(
            self,
            context,
            "job_batches.retry_failed",
            serde_json::to_value(input).map_err(|_| invalid_response())?,
        )
    }
}

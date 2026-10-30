//! Source association transactions shared by SDK/CLI; no filesystem/UI/model calls.
pub mod api;
pub mod media_api;
pub mod media_preparation;
mod native_processes;
mod ports;
mod requests;
mod review_cases;
mod review_evidence;
mod review_media;
pub mod schema;
mod service;
pub mod timeline_evidence;
use crate::{
    application::api::{ApiError, ErrorCode},
    domain::review_source::ValidationError,
};
pub(crate) use native_processes::NativeProcessState;
pub use native_processes::{NativeProcessLedger, NativeProcessObservation};
pub use ports::*;
pub use requests::*;
pub use review_cases::ReviewCases;
pub use review_evidence::{RetainedReviewEvidence, ReviewCaseEvidence};
pub use review_media::ReviewPackageMedia;
pub use service::*;

fn invalid(message: &str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
fn conflict(message: &str) -> ApiError {
    ApiError::new(ErrorCode::RevisionConflict, message)
}
fn validate(error: ValidationError) -> ApiError {
    match error {
        ValidationError::BindingChanged | ValidationError::RevisionConflict => {
            conflict("Review source binding or revision differs")
        }
        ValidationError::UnsupportedTimelineEvidence => ApiError::new(
            ErrorCode::Unsupported,
            "Verified timeline evidence adapter unavailable",
        ),
        _ => invalid("Invalid review source identity, revision, range or note"),
    }
}
#[cfg(test)]
mod tests;

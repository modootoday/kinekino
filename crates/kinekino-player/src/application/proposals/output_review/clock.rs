use crate::{application::api::ApiError, domain::proposals::output_review::ObservedRecordTime};

/// Adapter-local recording observation, not trusted external time or reviewer identity.
/// Read only for a new publication after target/criteria/evidence validation; an
/// identical retry must return its stored observation without another clock read.
pub trait OutputReviewClock: Send + Sync {
    fn observe_record_time(&self) -> Result<ObservedRecordTime, ApiError>;
}

use crate::{
    application::{
        api::{ApiError, ErrorCode},
        proposals::output_review::OutputReviewClock,
    },
    domain::{
        api_values::DecimalU64,
        proposals::output_review::{ObservedRecordTime, RecordClock},
    },
};
pub(in crate::infrastructure::proposals) struct SystemRecordClock;
impl OutputReviewClock for SystemRecordClock {
    fn observe_record_time(&self) -> Result<ObservedRecordTime, ApiError> {
        let elapsed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| {
                ApiError::new(
                    ErrorCode::EngineFailed,
                    "Local recording clock precedes Unix epoch",
                )
            })?;
        let milliseconds = u64::try_from(elapsed.as_millis()).map_err(|_| {
            ApiError::new(
                ErrorCode::EngineFailed,
                "Local recording clock exceeds u64 milliseconds",
            )
        })?;
        Ok(ObservedRecordTime {
            unix_ms: DecimalU64(milliseconds),
            clock: RecordClock::LocalSystemUnixMs,
        })
    }
}

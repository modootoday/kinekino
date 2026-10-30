//! Fresh explicit decision identifiers; never a reviewer identity or claimed/observed time.
use crate::application::api::{ApiError, ErrorCode};

pub fn next_decision_id() -> Result<String, ApiError> {
    let guid = crate::infrastructure::os_policy::fresh_guid_hex().map_err(|error| {
        ApiError::new(
            ErrorCode::EngineFailed,
            format!("Output decision identity allocation failed: {error}"),
        )
    })?;
    Ok(format!("output-{guid}"))
}

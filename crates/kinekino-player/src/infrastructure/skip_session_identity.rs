use crate::application::api::{ApiError, ErrorCode};
/// OS host generation; request payloads never select this identity.
pub fn new_host_generation() -> Result<String, ApiError> {
    let guid = crate::infrastructure::os_policy::fresh_guid_hex().map_err(|_| {
        ApiError::new(
            ErrorCode::EngineFailed,
            "Cannot create skip host generation",
        )
    })?;
    Ok(guid)
}

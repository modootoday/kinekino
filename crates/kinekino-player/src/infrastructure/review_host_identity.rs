//! OS-generated review host lifetime identity; never supplied by a request.
use crate::application::api::{ApiError, ErrorCode};

pub(crate) fn fresh_scope() -> Result<String, ApiError> {
    let guid = crate::infrastructure::os_policy::fresh_guid_hex()
        .map_err(|_| ApiError::new(ErrorCode::EngineFailed, "Cannot create review host scope"))?;
    Ok(guid)
}

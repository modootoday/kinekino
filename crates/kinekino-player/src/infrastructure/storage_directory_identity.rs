//! Actual identity from the original directory handle; never a path hash.
use crate::application::api::{ApiError, ErrorCode};
use crate::infrastructure::os_policy;
use serde::{Deserialize, Serialize};
use std::fs::File;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StorageDirectoryIdentity {
    volume: String,
    file: String,
}
impl StorageDirectoryIdentity {
    pub(crate) fn observe(directory: &File) -> Result<Self, ApiError> {
        let identity = os_policy::file_identity_128(directory).map_err(|error| {
            let code = match error.raw_os_error() {
                Some(5) => ErrorCode::PermissionDenied,
                Some(1 | 50 | 87) => ErrorCode::Unsupported,
                _ if error.kind() == std::io::ErrorKind::PermissionDenied => {
                    ErrorCode::PermissionDenied
                }
                _ => ErrorCode::EngineFailed,
            };
            let mut failure = ApiError::new(code, "Stable proposal root identity is unavailable");
            failure.remediation = "proposal_store_origin_unavailable".into();
            failure
        })?;
        Ok(Self {
            volume: format!("{:016x}", identity.volume),
            file: identity
                .file
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
        })
    }
}

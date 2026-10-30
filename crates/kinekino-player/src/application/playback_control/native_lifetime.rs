//! Intended application/playback_control/native_lifetime.rs.
use crate::{
    application::api::{ApiError, ErrorCode},
    domain::api_values::DecimalU64,
};

/// An in-process observation from the native owner, never a deserializable client receipt.
#[derive(Debug, serde::Serialize)]
pub struct StoppedNativeEngine {
    engine_instance_id: DecimalU64,
}
impl StoppedNativeEngine {
    pub fn engine_instance_id(&self) -> DecimalU64 {
        self.engine_instance_id
    }
    pub(crate) fn observed(
        engine_instance_id: DecimalU64,
        engine_exited: bool,
        ipc_worker_exited: bool,
        stderr_worker_exited: bool,
        stdout_worker_exited: bool,
    ) -> Result<Self, ApiError> {
        if engine_instance_id.0 == 0
            || !engine_exited
            || !ipc_worker_exited
            || !stderr_worker_exited
            || !stdout_worker_exited
        {
            return Err(ApiError::new(
                ErrorCode::EngineFailed,
                "Native process and worker shutdown was not fully observed",
            ));
        }
        Ok(Self { engine_instance_id })
    }
}

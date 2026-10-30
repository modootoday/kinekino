//! Intended infrastructure/mpv/shutdown_observation.rs. Common cleanup proof for legacy hosts.
use super::Mpv;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        playback_control::StoppedNativeEngine,
    },
    domain::api_values::DecimalU64,
};

impl Mpv {
    pub(crate) fn shutdown_observed(&mut self) -> Result<StoppedNativeEngine, ApiError> {
        self.shutdown().map_err(|_| {
            ApiError::new(
                ErrorCode::EngineFailed,
                "Native process shutdown could not be confirmed",
            )
        })?;
        let engine_exited = self
            .child
            .try_wait()
            .map_err(|_| {
                ApiError::new(
                    ErrorCode::EngineFailed,
                    "Native process exit could not be observed",
                )
            })?
            .is_some();
        StoppedNativeEngine::observed(
            DecimalU64(self.control_handle().instance_id()),
            engine_exited,
            self.ipc_worker.is_none(),
            self.stderr_worker.is_none(),
            self.stdout_worker.is_none(),
        )
    }
}

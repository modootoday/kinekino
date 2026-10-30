mod client;
mod diagnostics;
pub mod execution_host;
mod executor;
mod framing;
mod security;
mod server;
mod shutdown;
#[cfg(feature = "validation-export-delivery-loss")]
pub mod validation;
pub use client::EndpointClient;
pub use diagnostics::{TransportDiagnostics, TransportEvent};
pub use executor::DispatchState;
pub use server::LocalHost;
pub use shutdown::{LocalHostShutdownStatus, PendingLocalHost, WorkerJoin};

use crate::application::api::{ApiError, ErrorCode};
use std::time::Duration;

/// Transport deadlines are separate from the SDK request's operation timeout.
#[derive(Clone, Copy)]
pub struct TransportOptions {
    pub timeout: Duration,
}
impl Default for TransportOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_millis(crate::application::local_host::DEFAULT_TIMEOUT_MS),
        }
    }
}
impl TransportOptions {
    fn validate(self) -> Result<Self, ApiError> {
        if !(Duration::from_millis(10)..=Duration::from_secs(30)).contains(&self.timeout) {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid local transport timeout",
            ));
        }
        Ok(self)
    }
}

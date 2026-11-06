//! Host-owned browser capture for plugin Resolve (ADR 20261106032051).
//!
//! [`BrowserCaptureSession`] opens a session on a host [`crate::engine::port::BrowserEngine`],
//! navigates an admitted target, and returns capped observed network requests. The plugin maps
//! those observations to [`kinekino_core::source::StreamCandidate`]. [`crate::engine::port::NetworkInterceptor`]
//! stays Allow/Block only and is not the plugin-facing Resolve API.

mod admission;
mod mock;
mod observe;
mod session;
mod types;

#[cfg(test)]
mod tests;

pub use admission::{AllowlistAdmission, CaptureAdmission, host_of};
pub use mock::MockBrowserEngine;
pub use observe::NetworkObservationSource;
pub use session::{BrowserCaptureSession, DEFAULT_MAX_OBSERVATIONS};
pub use types::{CaptureError, CaptureNavigateTarget, CaptureWaitHint, ObservedNetworkRequest};

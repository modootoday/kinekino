//! Engine-side drain of observed network requests for capture sessions.

use super::types::ObservedNetworkRequest;

/// Companion to [`crate::engine::port::BrowserEngine`] for Resolve capture observation.
///
/// Distinct from [`crate::engine::port::NetworkInterceptor`] (Allow/Block only).
pub trait NetworkObservationSource {
    /// Drains requests observed since the last drain (typically after navigate).
    fn drain_observed_requests(&mut self) -> Vec<ObservedNetworkRequest>;
}

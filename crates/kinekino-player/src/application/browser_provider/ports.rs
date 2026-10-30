//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

/// All methods are admission/snapshot operations; no COM/network wait on the caller.
pub trait BrowserProviderPort: Send + Sync {
    fn issue_media_replay(
        &self,
        _stream: OwnedMediaStream,
        _policy: &dyn MediaReplayPolicy,
    ) -> Result<OwnedMediaReplay, ApiError> {
        Err(ApiError::new(
            crate::application::api::ErrorCode::Unsupported,
            "Original provider replay issuer is unavailable",
        ))
    }
    fn consume_media_replay(&self, _replay: &OwnedMediaReplay) -> Result<(), ApiError> {
        Err(ApiError::new(
            crate::application::api::ErrorCode::Unsupported,
            "Original provider replay issuer is unavailable",
        ))
    }
    fn check_media_replay(&self, _replay: &OwnedMediaReplay) -> Result<(), ApiError> {
        Err(ApiError::new(
            crate::application::api::ErrorCode::Unsupported,
            "Original provider replay issuer is unavailable",
        ))
    }
    fn admit_media_stream(
        &self,
        _ticket: &BrowserTicket,
        _generation: u64,
        _session: &crate::application::browser_network::ProtocolSession,
        _request_id: &str,
    ) -> Result<OwnedMediaStream, ApiError> {
        Err(ApiError::new(
            crate::application::api::ErrorCode::Unsupported,
            "Provider streaming admission is unavailable",
        ))
    }
    fn check_media_stream(&self, _stream: &OwnedMediaStream) -> Result<(), ApiError> {
        Err(ApiError::new(
            crate::application::api::ErrorCode::Unsupported,
            "Provider streaming admission is unavailable",
        ))
    }
    fn try_start(&self, request: ValidatedBrowserRequest) -> Result<BrowserTicket, ApiError>;
    fn poll(&self, ticket: &BrowserTicket) -> Result<BrowserSnapshot, ApiError>;
    fn cancel(&self, ticket: &BrowserTicket) -> Result<CancelDisposition, ApiError>;
    /// Admission of one fixed watch action, not a player/media readiness acknowledgment.
    fn activate_episode(
        &self,
        _ticket: &BrowserTicket,
        _expected_document_generation: u64,
    ) -> Result<(), ApiError> {
        Err(ApiError::new(
            crate::application::api::ErrorCode::Unsupported,
            "Episode activation is unavailable",
        ))
    }
    fn navigate(
        &self,
        ticket: &BrowserTicket,
        expected_document_generation: u64,
        url: &str,
    ) -> Result<(), ApiError>;
    /// Terminal-only explicit release; active work must never be silently evicted.
    fn release(&self, ticket: &BrowserTicket) -> Result<(), ApiError>;
}

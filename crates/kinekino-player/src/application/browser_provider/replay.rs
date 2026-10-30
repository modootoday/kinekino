//! Original-owner capability. No provider policy is enabled by default.
use super::{BrowserWork, OwnedMediaStream};
use crate::application::{
    api::{ApiError, ErrorCode},
    browser_network::{CredentialObservation, NetworkResponse, RequestHeaderEvidence},
};

/// Trusted in-process composition port, never deserialized from browser/RPC data.
/// Implementations must be bounded, nonblocking and explicitly provider-specific.
/// A successful metadata observation alone must not authorize a second request.
pub trait MediaReplayPolicy: Send + Sync {
    fn authorize(
        &self,
        response: &NetworkResponse,
        headers: &RequestHeaderEvidence,
    ) -> Result<(), ApiError>;
}

/// Non-Clone/non-Serialize grant retaining the exact original metadata lease.
pub struct OwnedMediaReplay {
    pub(super) stream: OwnedMediaStream,
}
impl OwnedMediaReplay {
    /// Exact canonical destination/referer matching, never a new authority source.
    pub fn check_request(&self, target: &str, referer: Option<&str>) -> Result<(), ApiError> {
        use crate::domain::request_referer::RequestReferer;
        let matches = |original: &str, requested: &str| {
            url::Url::parse(original)
                .ok()
                .is_some_and(|original| original.as_str() == requested)
        };
        let referer_matches = match &self.stream.headers().referer {
            RequestReferer::Absent => referer.is_none(),
            RequestReferer::Url(original) => {
                referer.is_some_and(|requested| matches(original, requested))
            }
            // Current bootstrap cannot represent an explicitly empty header.
            RequestReferer::Empty => {
                return Err(ApiError::new(
                    ErrorCode::Unsupported,
                    "Original empty Referer cannot be represented by this transport",
                ));
            }
        };
        if !matches(&self.stream.response().url, target) || !referer_matches {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Original replay request differs from transport request",
            ));
        }
        Ok(())
    }
    pub(crate) fn ticket(&self) -> &super::BrowserTicket {
        &self.stream.ticket
    }
}

impl BrowserWork {
    pub(crate) fn issue_media_replay(
        &mut self,
        stream: OwnedMediaStream,
        policy: &dyn MediaReplayPolicy,
    ) -> Result<OwnedMediaReplay, ApiError> {
        self.check_media_stream(&stream)?;
        // Shared original binding also prevents reissue through a work clone.
        // Failed/denied/panicking policy attempts remain spent.
        stream.admission.reserve_replay()?;
        if stream.headers().credentials != CredentialObservation::NoneObserved {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Original request credential state does not permit replay admission",
            ));
        }
        policy.authorize(stream.response(), stream.headers())?;
        self.check_media_stream(&stream)?;
        Ok(OwnedMediaReplay { stream })
    }
    pub(crate) fn consume_media_replay(&self, replay: &OwnedMediaReplay) -> Result<(), ApiError> {
        self.check_media_stream(&replay.stream)?;
        replay.stream.admission.consume_replay()
    }
    pub(crate) fn check_media_replay(&self, replay: &OwnedMediaReplay) -> Result<(), ApiError> {
        self.check_media_stream(&replay.stream)
    }
}

#[cfg(feature = "source-adapter-test-support")]
pub(crate) mod fixture;
#[cfg(test)]
pub(crate) mod tests;

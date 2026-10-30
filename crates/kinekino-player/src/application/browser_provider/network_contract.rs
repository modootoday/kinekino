use super::*;
use crate::application::browser_network::{ProtocolSession, ResponseCapture};

/// Native owner context is a cancellation/reload guard, not equality with protocol frame IDs.
#[derive(Clone)]
pub struct OwnedResponseCapture {
    pub session: ProtocolSession,
    pub capture: ResponseCapture,
    pub owner_document_generation: DecimalU64,
    pub owner_native_navigation_id: DecimalU64,
}

/// Stream metadata lease from the original work, independent of bounded body capture.
/// It is not serialized into RPC/project history and does not carry plaintext cookies.
pub struct OwnedMediaStream {
    pub(crate) admission: crate::application::browser_network::MediaStreamAdmission,
    pub(crate) ticket: BrowserTicket,
    pub(crate) document_generation: DecimalU64,
    pub(crate) native_navigation_id: DecimalU64,
}
impl OwnedMediaStream {
    #[allow(
        dead_code,
        reason = "Staged metadata getter; production replay consumer is not admitted"
    )]
    pub(crate) fn response(&self) -> &crate::application::browser_network::NetworkResponse {
        self.admission.response()
    }
    #[allow(
        dead_code,
        reason = "Staged metadata getter; production replay consumer is not admitted"
    )]
    pub(crate) fn headers(&self) -> &crate::application::browser_network::RequestHeaderEvidence {
        self.admission.headers()
    }
}

use super::{
    DocumentScopes, NetworkEvent, NetworkResponse, ProtocolDocumentScope, ProtocolSession,
};
use super::{
    RequestHeaderEvidence,
    request_headers::{HeaderAssociation, RequestHeaders},
};
use crate::application::api::{ApiError, ErrorCode};
use crate::domain::api_values::DecimalU64;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub const MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;

fn stream_expired() -> ApiError {
    ApiError::new(
        ErrorCode::SourceExpired,
        "Original media request or response is no longer current",
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponsePurpose {
    PlayerScript,
    PlayerBootstrap,
    ManifestCandidate,
    MediaProbe,
}

/// A one-use capture capability derived from a matched request/response/finished observation.
#[derive(Clone, PartialEq, Eq, Serialize)]
pub struct ResponseCapture {
    request_headers: Option<RequestHeaderEvidence>,
    request_method: String,
    resource_type: Option<String>,
    request_id: String,
    attempt: u64,
    scope: ProtocolDocumentScope,
    response: NetworkResponse,
    purpose: ResponsePurpose,
    referer: Option<String>,
    finished_encoded_bytes: u64,
}
impl ResponseCapture {
    pub fn request_headers(&self) -> Option<&RequestHeaderEvidence> {
        self.request_headers.as_ref()
    }
    pub fn request_method(&self) -> &str {
        &self.request_method
    }
    pub fn resource_type(&self) -> Option<&str> {
        self.resource_type.as_deref()
    }
    pub fn request_id(&self) -> &str {
        &self.request_id
    }
    pub fn scope(&self) -> &ProtocolDocumentScope {
        &self.scope
    }
    pub fn response(&self) -> &NetworkResponse {
        &self.response
    }
    pub fn purpose(&self) -> &ResponsePurpose {
        &self.purpose
    }
    pub fn referer(&self) -> Option<&str> {
        self.referer.as_deref()
    }
    pub fn attempt(&self) -> u64 {
        self.attempt
    }
    pub fn finished_encoded_bytes(&self) -> u64 {
        self.finished_encoded_bytes
    }
}
#[derive(Clone)]
pub struct ObservedResponseBody {
    pub capture: ResponseCapture,
    pub bytes: Arc<[u8]>,
}
#[derive(Clone, Serialize)]
pub struct RequestRecord {
    #[serde(skip)]
    stream_lease: Option<Arc<MediaStreamBinding>>,
    request_headers: Option<RequestHeaderEvidence>,
    header_association: HeaderAssociation,
    terminal: RequestTerminal,
    request_id: String,
    attempt: u64,
    scope: ProtocolDocumentScope,
    url: String,
    method: String,
    resource_type: Option<String>,
    referer: Option<String>,
    response: Option<NetworkResponse>,
    finished_bytes: Option<u64>,
    retired: bool,
    capture_issued: bool,
    body_consumed: bool,
}

struct MediaStreamBinding {
    replay_attempted: AtomicBool,
    replay_consumed: AtomicBool,
    scope: ProtocolDocumentScope,
    request_id: String,
    attempt: u64,
    response: NetworkResponse,
    headers: RequestHeaderEvidence,
}
/// Original live request/response admission; never serializable or constructible
/// from persisted resolution JSON. It does not assert that remote bytes are media.
pub struct MediaStreamAdmission {
    binding: Arc<MediaStreamBinding>,
}
impl MediaStreamAdmission {
    pub(crate) fn reserve_replay(&self) -> Result<(), ApiError> {
        self.binding
            .replay_attempted
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| ())
            .map_err(|_| {
                ApiError::new(
                    ErrorCode::RevisionConflict,
                    "Original replay issuance was already attempted",
                )
            })
    }
    pub(crate) fn consume_replay(&self) -> Result<(), ApiError> {
        if !self.binding.replay_attempted.load(Ordering::Acquire) {
            return Err(stream_expired());
        }
        self.binding
            .replay_consumed
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| ())
            .map_err(|_| {
                ApiError::new(
                    ErrorCode::RevisionConflict,
                    "Original replay was already consumed",
                )
            })
    }
    pub(crate) fn scope(&self) -> &ProtocolDocumentScope {
        &self.binding.scope
    }
    #[allow(
        dead_code,
        reason = "Staged metadata getter; production replay consumer is not admitted"
    )]
    pub(crate) fn response(&self) -> &NetworkResponse {
        &self.binding.response
    }
    #[allow(
        dead_code,
        reason = "Staged metadata getter; production replay consumer is not admitted"
    )]
    pub(crate) fn headers(&self) -> &RequestHeaderEvidence {
        &self.binding.headers
    }
}
#[derive(Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RequestTerminal {
    Pending,
    Finished,
    Failed { cancelled: Option<bool> },
    Redirected,
}
#[derive(Clone, Default)]
pub struct RequestLedger {
    headers: RequestHeaders,
    session: ProtocolSession,
    records: Vec<RequestRecord>,
    next_attempt: u64,
    closed: bool,
}
impl RequestLedger {
    pub(crate) fn admit_stream(
        &mut self,
        scopes: &DocumentScopes,
        request_id: &str,
    ) -> Result<MediaStreamAdmission, ApiError> {
        if self.closed {
            return Err(stream_expired());
        }
        let record = self
            .current_mut(scopes, request_id)
            .ok_or_else(stream_expired)?;
        if record.stream_lease.is_some() {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Original media stream was already admitted",
            ));
        }
        if !matches!(
            record.terminal,
            RequestTerminal::Pending | RequestTerminal::Finished
        ) || record.method != "GET"
            || !matches!(
                record.resource_type.as_deref(),
                Some("Media" | "XHR" | "Fetch")
            )
            || !matches!(record.header_association, HeaderAssociation::UniqueRequest)
        {
            return Err(stream_expired());
        }
        let response = record.response.as_ref().ok_or_else(stream_expired)?;
        // Whole-response transfer only. Range/partial admission is a separate resume contract.
        if response.status != 200 {
            return Err(ApiError::new(
                ErrorCode::NeedsInput,
                "A current complete media response is required",
            ));
        }
        let mime = response
            .mime_type
            .split(';')
            .next()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        // Plain-text HLS manifests are already observed by this provider. MIME is
        // candidate metadata, never a substitute for downstream byte/media validation.
        if mime.contains("html") {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Observed response is not a media candidate",
            ));
        }
        let headers = record.request_headers.clone().ok_or_else(stream_expired)?;
        if !headers.referer.allowed() {
            return Err(stream_expired());
        }
        let binding = Arc::new(MediaStreamBinding {
            replay_attempted: AtomicBool::new(false),
            replay_consumed: AtomicBool::new(false),
            scope: record.scope.clone(),
            request_id: record.request_id.clone(),
            attempt: record.attempt,
            response: response.clone(),
            headers,
        });
        record.stream_lease = Some(binding.clone());
        Ok(MediaStreamAdmission { binding })
    }
    pub(crate) fn check_stream(
        &self,
        scopes: &DocumentScopes,
        admission: &MediaStreamAdmission,
    ) -> Result<(), ApiError> {
        let bound = &admission.binding;
        if self.closed || !scopes.is_current(&bound.scope) {
            return Err(stream_expired());
        }
        let record = self
            .records
            .iter()
            .rfind(|record| record.request_id == bound.request_id)
            .ok_or_else(stream_expired)?;
        if record.retired
            || record.scope != bound.scope
            || record.attempt != bound.attempt
            || !matches!(
                record.terminal,
                RequestTerminal::Pending | RequestTerminal::Finished
            )
            || !matches!(record.header_association, HeaderAssociation::UniqueRequest)
            || record.response.as_ref() != Some(&bound.response)
            || record.request_headers.as_ref() != Some(&bound.headers)
            || !record
                .stream_lease
                .as_ref()
                .is_some_and(|original| Arc::ptr_eq(original, bound))
        {
            return Err(stream_expired());
        }
        Ok(())
    }
    pub fn for_session(session: ProtocolSession) -> Self {
        Self {
            session,
            ..Self::default()
        }
    }
    pub fn session(&self) -> &ProtocolSession {
        &self.session
    }
    pub fn close(&mut self) {
        self.closed = true;
    }
    pub fn records(&self) -> &[RequestRecord] {
        &self.records
    }
    fn refresh_headers(&mut self, id: &str) {
        let evidence = self.headers.observed(id).cloned();
        let association = self.headers.association(id);
        for record in self
            .records
            .iter_mut()
            .filter(|record| record.request_id == id)
        {
            record.request_headers = evidence.clone();
            record.header_association = association;
        }
    }
    pub fn evidence_capture(
        &self,
        scopes: &DocumentScopes,
        capture: &ResponseCapture,
    ) -> Option<ResponseCapture> {
        if self.closed {
            return None;
        }
        let record = self
            .records
            .iter()
            .rfind(|record| record.request_id == capture.request_id)?;
        if record.retired
            || !scopes.is_current(&record.scope)
            || record.scope != capture.scope
            || record.attempt != capture.attempt
            || record.response.as_ref() != Some(&capture.response)
            || record.method != capture.request_method
            || record.resource_type != capture.resource_type
        {
            return None;
        }
        let mut updated = capture.clone();
        updated.request_headers = record.request_headers.clone();
        Some(updated)
    }
    pub fn issued_count(&self) -> usize {
        self.records
            .iter()
            .filter(|record| record.capture_issued)
            .count()
    }
    pub fn reject_body(&mut self, scopes: &DocumentScopes, capture: &ResponseCapture) -> bool {
        if !self.can_accept_body(scopes, capture) {
            return false;
        }
        let Some(record) = self.current_mut(scopes, &capture.request_id) else {
            return false;
        };
        if record.attempt != capture.attempt
            || record.scope != capture.scope
            || !record.capture_issued
            || record.body_consumed
        {
            return false;
        }
        record.body_consumed = true;
        true
    }
    pub fn can_accept_body(&self, scopes: &DocumentScopes, capture: &ResponseCapture) -> bool {
        !self.closed
            && self
                .records
                .iter()
                .rfind(|record| record.request_id == capture.request_id)
                .is_some_and(|record| {
                    !record.retired
                        && scopes.is_current(&record.scope)
                        && record.attempt == capture.attempt
                        && record.method == capture.request_method
                        && record.resource_type == capture.resource_type
                        && record.scope == capture.scope
                        && record.response.as_ref() == Some(&capture.response)
                        && record.referer == capture.referer
                        && record.finished_bytes == Some(capture.finished_encoded_bytes)
                        && record.capture_issued
                        && !record.body_consumed
                })
    }
    pub fn observe(
        &mut self,
        scopes: &DocumentScopes,
        event: &NetworkEvent,
    ) -> Result<bool, ApiError> {
        self.observe_at(scopes, event, None)
    }
    pub fn observe_at(
        &mut self,
        scopes: &DocumentScopes,
        event: &NetworkEvent,
        received_unix_ms: Option<DecimalU64>,
    ) -> Result<bool, ApiError> {
        if self.closed {
            return Ok(false);
        }
        match event {
            NetworkEvent::Request {
                request_id,
                redirect,
                ..
            } => {
                self.headers.request(request_id, redirect.is_some());
                self.refresh_headers(request_id);
            }
            NetworkEvent::RequestExtraInfo {
                request_id,
                referer,
                credentials,
            } => {
                self.headers.extra_with_credentials(
                    request_id,
                    referer.clone(),
                    received_unix_ms,
                    *credentials,
                );
                self.refresh_headers(request_id);
                return Ok(true);
            }
            _ => {}
        }
        match event {
            NetworkEvent::Request {
                request_id,
                frame_id: Some(frame_id),
                loader_id,
                url,
                method,
                resource_type,
                referer,
                redirect,
                ..
            } => {
                let Some(scope) = scopes.current_in_session(&self.session, frame_id, loader_id)
                else {
                    return Ok(false);
                };
                if !crate::domain::navigation::media_transport_allowed(url) {
                    return Ok(false);
                }
                let previous = self
                    .records
                    .iter()
                    .rposition(|record| record.request_id == *request_id);
                if let Some(index) = previous {
                    let record = &self.records[index];
                    if scopes.is_current(&record.scope)
                        && redirect.as_ref().is_none_or(|response| {
                            !(300..400).contains(&response.status) || response.url != record.url
                        })
                    {
                        return Err(invalid(
                            "Request identity repeated without its observed redirect",
                        ));
                    }
                }
                if self.records.len() >= 64 {
                    return Err(limit());
                }
                self.next_attempt = self.next_attempt.checked_add(1).ok_or_else(limit)?;
                if let Some(index) = previous {
                    self.records[index].retired = true;
                    self.records[index].terminal = RequestTerminal::Redirected;
                    self.records[index].response = redirect.clone();
                }
                self.records.push(RequestRecord {
                    stream_lease: None,
                    request_headers: self.headers.observed(request_id).cloned(),
                    header_association: self.headers.association(request_id),
                    terminal: RequestTerminal::Pending,
                    request_id: request_id.clone(),
                    attempt: self.next_attempt,
                    scope,
                    url: url.clone(),
                    method: method.clone(),
                    resource_type: resource_type.clone(),
                    referer: referer.clone(),
                    response: None,
                    finished_bytes: None,
                    retired: false,
                    capture_issued: false,
                    body_consumed: false,
                });
                Ok(true)
            }
            NetworkEvent::Response {
                request_id,
                frame_id,
                loader_id,
                response,
                ..
            } => {
                let Some(record) = self.current_mut(scopes, request_id) else {
                    return Ok(false);
                };
                if record.scope.loader_id != *loader_id
                    || frame_id
                        .as_ref()
                        .is_some_and(|id| *id != record.scope.frame_id)
                    || record.url != response.url
                {
                    return Ok(false);
                }
                if let Some(original) = &record.response {
                    if original != response {
                        // The same current response identity is now ambiguous.
                        // Preserve the first observation, but revoke its lease.
                        record.retired = true;
                        return Ok(true);
                    }
                    return Ok(false);
                }
                record.response = Some(response.clone());
                Ok(true)
            }
            NetworkEvent::Finished {
                request_id,
                encoded_bytes,
            } => {
                let Some(record) = self.current_mut(scopes, request_id) else {
                    return Ok(false);
                };
                if record.finished_bytes.is_some() {
                    return Ok(false);
                }
                record.finished_bytes = Some(*encoded_bytes);
                record.terminal = RequestTerminal::Finished;
                Ok(true)
            }
            NetworkEvent::Failed {
                request_id,
                cancelled,
            } => {
                let Some(record) = self.current_mut(scopes, request_id) else {
                    return Ok(false);
                };
                record.retired = true;
                record.terminal = RequestTerminal::Failed {
                    cancelled: *cancelled,
                };
                Ok(true)
            }
            _ => Ok(false),
        }
    }
    fn current_mut(&mut self, scopes: &DocumentScopes, id: &str) -> Option<&mut RequestRecord> {
        self.records
            .iter_mut()
            .rfind(|record| record.request_id == id)
            .filter(|record| !record.retired && scopes.is_current(&record.scope))
    }
    pub fn take_body(&mut self, scopes: &DocumentScopes) -> Option<ResponseCapture> {
        if self.closed {
            return None;
        }
        for record in &mut self.records {
            if record.retired
                || record.capture_issued
                || !scopes.is_current(&record.scope)
                || record
                    .finished_bytes
                    .is_none_or(|bytes| bytes > MAX_RESPONSE_BYTES as u64)
            {
                continue;
            }
            let Some(response) = &record.response else {
                continue;
            };
            if !(100..=599).contains(&response.status) {
                continue;
            }
            let Some(purpose) = super::response_purpose::classify(
                &record.method,
                record.resource_type.as_deref(),
                &record.url,
                response,
            ) else {
                continue;
            };
            record.capture_issued = true;
            return Some(ResponseCapture {
                request_headers: record.request_headers.clone(),
                request_method: record.method.clone(),
                resource_type: record.resource_type.clone(),
                request_id: record.request_id.clone(),
                attempt: record.attempt,
                scope: record.scope.clone(),
                response: response.clone(),
                purpose,
                referer: record.referer.clone(),
                finished_encoded_bytes: record.finished_bytes?,
            });
        }
        None
    }
    pub fn accept_body(
        &mut self,
        scopes: &DocumentScopes,
        capture: &ResponseCapture,
        bytes: &[u8],
    ) -> Result<Option<ObservedResponseBody>, ApiError> {
        if !self.can_accept_body(scopes, capture) {
            return Ok(None);
        }
        let Some(record) = self.current_mut(scopes, &capture.request_id) else {
            return Ok(None);
        };
        if record.attempt != capture.attempt
            || record.scope != capture.scope
            || record.response.as_ref() != Some(&capture.response)
            || !record.capture_issued
            || record.body_consumed
        {
            return Ok(None);
        }
        if bytes.len() > MAX_RESPONSE_BYTES {
            return Err(limit());
        }
        record.body_consumed = true;
        Ok(Some(ObservedResponseBody {
            capture: capture.clone(),
            bytes: Arc::from(bytes),
        }))
    }
}
fn invalid(message: &'static str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
fn limit() -> ApiError {
    ApiError::new(
        ErrorCode::ResourceExhausted,
        "Owned network capture limit exceeded",
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn original_stream_retains_sanitized_credential_observation() {
        let (scopes, mut ledger) = media_stream_fixture(false);
        let extra = decode("Network.requestWillBeSentExtraInfo", &json!({"requestId":"media","headers":{"Authorization":"secret-marker"},"associatedCookies":[]}).to_string()).unwrap();
        ledger.observe(&scopes, &extra).unwrap();
        let admission = ledger.admit_stream(&scopes, "media").unwrap();
        assert_eq!(
            admission.headers().credentials,
            super::super::CredentialObservation::Present
        );
        assert!(
            !serde_json::to_string(ledger.records())
                .unwrap()
                .contains("secret-marker")
        );
        ledger.check_stream(&scopes, &admission).unwrap();
    }

    #[test]
    fn conflicting_current_response_invalidates_lease_but_exact_duplicate_does_not() {
        let (scopes, mut ledger) = media_stream_fixture(true);
        let admission = ledger.admit_stream(&scopes, "media").unwrap();
        let mut repeated = decode("Network.responseReceived", &json!({"requestId":"media","frameId":"child","loaderId":"child-loader","type":"XHR","response":{"url":"https://media.example.test/media/fixture.mp4","status":200,"mimeType":"video/mp4"}}).to_string()).unwrap();
        assert!(!ledger.observe(&scopes, &repeated).unwrap());
        ledger.check_stream(&scopes, &admission).unwrap();
        let NetworkEvent::Response { response, .. } = &mut repeated else {
            panic!("response fixture required");
        };
        response.status = 403;
        assert!(ledger.observe(&scopes, &repeated).unwrap());
        assert_eq!(
            ledger.check_stream(&scopes, &admission).unwrap_err().code,
            ErrorCode::SourceExpired
        );
        assert!(ledger.admit_stream(&scopes, "media").is_err());
    }

    fn media_stream_fixture(with_headers: bool) -> (DocumentScopes, RequestLedger) {
        let (scopes, mut ledger) = fixture();
        let request = decode("Network.requestWillBeSent", &json!({"requestId":"media","frameId":"child","loaderId":"child-loader","documentURL":"https://media.example.test/video/0123456789abcdef0123456789abcdef","type":"XHR","request":{"url":"https://media.example.test/media/fixture.mp4","method":"GET","headers":{}}}).to_string()).unwrap();
        ledger.observe(&scopes, &request).unwrap();
        if with_headers {
            ledger
                .observe(
                    &scopes,
                    &NetworkEvent::RequestExtraInfo {
                        request_id: "media".into(),
                        referer: crate::domain::request_referer::RequestReferer::Absent,
                        credentials: super::super::CredentialObservation::Unknown,
                    },
                )
                .unwrap();
        }
        let response = decode("Network.responseReceived", &json!({"requestId":"media","frameId":"child","loaderId":"child-loader","type":"XHR","response":{"url":"https://media.example.test/media/fixture.mp4","status":200,"mimeType":"video/mp4"}}).to_string()).unwrap();
        ledger.observe(&scopes, &response).unwrap();
        (scopes, ledger)
    }
    #[test]
    fn stream_metadata_admits_before_finish_without_expanding_body_budget() {
        let (scopes, mut ledger) = media_stream_fixture(true);
        let admission = ledger.admit_stream(&scopes, "media").unwrap();
        ledger.check_stream(&scopes, &admission).unwrap();
        assert!(ledger.take_body(&scopes).is_none());
        ledger
            .observe(
                &scopes,
                &decode(
                    "Network.loadingFinished",
                    r#"{"requestId":"media","encodedDataLength":4194304}"#,
                )
                .unwrap(),
            )
            .unwrap();
        ledger.check_stream(&scopes, &admission).unwrap();
        assert!(ledger.take_body(&scopes).is_none());
        assert_eq!(MAX_RESPONSE_BYTES, 2 * 1024 * 1024);
    }
    #[test]
    fn stream_admission_requires_observed_headers_and_cannot_be_reissued() {
        let (scopes, mut ledger) = media_stream_fixture(false);
        assert_eq!(
            ledger.admit_stream(&scopes, "media").err().unwrap().code,
            ErrorCode::SourceExpired
        );
        ledger
            .observe(
                &scopes,
                &NetworkEvent::RequestExtraInfo {
                    request_id: "media".into(),
                    referer: crate::domain::request_referer::RequestReferer::Absent,
                    credentials: super::super::CredentialObservation::Unknown,
                },
            )
            .unwrap();
        let admission = ledger.admit_stream(&scopes, "media").unwrap();
        assert_eq!(
            ledger.admit_stream(&scopes, "media").err().unwrap().code,
            ErrorCode::RevisionConflict
        );
        ledger.check_stream(&scopes, &admission).unwrap();
    }
    #[test]
    fn reload_and_close_invalidate_original_stream_without_url_rebinding() {
        let (mut scopes, mut ledger) = media_stream_fixture(true);
        let admission = ledger.admit_stream(&scopes, "media").unwrap();
        scopes.observe(&decode("Page.frameNavigated", r#"{"frame":{"id":"root","loaderId":"new-wrapper","url":"https://frame.example.test/tv-show.php?v=fixture"}}"#).unwrap()).unwrap();
        assert_eq!(
            ledger.check_stream(&scopes, &admission).unwrap_err().code,
            ErrorCode::SourceExpired
        );
        ledger.close();
        assert_eq!(
            ledger.check_stream(&scopes, &admission).unwrap_err().code,
            ErrorCode::SourceExpired
        );
    }
    #[test]
    fn ambiguous_extra_headers_invalidate_an_issued_stream() {
        let (scopes, mut ledger) = media_stream_fixture(true);
        let admission = ledger.admit_stream(&scopes, "media").unwrap();
        ledger
            .observe(
                &scopes,
                &NetworkEvent::RequestExtraInfo {
                    request_id: "media".into(),
                    referer: crate::domain::request_referer::RequestReferer::Absent,
                    credentials: super::super::CredentialObservation::Unknown,
                },
            )
            .unwrap();
        assert_eq!(
            ledger.check_stream(&scopes, &admission).unwrap_err().code,
            ErrorCode::SourceExpired
        );
    }
    use super::*;
    use crate::application::browser_network::decode;
    use serde_json::json;

    fn fixture() -> (DocumentScopes, RequestLedger) {
        let mut scopes = DocumentScopes::new("https://catalog.example.test/e/fixture").unwrap();
        for value in [
            json!({"frame":{"id":"root","loaderId":"episode","url":"https://catalog.example.test/e/fixture"}}),
            json!({"frame":{"id":"root","loaderId":"wrapper","url":"https://frame.example.test/tv-show.php?v=fixture"}}),
            json!({"frame":{"id":"child","parentId":"root","loaderId":"child-loader","url":"https://media.example.test/video/0123456789abcdef0123456789abcdef"}}),
        ] {
            scopes
                .observe(&decode("Page.frameNavigated", &value.to_string()).unwrap())
                .unwrap();
        }
        (scopes, RequestLedger::default())
    }
    fn request() -> NetworkEvent {
        decode("Network.requestWillBeSent", &json!({"requestId":"request","frameId":"child","loaderId":"child-loader","documentURL":"https://media.example.test/video/0123456789abcdef0123456789abcdef","type":"Script","request":{"url":"https://media.example.test/player/assets/scripts.php?v=fixture","method":"GET","headers":{}}}).to_string()).unwrap()
    }
    fn complete(scopes: &DocumentScopes, ledger: &mut RequestLedger) {
        ledger.observe(scopes, &decode("Network.responseReceived", &json!({"requestId":"request","frameId":"child","loaderId":"child-loader","type":"Script","response":{"url":"https://media.example.test/player/assets/scripts.php?v=fixture","status":200,"mimeType":"application/javascript"}}).to_string()).unwrap()).unwrap();
        ledger
            .observe(
                scopes,
                &decode(
                    "Network.loadingFinished",
                    r#"{"requestId":"request","encodedDataLength":12}"#,
                )
                .unwrap(),
            )
            .unwrap();
    }
    #[test]
    fn capture_requires_matching_response_and_finished_and_is_consumed_once() {
        let (scopes, mut ledger) = fixture();
        ledger.observe(&scopes, &request()).unwrap();
        assert!(ledger.take_body(&scopes).is_none());
        complete(&scopes, &mut ledger);
        let capture = ledger.take_body(&scopes).unwrap();
        assert!(ledger.take_body(&scopes).is_none());
        assert!(
            ledger
                .accept_body(&scopes, &capture, b"fixture body")
                .unwrap()
                .is_some()
        );
        assert!(
            ledger
                .accept_body(&scopes, &capture, b"fixture body")
                .unwrap()
                .is_none()
        );
    }
    #[test]
    fn root_reload_and_close_reject_late_body_without_url_rebinding() {
        let (mut scopes, mut ledger) = fixture();
        ledger.observe(&scopes, &request()).unwrap();
        complete(&scopes, &mut ledger);
        let capture = ledger.take_body(&scopes).unwrap();
        scopes.observe(&decode("Page.frameNavigated", r#"{"frame":{"id":"root","loaderId":"new-wrapper","url":"https://frame.example.test/tv-show.php?v=fixture"}}"#).unwrap()).unwrap();
        assert!(
            ledger
                .accept_body(&scopes, &capture, b"late")
                .unwrap()
                .is_none()
        );
        ledger.close();
        assert!(!ledger.observe(&scopes, &request()).unwrap());
    }
    #[test]
    fn equal_url_with_a_different_request_id_does_not_complete_the_request() {
        let (scopes, mut ledger) = fixture();
        ledger.observe(&scopes, &request()).unwrap();
        let response = decode("Network.responseReceived", &json!({"requestId":"other-request","frameId":"child","loaderId":"child-loader","type":"Script","response":{"url":"https://media.example.test/player/assets/scripts.php?v=fixture","status":200,"mimeType":"application/javascript"}}).to_string()).unwrap();
        assert!(!ledger.observe(&scopes, &response).unwrap());
        assert!(ledger.take_body(&scopes).is_none());
    }
    #[test]
    fn observed_redirect_retires_the_old_attempt_and_rejects_its_late_response() {
        let (scopes, mut ledger) = fixture();
        ledger.observe(&scopes, &request()).unwrap();
        let old_url = "https://media.example.test/player/assets/scripts.php?v=fixture";
        let new_url = "https://media.example.test/player/assets/scripts.php?v=redirect-fixture";
        let redirected = decode("Network.requestWillBeSent", &json!({"requestId":"request","frameId":"child","loaderId":"child-loader","documentURL":"https://media.example.test/video/0123456789abcdef0123456789abcdef","type":"Script","request":{"url":new_url,"method":"GET","headers":{}},"redirectResponse":{"url":old_url,"status":302,"mimeType":"text/html"}}).to_string()).unwrap();
        ledger.observe(&scopes, &redirected).unwrap();
        assert_eq!(ledger.records.len(), 2);
        assert!(ledger.records[0].retired);
        let late = decode("Network.responseReceived", &json!({"requestId":"request","frameId":"child","loaderId":"child-loader","type":"Script","response":{"url":old_url,"status":302,"mimeType":"text/html"}}).to_string()).unwrap();
        assert!(!ledger.observe(&scopes, &late).unwrap());
        let response = decode("Network.responseReceived", &json!({"requestId":"request","frameId":"child","loaderId":"child-loader","type":"Script","response":{"url":new_url,"status":200,"mimeType":"application/javascript"}}).to_string()).unwrap();
        ledger.observe(&scopes, &response).unwrap();
        ledger
            .observe(
                &scopes,
                &decode(
                    "Network.loadingFinished",
                    r#"{"requestId":"request","encodedDataLength":12}"#,
                )
                .unwrap(),
            )
            .unwrap();
        let capture = ledger.take_body(&scopes).unwrap();
        assert_eq!(capture.attempt, 2);
        assert_eq!(capture.response().url, new_url);
    }
}

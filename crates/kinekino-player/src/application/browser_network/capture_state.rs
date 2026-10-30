use super::*;
use crate::{
    application::api::{ApiError, ErrorCode},
    domain::api_values::DecimalU64,
};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct TimedResponseBody {
    pub session: ProtocolSession,
    pub body: ObservedResponseBody,
    pub observed_unix_ms: DecimalU64,
    pub owner_context: BodyOwnerContext,
    pub current: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyOwnerContext {
    pub document_generation: DecimalU64,
    pub native_navigation_id: DecimalU64,
}
#[derive(Clone)]
pub struct NetworkEvidence {
    pub unavailable_captures: Vec<UnavailableResponseCapture>,
    pub events: Vec<ProtocolObservation>,
    pub requests: Vec<RequestRecord>,
    pub bodies: Vec<TimedResponseBody>,
    pub unbound_session_events: DecimalU64,
    pub unavailable_bodies: DecimalU64,
    pub unscoped_requests: DecimalU64,
}
#[derive(Clone, Serialize)]
pub struct UnavailableResponseCapture {
    pub capture: ResponseCapture,
    pub owner_context: BodyOwnerContext,
    pub current: bool,
}
#[derive(Clone, Serialize)]
pub struct ProtocolObservation {
    session: ProtocolSession,
    received_unix_ms: Option<DecimalU64>,
    #[serde(flatten)]
    event: NetworkEvent,
}
impl Default for NetworkEvidence {
    fn default() -> Self {
        Self {
            unavailable_captures: Vec::new(),
            events: Vec::new(),
            requests: Vec::new(),
            bodies: Vec::new(),
            unbound_session_events: DecimalU64(0),
            unavailable_bodies: DecimalU64(0),
            unscoped_requests: DecimalU64(0),
        }
    }
}
#[derive(Clone)]
pub struct NetworkCaptureState {
    scopes: DocumentScopes,
    requests: Vec<RequestLedger>,
    evidence: NetworkEvidence,
    closed: bool,
}
impl NetworkCaptureState {
    pub(crate) fn admit_stream(
        &mut self,
        session: &ProtocolSession,
        request_id: &str,
    ) -> Result<MediaStreamAdmission, ApiError> {
        if self.closed || matches!(session, ProtocolSession::Unavailable) {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Original network session is unavailable",
            ));
        }
        self.requests
            .iter_mut()
            .find(|requests| requests.session() == session)
            .ok_or_else(|| {
                ApiError::new(
                    ErrorCode::SourceExpired,
                    "Original protocol session is absent",
                )
            })?
            .admit_stream(&self.scopes, request_id)
    }
    pub(crate) fn check_stream(&self, admission: &MediaStreamAdmission) -> Result<(), ApiError> {
        if self.closed {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Original network session closed",
            ));
        }
        self.requests
            .iter()
            .find(|requests| requests.session() == &admission.scope().session)
            .ok_or_else(|| {
                ApiError::new(
                    ErrorCode::SourceExpired,
                    "Original protocol session is absent",
                )
            })?
            .check_stream(&self.scopes, admission)
    }
    pub fn new(initial_url: &str) -> Result<Self, ApiError> {
        Ok(Self {
            scopes: DocumentScopes::new(initial_url)?,
            requests: vec![RequestLedger::default()],
            evidence: NetworkEvidence::default(),
            closed: false,
        })
    }
    pub fn close(&mut self) {
        self.closed = true;
        for requests in &mut self.requests {
            requests.close();
        }
    }
    pub fn snapshot(&self) -> NetworkEvidence {
        let mut evidence = self.evidence.clone();
        evidence.requests = self
            .requests
            .iter()
            .flat_map(|requests| requests.records().iter().cloned())
            .collect();
        for body in &mut evidence.bodies {
            let updated = self
                .requests
                .iter()
                .find(|requests| requests.session() == &body.session)
                .and_then(|requests| requests.evidence_capture(&self.scopes, &body.body.capture));
            body.current = updated.is_some();
            if let Some(updated) = updated {
                body.body.capture = updated;
            }
        }
        for failed in &mut evidence.unavailable_captures {
            failed.current = self.scopes.is_current(failed.capture.scope());
        }
        evidence
    }
    pub fn observe(
        &mut self,
        session: &ProtocolSession,
        event: NetworkEvent,
    ) -> Result<(), ApiError> {
        self.observe_at(session, event, None)
    }
    pub fn observe_at(
        &mut self,
        session: &ProtocolSession,
        event: NetworkEvent,
        received_unix_ms: Option<DecimalU64>,
    ) -> Result<(), ApiError> {
        if self.closed {
            return Ok(());
        }
        if session == &ProtocolSession::Unavailable {
            self.evidence.unbound_session_events.0 =
                self.evidence.unbound_session_events.0.saturating_add(1);
            return Ok(());
        }
        self.scopes.observe_session(session, &event)?;
        if session == &ProtocolSession::Default
            && let NetworkEvent::TargetAttached {
                session_id,
                target_type,
                ..
            } = &event
            && target_type == "iframe"
        {
            let target = ProtocolSession::Target(session_id.clone());
            if !self
                .requests
                .iter()
                .any(|requests| requests.session() == &target)
            {
                if self.requests.len() >= 9 {
                    return Err(ApiError::new(
                        ErrorCode::ResourceExhausted,
                        "Owned request session limit exceeded",
                    ));
                }
                self.requests.push(RequestLedger::for_session(target));
            }
        }
        if session == &ProtocolSession::Default
            && let NetworkEvent::TargetDetached { session_id, .. } = &event
        {
            for requests in &mut self.requests {
                if requests.session() == &ProtocolSession::Target(session_id.clone()) {
                    requests.close();
                }
            }
        }
        let Some(requests) = self
            .requests
            .iter_mut()
            .find(|requests| requests.session() == session)
        else {
            self.evidence.unbound_session_events.0 =
                self.evidence.unbound_session_events.0.saturating_add(1);
            return Ok(());
        };
        let relevant = match &event {
            NetworkEvent::FrameNavigated { url, .. } => {
                crate::domain::navigation::navigation_allowed(url)
            }
            NetworkEvent::FrameAttached { .. }
            | NetworkEvent::FrameTree { .. }
            | NetworkEvent::FrameDetached { .. }
            | NetworkEvent::TargetAttached { .. }
            | NetworkEvent::TargetDetached { .. } => true,
            _ => requests.observe_at(&self.scopes, &event, received_unix_ms)?,
        };
        if !relevant && matches!(event, NetworkEvent::Request { .. }) {
            self.evidence.unscoped_requests.0 = self.evidence.unscoped_requests.0.saturating_add(1);
        }
        if relevant {
            if self.evidence.events.len() >= 256 {
                return Err(ApiError::new(
                    ErrorCode::ResourceExhausted,
                    "Owned network evidence limit exceeded",
                ));
            }
            self.evidence.events.push(ProtocolObservation {
                session: session.clone(),
                received_unix_ms,
                event,
            });
        }
        Ok(())
    }
    pub fn take_body(&mut self) -> Option<ResponseCapture> {
        if self.closed
            || self
                .requests
                .iter()
                .map(RequestLedger::issued_count)
                .sum::<usize>()
                >= 4
        {
            None
        } else {
            self.requests
                .iter_mut()
                .find_map(|requests| requests.take_body(&self.scopes))
        }
    }
    pub fn has_current_player_script_body(
        &self,
        generation: DecimalU64,
        native_id: DecimalU64,
    ) -> bool {
        !self.closed
            && self.evidence.bodies.iter().any(|body| {
                body.owner_context.document_generation == generation
                    && body.owner_context.native_navigation_id == native_id
                    && self.scopes.is_current(body.body.capture.scope())
                    && body.body.capture.purpose() == &ResponsePurpose::PlayerScript
                    && (200..300).contains(&body.body.capture.response().status)
            })
    }
    pub fn body(
        &mut self,
        session: &ProtocolSession,
        capture: &ResponseCapture,
        json: &str,
        time: DecimalU64,
        owner_context: BodyOwnerContext,
    ) -> Result<(), ApiError> {
        if self.closed || session != &capture.scope().session {
            return Ok(());
        }
        let Some(requests) = self
            .requests
            .iter_mut()
            .find(|requests| requests.session() == session)
        else {
            return Ok(());
        };
        if !requests.can_accept_body(&self.scopes, capture) {
            return Ok(());
        }
        let bytes = decode_body(json)?;
        if let Some(body) = requests.accept_body(&self.scopes, capture, &bytes)? {
            if self.evidence.bodies.len() >= 4 {
                return Err(ApiError::new(
                    ErrorCode::ResourceExhausted,
                    "Owned network body evidence limit exceeded",
                ));
            }
            self.evidence.bodies.push(TimedResponseBody {
                session: session.clone(),
                body,
                observed_unix_ms: time,
                owner_context,
                current: true,
            });
        }
        Ok(())
    }
    pub fn unavailable(
        &mut self,
        session: &ProtocolSession,
        capture: &ResponseCapture,
        owner_context: BodyOwnerContext,
    ) {
        if session == &capture.scope().session
            && let Some(requests) = self
                .requests
                .iter_mut()
                .find(|requests| requests.session() == session)
            && requests.reject_body(&self.scopes, capture)
        {
            self.evidence.unavailable_bodies.0 =
                self.evidence.unavailable_bodies.0.saturating_add(1);
            self.evidence
                .unavailable_captures
                .push(UnavailableResponseCapture {
                    capture: capture.clone(),
                    owner_context,
                    current: true,
                });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_body_reads_still_consume_the_global_four_attempt_budget() {
        let mut state = NetworkCaptureState::new("https://catalog.example.test/e/fixture").unwrap();
        for index in 0..5 {
            for mut event in super::super::test_fixture::events() {
                match &mut event {
                    NetworkEvent::Request { request_id, .. }
                    | NetworkEvent::Response { request_id, .. }
                    | NetworkEvent::Finished { request_id, .. } => {
                        *request_id = format!("script-{index}")
                    }
                    _ => {}
                }
                state.observe(&ProtocolSession::Default, event).unwrap();
            }
            if index < 4 {
                let capture = state.take_body().unwrap();
                state.unavailable(&ProtocolSession::Default, &capture, owner());
            } else {
                assert!(state.take_body().is_none());
            }
        }
        assert_eq!(state.snapshot().unavailable_bodies.0, 4);
        assert_eq!(state.snapshot().unavailable_captures.len(), 4);
        assert!(state.snapshot().bodies.is_empty());
    }
    #[test]
    fn same_request_id_in_default_and_target_sessions_cannot_exchange_bodies() {
        let mut state = fixture();
        state
            .observe(
                &ProtocolSession::Default,
                NetworkEvent::FrameAttached {
                    frame_id: "protocol-child".into(),
                    parent_frame_id: "protocol-root".into(),
                },
            )
            .unwrap();
        state
            .observe(
                &ProtocolSession::Default,
                NetworkEvent::TargetAttached {
                    session_id: "named-session".into(),
                    target_id: "not-a-frame-id".into(),
                    target_type: "iframe".into(),
                    waiting_for_debugger: true,
                },
            )
            .unwrap();
        let target = ProtocolSession::Target("named-session".into());
        state
            .observe(
                &target,
                NetworkEvent::FrameTree {
                    frame_id: "protocol-child".into(),
                    parent_frame_id: None,
                    loader_id: "child-loader".into(),
                    url: "https://media.example.test/video/0123456789abcdef0123456789abcdef".into(),
                },
            )
            .unwrap();
        for event in super::super::test_fixture::events().into_iter().skip(3) {
            state.observe(&target, event).unwrap();
        }
        let default_capture = state.take_body().unwrap();
        let named_capture = state.take_body().unwrap();
        assert_eq!(default_capture.scope().session, ProtocolSession::Default);
        assert_eq!(named_capture.scope().session, target);
        state
            .body(
                &ProtocolSession::Default,
                &named_capture,
                "malformed wrong session",
                DecimalU64(5),
                owner(),
            )
            .unwrap();
        assert!(state.snapshot().bodies.is_empty());
        state
            .body(
                &target,
                &named_capture,
                r#"{"body":"target fixture body","base64Encoded":false}"#,
                DecimalU64(6),
                owner(),
            )
            .unwrap();
        let snapshot = state.snapshot();
        assert_eq!(snapshot.bodies.len(), 1);
        assert_eq!(snapshot.bodies[0].session, target);
        state
            .observe(
                &ProtocolSession::Default,
                NetworkEvent::RequestExtraInfo {
                    request_id: "script-request".into(),
                    referer: crate::domain::request_referer::RequestReferer::Absent,
                    credentials: CredentialObservation::Unknown,
                },
            )
            .unwrap();
        assert!(
            state.snapshot().bodies[0]
                .body
                .capture
                .request_headers()
                .is_none()
        );
        state
            .observe_at(
                &target,
                NetworkEvent::RequestExtraInfo {
                    request_id: "script-request".into(),
                    referer: crate::domain::request_referer::RequestReferer::Empty,
                    credentials: CredentialObservation::Unknown,
                },
                Some(DecimalU64(7)),
            )
            .unwrap();
        assert_eq!(
            state.snapshot().bodies[0]
                .body
                .capture
                .request_headers()
                .unwrap()
                .referer,
            crate::domain::request_referer::RequestReferer::Empty
        );
        assert_eq!(
            snapshot.bodies[0].body.bytes.as_ref(),
            b"target fixture body"
        );
    }
    #[test]
    fn attachment_evidence_does_not_grant_a_document_or_request_scope() {
        let mut state = NetworkCaptureState::new("https://catalog.example.test/e/fixture").unwrap();
        let event = decode(
            "Page.frameAttached",
            r#"{"frameId":"child","parentFrameId":"parent"}"#,
        )
        .unwrap();
        state.observe(&ProtocolSession::Default, event).unwrap();
        let evidence = state.snapshot();
        assert_eq!(evidence.events.len(), 1);
        assert!(evidence.requests.is_empty());
        assert!(state.scopes.current("child", "loader").is_none());
        assert!(state.take_body().is_none());
    }

    fn fixture() -> NetworkCaptureState {
        let mut state = NetworkCaptureState::new("https://catalog.example.test/e/fixture").unwrap();
        for event in super::super::test_fixture::events() {
            state.observe(&ProtocolSession::Default, event).unwrap();
        }
        state
    }
    fn owner() -> BodyOwnerContext {
        BodyOwnerContext {
            document_generation: DecimalU64(2),
            native_navigation_id: DecimalU64(43),
        }
    }
    #[test]
    fn child_reload_rejects_late_malformed_body_before_decoding() {
        let mut state = fixture();
        let capture = state.take_body().unwrap();
        let event = decode("Page.frameNavigated", r#"{"frame":{"id":"protocol-child","parentId":"protocol-root","loaderId":"new-child-loader","url":"https://media.example.test/video/0123456789abcdef0123456789abcdef"}}"#).unwrap();
        state.observe(&ProtocolSession::Default, event).unwrap();
        state
            .body(
                &ProtocolSession::Default,
                &capture,
                "malformed late JSON",
                DecimalU64(5),
                owner(),
            )
            .unwrap();
        assert!(state.snapshot().bodies.is_empty());
    }
    #[test]
    fn unbound_target_and_close_do_not_inherit_default_session_capture() {
        let mut state = NetworkCaptureState::new("https://catalog.example.test/e/fixture").unwrap();
        for event in super::super::test_fixture::events() {
            state
                .observe(&ProtocolSession::Target("different-target".into()), event)
                .unwrap();
        }
        assert!(state.take_body().is_none());
        assert!(state.snapshot().unbound_session_events.0 > 0);
        for event in super::super::test_fixture::events() {
            state.observe(&ProtocolSession::Default, event).unwrap();
        }
        let capture = state.take_body().unwrap();
        state.close();
        state
            .body(
                &ProtocolSession::Default,
                &capture,
                "malformed after close",
                DecimalU64(5),
                owner(),
            )
            .unwrap();
        assert!(state.snapshot().bodies.is_empty());
    }
    #[test]
    fn error_html_response_to_observed_script_is_preserved_without_becoming_runtime_success() {
        let mut state = NetworkCaptureState::new("https://catalog.example.test/e/fixture").unwrap();
        for mut event in super::super::test_fixture::events() {
            if let NetworkEvent::Response { response, .. } = &mut event {
                response.status = 404;
                response.mime_type = "text/html".into();
            }
            state.observe(&ProtocolSession::Default, event).unwrap();
        }
        let capture = state.take_body().unwrap();
        assert_eq!(capture.response().status, 404);
        state
            .body(
                &ProtocolSession::Default,
                &capture,
                r#"{"body":"fixture body","base64Encoded":false}"#,
                DecimalU64(5),
                owner(),
            )
            .unwrap();
        let snapshot = state.snapshot();
        assert_eq!(snapshot.bodies.len(), 1);
        assert_eq!(snapshot.bodies[0].body.capture.response().status, 404);
        assert_eq!(snapshot.bodies[0].body.bytes.as_ref(), b"fixture body");
    }
}

//! Explicit test-support feature only. Synthetic observations, no browser/HTTP.
use super::super::*;
use super::*;
use crate::application::browser_network::{ProtocolSession, decode};
use serde_json::json;
use std::sync::Mutex;
struct Port(Mutex<Option<BrowserWork>>);
impl BrowserProviderPort for Port {
    fn check_media_stream(&self, stream: &OwnedMediaStream) -> Result<(), ApiError> {
        self.0
            .lock()
            .unwrap()
            .as_ref()
            .ok_or_else(unsupported)?
            .check_media_stream(stream)
    }
    fn issue_media_replay(
        &self,
        stream: OwnedMediaStream,
        policy: &dyn MediaReplayPolicy,
    ) -> Result<OwnedMediaReplay, ApiError> {
        let mut slot = self.0.lock().unwrap();
        let work = slot.as_mut().ok_or_else(unsupported)?;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            work.issue_media_replay(stream, policy)
        }));
        drop(slot);
        match result {
            Ok(result) => result,
            Err(payload) => std::panic::resume_unwind(payload),
        }
    }
    fn check_media_replay(&self, grant: &OwnedMediaReplay) -> Result<(), ApiError> {
        self.0
            .lock()
            .unwrap()
            .as_ref()
            .ok_or_else(unsupported)?
            .check_media_replay(grant)
    }
    fn consume_media_replay(&self, grant: &OwnedMediaReplay) -> Result<(), ApiError> {
        self.0
            .lock()
            .unwrap()
            .as_ref()
            .ok_or_else(unsupported)?
            .consume_media_replay(grant)
    }
    fn try_start(&self, _: ValidatedBrowserRequest) -> Result<BrowserTicket, ApiError> {
        Err(unsupported())
    }
    fn poll(&self, ticket: &BrowserTicket) -> Result<BrowserSnapshot, ApiError> {
        let work = self.0.lock().unwrap();
        let snapshot = work.as_ref().ok_or_else(unsupported)?.snapshot();
        if snapshot.ticket != *ticket {
            return Err(unsupported());
        }
        Ok(snapshot)
    }
    fn cancel(&self, ticket: &BrowserTicket) -> Result<CancelDisposition, ApiError> {
        let mut slot = self.0.lock().unwrap();
        let work = slot.as_mut().ok_or_else(unsupported)?;
        if work.snapshot().ticket != *ticket {
            return Err(unsupported());
        }
        let result = work.cancel();
        work.apply(ticket, BrowserEvent::Cancelled);
        // Synthetic native completion only; never actual Capture/COM evidence.
        work.record_native_capture_released()?;
        Ok(result)
    }
    fn navigate(&self, _: &BrowserTicket, _: u64, _: &str) -> Result<(), ApiError> {
        Err(unsupported())
    }
    fn release(&self, ticket: &BrowserTicket) -> Result<(), ApiError> {
        let mut slot = self.0.lock().unwrap();
        let work = slot.as_ref().ok_or_else(unsupported)?;
        if work.snapshot().ticket != *ticket
            || !work.snapshot().state.terminal()
            || !work.native_capture_released()
        {
            return Err(unsupported());
        }
        slot.take();
        Ok(())
    }
}
fn unsupported() -> ApiError {
    ApiError::new(
        ErrorCode::Unsupported,
        "Test-support fixture has no browser",
    )
}
struct Policy;
impl MediaReplayPolicy for Policy {
    fn authorize(
        &self,
        _: &NetworkResponse,
        headers: &RequestHeaderEvidence,
    ) -> Result<(), ApiError> {
        assert_eq!(headers.credentials, CredentialObservation::NoneObserved);
        Ok(())
    }
}
pub fn fixture_grant_for_request(
    target: &str,
    deadline: std::time::Instant,
) -> (
    Arc<dyn BrowserProviderPort>,
    OwnedMediaReplay,
    BrowserDocumentSourceOwner,
) {
    let (port, stream, owner) = fixture_stream_for_request(target, deadline);
    let grant = port.issue_media_replay(stream, &Policy).unwrap();
    (port, grant, owner)
}
pub fn fixture_stream_for_request(
    target: &str,
    deadline: std::time::Instant,
) -> (
    Arc<dyn BrowserProviderPort>,
    OwnedMediaStream,
    BrowserDocumentSourceOwner,
) {
    let (work, stream, ticket) = fixture_work_for_request(target);
    let port: Arc<dyn BrowserProviderPort> = Arc::new(Port(Mutex::new(Some(work))));
    let owner = BrowserDocumentSession::fixture_admitted(port.clone(), ticket, deadline)
        .into_source_owner();
    (port, stream, owner)
}
pub(crate) fn fixture_work_for_request(
    target: &str,
) -> (BrowserWork, OwnedMediaStream, BrowserTicket) {
    let request = ValidatedBrowserRequest::new(BrowserRequest {
        request_id: "fixture".into(),
        operation: BrowserOperation::ResolveEpisode,
        url: "https://catalog.example.test/e/fixture".into(),
        timeout_ms: DecimalU64(1000),
        blocking_enabled: true,
    })
    .unwrap();
    let ticket = BrowserTicket::new("fixture-owner".into(), "fixture".into(), 1).unwrap();
    let mut work = BrowserWork::new(ticket.clone(), request).unwrap();
    work.apply(
        &ticket,
        BrowserEvent::Started {
            engine_version: "synthetic".into(),
            blocking_installed: true,
        },
    );
    work.apply(
        &ticket,
        BrowserEvent::Navigated {
            url: "https://catalog.example.test/e/fixture".into(),
            document_generation: 1,
        },
    );
    work.apply(
        &ticket,
        BrowserEvent::NavigationIdentity {
            document_generation: 1,
            native_navigation_id: 42,
        },
    );
    work.apply(
        &ticket,
        BrowserEvent::NavigationCompleted(MainNavigationDiagnostic {
            observed_unix_ms: DecimalU64(1),
            document_generation: DecimalU64(1),
            native_navigation_id: DecimalU64(42),
            http_status_code: Some(200),
            is_success: true,
            web_error_status: 0,
        }),
    );
    work.apply(
        &ticket,
        BrowserEvent::DocumentReady(BrowserDocument {
            kind: CaptureKind::DomSnapshot,
            final_url: "https://catalog.example.test/e/fixture".into(),
            document_generation: DecimalU64(1),
            observed_unix_ms: DecimalU64(1),
            body: Arc::from(b"fixture".as_slice()),
            response: None,
        }),
    );
    work.reserve_episode_activation(1).unwrap();
    work.apply(
        &ticket,
        BrowserEvent::Navigated {
            url: "https://frame.example.test/tv-show.php?v=fixture".into(),
            document_generation: 2,
        },
    );
    work.apply(
        &ticket,
        BrowserEvent::NavigationIdentity {
            document_generation: 2,
            native_navigation_id: 43,
        },
    );
    work.apply(
        &ticket,
        BrowserEvent::NavigationCompleted(MainNavigationDiagnostic {
            observed_unix_ms: DecimalU64(2),
            document_generation: DecimalU64(2),
            native_navigation_id: DecimalU64(43),
            http_status_code: Some(200),
            is_success: true,
            web_error_status: 0,
        }),
    );
    for (name, value) in [
        (
            "Page.frameNavigated",
            json!({"frame":{"id":"root","loaderId":"episode","url":"https://catalog.example.test/e/fixture"}}),
        ),
        (
            "Page.frameNavigated",
            json!({"frame":{"id":"root","loaderId":"wrapper","url":"https://frame.example.test/tv-show.php?v=fixture"}}),
        ),
        (
            "Page.frameNavigated",
            json!({"frame":{"id":"child","parentId":"root","loaderId":"child-loader","url":"https://media.example.test/video/0123456789abcdef0123456789abcdef"}}),
        ),
        (
            "Network.requestWillBeSent",
            json!({"requestId":"media","frameId":"child","loaderId":"child-loader","documentURL":"https://media.example.test/video/0123456789abcdef0123456789abcdef","type":"XHR","request":{"url":target,"method":"GET","headers":{}}}),
        ),
        (
            "Network.requestWillBeSentExtraInfo",
            json!({"requestId":"media","headers":{},"associatedCookies":[]}),
        ),
        (
            "Network.responseReceived",
            json!({"requestId":"media","frameId":"child","loaderId":"child-loader","type":"XHR","response":{"url":target,"status":200,"mimeType":"video/mp4"}}),
        ),
    ] {
        work.apply(
            &ticket,
            BrowserEvent::NetworkObserved {
                session: ProtocolSession::Default,
                event: decode(name, &value.to_string()).unwrap(),
            },
        );
    }
    let stream = work
        .admit_media_stream(2, &ProtocolSession::Default, "media")
        .unwrap();
    (work, stream, ticket)
}

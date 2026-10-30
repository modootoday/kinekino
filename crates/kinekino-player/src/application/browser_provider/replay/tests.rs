//! Synthetic provider policy only; no real service replay permission is asserted.
use super::*;
use crate::application::browser_network::ProtocolSession;
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};

struct FixturePort(std::sync::Mutex<BrowserWork>);
impl super::super::BrowserProviderPort for FixturePort {
    fn check_media_replay(&self, replay: &OwnedMediaReplay) -> Result<(), ApiError> {
        self.0.lock().unwrap().check_media_replay(replay)
    }
    fn consume_media_replay(&self, replay: &OwnedMediaReplay) -> Result<(), ApiError> {
        self.0.lock().unwrap().consume_media_replay(replay)
    }
    fn try_start(
        &self,
        _: super::super::ValidatedBrowserRequest,
    ) -> Result<super::super::BrowserTicket, ApiError> {
        Err(unsupported())
    }
    fn poll(
        &self,
        _: &super::super::BrowserTicket,
    ) -> Result<super::super::BrowserSnapshot, ApiError> {
        Err(unsupported())
    }
    fn cancel(
        &self,
        ticket: &super::super::BrowserTicket,
    ) -> Result<super::super::CancelDisposition, ApiError> {
        let mut work = self.0.lock().unwrap();
        if work.snapshot().ticket != *ticket {
            return Err(unsupported());
        }
        work.cancel();
        Ok(super::super::CancelDisposition::Requested)
    }
    fn navigate(&self, _: &super::super::BrowserTicket, _: u64, _: &str) -> Result<(), ApiError> {
        Err(unsupported())
    }
    fn release(&self, _: &super::super::BrowserTicket) -> Result<(), ApiError> {
        Err(unsupported())
    }
}
fn unsupported() -> ApiError {
    ApiError::new(
        ErrorCode::Unsupported,
        "Synthetic grant fixture has no browser",
    )
}
pub(crate) fn fixture_grant() -> (
    std::sync::Arc<dyn super::super::BrowserProviderPort>,
    OwnedMediaReplay,
) {
    let (mut work, stream) = fixture(clean());
    let grant = work.issue_media_replay(stream, &policy(true)).unwrap();
    (
        std::sync::Arc::new(FixturePort(std::sync::Mutex::new(work))),
        grant,
    )
}

struct FixturePolicy {
    calls: AtomicUsize,
    allow: bool,
}
#[test]
fn grant_binds_exact_target_and_observed_referer_absence() {
    let (mut work, stream) = fixture(clean());
    let grant = work.issue_media_replay(stream, &policy(true)).unwrap();
    grant
        .check_request("https://media.example.test/media/fixture.mp4", None)
        .unwrap();
    for (target, referer) in [
        (
            "https://media.example.test/media/fixture.mp4?changed=1",
            None,
        ),
        ("https://other.invalid/media/fixture.mp4", None),
        (
            "https://media.example.test/media/fixture.mp4",
            Some("https://media.example.test/video/other"),
        ),
    ] {
        assert_eq!(
            grant.check_request(target, referer).unwrap_err().code,
            ErrorCode::RevisionConflict
        );
    }
}
#[test]
fn explicit_empty_referer_is_not_silently_omitted() {
    let (mut work, stream) =
        fixture(json!({"requestId":"media","headers":{"Referer":""},"associatedCookies":[]}));
    let grant = work.issue_media_replay(stream, &policy(true)).unwrap();
    assert_eq!(
        grant
            .check_request("https://media.example.test/media/fixture.mp4", None)
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
}
#[test]
fn synthetic_fixture_port_uses_original_grant_and_rejects_second_consumption() {
    let (port, grant) = fixture_grant();
    port.check_media_replay(&grant).unwrap();
    port.consume_media_replay(&grant).unwrap();
    assert_eq!(
        port.consume_media_replay(&grant).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
}
impl MediaReplayPolicy for FixturePolicy {
    fn authorize(
        &self,
        response: &NetworkResponse,
        headers: &RequestHeaderEvidence,
    ) -> Result<(), ApiError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(response.status, 200);
        assert_eq!(headers.credentials, CredentialObservation::NoneObserved);
        if self.allow {
            Ok(())
        } else {
            Err(ApiError::new(
                ErrorCode::Unsupported,
                "Fixture provider denies replay",
            ))
        }
    }
}
fn policy(allow: bool) -> FixturePolicy {
    FixturePolicy {
        calls: AtomicUsize::new(0),
        allow,
    }
}
fn fixture(extra: serde_json::Value) -> (BrowserWork, OwnedMediaStream) {
    let (_, mut work) = super::super::media_stream_tests::observed_media_with_extra(extra);
    let stream = work
        .admit_media_stream(2, &ProtocolSession::Default, "media")
        .unwrap();
    (work, stream)
}
fn clean() -> serde_json::Value {
    json!({"requestId":"media","headers":{},"associatedCookies":[]})
}

#[test]
fn original_grant_is_consumed_once_across_work_candidates_and_cancel_revokes_it() {
    let (mut work, stream) = fixture(clean());
    let candidate = work.clone();
    let policy = policy(true);
    let grant = work.issue_media_replay(stream, &policy).unwrap();
    work.check_media_replay(&grant).unwrap();
    work.consume_media_replay(&grant).unwrap();
    assert_eq!(
        candidate.consume_media_replay(&grant).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(policy.calls.load(Ordering::SeqCst), 1);
    work.cancel();
    assert_eq!(
        work.check_media_replay(&grant).unwrap_err().code,
        ErrorCode::SourceExpired
    );
}
#[test]
fn denied_issuance_is_spent_and_metadata_cannot_be_reissued() {
    let (mut work, stream) = fixture(clean());
    let policy = policy(false);
    assert_eq!(
        work.issue_media_replay(stream, &policy).err().unwrap().code,
        ErrorCode::Unsupported
    );
    assert_eq!(
        work.admit_media_stream(2, &ProtocolSession::Default, "media")
            .err()
            .unwrap()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(policy.calls.load(Ordering::SeqCst), 1);
}
#[test]
fn unknown_or_present_credentials_never_reach_policy() {
    for extra in [
        json!({"requestId":"media","headers":{}}),
        json!({"requestId":"media","headers":{"Cookie":"secret-marker"},"associatedCookies":[]}),
    ] {
        let (mut work, stream) = fixture(extra);
        let policy = policy(true);
        assert_eq!(
            work.issue_media_replay(stream, &policy).err().unwrap().code,
            ErrorCode::Unsupported
        );
        assert_eq!(policy.calls.load(Ordering::SeqCst), 0);
    }
}
#[test]
fn foreign_work_cannot_issue_original_grant() {
    let (work, stream) = fixture(clean());
    let (mut foreign, _) = fixture(clean());
    let policy = policy(true);
    assert!(foreign.issue_media_replay(stream, &policy).is_err());
    assert_eq!(policy.calls.load(Ordering::SeqCst), 0);
    drop(work);
}

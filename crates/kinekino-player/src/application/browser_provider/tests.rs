use super::*;
use crate::application::api::ErrorCode;

fn fixture(operation: BrowserOperation) -> (BrowserTicket, BrowserWork) {
    let request = ValidatedBrowserRequest::new(BrowserRequest {
        request_id: "request".into(),
        operation,
        url: match operation {
            BrowserOperation::ReadCatalog => "https://catalog.example.test/방영중",
            BrowserOperation::ResolveEpisode => "https://catalog.example.test/e/one",
            BrowserOperation::ReadPlayerDocument => {
                "https://media.example.test/video/0123456789abcdef0123456789abcdef"
            }
        }
        .into(),
        timeout_ms: DecimalU64(30000),
        blocking_enabled: true,
    })
    .unwrap();
    let ticket = BrowserTicket::new("session".into(), "request".into(), 1).unwrap();
    let work = BrowserWork::new(ticket.clone(), request).unwrap();
    (ticket, work)
}
fn start(ticket: &BrowserTicket, work: &mut BrowserWork) {
    work.apply(
        ticket,
        BrowserEvent::Started {
            engine_version: "test-engine".into(),
            blocking_installed: true,
        },
    );
    work.apply(
        ticket,
        BrowserEvent::Navigated {
            url: work.request().request().url.clone(),
            document_generation: 1,
        },
    );
}
fn document(work: &BrowserWork, kind: CaptureKind) -> BrowserDocument {
    let snapshot = work.snapshot();
    BrowserDocument {
        kind,
        final_url: snapshot.current_url.unwrap(),
        document_generation: snapshot.document_generation,
        observed_unix_ms: DecimalU64(1000),
        body: Arc::from(b"<h1>actual bytes</h1>".as_slice()),
        response: (kind == CaptureKind::NetworkResponse).then_some(NetworkResponseMetadata {
            status: 200,
            content_type: "text/html".into(),
            content_encoding: None,
        }),
    }
}
#[test]
fn request_purpose_and_transport_are_validated_before_admission() {
    let (_, work) = fixture(BrowserOperation::ReadCatalog);
    for url in [
        "https://frame.example.test/a",
        "https://catalog.example.test/e/a",
        "https://user@catalog.example.test/방영중",
        "https://catalog.example.test:444/방영중",
    ] {
        assert!(!work.request().navigation_allowed(url));
    }
    let (_, work) = fixture(BrowserOperation::ResolveEpisode);
    assert!(
        work.request()
            .navigation_allowed("https://frame.example.test/tv-show.php?v=one")
    );
    for url in [
        "https://frame.example.test.evil.test/",
        "https://user@frame.example.test/",
        "https://frame.example.test:444/",
        "file:///a",
    ] {
        assert!(!work.request().navigation_allowed(url));
    }
    assert!(!crate::domain::navigation::media_transport_allowed(
        "https://user@cdn.example/a.m3u8"
    ));
    assert!(!crate::domain::navigation::media_transport_allowed(
        "https://cdn.example:444/a.m3u8"
    ));
    assert!(serde_json::from_value::<BrowserTicket>(serde_json::json!({"session_id":"session","request_id":"request","request_generation":"0"})).is_err());
}
#[test]
fn previous_ticket_and_document_cannot_publish_into_current_work() {
    let (ticket, mut work) = fixture(BrowserOperation::ReadCatalog);
    start(&ticket, &mut work);
    let old = BrowserTicket::new("old-session".into(), "request".into(), 1).unwrap();
    assert_eq!(
        work.apply(
            &old,
            BrowserEvent::DocumentReady(document(&work, CaptureKind::NetworkResponse))
        ),
        EventDisposition::Stale
    );
    let mut stale = document(&work, CaptureKind::NetworkResponse);
    stale.document_generation = DecimalU64(2);
    assert_eq!(
        work.apply(&ticket, BrowserEvent::DocumentReady(stale)),
        EventDisposition::Stale
    );
    assert_eq!(work.snapshot().state, BrowserWorkState::Running);
    let accepted = document(&work, CaptureKind::NetworkResponse);
    work.apply(&ticket, BrowserEvent::DocumentReady(accepted));
    assert_eq!(work.snapshot().state, BrowserWorkState::Succeeded);
    assert_eq!(work.cancel(), CancelDisposition::AlreadyTerminal);
    assert!(work.snapshot().result.is_some());
}
#[test]
fn cancel_requires_worker_ack_and_discards_late_capture() {
    let (ticket, mut work) = fixture(BrowserOperation::ReadCatalog);
    start(&ticket, &mut work);
    let late = document(&work, CaptureKind::NetworkResponse);
    assert_eq!(work.cancel(), CancelDisposition::Requested);
    assert_eq!(work.snapshot().state, BrowserWorkState::Cancelling);
    assert_eq!(
        work.apply(&ticket, BrowserEvent::DocumentReady(late)),
        EventDisposition::Stale
    );
    assert!(work.snapshot().result.is_none());
    work.apply(&ticket, BrowserEvent::Cancelled);
    assert_eq!(work.snapshot().state, BrowserWorkState::Cancelled);
}
#[test]
fn staged_resolution_preserves_document_then_advances_to_media() {
    let (ticket, mut work) = fixture(BrowserOperation::ResolveEpisode);
    start(&ticket, &mut work);
    work.apply(
        &ticket,
        BrowserEvent::DocumentReady(document(&work, CaptureKind::DomSnapshot)),
    );
    assert_eq!(work.snapshot().state, BrowserWorkState::Running);
    assert!(work.snapshot().document.is_some());
    let player = "https://frame.example.test/tv-show.php?v=one";
    work.reserve_navigation(1, player).unwrap();
    assert_eq!(
        work.reserve_navigation(1, player).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    work.apply(
        &ticket,
        BrowserEvent::Navigated {
            url: player.into(),
            document_generation: 2,
        },
    );
    let media = BrowserMedia {
        document_url: player.into(),
        document_generation: DecimalU64(2),
        observed_unix_ms: DecimalU64(2000),
        source: MediaSource::from_manifest(
            "https://cdn.example/manifest.txt".into(),
            player.into(),
            b"#EXTM3U\n#EXT-X-VERSION:3",
        )
        .unwrap(),
        manifest_prefix: Some(Arc::from(b"#EXTM3U\n#EXT-X-VERSION:3".as_slice())),
    };
    work.apply(&ticket, BrowserEvent::MediaReady(media));
    assert_eq!(work.snapshot().state, BrowserWorkState::Succeeded);
    assert!(matches!(
        work.snapshot().result.as_deref(),
        Some(BrowserResult::Media(_))
    ));
}
#[test]
fn dom_does_not_claim_network_status_and_filter_failure_is_terminal() {
    let (ticket, mut work) = fixture(BrowserOperation::ReadCatalog);
    start(&ticket, &mut work);
    let mut invalid = document(&work, CaptureKind::DomSnapshot);
    invalid.response = Some(NetworkResponseMetadata {
        status: 200,
        content_type: "text/html".into(),
        content_encoding: None,
    });
    work.apply(&ticket, BrowserEvent::DocumentReady(invalid));
    assert_eq!(work.snapshot().state, BrowserWorkState::Failed);
    let (ticket, mut work) = fixture(BrowserOperation::ReadCatalog);
    work.apply(
        &ticket,
        BrowserEvent::Started {
            engine_version: "test-engine".into(),
            blocking_installed: false,
        },
    );
    assert_eq!(
        work.snapshot().failure.unwrap().code,
        ErrorCode::EngineFailed
    );
}
#[test]
fn timeout_invalidates_completion_without_fabricating_cancellation_ack() {
    let (ticket, mut work) = fixture(BrowserOperation::ReadCatalog);
    start(&ticket, &mut work);
    let late = document(&work, CaptureKind::NetworkResponse);
    work.expire();
    assert_eq!(work.snapshot().failure.unwrap().code, ErrorCode::Timeout);
    assert_eq!(
        work.apply(&ticket, BrowserEvent::DocumentReady(late)),
        EventDisposition::AlreadyTerminal
    );
    assert!(work.snapshot().result.is_none());
}

#[test]
fn player_document_scope_is_exact_and_document_success_is_not_media_success() {
    let (ticket, mut work) = fixture(BrowserOperation::ReadPlayerDocument);
    assert!(
        work.request()
            .navigation_allowed(work.request().request().url.as_str())
    );
    for url in [
        "https://media.example.test/video/ffffffffffffffffffffffffffffffff",
        "https://frame.example.test/tv-show.php?v=1",
        "https://127.0.0.1/video/0123456789abcdef0123456789abcdef",
    ] {
        assert!(!work.request().navigation_allowed(url));
    }
    start(&ticket, &mut work);
    let captured = document(&work, CaptureKind::DomSnapshot);
    work.apply(&ticket, BrowserEvent::DocumentReady(captured));
    assert_eq!(work.snapshot().state, BrowserWorkState::Succeeded);
    assert!(matches!(
        work.snapshot().result.as_deref(),
        Some(BrowserResult::Document(_))
    ));
}
#[test]
fn secondary_provider_request_stays_exact_and_document_capture_is_not_media_success() {
    let (_, original) = fixture(BrowserOperation::ReadPlayerDocument);
    let mut request = original.request().request().clone();
    request.url = "https://michealcdn.com/video/0123456789abcdef0123456789abcdef".into();
    let request = ValidatedBrowserRequest::new(request).unwrap();
    assert!(request.navigation_allowed(&request.request().url));
    for foreign in [
        "https://michealcdn.com/",
        "https://michealcdn.com/video/ffffffffffffffffffffffffffffffff",
        "https://media.example.test/video/0123456789abcdef0123456789abcdef",
        "https://frame.example.test/tv-show.php?v=1",
    ] {
        assert!(!request.navigation_allowed(foreign));
    }
    let ticket =
        BrowserTicket::new("secondary-provider-session".into(), "request".into(), 1).unwrap();
    let mut work = BrowserWork::new(ticket.clone(), request).unwrap();
    start(&ticket, &mut work);
    let captured = document(&work, CaptureKind::DomSnapshot);
    work.apply(&ticket, BrowserEvent::DocumentReady(captured));
    assert_eq!(work.snapshot().state, BrowserWorkState::Succeeded);
    assert!(matches!(
        work.snapshot().result.as_deref(),
        Some(BrowserResult::Document(_))
    ));
    assert!(work.snapshot().media_requests.is_empty());
}

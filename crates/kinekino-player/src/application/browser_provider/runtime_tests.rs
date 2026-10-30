//! Synthetic app/adapter boundary checks; not native runtime evidence.
use super::*;
use std::time::Instant;

fn prepared() -> (BrowserTicket, BrowserWork) {
    let (ticket, mut work) = super::network_tests::prepared();
    super::network_tests::complete_parent(&ticket, &mut work);
    let identity = FrameDocumentIdentity {
        owner_document_generation: DecimalU64(2),
        owner_native_navigation_id: DecimalU64(43),
        frame_id: DecimalU64(0),
        document_generation: DecimalU64(1),
        native_navigation_id: DecimalU64(61),
        url: "https://media.example.test/video/0123456789abcdef0123456789abcdef".into(),
    };
    for event in [
        FrameDocumentEvent::Started(identity.clone()),
        FrameDocumentEvent::Completed(
            identity.clone(),
            FrameNavigationDiagnostic {
                observed_unix_ms: DecimalU64(4),
                http_status_code: Some(200),
                is_success: true,
                web_error_status: 0,
            },
        ),
        FrameDocumentEvent::Document(
            identity.clone(),
            BrowserDocument {
                kind: CaptureKind::DomSnapshot,
                final_url: identity.url.clone(),
                document_generation: identity.document_generation,
                observed_unix_ms: DecimalU64(5),
                body: Arc::from(b"synthetic child DOM".as_slice()),
                response: None,
            },
        ),
    ] {
        assert_eq!(
            work.apply(&ticket, BrowserEvent::FrameDocument(event)),
            EventDisposition::Applied
        );
    }
    (ticket, work)
}

fn accept_asset(ticket: &BrowserTicket, work: &mut BrowserWork) {
    let read = work.take_network_body().unwrap();
    assert_eq!(
        work.apply(
            ticket,
            BrowserEvent::NetworkBody {
                read,
                json: r#"{"body":"synthetic script","base64Encoded":false}"#.into(),
                observed_unix_ms: DecimalU64(6)
            }
        ),
        EventDisposition::Applied
    );
}

#[test]
fn child_runtime_requires_current_script_and_cancel_wins_pending_callback() {
    let (ticket, mut work) = prepared();
    assert!(work.take_frame_runtime_probe(Instant::now()).is_none());
    accept_asset(&ticket, &mut work);
    let request = work.take_frame_runtime_probe(Instant::now()).unwrap();
    assert_eq!(request.identity().frame_id, DecimalU64(0));
    assert_eq!(request.identity().native_navigation_id, DecimalU64(61));
    assert!(work.take_frame_runtime_probe(Instant::now()).is_none());
    work.cancel();
    assert_eq!(
        work.apply(
            &ticket,
            BrowserEvent::FrameRuntime {
                request,
                json: "malformed after cancel".into(),
                observed_unix_ms: DecimalU64(7)
            }
        ),
        EventDisposition::Stale
    );
    assert_eq!(work.snapshot().state, BrowserWorkState::Cancelling);
    assert!(
        work.snapshot().frame_documents[0]
            .runtime
            .observations()
            .is_empty()
    );
}

#[test]
fn parent_reload_rejects_late_runtime_before_decoding() {
    let (ticket, mut work) = prepared();
    accept_asset(&ticket, &mut work);
    let request = work.take_frame_runtime_probe(Instant::now()).unwrap();
    work.apply(
        &ticket,
        BrowserEvent::Navigated {
            url: "https://frame.example.test/tv-show.php?v=fixture".into(),
            document_generation: 3,
        },
    );
    assert_eq!(
        work.apply(
            &ticket,
            BrowserEvent::FrameRuntime {
                request,
                json: "malformed old document".into(),
                observed_unix_ms: DecimalU64(7)
            }
        ),
        EventDisposition::Stale
    );
    assert_eq!(work.snapshot().state, BrowserWorkState::Running);
    assert!(work.snapshot().frame_documents.is_empty());
}

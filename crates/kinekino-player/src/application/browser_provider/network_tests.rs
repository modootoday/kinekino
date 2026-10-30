use super::*;
use crate::application::browser_network::{ProtocolSession, test_fixture};

#[test]
fn observed_plain_text_manifest_keeps_actual_referer_and_rejects_owner_reload() {
    use crate::application::{browser_network::decode, episode_resolve::observed_manifest};
    use serde_json::json;
    let (ticket, mut work) = prepared();
    complete_parent(&ticket, &mut work);
    let script = work.take_network_body().unwrap();
    work.apply(
        &ticket,
        BrowserEvent::NetworkBody {
            read: script,
            json: r#"{"body":"synthetic script","base64Encoded":false}"#.into(),
            observed_unix_ms: DecimalU64(5),
        },
    );
    let url = "https://cdn.example/master.txt";
    let document = "https://media.example.test/video/0123456789abcdef0123456789abcdef";
    for (name, value) in [
        (
            "Network.requestWillBeSent",
            json!({"requestId":"manifest","frameId":"protocol-child","loaderId":"child-loader","documentURL":document,"type":"XHR","request":{"url":url,"method":"GET","headers":{"Referer":document}}}),
        ),
        (
            "Network.responseReceived",
            json!({"requestId":"manifest","frameId":"protocol-child","loaderId":"child-loader","type":"XHR","response":{"url":url,"status":200,"mimeType":"text/plain"}}),
        ),
        (
            "Network.loadingFinished",
            json!({"requestId":"manifest","encodedDataLength":100}),
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
    let read = work.take_network_body().unwrap();
    work.apply(&ticket, BrowserEvent::NetworkBody { read, json: json!({"body":"#EXTM3U\n#EXT-X-TARGETDURATION:5\n#EXTINF:4,\nsegment.ts\n","base64Encoded":false}).to_string(), observed_unix_ms: DecimalU64(6) });
    assert!(
        observed_manifest(&work.snapshot(), document)
            .unwrap()
            .is_none()
    );
    for (headers, expected) in [
        (
            json!({"Referer":document}),
            crate::domain::request_referer::RequestReferer::Url(document.into()),
        ),
        (
            json!({}),
            crate::domain::request_referer::RequestReferer::Absent,
        ),
        (
            json!({"Referer":""}),
            crate::domain::request_referer::RequestReferer::Empty,
        ),
    ] {
        // Each is a separate synthetic network ledger, never a duplicate event merged into a hop.
        let mut observed = work.clone();
        observed.apply(
            &ticket,
            BrowserEvent::NetworkObservedAt {
                session: ProtocolSession::Default,
                event: decode(
                    "Network.requestWillBeSentExtraInfo",
                    &json!({"requestId":"manifest","headers":headers}).to_string(),
                )
                .unwrap(),
                received_unix_ms: DecimalU64(7),
            },
        );
        let selected = observed_manifest(&observed.snapshot(), document)
            .unwrap()
            .unwrap();
        assert_eq!(selected.source.referer, expected);
        assert_eq!(
            selected
                .body
                .body
                .capture
                .request_headers()
                .unwrap()
                .received_unix_ms,
            Some(DecimalU64(7))
        );
        assert!(observed.take_network_body().is_none());
    }
    work.apply(
        &ticket,
        BrowserEvent::NetworkObserved {
            session: ProtocolSession::Default,
            event: decode(
                "Network.requestWillBeSentExtraInfo",
                &json!({"requestId":"manifest","headers":{"Referer":document}}).to_string(),
            )
            .unwrap(),
        },
    );
    let selected = observed_manifest(&work.snapshot(), document)
        .unwrap()
        .unwrap();
    assert_eq!(selected.source.url, url);
    assert_eq!(
        selected.source.referer,
        crate::domain::request_referer::RequestReferer::Url(document.into())
    );
    assert_eq!(selected.manifest.summary().segments, 1);
    work.apply(
        &ticket,
        BrowserEvent::Navigated {
            url: "https://frame.example.test/tv-show.php?v=fixture".into(),
            document_generation: 3,
        },
    );
    assert!(
        observed_manifest(&work.snapshot(), document)
            .unwrap()
            .is_none()
    );
}

pub(super) fn prepared() -> (BrowserTicket, BrowserWork) {
    let (ticket, mut work) = super::activation_tests::episode();
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
    for event in test_fixture::events() {
        work.apply(
            &ticket,
            BrowserEvent::NetworkObserved {
                session: ProtocolSession::Default,
                event,
            },
        );
    }
    (ticket, work)
}
pub(super) fn complete_parent(ticket: &BrowserTicket, work: &mut BrowserWork) {
    work.apply(
        ticket,
        BrowserEvent::NavigationCompleted(MainNavigationDiagnostic {
            observed_unix_ms: DecimalU64(4),
            document_generation: DecimalU64(2),
            native_navigation_id: DecimalU64(43),
            http_status_code: Some(200),
            is_success: true,
            web_error_status: 0,
        }),
    );
}
#[test]
fn response_read_waits_for_native_owner_completion_and_rejects_same_url_owner_reload() {
    let (ticket, mut work) = prepared();
    assert!(work.take_network_body().is_none());
    complete_parent(&ticket, &mut work);
    let read = work.take_network_body().unwrap();
    assert_eq!(read.owner_document_generation, DecimalU64(2));
    assert_eq!(read.owner_native_navigation_id, DecimalU64(43));
    assert_eq!(read.capture.scope().frame_id, "protocol-child");
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
            BrowserEvent::NetworkBody {
                read,
                json: "malformed old response".into(),
                observed_unix_ms: DecimalU64(5)
            }
        ),
        EventDisposition::Stale
    );
    assert!(work.snapshot().network.unwrap().bodies.is_empty());
    assert_eq!(work.snapshot().state, BrowserWorkState::Running);
}
#[test]
fn accepted_body_remains_evidence_and_cancel_wins_late_duplicate() {
    let (ticket, mut work) = prepared();
    complete_parent(&ticket, &mut work);
    let read = work.take_network_body().unwrap();
    let body = r#"{"body":"fixture body","base64Encoded":false}"#;
    work.apply(
        &ticket,
        BrowserEvent::NetworkBody {
            read: read.clone(),
            json: body.into(),
            observed_unix_ms: DecimalU64(5),
        },
    );
    let snapshot = work.snapshot();
    assert_eq!(snapshot.network.unwrap().bodies.len(), 1);
    assert_eq!(snapshot.state, BrowserWorkState::Running);
    assert!(snapshot.result.is_none());
    work.cancel();
    assert_eq!(
        work.apply(
            &ticket,
            BrowserEvent::NetworkBody {
                read,
                json: body.into(),
                observed_unix_ms: DecimalU64(6)
            }
        ),
        EventDisposition::Stale
    );
    assert_eq!(work.snapshot().network.unwrap().bodies.len(), 1);
}

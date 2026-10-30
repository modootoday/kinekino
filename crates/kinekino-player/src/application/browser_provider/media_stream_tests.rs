//! Metadata/current lifetime only. No replay permission, browser or HTTP execution.
use super::*;
use crate::application::{
    api::ErrorCode,
    browser_network::{ProtocolSession, decode},
};
use serde_json::json;
fn observed_media() -> (BrowserTicket, BrowserWork) {
    observed_media_with_extra(json!({"requestId":"media","headers":{},"associatedCookies":[]}))
}
pub(super) fn observed_media_with_extra(extra: serde_json::Value) -> (BrowserTicket, BrowserWork) {
    let (ticket, mut work) = network_tests::prepared();
    network_tests::complete_parent(&ticket, &mut work);
    for (name, value) in [
        (
            "Network.requestWillBeSent",
            json!({"requestId":"media","frameId":"protocol-child","loaderId":"child-loader","documentURL":"https://media.example.test/video/0123456789abcdef0123456789abcdef","type":"XHR","request":{"url":"https://media.example.test/media/fixture.mp4","method":"GET","headers":{}}}),
        ),
        ("Network.requestWillBeSentExtraInfo", extra),
        (
            "Network.responseReceived",
            json!({"requestId":"media","frameId":"protocol-child","loaderId":"child-loader","type":"XHR","response":{"url":"https://media.example.test/media/fixture.mp4","status":200,"mimeType":"video/mp4"}}),
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
    (ticket, work)
}
#[test]
fn metadata_keeps_original_owner_and_is_not_reissued_after_reload() {
    let (ticket, mut work) = observed_media();
    assert_eq!(
        work.admit_media_stream(1, &ProtocolSession::Default, "media")
            .err()
            .unwrap()
            .code,
        ErrorCode::SourceExpired
    );
    let stream = work
        .admit_media_stream(2, &ProtocolSession::Default, "media")
        .unwrap();
    work.check_media_stream(&stream).unwrap();
    assert_eq!(stream.response().status, 200);
    let _headers = stream.headers();
    assert_eq!(
        work.admit_media_stream(2, &ProtocolSession::Default, "media")
            .err()
            .unwrap()
            .code,
        ErrorCode::RevisionConflict
    );
    work.apply(
        &ticket,
        BrowserEvent::Navigated {
            url: "https://frame.example.test/tv-show.php?v=fixture".into(),
            document_generation: 3,
        },
    );
    assert_eq!(
        work.check_media_stream(&stream).unwrap_err().code,
        ErrorCode::SourceExpired
    );
}
#[test]
fn cancellation_invalidates_original_metadata_lease() {
    let (_, mut work) = observed_media();
    let stream = work
        .admit_media_stream(2, &ProtocolSession::Default, "media")
        .unwrap();
    work.cancel();
    assert_eq!(
        work.check_media_stream(&stream).unwrap_err().code,
        ErrorCode::SourceExpired
    );
}

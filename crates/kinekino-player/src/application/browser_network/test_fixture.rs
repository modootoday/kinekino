//! Synthetic protocol events only; never evidence of native browser lineage or HTTP success.
use super::{NetworkEvent, decode};
use serde_json::json;

pub(crate) fn events() -> Vec<NetworkEvent> {
    let mut events = vec![
        decode("Page.frameNavigated", &json!({"frame":{"id":"protocol-root","loaderId":"episode-loader","url":"https://catalog.example.test/e/fixture"}}).to_string()).unwrap(),
        decode("Page.frameNavigated", &json!({"frame":{"id":"protocol-root","loaderId":"wrapper-loader","url":"https://frame.example.test/tv-show.php?v=fixture"}}).to_string()).unwrap(),
        decode("Page.frameNavigated", &json!({"frame":{"id":"protocol-child","parentId":"protocol-root","loaderId":"child-loader","url":"https://media.example.test/video/0123456789abcdef0123456789abcdef"}}).to_string()).unwrap(),
    ];
    events.push(decode("Network.requestWillBeSent", &json!({"requestId":"script-request","frameId":"protocol-child","loaderId":"child-loader","documentURL":"https://media.example.test/video/0123456789abcdef0123456789abcdef","type":"Script","request":{"url":"https://media.example.test/player/assets/scripts.php?v=fixture","method":"GET","headers":{"Referer":"https://media.example.test/video/0123456789abcdef0123456789abcdef"}}}).to_string()).unwrap());
    events.push(decode("Network.responseReceived", &json!({"requestId":"script-request","frameId":"protocol-child","loaderId":"child-loader","type":"Script","response":{"url":"https://media.example.test/player/assets/scripts.php?v=fixture","status":200,"mimeType":"application/javascript"}}).to_string()).unwrap());
    events.push(
        decode(
            "Network.loadingFinished",
            r#"{"requestId":"script-request","encodedDataLength":12}"#,
        )
        .unwrap(),
    );
    events
}

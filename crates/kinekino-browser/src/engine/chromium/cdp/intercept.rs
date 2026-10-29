//! Fetch-domain request interception bridging CDP to the NetworkInterceptor port.
use crate::engine::port::{
    BrowserEvent, EngineError, EngineResult, InterceptedRequest, NetworkInterceptor,
    RequestDecision,
};
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::fetch::{
    ContinueRequestParams, EnableParams, EventRequestPaused, FailRequestParams, RequestPattern,
};
use chromiumoxide::cdp::browser_protocol::network::ErrorReason;
use futures::StreamExt;
use std::sync::{Arc, Mutex};

pub(super) async fn install(
    page: Page,
    interceptor: Arc<dyn NetworkInterceptor>,
    events: Arc<Mutex<Vec<BrowserEvent>>>,
) -> EngineResult<()> {
    let call = |e: chromiumoxide::error::CdpError| EngineError::Call(e.to_string());
    let mut paused = page
        .event_listener::<EventRequestPaused>()
        .await
        .map_err(call)?;
    let pattern = RequestPattern::builder().url_pattern("*").build();
    page.execute(EnableParams::builder().pattern(pattern).build())
        .await
        .map_err(call)?;
    tokio::spawn(async move {
        while let Some(ev) = paused.next().await {
            let req = InterceptedRequest {
                url: ev.request.url.clone(),
                method: ev.request.method.clone(),
                resource_type: format!("{:?}", ev.resource_type),
            };
            let id = ev.request_id.clone();
            let _ = match interceptor.decide(&req) {
                RequestDecision::Allow => page
                    .execute(ContinueRequestParams::new(id))
                    .await
                    .map(|_| ()),
                RequestDecision::Block => {
                    if let Ok(mut e) = events.lock() {
                        e.push(BrowserEvent::RequestBlocked { url: req.url });
                    }
                    page.execute(FailRequestParams::new(id, ErrorReason::BlockedByClient))
                        .await
                        .map(|_| ())
                }
            };
        }
    });
    Ok(())
}

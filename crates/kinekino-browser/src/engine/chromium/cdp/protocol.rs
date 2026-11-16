//! DevTools protocol pass-through for Chrome over CDP (ADR 20261109071720 decision 2).
//!
//! Calls go to the page session, or to a child target chromiumoxide attached. Events are
//! forwarded for the fixed set the browser provider reads: network requests and responses,
//! frames and child targets. chromiumoxide only offers typed listeners, so an event outside
//! [`FORWARDED`] does not reach [`ProtocolEvent`]s.
use crate::engine::port::{EngineError, EngineResult, ProtocolEvent};
use chromiumoxide::cdp::browser_protocol::network::{
    EventLoadingFailed, EventLoadingFinished, EventRequestWillBeSent, EventResponseReceived,
};
use chromiumoxide::cdp::browser_protocol::page::{
    EventFrameAttached, EventFrameDetached, EventFrameNavigated,
};
use chromiumoxide::cdp::browser_protocol::target::{
    EventAttachedToTarget, EventDetachedFromTarget,
};
use chromiumoxide::{Browser, Command, Method, MethodType, Page};
use futures::StreamExt;
use std::borrow::Cow;
use std::sync::{Arc, Mutex};

/// Protocol events forwarded to the session, by method name.
pub const FORWARDED: [&str; 9] = [
    "Network.requestWillBeSent",
    "Network.responseReceived",
    "Network.loadingFinished",
    "Network.loadingFailed",
    "Page.frameAttached",
    "Page.frameDetached",
    "Page.frameNavigated",
    "Target.attachedToTarget",
    "Target.detachedFromTarget",
];

/// A protocol method named at run time, with its JSON params and JSON result.
#[derive(Debug)]
pub(super) struct RawCommand {
    method: String,
    params: serde_json::Value,
}

impl RawCommand {
    pub(super) fn new(method: &str, params: serde_json::Value) -> Self {
        Self {
            method: method.to_owned(),
            params,
        }
    }
}

impl serde::Serialize for RawCommand {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.params.serialize(s)
    }
}

impl Method for RawCommand {
    fn identifier(&self) -> chromiumoxide::types::MethodId {
        Cow::Owned(self.method.clone())
    }
}

impl Command for RawCommand {
    type Response = serde_json::Value;
}

fn call_error(e: impl std::fmt::Display) -> EngineError {
    EngineError::Call(e.to_string())
}

/// The page, or the attached child target whose session id is `session`.
pub(super) async fn target(
    browser: &Browser,
    page: &Page,
    session: Option<&str>,
) -> EngineResult<Page> {
    let Some(session) = session else {
        return Ok(page.clone());
    };
    browser
        .pages()
        .await
        .map_err(call_error)?
        .into_iter()
        .find(|p| p.session_id().as_ref() == session)
        .ok_or_else(|| EngineError::Call(format!("no attached session {session}")))
}

/// Forwards every [`FORWARDED`] event of `page` into `events`.
pub(super) async fn listen(
    page: &Page,
    events: Arc<Mutex<Vec<ProtocolEvent>>>,
) -> EngineResult<()> {
    macro_rules! forward {
        ($($event:ty),* $(,)?) => {$(
            let mut stream = page.event_listener::<$event>().await.map_err(call_error)?;
            let sink = events.clone();
            tokio::spawn(async move {
                let method = <$event as MethodType>::method_id().into_owned();
                while let Some(event) = stream.next().await {
                    let params = serde_json::to_value(&*event).unwrap_or_default();
                    sink.lock()
                        .unwrap_or_else(|p| p.into_inner())
                        .push(ProtocolEvent { session: None, method: method.clone(), params });
                }
            });
        )*};
    }
    forward!(
        EventRequestWillBeSent,
        EventResponseReceived,
        EventLoadingFinished,
        EventLoadingFailed,
        EventFrameAttached,
        EventFrameDetached,
        EventFrameNavigated,
        EventAttachedToTarget,
        EventDetachedFromTarget,
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chromiumoxide::cdp::browser_protocol::page::EventFrameNavigated;

    #[test]
    fn raw_commands_carry_the_runtime_name_and_bare_params() {
        let cmd = RawCommand::new("Runtime.evaluate", serde_json::json!({"expression": "1"}));
        assert_eq!(cmd.identifier(), "Runtime.evaluate");
        assert_eq!(
            serde_json::to_value(&cmd).unwrap(),
            serde_json::json!({"expression": "1"})
        );
    }

    #[test]
    fn forwarded_names_match_the_typed_listeners() {
        assert!(FORWARDED.contains(&&*EventFrameNavigated::method_id()));
        assert!(FORWARDED.contains(&&*EventResponseReceived::method_id()));
        assert!(FORWARDED.contains(&&*EventAttachedToTarget::method_id()));
    }
}

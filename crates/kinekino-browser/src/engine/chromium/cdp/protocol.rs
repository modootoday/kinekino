//! Message routing for the DevTools protocol session (ADR 20261109071720 decision 2).
//!
//! The session owns one page session and the child sessions auto-attach adds under it.
//! Replies settle pending calls. Events on the page session carry no session id; events on a
//! child carry the child's id. Messages for any other session (chromiumoxide's own, the
//! browser target) are not the session's and are dropped.
use crate::engine::port::{EngineError, EngineResult, ProtocolEvent};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use tokio::sync::oneshot;

/// The reply to one call: its result object, or the protocol error message.
pub(super) type Reply = Result<Value, String>;

/// Calls in flight, the sessions the protocol session owns and the events not yet drained.
#[derive(Default)]
pub(super) struct Routes {
    pending: HashMap<u64, oneshot::Sender<Reply>>,
    page: Option<String>,
    children: HashSet<String>,
    events: Vec<ProtocolEvent>,
    closed: bool,
}

impl Routes {
    /// Registers a call; replies to it go to the returned receiver.
    pub(super) fn expect(&mut self, id: u64) -> EngineResult<oneshot::Receiver<Reply>> {
        if self.closed {
            return Err(closed());
        }
        let (tx, rx) = oneshot::channel();
        self.pending.insert(id, tx);
        Ok(rx)
    }

    /// Drops a call that was never sent or timed out.
    pub(super) fn forget(&mut self, id: u64) {
        self.pending.remove(&id);
    }

    /// The page session events and calls without a session id belong to.
    pub(super) fn set_page(&mut self, session: String) {
        self.page = Some(session);
    }

    /// The wire session id for a call: the page for `None`, or an attached child.
    pub(super) fn session_for(&self, session: Option<&str>) -> EngineResult<String> {
        match session {
            None => self
                .page
                .clone()
                .ok_or_else(|| EngineError::Call("protocol session not attached".into())),
            Some(child) if self.children.contains(child) => Ok(child.to_owned()),
            Some(other) => Err(EngineError::Call(format!("no attached session {other}"))),
        }
    }

    pub(super) fn drain(&mut self) -> Vec<ProtocolEvent> {
        std::mem::take(&mut self.events)
    }

    /// Handles one message read from the connection.
    pub(super) fn route(&mut self, message: Value) {
        if let Some(id) = message.get("id").and_then(Value::as_u64) {
            if let Some(tx) = self.pending.remove(&id) {
                let reply = match message.get("error") {
                    Some(error) => Err(error["message"].as_str().unwrap_or("error").to_owned()),
                    None => Ok(message.get("result").cloned().unwrap_or_default()),
                };
                let _ = tx.send(reply);
            }
            return;
        }
        let (Some(method), Some(from)) = (
            message.get("method").and_then(Value::as_str),
            message.get("sessionId").and_then(Value::as_str),
        ) else {
            return;
        };
        let session = if self.page.as_deref() == Some(from) {
            None
        } else if self.children.contains(from) {
            Some(from.to_owned())
        } else {
            return;
        };
        let params = message.get("params").cloned().unwrap_or_default();
        let child = params["sessionId"].as_str().map(str::to_owned);
        match (method, child) {
            ("Target.attachedToTarget", Some(child)) => {
                self.children.insert(child);
            }
            ("Target.detachedFromTarget", Some(child)) => {
                self.children.remove(&child);
            }
            _ => {}
        }
        self.events.push(ProtocolEvent {
            session,
            method: method.to_owned(),
            params,
        });
    }

    /// The connection ended: every call in flight and every later call fails.
    pub(super) fn close(&mut self) {
        self.closed = true;
        for (_, tx) in self.pending.drain() {
            let _ = tx.send(Err("protocol connection closed".into()));
        }
    }
}

fn closed() -> EngineError {
    EngineError::Call("protocol connection closed".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn attached() -> Routes {
        let mut routes = Routes::default();
        routes.set_page("P".into());
        routes
    }

    #[test]
    fn replies_settle_their_call() {
        let mut routes = attached();
        let mut ok = routes.expect(1).unwrap();
        let mut failed = routes.expect(2).unwrap();
        routes.route(json!({"id": 1, "result": {"v": 1}}));
        routes.route(json!({"id": 2, "error": {"code": -32601, "message": "not found"}}));
        assert_eq!(ok.try_recv().unwrap(), Ok(json!({"v": 1})));
        assert_eq!(failed.try_recv().unwrap(), Err("not found".into()));
    }

    #[test]
    fn page_events_have_no_session_and_children_keep_theirs() {
        let mut routes = attached();
        routes.route(
            json!({"sessionId": "P", "method": "Target.attachedToTarget",
            "params": {"sessionId": "C", "targetInfo": {"type": "worker"}}}),
        );
        routes.route(json!({"sessionId": "C", "method": "Runtime.consoleAPICalled", "params": {}}));
        routes
            .route(json!({"sessionId": "X", "method": "Network.requestWillBeSent", "params": {}}));
        routes.route(json!({"method": "Target.targetCreated", "params": {}}));
        let events = routes.drain();
        let seen: Vec<_> = events
            .iter()
            .map(|e| (e.session.as_deref(), e.method.as_str()))
            .collect();
        assert_eq!(
            seen,
            [
                (None, "Target.attachedToTarget"),
                (Some("C"), "Runtime.consoleAPICalled")
            ]
        );
        assert!(routes.drain().is_empty());
    }

    #[test]
    fn calls_reach_the_page_or_an_attached_child_only() {
        let mut routes = Routes::default();
        assert!(routes.session_for(None).is_err(), "not attached yet");
        routes.set_page("P".into());
        assert_eq!(routes.session_for(None).unwrap(), "P");
        assert!(routes.session_for(Some("C")).is_err());
        routes.route(
            json!({"sessionId": "P", "method": "Target.attachedToTarget",
            "params": {"sessionId": "C"}}),
        );
        assert_eq!(routes.session_for(Some("C")).unwrap(), "C");
        routes.route(
            json!({"sessionId": "C", "method": "Target.attachedToTarget",
            "params": {"sessionId": "G"}}),
        );
        assert_eq!(routes.session_for(Some("G")).unwrap(), "G", "nested child");
        routes.route(
            json!({"sessionId": "P", "method": "Target.detachedFromTarget",
            "params": {"sessionId": "C"}}),
        );
        assert!(routes.session_for(Some("C")).is_err(), "detached");
    }

    #[test]
    fn closing_fails_calls_in_flight_and_later() {
        let mut routes = attached();
        let mut waiting = routes.expect(7).unwrap();
        routes.close();
        assert!(waiting.try_recv().unwrap().is_err());
        assert!(routes.expect(8).is_err());
    }
}

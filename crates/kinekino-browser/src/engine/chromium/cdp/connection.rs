//! The protocol session's own DevTools connection (ADR 20261109071720 decision 2).
//!
//! chromiumoxide hands events to typed listeners without the session they came from, and does
//! not route calls to children attached under a page. The protocol session therefore opens a
//! second connection to the same browser and attaches to the engine's page target there.
//! [`super::protocol::Routes`] decides what each message means.
use super::protocol::{Reply, Routes};
use crate::engine::port::{EngineError, EngineResult, ProtocolEvent};
use async_tungstenite::tungstenite::Message;
use futures::StreamExt;
use serde_json::{Value, json};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;
use tokio::sync::mpsc;

/// How long one call may wait for its reply.
const CALL_TIMEOUT: Duration = Duration::from_secs(30);

/// Domains enabled on the page session at attach, so network and frame events flow before
/// the caller enables anything. Other domains, and every child, are the caller's to enable.
const ENABLED: [&str; 2] = ["Page.enable", "Network.enable"];

pub(super) struct ProtocolConnection {
    out: mpsc::UnboundedSender<String>,
    routes: Arc<Mutex<Routes>>,
    next: AtomicU64,
}

fn call_error(e: impl std::fmt::Display) -> EngineError {
    EngineError::Call(e.to_string())
}

impl ProtocolConnection {
    /// Connects to the browser at `ws_url` and attaches to the page target `target_id`.
    pub(super) async fn attach(ws_url: &str, target_id: &str) -> EngineResult<Self> {
        let (socket, _) = async_tungstenite::tokio::connect_async(ws_url)
            .await
            .map_err(call_error)?;
        let (mut sink, mut stream) = socket.split();
        let (out, mut outgoing) = mpsc::unbounded_channel::<String>();
        tokio::spawn(async move {
            while let Some(text) = outgoing.recv().await {
                if sink.send(Message::text(text)).await.is_err() {
                    break;
                }
            }
        });
        let routes = Arc::new(Mutex::new(Routes::default()));
        let reader = Arc::clone(&routes);
        tokio::spawn(async move {
            while let Some(Ok(message)) = stream.next().await {
                let Message::Text(text) = message else {
                    continue;
                };
                if let Ok(value) = serde_json::from_str::<Value>(text.as_str()) {
                    lock(&reader).route(value);
                }
            }
            lock(&reader).close();
        });
        let connection = Self {
            out,
            routes,
            next: AtomicU64::new(1),
        };
        let attached = connection
            .send(
                None,
                "Target.attachToTarget",
                json!({"targetId": target_id, "flatten": true}),
            )
            .await?;
        let page = attached["sessionId"]
            .as_str()
            .ok_or_else(|| EngineError::Call("attach returned no session".into()))?;
        lock(&connection.routes).set_page(page.to_owned());
        for method in ENABLED {
            connection.call(None, method, json!({})).await?;
        }
        Ok(connection)
    }

    /// Sends `method` to the page (`None`) or an attached child and waits for the result.
    pub(super) async fn call(
        &self,
        session: Option<&str>,
        method: &str,
        params: Value,
    ) -> EngineResult<Value> {
        let wire = lock(&self.routes).session_for(session)?;
        self.send(Some(&wire), method, params).await
    }

    pub(super) fn drain(&self) -> Vec<ProtocolEvent> {
        lock(&self.routes).drain()
    }

    async fn send(
        &self,
        session: Option<&str>,
        method: &str,
        params: Value,
    ) -> EngineResult<Value> {
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        let reply = lock(&self.routes).expect(id)?;
        let mut message = json!({"id": id, "method": method, "params": params});
        if let Some(session) = session {
            message["sessionId"] = json!(session);
        }
        if self.out.send(message.to_string()).is_err() {
            lock(&self.routes).forget(id);
            return Err(EngineError::Call("protocol connection closed".into()));
        }
        let settled: Option<Reply> = match tokio::time::timeout(CALL_TIMEOUT, reply).await {
            Ok(received) => received.ok(),
            Err(_) => {
                lock(&self.routes).forget(id);
                return Err(EngineError::Call(format!("{method} timed out")));
            }
        };
        match settled {
            Some(Ok(result)) => Ok(result),
            Some(Err(message)) => Err(EngineError::Call(format!("{method}: {message}"))),
            None => Err(EngineError::Call("protocol connection closed".into())),
        }
    }
}

fn lock(routes: &Mutex<Routes>) -> MutexGuard<'_, Routes> {
    routes.lock().unwrap_or_else(|p| p.into_inner())
}

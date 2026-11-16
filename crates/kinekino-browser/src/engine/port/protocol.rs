//! DevTools protocol pass-through (ADR 20261109071720 decision 2).
//!
//! The browser provider reads network responses, response bodies, frames and child targets,
//! which the rest of the port does not carry. An adapter offers this only when it really
//! passes protocol calls through, and declares [`super::Capability::ProtocolSession`].

/// One protocol event, with the child session it came from when auto-attach is on.
#[derive(Debug, Clone, PartialEq)]
pub struct ProtocolEvent {
    pub session: Option<String>,
    pub method: String,
    pub params: serde_json::Value,
}

/// A DevTools protocol session on the engine's page target.
pub trait ProtocolSession {
    /// Sends `method` with JSON `params` to the page (`session: None`) or to an attached
    /// child session, and returns the result object.
    fn call(
        &mut self,
        session: Option<&str>,
        method: &str,
        params: serde_json::Value,
    ) -> super::EngineResult<serde_json::Value>;
    /// Drains protocol events observed since the last call.
    fn drain_protocol_events(&mut self) -> Vec<ProtocolEvent>;
    /// Attaches to child targets (frames, workers) as they appear, paused until resumed by
    /// the caller, so their events carry a session id.
    fn set_auto_attach(&mut self, on: bool) -> super::EngineResult<()>;
}

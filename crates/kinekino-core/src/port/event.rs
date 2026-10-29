//! Event convention: ports expose typed events by pull (`drain_events`), not callbacks.

pub trait EventSource {
    type Event;
    /// Events observed since the previous call, oldest first.
    fn drain_events(&mut self) -> Vec<Self::Event>;
}

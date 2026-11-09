//! The mascot event port and its no-op adapter.
use super::MascotEvent;

/// Receives app events for an optional mascot.
///
/// Fire and forget: `notify` returns at once, never blocks the caller's thread and never
/// reports failure, because a mascot is decoration and must not affect playback or jobs.
/// An adapter that talks to a mascot process queues or drops events on its own side.
pub trait MascotEventPort: Send + Sync {
    fn notify(&self, event: MascotEvent);
}

/// The default adapter: no mascot is installed, so every event is ignored.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoMascot;

impl MascotEventPort for NoMascot {
    fn notify(&self, _event: MascotEvent) {}
}

//! Mascot event port (plan item 39, roadmap M6-4): the host tells an optional mascot what the
//! app is doing. The mascot itself (motion presets, renderer) is outside this repository; the
//! host only emits app-level events and never waits for or depends on a mascot.
mod event;
mod port;

pub use event::MascotEvent;
pub use port::{MascotEventPort, NoMascot};

#[cfg(test)]
mod tests;

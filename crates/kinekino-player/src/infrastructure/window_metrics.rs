//! Read-only geometry of the actual player window (ADR 0009 port).
use super::os_policy::{native_window, window};
use anyhow::Result;
use kinekino_core::platform::window::NativeWindow;
use raw_window_handle::HasWindowHandle;
use serde::Serialize;

/// Geometry probe bound to the live player window, separate from monitor probes.
pub struct WindowMetrics(NativeWindow);

#[derive(Serialize)]
pub struct Snapshot {
    dpi: u32,
    window: [i32; 4],
    client: [i32; 4],
    monitor: [i32; 4],
    primary_monitor: bool,
}

impl WindowMetrics {
    pub fn new(parent: &impl HasWindowHandle) -> Result<Self> {
        Ok(Self(native_window(parent)?))
    }

    pub fn snapshot(&self) -> Result<Snapshot> {
        let g = window().geometry(self.0)?;
        Ok(Snapshot {
            dpi: g.dpi,
            window: g.window,
            client: g.client,
            monitor: g.monitor,
            primary_monitor: g.primary_monitor,
        })
    }
}

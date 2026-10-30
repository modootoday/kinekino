//! mpv render parent, created through the native window port (ADR 0009).
use super::os_policy::{native_window, window};
use anyhow::Result;
use raw_window_handle::HasWindowHandle;

/// Owns the child window used exclusively as mpv's rendering parent.
pub struct VideoSurface(Box<dyn kinekino_core::platform::window::VideoSurface>);
impl VideoSurface {
    pub fn new(parent: &impl HasWindowHandle) -> Result<Self> {
        Ok(Self(window().video_surface(native_window(parent)?)?))
    }
    pub fn id(&self) -> u32 {
        self.0.id()
    }
    pub fn layout(&self, bounds: [f64; 4], scale: f64, visible: bool) -> Result<()> {
        Ok(self.0.layout(bounds, scale, visible)?)
    }
}

//! Pointer activity over the foreground player window (ADR 0009 port).
use super::os_policy::{native_window, window};
use anyhow::Result;
use raw_window_handle::HasWindowHandle;
pub struct WindowActivity(Box<dyn kinekino_core::platform::window::PointerActivity>);
impl WindowActivity {
    pub fn new(parent: &impl HasWindowHandle) -> Result<Self> {
        Ok(Self(window().pointer_activity(native_window(parent)?)?))
    }
    pub fn pointer_moved(&mut self) -> bool {
        self.0.pointer_moved()
    }
}

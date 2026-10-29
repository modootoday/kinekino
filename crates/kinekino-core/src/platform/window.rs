//! Native window port (ADR 0009): mpv render parent, pointer activity and geometry.
//!
//! Handles are OS-neutral numbers; the adapter interprets them (`Win32` = HWND, `X11` = XID).
//! Objects are UI-thread owned, so they are not `Send`.
use std::io;

/// A live native window owned by the UI toolkit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeWindow {
    /// Win32 HWND value.
    Win32(u64),
    /// X11 window id.
    X11(u64),
}

/// Child surface used exclusively as the native video renderer's parent.
pub trait VideoSurface {
    /// Renderer-visible window id (mpv `--wid`).
    fn id(&self) -> u32;
    /// Places the surface in parent client coordinates (logical bounds × scale).
    fn layout(&self, bounds: [f64; 4], scale: f64, visible: bool) -> io::Result<()>;
}

/// Pointer movement over the foreground parent window.
pub trait PointerActivity {
    /// True when the pointer moved since the last call while inside the foreground parent.
    fn pointer_moved(&mut self) -> bool;
}

/// Read-only geometry snapshot: rectangles are `[left, top, right, bottom]` in device pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowGeometry {
    pub dpi: u32,
    pub window: [i32; 4],
    pub client: [i32; 4],
    pub monitor: [i32; 4],
    pub primary_monitor: bool,
}

/// Native window operations selected by the OS adapter.
pub trait WindowPort: Send + Sync {
    fn video_surface(&self, parent: NativeWindow) -> io::Result<Box<dyn VideoSurface>>;
    fn pointer_activity(&self, parent: NativeWindow) -> io::Result<Box<dyn PointerActivity>>;
    fn geometry(&self, window: NativeWindow) -> io::Result<WindowGeometry>;
}

/// Adapter for hosts without a native window implementation (capability false).
#[derive(Debug, Default)]
pub struct UnsupportedWindows;

fn unsupported() -> io::Error {
    io::Error::new(
        io::ErrorKind::Unsupported,
        "Native window port is not implemented on this OS",
    )
}

impl WindowPort for UnsupportedWindows {
    fn video_surface(&self, _: NativeWindow) -> io::Result<Box<dyn VideoSurface>> {
        Err(unsupported())
    }
    fn pointer_activity(&self, _: NativeWindow) -> io::Result<Box<dyn PointerActivity>> {
        Err(unsupported())
    }
    fn geometry(&self, _: NativeWindow) -> io::Result<WindowGeometry> {
        Err(unsupported())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unsupported_adapter_refuses_every_operation() {
        let port = UnsupportedWindows;
        let parent = NativeWindow::X11(7);
        assert!(port.video_surface(parent).is_err());
        assert!(port.pointer_activity(parent).is_err());
        assert_eq!(
            port.geometry(parent).unwrap_err().kind(),
            io::ErrorKind::Unsupported
        );
    }
}

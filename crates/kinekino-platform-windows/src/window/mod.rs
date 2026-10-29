//! Win32 native window adapter (ADR 0009), moved from the player.
use kinekino_core::platform::window::{
    NativeWindow, PointerActivity, VideoSurface, WindowGeometry, WindowPort,
};
use std::io;
use windows::{
    Win32::{
        Foundation::{HWND, POINT, RECT},
        Graphics::Gdi::{
            GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow,
        },
        UI::{
            HiDpi::GetDpiForWindow,
            Input::KeyboardAndMouse::{GetFocus, SetFocus},
            WindowsAndMessaging::*,
        },
    },
    core::w,
};

/// Win32 window port.
#[derive(Debug, Default)]
pub struct WindowsWindows;

fn hwnd(window: NativeWindow) -> io::Result<HWND> {
    match window {
        NativeWindow::Win32(value) => Ok(HWND(value as usize as *mut _)),
        NativeWindow::X11(_) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Win32 parent required",
        )),
    }
}

fn os(error: windows::core::Error) -> io::Error {
    io::Error::other(error)
}

struct Win32Surface(HWND, HWND);
impl VideoSurface for Win32Surface {
    fn id(&self) -> u32 {
        self.0.0 as usize as u32
    }
    fn layout(&self, bounds: [f64; 4], scale: f64, visible: bool) -> io::Result<()> {
        let [x, y, w, h] = bounds.map(|v| (v * scale).round() as i32);
        // SAFETY: this adapter owns the live child window on the UI thread.
        unsafe {
            SetWindowPos(
                self.0,
                None,
                x,
                y,
                w.max(1),
                h.max(1),
                SWP_NOACTIVATE | SWP_NOZORDER,
            )
            .map_err(os)?;
            // Keep the render-only child from taking keyboard shortcuts away from the Rust shell.
            if visible && GetForegroundWindow() == self.1 && GetFocus() != self.1 {
                let _ = SetFocus(Some(self.1));
            }
            let _ = ShowWindow(self.0, if visible { SW_SHOWNA } else { SW_HIDE });
        }
        Ok(())
    }
}
impl Drop for Win32Surface {
    fn drop(&mut self) {
        // SAFETY: the child is owned by this adapter; mpv is dropped first by PlayerApp.
        unsafe {
            let _ = DestroyWindow(self.0);
        }
    }
}

struct Win32Activity {
    parent: HWND,
    last: Option<(i32, i32)>,
}
impl PointerActivity for Win32Activity {
    fn pointer_moved(&mut self) -> bool {
        // SAFETY: output buffers are valid; parent belongs to the live UI window.
        unsafe {
            let mut point = POINT::default();
            let mut bounds = RECT::default();
            if GetCursorPos(&mut point).is_err() || GetWindowRect(self.parent, &mut bounds).is_err()
            {
                return false;
            }
            let pos = (point.x, point.y);
            let changed = self.last != Some(pos);
            self.last = Some(pos);
            changed
                && GetForegroundWindow() == self.parent
                && point.x >= bounds.left
                && point.x < bounds.right
                && point.y >= bounds.top
                && point.y < bounds.bottom
        }
    }
}

fn rect(r: RECT) -> [i32; 4] {
    [r.left, r.top, r.right, r.bottom]
}

impl WindowPort for WindowsWindows {
    fn video_surface(&self, parent: NativeWindow) -> io::Result<Box<dyn VideoSurface>> {
        let parent = hwnd(parent)?;
        // SAFETY: parent is borrowed from the live UI window; STATIC is a system class.
        let child = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("STATIC"),
                w!(""),
                WS_CHILD | WS_DISABLED | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
                0,
                0,
                1,
                1,
                Some(parent),
                None,
                None,
                None,
            )
        }
        .map_err(os)?;
        Ok(Box::new(Win32Surface(child, parent)))
    }
    fn pointer_activity(&self, parent: NativeWindow) -> io::Result<Box<dyn PointerActivity>> {
        Ok(Box::new(Win32Activity {
            parent: hwnd(parent)?,
            last: None,
        }))
    }
    fn geometry(&self, window: NativeWindow) -> io::Result<WindowGeometry> {
        let window = hwnd(window)?;
        // SAFETY: all buffers are initialized; the HWND is the live UI-thread window.
        unsafe {
            let mut outer = RECT::default();
            let mut client = RECT::default();
            GetWindowRect(window, &mut outer).map_err(os)?;
            GetClientRect(window, &mut client).map_err(os)?;
            let monitor = MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST);
            let mut info = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };
            GetMonitorInfoW(monitor, &mut info).ok().map_err(os)?;
            Ok(WindowGeometry {
                dpi: GetDpiForWindow(window),
                window: rect(outer),
                client: rect(client),
                monitor: rect(info.rcMonitor),
                primary_monitor: info.dwFlags & 1 != 0,
            })
        }
    }
}

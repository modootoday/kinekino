use anyhow::Result;
use serde::Serialize;
use std::mem::size_of;
use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{
            EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO, MONITORINFOEXW,
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            HiDpi::{
                DPI_AWARENESS_CONTEXT, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, GetDpiForWindow,
                SetThreadDpiAwarenessContext,
            },
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassW, WNDCLASSW,
                WS_EX_NOACTIVATE, WS_POPUP,
            },
        },
    },
    core::{BOOL, w},
};

#[derive(Serialize)]
struct Monitor {
    device: String,
    bounds: [i32; 4],
    primary: bool,
    probe_window_dpi: u32,
}

struct Awareness(DPI_AWARENESS_CONTEXT);

unsafe extern "system" fn window_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // SAFETY: forward the OS-supplied window message unchanged to the default procedure.
    unsafe { DefWindowProcW(window, message, wparam, lparam) }
}
impl Drop for Awareness {
    fn drop(&mut self) {
        // SAFETY: restore the previous context on the same thread.
        unsafe {
            SetThreadDpiAwarenessContext(self.0);
        }
    }
}

unsafe extern "system" fn collect(monitor: HMONITOR, _: HDC, _: *mut RECT, data: LPARAM) -> BOOL {
    // SAFETY: EnumDisplayMonitors invokes synchronously with the live vector pointer.
    unsafe {
        let monitors = &mut *(data.0 as *mut Vec<Monitor>);
        let mut info = MONITORINFOEXW::default();
        info.monitorInfo.cbSize = size_of::<MONITORINFOEXW>() as u32;
        if GetMonitorInfoW(
            monitor,
            (&mut info as *mut MONITORINFOEXW).cast::<MONITORINFO>(),
        )
        .as_bool()
        {
            let len = info
                .szDevice
                .iter()
                .position(|c| *c == 0)
                .unwrap_or(info.szDevice.len());
            let r = info.monitorInfo.rcMonitor;
            monitors.push(Monitor {
                device: String::from_utf16_lossy(&info.szDevice[..len]),
                bounds: [r.left, r.top, r.right, r.bottom],
                primary: info.monitorInfo.dwFlags & 1 != 0,
                probe_window_dpi: 0,
            });
        }
    }
    BOOL(1)
}

pub fn observe() -> Result<serde_json::Value> {
    // SAFETY: the context is thread-local; all probe windows are owned, hidden and destroyed.
    unsafe {
        let previous = SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        anyhow::ensure!(!previous.0.is_null(), "DPI awareness context unavailable");
        let _guard = Awareness(previous);
        let mut monitors = Vec::<Monitor>::new();
        EnumDisplayMonitors(
            None,
            None,
            Some(collect),
            LPARAM((&mut monitors as *mut Vec<Monitor>) as isize),
        )
        .ok()?;
        let instance = GetModuleHandleW(None)?;
        let class = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: instance.into(),
            lpszClassName: w!("SourceValidationDpiProbe"),
            ..Default::default()
        };
        anyhow::ensure!(
            RegisterClassW(&class) != 0,
            "cannot register owned DPI probe"
        );
        for monitor in &mut monitors {
            let window = CreateWindowExW(
                WS_EX_NOACTIVATE,
                w!("SourceValidationDpiProbe"),
                w!("Source DPI Probe"),
                WS_POPUP,
                monitor.bounds[0] + 16,
                monitor.bounds[1] + 16,
                256,
                128,
                None,
                None,
                Some(instance.into()),
                None,
            )?;
            monitor.probe_window_dpi = GetDpiForWindow(window);
            DestroyWindow(window)?;
        }
        Ok(
            serde_json::json!({"monitors":monitors,"awareness":"PER_MONITOR_AWARE_V2",
            "scope":"actual GetDpiForWindow on hidden owned probe per monitor; no scale/topology modification"}),
        )
    }
}

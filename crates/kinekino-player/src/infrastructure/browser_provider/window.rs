use super::error;
use crate::application::api::{ApiError, ErrorCode};
use std::num::NonZeroIsize;
use windows::{
    Win32::{
        Foundation::HWND,
        System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize},
        UI::WindowsAndMessaging::*,
    },
    core::w,
};
use wry::raw_window_handle::{HandleError, HasWindowHandle, Win32WindowHandle, WindowHandle};

pub(crate) struct Apartment;
impl Apartment {
    pub fn new() -> Result<Self, ApiError> {
        // SAFETY: worker owns this STA and drops all COM objects before Apartment.
        unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok() }
            .map_err(|_| error(ErrorCode::EngineFailed, "Browser STA initialization failed"))?;
        Ok(Self)
    }
}
impl Drop for Apartment {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}

pub(crate) struct HiddenWindow(HWND);
impl HiddenWindow {
    pub fn new() -> Result<Self, ApiError> {
        // SAFETY: system STATIC class; no WS_VISIBLE, no activation or user app handle.
        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                w!("STATIC"),
                w!(""),
                WS_OVERLAPPED | WS_CLIPCHILDREN,
                0,
                0,
                1000,
                800,
                None,
                None,
                None,
                None,
            )
        }
        .map_err(|_| {
            error(
                ErrorCode::EngineFailed,
                "Hidden browser parent creation failed",
            )
        })?;
        Ok(Self(hwnd))
    }
    pub fn pump(&self) {
        let mut message = MSG::default();
        // SAFETY: only this worker's message queue is consumed; no cross-thread GUI control.
        unsafe {
            for _ in 0..64 {
                if !PeekMessageW(&mut message, None, 0, 0, PM_REMOVE).as_bool() {
                    break;
                }
                let _ = TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
    }
}
impl HasWindowHandle for HiddenWindow {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let hwnd = NonZeroIsize::new(self.0.0 as isize).ok_or(HandleError::Unavailable)?;
        // SAFETY: borrowed handle cannot outlive this owned worker window.
        Ok(unsafe { WindowHandle::borrow_raw(Win32WindowHandle::new(hwnd).into()) })
    }
}
impl Drop for HiddenWindow {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.0);
        }
    }
}

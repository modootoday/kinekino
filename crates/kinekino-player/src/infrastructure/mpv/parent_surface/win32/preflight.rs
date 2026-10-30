use super::ProbeError;
use meari_native::infrastructure::window::NativeScopedParent;
use windows::Win32::{
    Foundation::HWND, System::Threading::GetCurrentThreadId,
    UI::WindowsAndMessaging::GetWindowThreadProcessId,
};
use wry::raw_window_handle::{HasWindowHandle, RawWindowHandle};

pub(super) fn parent(parent: &NativeScopedParent<'_>) -> Result<HWND, ProbeError> {
    let handle = parent
        .window_handle()
        .map_err(|_| ProbeError::HandleUnavailable)?;
    let RawWindowHandle::Win32(raw) = handle.as_raw() else {
        return Err(ProbeError::HandleUnavailable);
    };
    let hwnd = HWND(raw.hwnd.get() as *mut std::ffi::c_void);
    // SAFETY: The original scoped parent borrow remains live during preflight and creation.
    unsafe {
        require_thread(hwnd, GetCurrentThreadId())?;
    }
    Ok(hwnd)
}
pub(super) unsafe fn require_thread(hwnd: HWND, expected: u32) -> Result<(), ProbeError> {
    // SAFETY: This read does not dereference a handle in Rust. Zero/foreign thread is rejected.
    if unsafe { GetWindowThreadProcessId(hwnd, None) } != expected || expected == 0 {
        return Err(ProbeError::ForeignThread);
    }
    Ok(())
}

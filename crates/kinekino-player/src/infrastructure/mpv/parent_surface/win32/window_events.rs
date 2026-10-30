use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicIsize, AtomicU32, Ordering},
};
use windows::Win32::{
    Foundation::{GetLastError, HWND, LPARAM, LRESULT, SetLastError, WIN32_ERROR, WPARAM},
    UI::WindowsAndMessaging::*,
};

pub(super) struct Events {
    pub generation: u64,
    pub installed: AtomicBool,
    pub destroyed: AtomicBool,
    pub wake: AtomicBool,
    pub hwnd: AtomicIsize,
    pub detached: AtomicBool,
    pub destroy_entered: AtomicBool,
    pub callback_error: AtomicU32,
}
impl Events {
    pub fn new(generation: u64) -> Self {
        Self {
            generation,
            installed: AtomicBool::new(false),
            destroyed: AtomicBool::new(false),
            wake: AtomicBool::new(false),
            hwnd: AtomicIsize::new(0),
            detached: AtomicBool::new(false),
            destroy_entered: AtomicBool::new(false),
            callback_error: AtomicU32::new(0),
        }
    }
    pub fn complete(&self, generation: u64) -> bool {
        self.generation == generation
            && self.installed.load(Ordering::Acquire)
            && self.destroyed.load(Ordering::Acquire)
            && self.detached.load(Ordering::Acquire)
    }
}
pub(super) fn checked_write(write: impl FnOnce() -> (isize, u32)) -> Result<isize, u32> {
    let (previous, error) = write();
    if previous == 0 && error != 0 {
        Err(error)
    } else {
        Ok(previous)
    }
}
unsafe fn set_userdata(hwnd: HWND, value: isize) -> Result<isize, u32> {
    checked_write(|| unsafe {
        SetLastError(WIN32_ERROR(0));
        let previous = SetWindowLongPtrW(hwnd, GWLP_USERDATA, value);
        (previous, GetLastError().0)
    })
}
pub(super) fn install_events(
    events: &Events,
    hwnd: isize,
    write: impl FnOnce() -> Result<isize, u32>,
) -> bool {
    if let Err(error) = write() {
        events.callback_error.store(error, Ordering::Release);
        return false;
    }
    events.hwnd.store(hwnd, Ordering::Release);
    events.installed.store(true, Ordering::Release);
    true
}
/// Owns exactly one transferred Arc reference only after successful installation.
/// Failed detach deliberately keeps that reference alive; it never acknowledges completion.
pub(super) unsafe fn finish_destroy(
    events: *const Events,
    detach: impl FnOnce() -> Result<isize, u32>,
    downstream: impl FnOnce() -> LRESULT,
) -> LRESULT {
    unsafe {
        if (*events).destroy_entered.swap(true, Ordering::AcqRel) {
            return downstream();
        }
        let detached = match detach() {
            Ok(_) => {
                (*events).detached.store(true, Ordering::Release);
                true
            }
            Err(error) => {
                (*events).callback_error.store(error, Ordering::Release);
                false
            }
        };
        let result = downstream();
        (*events).destroyed.store(true, Ordering::Release);
        (*events).wake.store(true, Ordering::Release);
        if detached {
            drop(Arc::from_raw(events));
        }
        result
    }
}
pub(super) unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // SAFETY: Only our registered private class uses this procedure. Creation supplies one Arc ref.
    unsafe {
        if message == WM_NCCREATE {
            let create = &*(lparam.0 as *const CREATESTRUCTW);
            let events = create.lpCreateParams as *const Events;
            if events.is_null() {
                return LRESULT(0);
            }
            if !install_events(&*events, hwnd.0 as isize, || {
                set_userdata(hwnd, events as isize)
            }) {
                return LRESULT(0);
            }
        }
        let events = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Events;
        if message == WM_NCDESTROY && !events.is_null() {
            return finish_destroy(
                events,
                || set_userdata(hwnd, 0),
                || DefWindowProcW(hwnd, message, wparam, lparam),
            );
        }
        DefWindowProcW(hwnd, message, wparam, lparam)
    }
}

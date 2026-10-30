use super::{HiddenSurfaceProbe, ProbeError, preflight};
use std::sync::{Arc, atomic::Ordering};
use windows::Win32::{
    System::Threading::GetCurrentThreadId,
    UI::WindowsAndMessaging::{DestroyWindow, GWLP_USERDATA, GetWindowLongPtrW, UnregisterClassW},
};
use windows::core::PCWSTR;

#[derive(Debug, PartialEq, Eq)]
pub enum ProbeRelease {
    Complete { generation: u64 },
    Unobserved,
    NotCreated,
}
pub(super) fn destroy_then_observe<E>(
    destroy: impl FnOnce() -> Result<(), E>,
    observe: impl FnOnce() -> bool,
) -> Result<bool, E> {
    destroy()?;
    Ok(observe())
}
pub(super) fn release_class<E>(
    registered: &mut bool,
    release: impl FnOnce() -> Result<(), E>,
) -> Result<(), E> {
    if *registered {
        release()?;
        *registered = false;
    }
    Ok(())
}
impl HiddenSurfaceProbe {
    fn finish_class(&mut self) -> Result<(), ProbeError> {
        release_class(&mut self.class_registered, || unsafe {
            UnregisterClassW(PCWSTR(self.class.as_ptr()), Some(self.instance))
                .map_err(ProbeError::Os)
        })
    }
    /// Local OS-owner release only; not a NativeChild cleanup driver or an engine join proof.
    pub fn release(&mut self) -> Result<ProbeRelease, ProbeError> {
        self.synchronize_construction()?;
        if self.thread == 0 && !self.class_registered && self.hwnd.is_none() {
            return Ok(ProbeRelease::NotCreated);
        }
        // SAFETY: No foreign-thread DestroyWindow call is issued.
        if unsafe { GetCurrentThreadId() } != self.thread {
            return Err(ProbeError::ForeignThread);
        }
        if self.events.complete(self.generation) {
            self.hwnd = None;
            self.finish_class()?;
            return Ok(ProbeRelease::Complete {
                generation: self.generation,
            });
        }
        let Some(hwnd) = self.hwnd else {
            if !self.events.installed.load(Ordering::Acquire) {
                self.finish_class()?;
                return Ok(ProbeRelease::NotCreated);
            }
            return Ok(ProbeRelease::Unobserved);
        };
        let callback_error = self.events.callback_error.load(Ordering::Acquire);
        if callback_error != 0 {
            return Err(ProbeError::CallbackFault(callback_error));
        }
        // SAFETY: Recheck original callback state before destruction. No numeric-handle-only ACK.
        unsafe {
            preflight::require_thread(hwnd, self.thread)?;
            if GetWindowLongPtrW(hwnd, GWLP_USERDATA) != Arc::as_ptr(&self.events) as isize {
                return Err(ProbeError::ObservationMismatch);
            }
            destroy_then_observe(
                || DestroyWindow(hwnd).map_err(ProbeError::Os),
                || self.events.complete(self.generation),
            )?;
        }
        if self.events.complete(self.generation) {
            self.hwnd = None;
            self.finish_class()?;
            Ok(ProbeRelease::Complete {
                generation: self.generation,
            })
        } else {
            let callback_error = self.events.callback_error.load(Ordering::Acquire);
            if callback_error != 0 {
                return Err(ProbeError::CallbackFault(callback_error));
            }
            Ok(ProbeRelease::Unobserved)
        }
    }
}

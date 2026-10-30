//! Observe the process handle captured from this fixture's native WebView PID.
use crate::infrastructure::browser_provider::HiddenWindow;
use std::time::{Duration, Instant};
use windows::Win32::{
    Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT},
    System::Threading::{OpenProcess, PROCESS_SYNCHRONIZE, WaitForSingleObject},
};

pub(super) struct OwnedProcess(HANDLE);
impl OwnedProcess {
    pub(super) fn capture(pid: u32) -> Self {
        assert_ne!(pid, 0);
        // SAFETY: read-only synchronization handle for the PID supplied by our
        // live WebView, captured before releasing it; no process discovery/kill.
        Self(unsafe { OpenProcess(PROCESS_SYNCHRONIZE, false, pid) }.unwrap())
    }
    pub(super) fn wait(&self, window: &HiddenWindow) -> bool {
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            window.pump();
            // SAFETY: the owned synchronization handle is live for this entire wait.
            let result = unsafe { WaitForSingleObject(self.0, 0) };
            if result == WAIT_OBJECT_0 {
                return true;
            }
            assert_eq!(result, WAIT_TIMEOUT, "Owned process observation failed");
            if Instant::now() >= deadline {
                return false;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}
impl Drop for OwnedProcess {
    fn drop(&mut self) {
        // SAFETY: exactly one owner for the successful OpenProcess handle.
        let _ = unsafe { CloseHandle(self.0) };
    }
}

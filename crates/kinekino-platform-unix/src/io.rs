//! Unix raw-descriptor adapter.
use kinekino_core::platform::CloseFailure;
use std::fs::File;
use std::os::fd::IntoRawFd;

/// Moves `file` into raw ownership and closes exactly that original descriptor once.
pub fn close_original(file: File) -> Result<(), CloseFailure> {
    let raw = file.into_raw_fd();
    // SAFETY: `raw` was just moved out of a live File; nothing else owns or closes it.
    if unsafe { libc::close(raw) } == 0 {
        Ok(())
    } else {
        Err(CloseFailure {
            raw: raw as usize,
            code: std::io::Error::last_os_error().raw_os_error().unwrap_or(-1),
        })
    }
}

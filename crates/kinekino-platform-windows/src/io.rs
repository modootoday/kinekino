//! Windows raw-handle adapter.
use kinekino_core::platform::CloseFailure;
use std::fs::File;
use std::os::windows::io::IntoRawHandle;

/// Moves `file` into raw ownership and closes exactly that original handle once.
pub fn close_original(file: File) -> Result<(), CloseFailure> {
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    let raw = file.into_raw_handle();
    // SAFETY: `raw` was just moved out of a live File; nothing else owns or closes it.
    unsafe { CloseHandle(HANDLE(raw)) }.map_err(|error| CloseFailure {
        raw: raw as usize,
        code: error.code().0,
    })
}

use super::handles::error;
use std::{ffi::OsString, io, os::windows::ffi::OsStringExt, path::PathBuf};
use windows::{
    Win32::{
        Foundation::{FILETIME, HANDLE},
        System::Threading::{GetProcessTimes, PROCESS_NAME_WIN32, QueryFullProcessImageNameW},
    },
    core::PWSTR,
};

pub(super) fn creation(process: HANDLE) -> io::Result<u64> {
    let mut created = FILETIME::default();
    let mut exited = FILETIME::default();
    let mut kernel = FILETIME::default();
    let mut user = FILETIME::default();
    // SAFETY: live owned process and initialized FILETIME outputs.
    unsafe {
        GetProcessTimes(process, &mut created, &mut exited, &mut kernel, &mut user)
            .map_err(error)?;
    }
    Ok((u64::from(created.dwHighDateTime) << 32) | u64::from(created.dwLowDateTime))
}
pub(super) fn image(process: HANDLE) -> io::Result<PathBuf> {
    let mut buffer = vec![0u16; 32768];
    let mut length = buffer.len() as u32;
    // SAFETY: writable buffer, bounded length, live original process handle.
    unsafe {
        QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut length,
        )
        .map_err(error)?;
    }
    buffer.truncate(length as usize);
    Ok(PathBuf::from(OsString::from_wide(&buffer)))
}

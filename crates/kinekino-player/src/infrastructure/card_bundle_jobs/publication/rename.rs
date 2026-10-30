//! Same create-only FILE_RENAME_INFO pattern as the existing download adapter.
use crate::application::api::{ApiError, ErrorCode};
use std::{
    fs::File,
    os::windows::{ffi::OsStrExt, io::AsRawHandle},
    path::Path,
};
use windows::Win32::{
    Foundation::HANDLE,
    Storage::FileSystem::{FILE_RENAME_INFO, FileRenameInfo, SetFileInformationByHandle},
};
pub(super) fn rename_new(file: &File, target: &Path) -> Result<(), ApiError> {
    let invalid = || ApiError::new(ErrorCode::InvalidArgument, "Bundle output path is too long");
    let mut name: Vec<u16> = target.as_os_str().encode_wide().collect();
    let name_bytes = name.len().checked_mul(2).ok_or_else(invalid)?;
    let name_length = u32::try_from(name_bytes).map_err(|_| invalid())?;
    name.push(0);
    let offset = std::mem::offset_of!(FILE_RENAME_INFO, FileName);
    let bytes = offset
        .checked_add(name_bytes.checked_add(2).ok_or_else(invalid)?)
        .ok_or_else(invalid)?;
    let length = u32::try_from(bytes).map_err(|_| invalid())?;
    let mut storage = vec![
        0_usize;
        bytes
            .max(std::mem::size_of::<FILE_RENAME_INFO>())
            .div_ceil(std::mem::size_of::<usize>())
    ];
    let info = storage.as_mut_ptr().cast::<FILE_RENAME_INFO>();
    // Zero initialization leaves ReplaceIfExists=false; preserve the original owned handle.
    unsafe {
        (*info).FileNameLength = name_length;
        std::ptr::copy_nonoverlapping(
            name.as_ptr(),
            storage.as_mut_ptr().cast::<u8>().add(offset).cast::<u16>(),
            name.len(),
        );
        SetFileInformationByHandle(
            HANDLE(file.as_raw_handle()),
            FileRenameInfo,
            info.cast(),
            length,
        )
    }
    .map_err(|e| {
        if target.symlink_metadata().is_ok() {
            ApiError::new(
                ErrorCode::RevisionConflict,
                "Bundle output appeared before publication; no overwrite",
            )
        } else {
            kinekino_core::file_publication::io_error(std::io::Error::from_raw_os_error(
                (e.code().0 as u32 & 0xffff) as i32,
            ))
        }
    })
}

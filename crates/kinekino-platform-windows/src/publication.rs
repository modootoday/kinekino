//! Windows complete-file publication: handle rename via `FILE_RENAME_INFO`, delete-on-close locks.
use std::fs::{File, OpenOptions};
use std::io;
use std::os::windows::{ffi::OsStrExt, fs::OpenOptionsExt, io::AsRawHandle};
use std::path::Path;
use windows::Win32::{
    Foundation::HANDLE,
    Storage::FileSystem::{FILE_RENAME_INFO, FileRenameInfo, SetFileInformationByHandle},
};

const GENERIC_READ_WRITE_DELETE: u32 = 0xC001_0000;
const GENERIC_WRITE_DELETE: u32 = 0x4001_0000;
const FILE_SHARE_READ: u32 = 1;
const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
const FILE_FLAG_DELETE_ON_CLOSE: u32 = 0x0400_0000;
const FILE_ATTRIBUTE_TEMPORARY: u32 = 0x100;

/// New staging file holding `DELETE` so the open handle can be renamed; readers may share.
pub fn create_stage(path: &Path, readable: bool) -> io::Result<File> {
    OpenOptions::new()
        .write(true)
        .access_mode(if readable {
            GENERIC_READ_WRITE_DELETE
        } else {
            GENERIC_WRITE_DELETE
        })
        .create_new(true)
        .share_mode(FILE_SHARE_READ)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
}

/// `SetFileInformationByHandle(FileRenameInfo)` with `ReplaceIfExists = false`.
pub fn rename_no_replace(file: &File, _staged: &Path, target: &Path) -> io::Result<()> {
    let invalid = || io::Error::from(io::ErrorKind::InvalidFilename);
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
    // SAFETY: `storage` is zeroed (ReplaceIfExists = false), aligned and large enough for the
    // header plus the NUL-terminated name; the handle stays owned by `file`.
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
    .map_err(|error| io::Error::from_raw_os_error((error.code().0 as u32 & 0xffff) as i32))
}

/// Exclusive, delete-on-close temporary file; an existing (held) lock fails with `AlreadyExists`.
pub fn create_transient_lock(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .access_mode(GENERIC_READ_WRITE_DELETE)
        .share_mode(0)
        .custom_flags(FILE_FLAG_DELETE_ON_CLOSE | FILE_ATTRIBUTE_TEMPORARY)
        .open(path)
}

use std::{
    fs::File,
    io,
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
};
use windows::Win32::{
    Foundation::{DUPLICATE_SAME_ACCESS, DuplicateHandle, HANDLE},
    System::Threading::GetCurrentProcess,
};

pub(super) struct Handle(OwnedHandle);
impl Handle {
    pub(super) fn inheritable(&self) -> io::Result<Self> {
        let mut target = HANDLE::default();
        unsafe {
            DuplicateHandle(
                GetCurrentProcess(),
                self.raw(),
                GetCurrentProcess(),
                &mut target,
                0,
                true,
                DUPLICATE_SAME_ACCESS,
            )
            .map_err(error)?;
        }
        Ok(unsafe { Self::from_raw(target) })
    }
    /// Caller transfers a newly returned owned kernel handle exactly once.
    pub(super) unsafe fn from_raw(raw: HANDLE) -> Self {
        // SAFETY: delegated caller ownership contract; no pseudo handles are accepted internally.
        Self(unsafe { OwnedHandle::from_raw_handle(raw.0) })
    }
    pub(super) fn raw(&self) -> HANDLE {
        HANDLE(self.0.as_raw_handle())
    }
    pub(super) fn into_file(self) -> File {
        File::from(self.0)
    }
    pub(super) fn duplicate_private(&self) -> io::Result<Self> {
        let mut target = HANDLE::default();
        // SAFETY: original owned handle; private duplicate is explicitly non-inheritable.
        unsafe {
            DuplicateHandle(
                GetCurrentProcess(),
                self.raw(),
                GetCurrentProcess(),
                &mut target,
                0,
                false,
                DUPLICATE_SAME_ACCESS,
            )
            .map_err(error)?;
        }
        Ok(unsafe { Self::from_raw(target) })
    }
    pub(super) fn inheritable_file(file: &File) -> io::Result<Self> {
        let mut target = HANDLE::default();
        // SAFETY: live file/current-process handles and writable output; duplicate is owned here.
        unsafe {
            DuplicateHandle(
                GetCurrentProcess(),
                HANDLE(file.as_raw_handle()),
                GetCurrentProcess(),
                &mut target,
                0,
                true,
                DUPLICATE_SAME_ACCESS,
            )
            .map_err(error)?;
        }
        Ok(unsafe { Self::from_raw(target) })
    }
    /// Private duplicate of a regular readable file with read-only rights (child stdin).
    pub(super) fn read_only_duplicate(file: &File) -> io::Result<File> {
        use std::io::{Read, Seek, SeekFrom};
        use windows::Win32::{
            Foundation::DUPLICATE_HANDLE_OPTIONS, Storage::FileSystem::FILE_GENERIC_READ,
        };
        let mut probe = file;
        probe.seek(SeekFrom::Start(0))?;
        // Empty regular readers are valid here; this only probes read permission.
        let _readability_probe_bytes = probe.read(&mut [0; 1])?;
        probe.seek(SeekFrom::Start(0))?;
        let mut reader = HANDLE::default();
        // SAFETY: original caller-owned readable file; private duplicate removes write/delete.
        unsafe {
            DuplicateHandle(
                GetCurrentProcess(),
                HANDLE(file.as_raw_handle()),
                GetCurrentProcess(),
                &mut reader,
                FILE_GENERIC_READ.0,
                false,
                DUPLICATE_HANDLE_OPTIONS(0),
            )
            .map_err(error)?;
        }
        Ok(unsafe { Self::from_raw(reader) }.into_file())
    }
}
pub(super) fn error(error: windows::core::Error) -> io::Error {
    io::Error::other(error)
}

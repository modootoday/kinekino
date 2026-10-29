//! Owned source lanes as single-instance overlapped named pipes, local clients only.
pub(super) mod operation;
use self::operation::Operation;
use super::handles::{Handle, error};
use kinekino_core::platform::process::{
    PipeOperation, SourceChildEndpoint, SourcePipe, SourcePipePair,
};
use std::{
    any::Any,
    io,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use windows::{
    Win32::{
        Foundation::{
            GENERIC_READ, GENERIC_WRITE, GetHandleInformation, HANDLE, HANDLE_FLAG_INHERIT,
            INVALID_HANDLE_VALUE,
        },
        Storage::FileSystem::{
            CreateFileW, FILE_FLAG_FIRST_PIPE_INSTANCE, FILE_FLAG_OVERLAPPED, FILE_SHARE_MODE,
            OPEN_EXISTING, PIPE_ACCESS_DUPLEX,
        },
        System::Pipes::{
            CreateNamedPipeW, GetNamedPipeClientProcessId, GetNamedPipeInfo, PIPE_READMODE_BYTE,
            PIPE_REJECT_REMOTE_CLIENTS, PIPE_TYPE_BYTE, PIPE_WAIT,
        },
    },
    core::PCWSTR,
};

static NEXT: AtomicU64 = AtomicU64::new(0);

/// Child-side original pipe client handle (non-inheritable until spawn duplicates it).
pub(super) struct ChildPipe(pub(super) Handle);
impl SourceChildEndpoint for ChildPipe {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct WindowsPipe {
    handle: Arc<Handle>,
    server: bool,
}
impl SourcePipe for WindowsPipe {
    fn connect(self: Arc<Self>) -> io::Result<Box<dyn PipeOperation>> {
        Ok(Box::new(Operation::connect(self.handle.clone())?))
    }
    fn read(self: Arc<Self>, maximum: usize) -> io::Result<Box<dyn PipeOperation>> {
        Ok(Box::new(Operation::read(self.handle.clone(), maximum)?))
    }
    fn write(self: Arc<Self>, bytes: &[u8]) -> io::Result<Box<dyn PipeOperation>> {
        Ok(Box::new(Operation::write(self.handle.clone(), bytes)?))
    }
    fn verify_peer(&self) -> io::Result<()> {
        if !self.server {
            return Ok(());
        }
        let mut client = 0;
        unsafe {
            GetNamedPipeClientProcessId(self.handle.raw(), &mut client).map_err(error)?;
        }
        if client != std::process::id() {
            return Err(io::Error::other("unexpected source pipe client"));
        }
        Ok(())
    }
    fn is_inheritable(&self) -> io::Result<bool> {
        let mut flags = 0;
        unsafe {
            GetHandleInformation(self.handle.raw(), &mut flags).map_err(error)?;
        }
        Ok(flags & HANDLE_FLAG_INHERIT.0 != 0)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub(super) fn pair(maximum: usize) -> io::Result<SourcePipePair> {
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    let name = format!("\\\\.\\pipe\\owned-source-{}-{id}", std::process::id());
    let wide: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
    let server = unsafe {
        CreateNamedPipeW(
            PCWSTR(wide.as_ptr()),
            PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED | FILE_FLAG_FIRST_PIPE_INSTANCE,
            PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
            1,
            maximum as u32,
            maximum as u32,
            0,
            None,
        )
    };
    if server == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    let server = Arc::new(unsafe { Handle::from_raw(server) });
    let client = unsafe {
        CreateFileW(
            PCWSTR(wide.as_ptr()),
            GENERIC_READ.0 | GENERIC_WRITE.0,
            FILE_SHARE_MODE(0),
            None,
            OPEN_EXISTING,
            FILE_FLAG_OVERLAPPED,
            None,
        )
        .map_err(error)?
    };
    let client = unsafe { Handle::from_raw(client) };
    Ok(SourcePipePair {
        parent: Arc::new(WindowsPipe {
            handle: server,
            server: true,
        }),
        child: Box::new(ChildPipe(client)),
        name: Some(name),
        // Do not start an overlapped connect before both lanes reach their close owner.
        needs_connect: true,
    })
}

/// # Safety
/// `value` is the original inherited pipe handle, transferred exactly once.
pub(super) unsafe fn adopt(value: usize) -> io::Result<Arc<dyn SourcePipe>> {
    if value == 0 || value == usize::MAX {
        return Err(io::Error::other("invalid source endpoint"));
    }
    let handle = unsafe { Handle::from_raw(HANDLE(value as *mut std::ffi::c_void)) };
    unsafe {
        GetNamedPipeInfo(handle.raw(), None, None, None, None).map_err(error)?;
    }
    Ok(Arc::new(WindowsPipe {
        handle: Arc::new(handle),
        server: false,
    }))
}

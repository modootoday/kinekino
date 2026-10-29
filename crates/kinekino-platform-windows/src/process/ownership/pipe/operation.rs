use super::super::handles::Handle;
use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use windows::{
    Win32::{
        Foundation::{
            ERROR_BROKEN_PIPE, ERROR_IO_INCOMPLETE, ERROR_IO_PENDING, ERROR_OPERATION_ABORTED,
            ERROR_PIPE_CONNECTED,
        },
        Storage::FileSystem::{ReadFile, WriteFile},
        System::{
            IO::{CancelIoEx, GetOverlappedResult, OVERLAPPED},
            Pipes::ConnectNamedPipe,
            Threading::CreateEventW,
        },
    },
    core::PCWSTR,
};

static EMERGENCY_RETAINED: AtomicU64 = AtomicU64::new(0);
pub(in crate::process::ownership) fn emergency_retained_operations() -> u64 {
    EMERGENCY_RETAINED.load(Ordering::Relaxed)
}

struct KernelOperation {
    overlapped: OVERLAPPED,
    buffer: Box<[u8]>,
    _event: Handle,
    pipe: Arc<Handle>,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Connect,
    Read,
    Write,
}
#[must_use = "Poll completion after cancel; a request is not a completion"]
pub(in crate::process::ownership) struct Operation {
    inner: Option<Box<KernelOperation>>,
    kind: Kind,
    pending: bool,
    result: Option<Result<Completion, io::Error>>,
}
use kinekino_core::platform::process::{PipeCompletion as Completion, PipeOperation};
// The allocation and its internal OVERLAPPED/event/buffer never move after a
// kernel call. Exclusive &mut access serializes poll/cancel; no Sync is exposed.
unsafe impl Send for Operation {}

impl Operation {
    fn new(pipe: Arc<Handle>, buffer: Box<[u8]>, kind: Kind) -> io::Result<Self> {
        let event = unsafe {
            Handle::from_raw(
                CreateEventW(None, true, false, PCWSTR::null()).map_err(io::Error::other)?,
            )
        };
        let overlapped = OVERLAPPED {
            hEvent: event.raw(),
            ..Default::default()
        };
        Ok(Self {
            inner: Some(Box::new(KernelOperation {
                overlapped,
                buffer,
                _event: event,
                pipe,
            })),
            kind,
            pending: false,
            result: None,
        })
    }
    pub(in crate::process::ownership) fn read(
        pipe: Arc<Handle>,
        maximum: usize,
    ) -> io::Result<Self> {
        let mut operation = Self::new(pipe, vec![0; maximum].into_boxed_slice(), Kind::Read)?;
        let inner = operation.inner.as_mut().expect("new operation");
        let result = unsafe {
            ReadFile(
                inner.pipe.raw(),
                Some(&mut inner.buffer),
                None,
                Some(&mut inner.overlapped),
            )
        };
        operation.started(result);
        Ok(operation)
    }
    pub(in crate::process::ownership) fn write(
        pipe: Arc<Handle>,
        bytes: &[u8],
    ) -> io::Result<Self> {
        let mut operation = Self::new(pipe, bytes.into(), Kind::Write)?;
        let inner = operation.inner.as_mut().expect("new operation");
        let result = unsafe {
            WriteFile(
                inner.pipe.raw(),
                Some(&inner.buffer),
                None,
                Some(&mut inner.overlapped),
            )
        };
        operation.started(result);
        Ok(operation)
    }
    pub(in crate::process::ownership) fn connect(pipe: Arc<Handle>) -> io::Result<Self> {
        let mut operation = Self::new(pipe, Vec::new().into_boxed_slice(), Kind::Connect)?;
        let inner = operation.inner.as_mut().expect("new operation");
        let result = unsafe { ConnectNamedPipe(inner.pipe.raw(), Some(&mut inner.overlapped)) };
        if result
            .as_ref()
            .is_err_and(|e| e.code() == ERROR_PIPE_CONNECTED.to_hresult())
        {
            operation.result = Some(Ok(Completion {
                bytes: Vec::new(),
                count: 0,
                eof: false,
                cancelled: false,
            }));
        } else {
            operation.started(result);
        }
        Ok(operation)
    }
    fn started(&mut self, result: windows::core::Result<()>) {
        match result {
            Ok(()) => self.pending = true, // Reap even synchronous success using the same OVERLAPPED.
            Err(error) if error.code() == ERROR_IO_PENDING.to_hresult() => self.pending = true,
            Err(error)
                if error.code() == ERROR_BROKEN_PIPE.to_hresult() && self.kind == Kind::Read =>
            {
                self.result = Some(Ok(Completion {
                    bytes: Vec::new(),
                    count: 0,
                    eof: true,
                    cancelled: false,
                }))
            }
            Err(error) => self.result = Some(Err(io::Error::other(error))),
        }
    }
    fn cancel_pending(&mut self) -> io::Result<()> {
        if !self.pending {
            return Ok(());
        }
        let inner = self.inner.as_mut().expect("pending operation");
        match unsafe { CancelIoEx(inner.pipe.raw(), Some(&inner.overlapped)) } {
            Ok(()) => Ok(()),
            Err(error)
                if error.code() == windows::Win32::Foundation::ERROR_NOT_FOUND.to_hresult() =>
            {
                Ok(())
            }
            Err(error) => Err(io::Error::other(error)),
        }
    }
    fn poll_once(&mut self) -> Option<Result<Completion, io::Error>> {
        if let Some(result) = self.result.take() {
            return Some(result);
        }
        if !self.pending {
            return Some(Err(io::Error::other("operation result already consumed")));
        }
        let inner = self.inner.as_mut().expect("pending allocation");
        let mut count = 0;
        let result =
            unsafe { GetOverlappedResult(inner.pipe.raw(), &inner.overlapped, &mut count, false) };
        if result
            .as_ref()
            .is_err_and(|e| e.code() == ERROR_IO_INCOMPLETE.to_hresult())
        {
            return None;
        }
        // Unknown observation errors are not proof that kernel references ended.
        if result.is_ok()
            || result.as_ref().is_err_and(|e| {
                e.code() == ERROR_OPERATION_ABORTED.to_hresult()
                    || e.code() == ERROR_BROKEN_PIPE.to_hresult()
            })
        {
            self.pending = false;
        }
        match result {
            Ok(()) => Some(
                checked_count(self.kind, inner.buffer.len(), count).map(|count| Completion {
                    bytes: if self.kind == Kind::Read {
                        inner.buffer[..count].to_vec()
                    } else {
                        Vec::new()
                    },
                    count,
                    eof: self.kind == Kind::Read && count == 0,
                    cancelled: false,
                }),
            ),
            Err(error) if error.code() == ERROR_OPERATION_ABORTED.to_hresult() => {
                Some(Ok(Completion {
                    bytes: Vec::new(),
                    count: 0,
                    eof: false,
                    cancelled: true,
                }))
            }
            Err(error)
                if error.code() == ERROR_BROKEN_PIPE.to_hresult() && self.kind == Kind::Read =>
            {
                Some(Ok(Completion {
                    bytes: Vec::new(),
                    count: 0,
                    eof: true,
                    cancelled: false,
                }))
            }
            Err(error) => Some(Err(io::Error::other(error))),
        }
    }
}
impl PipeOperation for Operation {
    fn poll(&mut self) -> Option<io::Result<Completion>> {
        self.poll_once()
    }
    fn cancel(&mut self) -> io::Result<()> {
        self.cancel_pending()
    }
    fn reaped(&self) -> bool {
        !self.pending
    }
}
fn checked_count(kind: Kind, capacity: usize, reported: u32) -> io::Result<usize> {
    // Microsoft GetOverlappedResult: ConnectNamedPipe's transferred count is
    // undefined. Never compare it to a zero-length connect buffer or emit EOF.
    if kind == Kind::Connect {
        return Ok(0);
    }
    let count = reported as usize;
    if count > capacity {
        Err(io::Error::other(
            "overlapped byte count exceeds original buffer",
        ))
    } else {
        Ok(count)
    }
}

impl Drop for Operation {
    fn drop(&mut self) {
        if !self.pending {
            return;
        }
        let _cancel = self.cancel_pending();
        let _observation = self.poll_once();
        if self.pending {
            // Emergency memory safety only: preserve the ORIGINAL allocation,
            // event and pipe. No wait, pump, new owner or cleanup ACK. Explicit
            // poll_close is required for normal reclamation; this path leaks.
            if let Some(inner) = self.inner.take() {
                std::mem::forget(inner);
            }
            EMERGENCY_RETAINED.fetch_add(1, Ordering::Relaxed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn connect_count_is_undefined_and_never_read_as_bytes_or_eof() {
        assert_eq!(checked_count(Kind::Connect, 0, u32::MAX).unwrap(), 0);
        assert!(checked_count(Kind::Read, 4, 5).is_err());
        assert!(checked_count(Kind::Write, 4, 5).is_err());
        assert_eq!(checked_count(Kind::Read, 4, 4).unwrap(), 4);
    }
}

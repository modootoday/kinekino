//! Named-pipe adapter for the IPC port (ADR 0007). Moved from `kinekino-player` local_host.
//!
//! Address `\\.\pipe\<EndpointId::name>`. Owner-only DACL `D:P(A;;GA;;;<sid>)`, remote clients
//! rejected, at most two instances (one connected, one listening) so the name stays owned.
//! Peer identity: `GetNamedPipe{Client,Server}ProcessId`, `GetProcessTimes` creation time and the
//! token's SID / integrity RID / elevation; per message the client is impersonated.
pub(crate) mod token;

use kinekino_core::platform::ipc::{
    Accept, EndpointId, IpcCapabilities, IpcFailure, IpcListener, IpcStream, IpcTransport,
    MessagePeer, PeerIdentity, PeerProcess,
};
use std::{
    ffi::c_void,
    io,
    os::windows::io::AsRawHandle,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::windows::named_pipe::{ClientOptions, NamedPipeClient, NamedPipeServer, ServerOptions},
};
use windows::{
    Win32::{
        Foundation::{CloseHandle, FILETIME, HANDLE, HLOCAL, LocalFree},
        Security::{
            Authorization::{
                ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
            },
            Cryptography::{BCRYPT_USE_SYSTEM_PREFERRED_RNG, BCryptGenRandom},
            PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES,
        },
        System::{
            Pipes::{GetNamedPipeClientProcessId, GetNamedPipeServerProcessId},
            Threading::{
                GetCurrentProcess, GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
            },
        },
    },
    core::PCWSTR,
};

pub(crate) fn fail(operation: &'static str, error: windows::core::Error) -> IpcFailure {
    IpcFailure::new(operation, win32_error(error))
}
/// Diagnostics carry Win32 codes (e.g. 233 ERROR_PIPE_NOT_CONNECTED). `windows::core::Error`
/// converts to an `io::Error` holding the HRESULT (`0x8007_xxxx` for FACILITY_WIN32), so unwrap
/// that facility back to the Win32 code; any other HRESULT is kept as-is.
fn win32_error(error: windows::core::Error) -> io::Error {
    let hresult = error.code().0 as u32;
    if hresult & 0xFFFF_0000 == 0x8007_0000 {
        io::Error::from_raw_os_error((hresult & 0xFFFF) as i32)
    } else {
        io::Error::from(error)
    }
}
pub(crate) fn invalid(operation: &'static str) -> IpcFailure {
    IpcFailure::new(operation, io::Error::from(io::ErrorKind::PermissionDenied))
}

struct OwnedHandle(HANDLE);
impl Drop for OwnedHandle {
    fn drop(&mut self) {
        // SAFETY: the handle was opened by this module and is closed once.
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}
pub(crate) struct LocalAllocation(pub(crate) *mut c_void);
impl Drop for LocalAllocation {
    fn drop(&mut self) {
        // SAFETY: LocalAlloc'd by the API that returned it.
        unsafe {
            let _ = LocalFree(Some(HLOCAL(self.0)));
        }
    }
}

fn start_time(process: HANDLE) -> Result<u64, IpcFailure> {
    let (mut creation, mut exit, mut kernel, mut user) = (
        FILETIME::default(),
        FILETIME::default(),
        FILETIME::default(),
        FILETIME::default(),
    );
    // SAFETY: all out-pointers are valid locals.
    unsafe {
        GetProcessTimes(process, &mut creation, &mut exit, &mut kernel, &mut user)
            .map_err(|e| fail("GetProcessTimes", e))?;
    }
    Ok(((creation.dwHighDateTime as u64) << 32) | creation.dwLowDateTime as u64)
}
fn open(pid: u32, operation: &'static str) -> Result<OwnedHandle, IpcFailure> {
    // SAFETY: plain query-limited open; the handle is owned by the guard.
    Ok(OwnedHandle(unsafe {
        OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
            .map_err(|e| fail(operation, e))?
    }))
}

/// Named-pipe transport.
#[derive(Debug, Default, Clone, Copy)]
pub struct WindowsIpc;

/// Same SID, integrity and UAC state; the local token itself must be a sane interactive one.
pub fn permits(local: &PeerIdentity, peer: &PeerIdentity) -> bool {
    !local.principal.is_empty()
        && matches!(local.tier, 0x1000 | 0x2000 | 0x2100 | 0x3000 | 0x4000)
        && matches!(local.elevation_kind, 1..=3)
        && !(local.elevation_kind == 2 && !local.elevated)
        && !(local.elevation_kind == 3 && local.elevated)
        && local == peer
}

pub(crate) fn create_instance(path: &str, first: bool) -> Result<NamedPipeServer, IpcFailure> {
    let sid = token::current_sid()?;
    let descriptor: Vec<u16> = format!("D:P(A;;GA;;;{sid})")
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let mut security = PSECURITY_DESCRIPTOR::default();
    // SAFETY: NUL-terminated SDDL; the descriptor is freed by the guard below.
    unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            PCWSTR(descriptor.as_ptr()),
            SDDL_REVISION_1,
            &mut security,
            None,
        )
        .map_err(|e| fail("ConvertStringSecurityDescriptor", e))?;
    }
    let _allocation = LocalAllocation(security.0);
    let attributes = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: security.0,
        bInheritHandle: false.into(),
    };
    // SAFETY: Windows copies the descriptor while creating the non-inheritable pipe handle.
    unsafe {
        ServerOptions::new()
            .first_pipe_instance(first)
            .max_instances(2)
            .reject_remote_clients(true)
            .in_buffer_size(65536)
            .out_buffer_size(65536)
            .create_with_security_attributes_raw(
                path,
                (&attributes as *const SECURITY_ATTRIBUTES)
                    .cast_mut()
                    .cast(),
            )
            .map_err(|e| IpcFailure::new("CreateNamedPipe", e))
    }
}

/// The previous session's instance can briefly outlive its drop while its cancelled I/O
/// drains; with `max_instances(2)` creation then reports ERROR_PIPE_BUSY. Retry inside a
/// bounded window instead of failing the whole accept loop; any other failure is immediate.
async fn next_instance(path: &str) -> io::Result<NamedPipeServer> {
    const ERROR_PIPE_BUSY: i32 = 231;
    const ATTEMPTS: u32 = 200;
    for _ in 0..ATTEMPTS {
        match create_instance(path, false) {
            Ok(pipe) => return Ok(pipe),
            Err(failure) if failure.source.raw_os_error() == Some(ERROR_PIPE_BUSY) => {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            Err(failure) => return Err(failure.source),
        }
    }
    create_instance(path, false).map_err(|f| f.source)
}

impl IpcTransport for WindowsIpc {
    fn capabilities(&self) -> IpcCapabilities {
        IpcCapabilities {
            local_only: true,
            owner_only_acl: true,
            message_peer: MessagePeer::PerMessage,
            raw_client: true,
        }
    }
    fn process_start(&self, process_id: u32) -> io::Result<u64> {
        let process = open(process_id, "OpenProcess").map_err(|f| f.source)?;
        start_time(process.0).map_err(|f| f.source)
    }
    fn fresh_endpoint(&self, product: &str) -> io::Result<EndpointId> {
        let mut nonce = [0u8; 16];
        // SAFETY: writes exactly `nonce.len()` bytes.
        unsafe {
            BCryptGenRandom(None, &mut nonce, BCRYPT_USE_SYSTEM_PREFERRED_RNG)
                .ok()
                .map_err(io::Error::from)?;
        }
        // SAFETY: pseudo-handle, never closed.
        let start = start_time(unsafe { GetCurrentProcess() }).map_err(|f| f.source)?;
        Ok(EndpointId {
            product: product.into(),
            process_id: std::process::id(),
            process_start: start,
            nonce,
        })
    }
    fn address(&self, endpoint: &EndpointId) -> io::Result<String> {
        let name = endpoint.name();
        EndpointId::parse(&endpoint.product, &name)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "bad endpoint"))?;
        Ok(format!(r"\\.\pipe\{name}"))
    }
    fn listen(&self, endpoint: &EndpointId) -> Result<Box<dyn IpcListener>, IpcFailure> {
        let path = self
            .address(endpoint)
            .map_err(|e| IpcFailure::new("address", e))?;
        let current = create_instance(&path, true)?;
        Ok(Box::new(PipeListener {
            path,
            current: Some(current),
        }))
    }
    fn connect(&self, endpoint: &EndpointId) -> Result<Box<dyn IpcStream>, IpcFailure> {
        let path = self
            .address(endpoint)
            .map_err(|e| IpcFailure::new("address", e))?;
        self.connect_address(&path)
    }
    fn connect_address(&self, address: &str) -> Result<Box<dyn IpcStream>, IpcFailure> {
        let client = ClientOptions::new().open(address).map_err(|e| {
            // All instances busy: report the portable "retry shortly" kind (ERROR_PIPE_BUSY).
            if e.raw_os_error() == Some(231) {
                IpcFailure::new(
                    "CreateFile_pipe",
                    io::Error::new(io::ErrorKind::WouldBlock, e),
                )
            } else {
                IpcFailure::new("CreateFile_pipe", e)
            }
        })?;
        Ok(Box::new(PipeStream::Client(client)))
    }
    fn local_peer(&self) -> Result<PeerIdentity, IpcFailure> {
        token::current()
    }
    fn permits(&self, local: &PeerIdentity, peer: &PeerIdentity) -> bool {
        permits(local, peer)
    }
    fn ensure_thread_identity(&self) -> Result<(), IpcFailure> {
        token::require_unimpersonated()
    }
}

pub(crate) fn cancel(pipe: &NamedPipeServer) {
    // SAFETY: this owner exclusively owns the pipe; cancel its overlapped I/O.
    unsafe {
        let _ = windows::Win32::System::IO::CancelIoEx(HANDLE(pipe.as_raw_handle()), None);
    }
    let _ = pipe.disconnect();
}

struct PipeListener {
    path: String,
    current: Option<NamedPipeServer>,
}
impl IpcListener for PipeListener {
    fn accept(&mut self) -> Accept<'_> {
        Box::pin(async move {
            let pipe = self
                .current
                .as_ref()
                .ok_or_else(|| io::Error::new(io::ErrorKind::Interrupted, "listener cancelled"))?;
            pipe.connect().await?;
            // Create the next instance before handing out the connected one, so at least one
            // server handle owns the name at all times.
            let next = next_instance(&self.path).await?;
            let connected = self.current.replace(next).expect("current instance");
            Ok(Box::new(PipeStream::Server(connected)) as Box<dyn IpcStream>)
        })
    }
    fn cancel(&self) {
        if let Some(pipe) = &self.current {
            cancel(pipe);
        }
    }
}
impl Drop for PipeListener {
    fn drop(&mut self) {
        IpcListener::cancel(self);
    }
}

enum PipeStream {
    Server(NamedPipeServer),
    Client(NamedPipeClient),
}
impl PipeStream {
    fn handle(&self) -> HANDLE {
        match self {
            Self::Server(p) => HANDLE(p.as_raw_handle()),
            Self::Client(p) => HANDLE(p.as_raw_handle()),
        }
    }
}
impl Drop for PipeStream {
    fn drop(&mut self) {
        if let Self::Server(pipe) = self {
            cancel(pipe);
        }
    }
}
impl IpcStream for PipeStream {
    fn peer(&self) -> Result<PeerProcess, IpcFailure> {
        let mut pid = 0;
        let (operation, open_op) = match self {
            Self::Server(_) => ("GetNamedPipeClientProcessId", "OpenProcess_client"),
            Self::Client(_) => ("GetNamedPipeServerProcessId", "OpenProcess_server"),
        };
        // SAFETY: valid pipe handle and out-pointer.
        unsafe {
            match self {
                Self::Server(_) => GetNamedPipeClientProcessId(self.handle(), &mut pid),
                Self::Client(_) => GetNamedPipeServerProcessId(self.handle(), &mut pid),
            }
            .map_err(|e| fail(operation, e))?;
        }
        let process = open(pid, open_op)?;
        Ok(PeerProcess {
            process_id: pid,
            process_start: start_time(process.0)?,
            identity: token::process(process.0)?,
        })
    }
    fn message_peer(&mut self) -> Result<PeerIdentity, IpcFailure> {
        match self {
            Self::Server(_) => token::message(self.handle()),
            Self::Client(_) => self.peer().map(|p| p.identity),
        }
    }
    fn try_write_now(&self, bytes: &[u8]) -> io::Result<usize> {
        match self {
            Self::Server(p) => p.try_write(bytes),
            Self::Client(p) => p.try_write(bytes),
        }
    }
}
impl AsyncRead for PipeStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        match self.get_mut() {
            Self::Server(p) => Pin::new(p).poll_read(cx, buf),
            Self::Client(p) => Pin::new(p).poll_read(cx, buf),
        }
    }
}
impl AsyncWrite for PipeStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match self.get_mut() {
            Self::Server(p) => Pin::new(p).poll_write(cx, buf),
            Self::Client(p) => Pin::new(p).poll_write(cx, buf),
        }
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.get_mut() {
            Self::Server(p) => Pin::new(p).poll_flush(cx),
            Self::Client(p) => Pin::new(p).poll_flush(cx),
        }
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.get_mut() {
            Self::Server(p) => Pin::new(p).poll_shutdown(cx),
            Self::Client(p) => Pin::new(p).poll_shutdown(cx),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn facility_win32_hresults_report_the_win32_code() {
        let not_connected =
            windows::core::Error::from_hresult(windows::core::HRESULT(0x8007_00E9_u32 as i32));
        assert_eq!(fail("op", not_connected).source.raw_os_error(), Some(233));
        let other =
            windows::core::Error::from_hresult(windows::core::HRESULT(0x8000_4005_u32 as i32));
        assert_eq!(
            fail("op", other).source.raw_os_error(),
            Some(0x8000_4005_u32 as i32)
        );
    }
    fn id(sid: &str, tier: u32, elevated: bool, kind: i32) -> PeerIdentity {
        PeerIdentity {
            principal: sid.into(),
            tier,
            elevated,
            elevation_kind: kind,
        }
    }
    #[test]
    fn equal_sid_does_not_authorize_cross_integrity_or_elevation() {
        let medium = id("S-1-5-21-1", 0x2000, false, 3);
        let high = id("S-1-5-21-1", 0x3000, true, 2);
        assert!(permits(&medium, &medium.clone()));
        assert!(!permits(&medium, &high));
        assert!(!permits(&high, &medium));
        assert!(!permits(&medium, &id("S-1-5-21-2", 0x2000, false, 3)));
    }
    #[test]
    fn real_pipe_rejects_second_first_instance_and_reports_own_peer() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let _entered = runtime.enter();
        let ipc = WindowsIpc;
        let endpoint = ipc.fresh_endpoint("kinekino").unwrap();
        let _listener = ipc.listen(&endpoint).unwrap();
        assert!(ipc.listen(&endpoint).is_err());
        let client = ipc.connect(&endpoint).unwrap();
        let peer = client.peer().unwrap();
        assert_eq!(peer.process_id, std::process::id());
        assert_eq!(peer.process_start, endpoint.process_start);
        assert!(ipc.permits(&ipc.local_peer().unwrap(), &peer.identity));
    }
}

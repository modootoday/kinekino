//! Unix-domain-socket adapter for the IPC port (ADR 0007).
//!
//! Address: `$XDG_RUNTIME_DIR/<product>-ipc/<nonce>.sock`. The directory is created `0700`, must be
//! owned by the current uid and must not be group/other accessible; there is no `/tmp` fallback.
//! Peer identity is the kernel `SO_PEERCRED` uid/pid plus the `/proc/<pid>/stat` start time.
use kinekino_core::platform::ipc::{
    Accept, EndpointId, IpcCapabilities, IpcFailure, IpcListener, IpcStream, IpcTransport,
    MessagePeer, PeerIdentity, PeerProcess,
};
use std::{
    io,
    os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll},
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::{UnixListener, UnixStream},
    sync::Notify,
};

/// Unix IPC transport. `runtime_dir: None` reads `XDG_RUNTIME_DIR` at each call.
#[derive(Debug, Default, Clone)]
pub struct UnixIpc {
    pub runtime_dir: Option<PathBuf>,
}

fn current_uid() -> u32 {
    // SAFETY: getuid has no preconditions and cannot fail.
    unsafe { libc::getuid() }
}

pub fn identity_of(uid: u32) -> PeerIdentity {
    PeerIdentity {
        principal: format!("uid:{uid}"),
        tier: 0,
        elevated: uid == 0,
        elevation_kind: 0,
    }
}

/// Start time (clock ticks since boot) of `pid` from `/proc/<pid>/stat` field 22.
pub fn process_start(pid: u32) -> io::Result<u64> {
    if !cfg!(target_os = "linux") {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "process start time is only implemented for Linux /proc",
        ));
    }
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat"))?;
    parse_start(&stat).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "bad /proc stat"))
}

fn parse_start(stat: &str) -> Option<u64> {
    // comm (field 2) may contain spaces and ')'; fields after the last ')' start at field 3.
    let rest = &stat[stat.rfind(')')? + 1..];
    rest.split_whitespace()
        .nth(19)?
        .parse()
        .ok()
        .map(|s: u64| s.max(1))
}

/// Verifies `dir` is a real directory owned by `uid` with no group/other permissions.
pub fn check_private_dir(dir: &Path, uid: u32) -> io::Result<()> {
    let meta = std::fs::symlink_metadata(dir)?;
    if !meta.is_dir() || meta.uid() != uid || meta.mode() & 0o077 != 0 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("{} is not a private directory of uid {uid}", dir.display()),
        ));
    }
    Ok(())
}

impl UnixIpc {
    fn socket_dir(&self, product: &str) -> io::Result<PathBuf> {
        let base = match &self.runtime_dir {
            Some(dir) => dir.clone(),
            None => std::env::var_os("XDG_RUNTIME_DIR")
                .filter(|v| !v.is_empty())
                .map(PathBuf::from)
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "XDG_RUNTIME_DIR unset"))?,
        };
        let uid = current_uid();
        check_private_dir(&base, uid)?;
        let dir = base.join(format!("{product}-ipc"));
        match std::fs::DirBuilder::new().mode(0o700).create(&dir) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e),
        }
        check_private_dir(&dir, uid)?;
        Ok(dir)
    }
    fn path(&self, endpoint: &EndpointId) -> io::Result<PathBuf> {
        let name = endpoint.name();
        if EndpointId::parse(&endpoint.product, &name).is_none()
            || endpoint.product.is_empty()
            || endpoint.product.starts_with('.')
            || !endpoint
                .product
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "bad endpoint"));
        }
        // sun_path is 108 bytes on Linux (104 on BSD/macOS); the nonce alone is unique and the
        // pid/start part of the id is verified against the kernel peer, not the file name.
        let nonce: String = endpoint.nonce.iter().map(|b| format!("{b:02x}")).collect();
        let path = self
            .socket_dir(&endpoint.product)?
            .join(format!("{nonce}.sock"));
        if path.as_os_str().len() >= 104 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "socket path exceeds sun_path",
            ));
        }
        Ok(path)
    }
}

/// Adapter policy: same uid, and never a non-root/root mix (no privilege brokering).
pub fn permits(local: &PeerIdentity, peer: &PeerIdentity) -> bool {
    local.principal == peer.principal && local.elevated == peer.elevated
}

impl IpcTransport for UnixIpc {
    fn capabilities(&self) -> IpcCapabilities {
        IpcCapabilities {
            local_only: true,
            owner_only_acl: true,
            message_peer: MessagePeer::ConnectionBound,
            raw_client: true,
        }
    }
    fn process_start(&self, process_id: u32) -> io::Result<u64> {
        process_start(process_id)
    }
    fn fresh_endpoint(&self, product: &str) -> io::Result<EndpointId> {
        Ok(EndpointId {
            product: product.into(),
            process_id: std::process::id(),
            process_start: process_start(std::process::id())?,
            nonce: crate::id::fresh_guid()?,
        })
    }
    fn address(&self, endpoint: &EndpointId) -> io::Result<String> {
        self.path(endpoint)?
            .into_os_string()
            .into_string()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "non-UTF-8 socket path"))
    }
    fn listen(&self, endpoint: &EndpointId) -> Result<Box<dyn IpcListener>, IpcFailure> {
        let path = self
            .path(endpoint)
            .map_err(|e| IpcFailure::new("socket_dir", e))?;
        let listener = UnixListener::bind(&path).map_err(|e| IpcFailure::new("bind", e))?;
        let guard = SocketFile(path);
        std::fs::set_permissions(&guard.0, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| IpcFailure::new("chmod", e))?;
        Ok(Box::new(UnixIpcListener {
            listener,
            cancelled: Arc::default(),
            notify: Arc::default(),
            _file: guard,
        }))
    }
    fn connect(&self, endpoint: &EndpointId) -> Result<Box<dyn IpcStream>, IpcFailure> {
        let path = self
            .path(endpoint)
            .map_err(|e| IpcFailure::new("socket_dir", e))?;
        connect_path(&path)
    }
    fn connect_address(&self, address: &str) -> Result<Box<dyn IpcStream>, IpcFailure> {
        connect_path(Path::new(address))
    }
    fn local_peer(&self) -> Result<PeerIdentity, IpcFailure> {
        Ok(identity_of(current_uid()))
    }
    fn permits(&self, local: &PeerIdentity, peer: &PeerIdentity) -> bool {
        permits(local, peer)
    }
}

fn connect_path(path: &Path) -> Result<Box<dyn IpcStream>, IpcFailure> {
    let std =
        std::os::unix::net::UnixStream::connect(path).map_err(|e| IpcFailure::new("connect", e))?;
    std.set_nonblocking(true)
        .map_err(|e| IpcFailure::new("set_nonblocking", e))?;
    let stream = UnixStream::from_std(std).map_err(|e| IpcFailure::new("register", e))?;
    Ok(Box::new(UnixIpcStream(stream)))
}

/// Removes the socket file when the listener goes away.
struct SocketFile(PathBuf);
impl Drop for SocketFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

struct UnixIpcListener {
    listener: UnixListener,
    cancelled: Arc<AtomicBool>,
    notify: Arc<Notify>,
    _file: SocketFile,
}
impl IpcListener for UnixIpcListener {
    fn accept(&mut self) -> Accept<'_> {
        Box::pin(async move {
            let cancelled = || io::Error::new(io::ErrorKind::Interrupted, "listener cancelled");
            if self.cancelled.load(Ordering::Acquire) {
                return Err(cancelled());
            }
            let notified = self.notify.notified();
            tokio::select! {
                accepted = self.listener.accept() => {
                    let (stream, _) = accepted?;
                    Ok(Box::new(UnixIpcStream(stream)) as Box<dyn IpcStream>)
                }
                _ = notified => Err(cancelled()),
            }
        })
    }
    fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
        self.notify.notify_waiters();
    }
}

struct UnixIpcStream(UnixStream);
impl IpcStream for UnixIpcStream {
    fn peer(&self) -> Result<PeerProcess, IpcFailure> {
        let cred = self
            .0
            .peer_cred()
            .map_err(|e| IpcFailure::new("SO_PEERCRED", e))?;
        let pid = cred
            .pid()
            .and_then(|p| u32::try_from(p).ok())
            .filter(|p| *p != 0)
            .ok_or_else(|| {
                IpcFailure::new(
                    "SO_PEERCRED",
                    io::Error::new(io::ErrorKind::Unsupported, "peer pid unavailable"),
                )
            })?;
        Ok(PeerProcess {
            process_id: pid,
            process_start: process_start(pid).map_err(|e| IpcFailure::new("/proc stat", e))?,
            identity: identity_of(cred.uid()),
        })
    }
    fn message_peer(&mut self) -> Result<PeerIdentity, IpcFailure> {
        self.peer().map(|p| p.identity)
    }
    fn try_write_now(&self, bytes: &[u8]) -> io::Result<usize> {
        self.0.try_write(bytes)
    }
}
impl AsyncRead for UnixIpcStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}
impl AsyncWrite for UnixIpcStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}

#[cfg(test)]
mod tests;

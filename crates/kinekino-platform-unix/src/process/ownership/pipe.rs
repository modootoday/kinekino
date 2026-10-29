//! Owned source lanes as an anonymous `socketpair(AF_UNIX, SOCK_STREAM)`: no filesystem name,
//! close-on-exec by default, the child end is inherited by exactly one child.
use kinekino_core::platform::process::{
    PipeCompletion, PipeOperation, SourceChildEndpoint, SourcePipe, SourcePipePair,
};
use std::{
    any::Any,
    io,
    os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd},
    sync::Arc,
};

pub(super) struct ChildSocket(pub(super) OwnedFd);
impl SourceChildEndpoint for ChildSocket {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn check(result: libc::c_int) -> io::Result<libc::c_int> {
    if result < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(result)
    }
}
fn set_cloexec_nonblocking(fd: RawFd, nonblocking: bool) -> io::Result<()> {
    // SAFETY: fcntl on a descriptor owned by the caller.
    unsafe {
        let flags = check(libc::fcntl(fd, libc::F_GETFD))?;
        check(libc::fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC))?;
        let status = check(libc::fcntl(fd, libc::F_GETFL))?;
        let status = if nonblocking {
            status | libc::O_NONBLOCK
        } else {
            status & !libc::O_NONBLOCK
        };
        check(libc::fcntl(fd, libc::F_SETFL, status))?;
    }
    #[cfg(target_vendor = "apple")]
    {
        let one: libc::c_int = 1;
        // SAFETY: valid option buffer on an owned socket.
        check(unsafe {
            libc::setsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_NOSIGPIPE,
                (&one as *const libc::c_int).cast(),
                size_of::<libc::c_int>() as libc::socklen_t,
            )
        })?;
    }
    Ok(())
}

pub(super) fn pair() -> io::Result<SourcePipePair> {
    let mut fds = [0 as RawFd; 2];
    #[cfg(target_os = "linux")]
    let kind = libc::SOCK_STREAM | libc::SOCK_CLOEXEC;
    #[cfg(not(target_os = "linux"))]
    let kind = libc::SOCK_STREAM;
    // SAFETY: valid two-element output array.
    check(unsafe { libc::socketpair(libc::AF_UNIX, kind, 0, fds.as_mut_ptr()) })?;
    // SAFETY: socketpair returned two new descriptors owned exactly once here.
    let (parent, child) = unsafe { (OwnedFd::from_raw_fd(fds[0]), OwnedFd::from_raw_fd(fds[1])) };
    set_cloexec_nonblocking(parent.as_raw_fd(), true)?;
    set_cloexec_nonblocking(child.as_raw_fd(), false)?;
    Ok(SourcePipePair {
        parent: Arc::new(UnixSocket(parent)),
        child: Box::new(ChildSocket(child)),
        name: None,
        needs_connect: false,
    })
}

/// # Safety
/// `value` is the original inherited descriptor, transferred exactly once.
pub(super) unsafe fn adopt(value: usize) -> io::Result<Arc<dyn SourcePipe>> {
    let fd = RawFd::try_from(value).map_err(|_| io::Error::other("invalid source endpoint"))?;
    if fd < 3 {
        return Err(io::Error::other("invalid source endpoint"));
    }
    // SAFETY: zeroed stat output for fstat.
    let mut stat: libc::stat = unsafe { std::mem::zeroed() };
    check(unsafe { libc::fstat(fd, &mut stat) })?;
    if stat.st_mode & libc::S_IFMT != libc::S_IFSOCK {
        return Err(io::Error::other(
            "inherited source endpoint is not a socket",
        ));
    }
    // SAFETY: delegated exactly-once ownership contract.
    let owned = unsafe { OwnedFd::from_raw_fd(fd) };
    set_cloexec_nonblocking(fd, true)?;
    Ok(Arc::new(UnixSocket(owned)))
}

struct UnixSocket(OwnedFd);
impl SourcePipe for UnixSocket {
    fn connect(self: Arc<Self>) -> io::Result<Box<dyn PipeOperation>> {
        Ok(Box::new(SocketOperation::done(self, Kind::Connect)))
    }
    fn read(self: Arc<Self>, maximum: usize) -> io::Result<Box<dyn PipeOperation>> {
        Ok(Box::new(SocketOperation::new(self, Kind::Read(maximum))))
    }
    fn write(self: Arc<Self>, bytes: &[u8]) -> io::Result<Box<dyn PipeOperation>> {
        Ok(Box::new(SocketOperation::new(
            self,
            Kind::Write(bytes.to_vec()),
        )))
    }
    fn verify_peer(&self) -> io::Result<()> {
        #[cfg(target_os = "linux")]
        {
            let mut cred = libc::ucred {
                pid: 0,
                uid: 0,
                gid: 0,
            };
            let mut length = size_of::<libc::ucred>() as libc::socklen_t;
            // SAFETY: correctly sized ucred output on an owned socket.
            check(unsafe {
                libc::getsockopt(
                    self.0.as_raw_fd(),
                    libc::SOL_SOCKET,
                    libc::SO_PEERCRED,
                    (&mut cred as *mut libc::ucred).cast(),
                    &mut length,
                )
            })?;
            // socketpair credentials are those of the creating process (this one).
            if cred.uid != unsafe { libc::geteuid() } {
                return Err(io::Error::other("unexpected source socket peer"));
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            let (mut uid, mut gid) = (0, 0);
            // SAFETY: valid outputs on an owned socket.
            check(unsafe { libc::getpeereid(self.0.as_raw_fd(), &mut uid, &mut gid) })?;
            if uid != unsafe { libc::geteuid() } {
                return Err(io::Error::other("unexpected source socket peer"));
            }
        }
        Ok(())
    }
    fn is_inheritable(&self) -> io::Result<bool> {
        // SAFETY: fcntl query on an owned descriptor.
        let flags = check(unsafe { libc::fcntl(self.0.as_raw_fd(), libc::F_GETFD) })?;
        Ok(flags & libc::FD_CLOEXEC == 0)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

enum Kind {
    Connect,
    Read(usize),
    Write(Vec<u8>),
}
/// One readiness-polled non-blocking operation; the kernel never retains its buffer.
struct SocketOperation {
    socket: Arc<UnixSocket>,
    kind: Kind,
    cancelled: bool,
    finished: bool,
}
impl SocketOperation {
    fn new(socket: Arc<UnixSocket>, kind: Kind) -> Self {
        Self {
            socket,
            kind,
            cancelled: false,
            finished: false,
        }
    }
    fn done(socket: Arc<UnixSocket>, kind: Kind) -> Self {
        Self::new(socket, kind)
    }
    fn attempt(&mut self) -> Option<io::Result<PipeCompletion>> {
        let fd = self.socket.0.as_raw_fd();
        #[cfg(target_os = "linux")]
        let flags = libc::MSG_NOSIGNAL;
        #[cfg(not(target_os = "linux"))]
        let flags = 0;
        match &self.kind {
            Kind::Connect => Some(Ok(PipeCompletion::default())),
            Kind::Read(maximum) => {
                let mut buffer = vec![0u8; *maximum];
                // SAFETY: writable buffer of the stated length on an owned socket.
                let count = unsafe { libc::recv(fd, buffer.as_mut_ptr().cast(), buffer.len(), 0) };
                if count < 0 {
                    return would_block(io::Error::last_os_error());
                }
                buffer.truncate(count as usize);
                Some(Ok(PipeCompletion {
                    count: count as usize,
                    eof: count == 0,
                    bytes: buffer,
                    cancelled: false,
                }))
            }
            Kind::Write(bytes) => {
                // SAFETY: readable buffer of the stated length on an owned socket.
                let count = unsafe { libc::send(fd, bytes.as_ptr().cast(), bytes.len(), flags) };
                if count < 0 {
                    let error = io::Error::last_os_error();
                    return matches!(
                        error.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                    )
                    .then_some(())
                    .map_or(Some(Err(error)), |()| None);
                }
                Some(Ok(PipeCompletion {
                    count: count as usize,
                    ..PipeCompletion::default()
                }))
            }
        }
    }
}
fn would_block(error: io::Error) -> Option<io::Result<PipeCompletion>> {
    match error.kind() {
        io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted => None,
        io::ErrorKind::BrokenPipe | io::ErrorKind::ConnectionReset => Some(Ok(PipeCompletion {
            eof: true,
            ..PipeCompletion::default()
        })),
        _ => Some(Err(error)),
    }
}
impl PipeOperation for SocketOperation {
    fn poll(&mut self) -> Option<io::Result<PipeCompletion>> {
        if self.finished {
            return Some(Err(io::Error::other("operation result already consumed")));
        }
        let result = if self.cancelled {
            Some(Ok(PipeCompletion {
                cancelled: true,
                ..PipeCompletion::default()
            }))
        } else {
            self.attempt()
        };
        self.finished = result.is_some();
        result
    }
    fn cancel(&mut self) -> io::Result<()> {
        if !self.finished {
            self.cancelled = true;
        }
        Ok(())
    }
    fn reaped(&self) -> bool {
        self.finished
    }
}

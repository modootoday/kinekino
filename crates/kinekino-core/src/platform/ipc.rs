//! Local IPC transport port (ADR 0007). One adapter per OS implements [`IpcTransport`]; domain
//! crates reach it only through [`super::OsPolicy::ipc`].
//!
//! Endpoint identity ([`EndpointId`]) is OS-neutral. The adapter alone maps it to an address
//! (Windows `\\.\pipe\…`, Unix a filesystem socket in a private `0700` directory) and decides
//! which peers are acceptable ([`IpcTransport::permits`]). `listen`/`connect` register with the
//! ambient tokio reactor, so they must run inside a tokio runtime context.
use std::{fmt, future::Future, io, pin::Pin};
use tokio::io::{AsyncRead, AsyncWrite};

/// OS-neutral identity of one host endpoint. Not a path, pipe name or URL.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EndpointId {
    pub product: String,
    pub process_id: u32,
    /// Adapter-defined process start stamp ([`IpcTransport::process_start`]); never zero.
    pub process_start: u64,
    pub nonce: [u8; 16],
}
impl EndpointId {
    /// Canonical name `<product>.v1.<pid>.<start:016x>.<nonce:032x>`.
    pub fn name(&self) -> String {
        let nonce: String = self.nonce.iter().map(|b| format!("{b:02x}")).collect();
        format!(
            "{}.v1.{}.{:016x}.{nonce}",
            self.product, self.process_id, self.process_start
        )
    }
    /// Parses [`Self::name`] for `product`; rejects anything non-canonical.
    pub fn parse(product: &str, name: &str) -> Option<Self> {
        let rest = name.strip_prefix(product)?.strip_prefix(".v1.")?;
        let parts: Vec<&str> = rest.split('.').collect();
        let [pid, start, nonce] = parts.as_slice() else {
            return None;
        };
        let hex = |s: &str| {
            s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        };
        if pid.is_empty()
            || pid.len() > 10
            || pid.starts_with('0')
            || !pid.bytes().all(|b| b.is_ascii_digit())
            || start.len() != 16
            || nonce.len() != 32
            || !hex(start)
            || !hex(nonce)
            || name.len() > 160
        {
            return None;
        }
        let process_start = u64::from_str_radix(start, 16).ok().filter(|s| *s != 0)?;
        let mut bytes = [0u8; 16];
        for (i, byte) in bytes.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&nonce[i * 2..i * 2 + 2], 16).ok()?;
        }
        Some(Self {
            product: product.into(),
            process_id: pid.parse().ok()?,
            process_start,
            nonce: bytes,
        })
    }
}

/// OS-derived peer identity, never taken from request bytes or endpoint text.
/// Windows: user SID, mandatory integrity RID, elevation flag/type. Unix: `uid:<n>`, tier 0,
/// `elevated = uid == 0`, `elevation_kind` 0.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeerIdentity {
    pub principal: String,
    pub tier: u32,
    pub elevated: bool,
    pub elevation_kind: i32,
}

/// The connected peer process as reported by the kernel for this connection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeerProcess {
    pub process_id: u32,
    pub process_start: u64,
    pub identity: PeerIdentity,
}

/// A failed OS step, named for diagnostics (e.g. `GetNamedPipeServerProcessId`, `SO_PEERCRED`).
#[derive(Debug)]
pub struct IpcFailure {
    pub operation: &'static str,
    pub source: io::Error,
    /// The calling thread's security context could not be restored; the host must stop.
    pub fatal: bool,
}
impl IpcFailure {
    pub fn new(operation: &'static str, source: io::Error) -> Self {
        Self {
            operation,
            source,
            fatal: false,
        }
    }
    pub fn fatal(operation: &'static str, source: io::Error) -> Self {
        Self {
            operation,
            source,
            fatal: true,
        }
    }
}
impl fmt::Display for IpcFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.operation, self.source)
    }
}
impl std::error::Error for IpcFailure {}

/// How the per-message peer check is backed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessagePeer {
    /// Re-derived per message from the OS (Windows client impersonation token).
    PerMessage,
    /// Fixed at connect time by the kernel (Unix `SO_PEERCRED`); a message cannot change it.
    ConnectionBound,
}

/// What this adapter really guarantees.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IpcCapabilities {
    /// Remote (network) clients cannot reach the endpoint.
    pub local_only: bool,
    /// Access is restricted to the owner principal by an OS ACL / file mode.
    pub owner_only_acl: bool,
    pub message_peer: MessagePeer,
    /// The adapter can also connect to foreign servers by raw address (mpv JSON IPC).
    pub raw_client: bool,
}

/// One connected byte stream.
pub trait IpcStream: AsyncRead + AsyncWrite + Unpin + Send {
    /// The other end's process (server side: the client; client side: the server).
    fn peer(&self) -> Result<PeerProcess, IpcFailure>;
    /// Peer identity checked for the message just read (see [`MessagePeer`]).
    fn message_peer(&mut self) -> Result<PeerIdentity, IpcFailure>;
    /// Non-blocking best-effort write (e.g. a final acknowledgement byte).
    fn try_write_now(&self, bytes: &[u8]) -> io::Result<usize>;
}

pub type Accept<'a> = Pin<Box<dyn Future<Output = io::Result<Box<dyn IpcStream>>> + Send + 'a>>;

/// A bound endpoint accepting one client at a time.
pub trait IpcListener: Send {
    /// Waits for the next client. Cancel-safe: dropping the future leaves the listener usable.
    fn accept(&mut self) -> Accept<'_>;
    /// Cancels pending accept I/O; later accepts fail.
    fn cancel(&self);
}

pub trait IpcTransport: Send + Sync {
    fn capabilities(&self) -> IpcCapabilities;
    /// Kernel start stamp of `process_id` (0 is never returned).
    fn process_start(&self, process_id: u32) -> io::Result<u64>;
    /// Fresh endpoint for the current process (random nonce, current start stamp).
    fn fresh_endpoint(&self, product: &str) -> io::Result<EndpointId>;
    /// OS address of `endpoint`.
    fn address(&self, endpoint: &EndpointId) -> io::Result<String>;
    /// Binds `endpoint` owner-only and local-only. Fails if it already exists.
    fn listen(&self, endpoint: &EndpointId) -> Result<Box<dyn IpcListener>, IpcFailure>;
    /// Connects to an owned endpoint.
    fn connect(&self, endpoint: &EndpointId) -> Result<Box<dyn IpcStream>, IpcFailure>;
    /// Connects to a raw OS address (foreign server such as mpv `--input-ipc-server`).
    fn connect_address(&self, address: &str) -> Result<Box<dyn IpcStream>, IpcFailure>;
    /// Identity of the current process.
    fn local_peer(&self) -> Result<PeerIdentity, IpcFailure>;
    /// Adapter policy: may `local` talk to `peer`? Same-principal IPC must never become a
    /// privilege broker (integrity/UAC on Windows, root on Unix).
    fn permits(&self, local: &PeerIdentity, peer: &PeerIdentity) -> bool;
    /// Fails if the calling thread runs under a borrowed identity (Windows impersonation) that a
    /// new worker thread would silently drop. Unix has no per-thread identity: always `Ok`.
    fn ensure_thread_identity(&self) -> Result<(), IpcFailure> {
        Ok(())
    }
}

/// Adapter for builds without an IPC implementation: every operation fails `Unsupported`.
#[derive(Debug, Default, Clone, Copy)]
pub struct UnsupportedIpc;
fn unsupported() -> io::Error {
    io::Error::new(
        io::ErrorKind::Unsupported,
        "local IPC is unsupported on this OS",
    )
}
impl IpcTransport for UnsupportedIpc {
    fn capabilities(&self) -> IpcCapabilities {
        IpcCapabilities {
            local_only: false,
            owner_only_acl: false,
            message_peer: MessagePeer::ConnectionBound,
            raw_client: false,
        }
    }
    fn process_start(&self, _: u32) -> io::Result<u64> {
        Err(unsupported())
    }
    fn fresh_endpoint(&self, _: &str) -> io::Result<EndpointId> {
        Err(unsupported())
    }
    fn address(&self, _: &EndpointId) -> io::Result<String> {
        Err(unsupported())
    }
    fn listen(&self, _: &EndpointId) -> Result<Box<dyn IpcListener>, IpcFailure> {
        Err(IpcFailure::new("listen", unsupported()))
    }
    fn connect(&self, _: &EndpointId) -> Result<Box<dyn IpcStream>, IpcFailure> {
        Err(IpcFailure::new("connect", unsupported()))
    }
    fn connect_address(&self, _: &str) -> Result<Box<dyn IpcStream>, IpcFailure> {
        Err(IpcFailure::new("connect", unsupported()))
    }
    fn local_peer(&self) -> Result<PeerIdentity, IpcFailure> {
        Err(IpcFailure::new("local_peer", unsupported()))
    }
    fn permits(&self, _: &PeerIdentity, _: &PeerIdentity) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_name_round_trips_and_rejects_non_canonical_forms() {
        let id = EndpointId {
            product: "kinekino".into(),
            process_id: 42,
            process_start: 0xabc,
            nonce: [0x5a; 16],
        };
        let name = id.name();
        assert_eq!(
            name,
            format!("kinekino.v1.42.0000000000000abc.{}", "5a".repeat(16))
        );
        assert_eq!(EndpointId::parse("kinekino", &name), Some(id));
        for bad in [
            name.replace(".42.", ".042."),
            name.replace("0abc", "0ABC"),
            name.replace("0000000000000abc", "0000000000000000"),
            format!("{name}.x"),
            name.replace("kinekino", "other"),
        ] {
            assert_eq!(EndpointId::parse("kinekino", &bad), None, "{bad}");
        }
    }
}

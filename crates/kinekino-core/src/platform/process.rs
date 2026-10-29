//! Process-family ownership port (ADR 0006). One OS adapter per OS implements
//! [`ProcessOwnership`]; domain crates reach it only through [`super::OsPolicy::process`].
//!
//! The port exposes what the OS can really guarantee via [`ProcessCapabilities`]. Windows Job
//! Objects and POSIX process groups differ, and callers/tests compare against the capability
//! instead of assuming Windows semantics.
use std::{
    any::Any,
    collections::BTreeMap,
    ffi::OsString,
    fs::File,
    io,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

/// How strongly an adapter enforces a guarantee.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Enforcement {
    /// The OS enforces it for the whole owned family.
    Enforced,
    /// Best effort: applied, but a cooperating-only or per-process mechanism (can be escaped).
    Advisory,
    /// Not available on this OS/build.
    Unsupported,
}

/// What one OS adapter really guarantees for an owned process family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessCapabilities {
    /// Descendants stay inside the owned family (Job Object vs. escapable process group).
    pub family_containment: Enforcement,
    /// Family-wide termination reaches every descendant still in the family.
    pub family_termination: Enforcement,
    /// The family dies when the owner process dies without cleanup.
    pub cleanup_on_owner_death: Enforcement,
    /// [`FamilyBudget::max_committed_bytes`] is enforced.
    pub memory_budget: Enforcement,
    /// [`FamilyBudget::affinity_mask`] is enforced.
    pub affinity_budget: Enforcement,
    /// [`OwnedFamily::memory_peaks`] reports family-wide peaks (vs. the direct child only).
    pub family_memory_peaks: Enforcement,
    /// [`FamilyObservation::Active`] carries an exact process count.
    pub exact_family_count: bool,
}

/// Both cutoffs are established before spawning; no phase starts a fresh timeout.
#[derive(Clone, Copy, Debug)]
pub struct Deadline {
    operation: Instant,
    end: Instant,
}
impl Deadline {
    /// Core may shorten physical cleanup without creating a fresh budget.
    pub fn constrained_end(self, requested: Instant) -> Self {
        Self {
            operation: self.operation.min(requested),
            end: self.end.min(requested),
        }
    }
    /// Later phases can shorten, but never renew, either original cutoff.
    pub fn constrained_by(self, requested: Self) -> Self {
        Self {
            operation: self.operation.min(requested.operation),
            end: self.end.min(requested.end),
        }
    }
    pub fn new(start: Instant, total: Duration, cleanup: Duration) -> io::Result<Self> {
        if total.is_zero() || cleanup.is_zero() || cleanup >= total {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "positive operation and cleanup budgets required",
            ));
        }
        let end = start
            .checked_add(total)
            .ok_or_else(|| io::Error::other("deadline overflow"))?;
        let operation = end
            .checked_sub(cleanup)
            .ok_or_else(|| io::Error::other("cleanup reserve overflow"))?;
        Ok(Self { operation, end })
    }
    pub fn operation_remaining(self, now: Instant) -> Duration {
        self.operation.saturating_duration_since(now)
    }
    pub fn remaining(self, now: Instant) -> Duration {
        self.end.saturating_duration_since(now)
    }
    pub fn request_budget(self, now: Instant, maximum: Duration) -> Duration {
        self.operation_remaining(now).min(maximum)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessIdentity {
    pub pid: u32,
    /// None means the original creation-time query failed; never synthesized from a PID.
    /// Windows: FILETIME. Unix: boot-relative start ticks where the OS exposes them.
    pub creation_filetime: Option<u64>,
    pub executable: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FamilyObservation {
    Empty,
    /// Active members. Exact only when [`ProcessCapabilities::exact_family_count`].
    Active(u32),
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockError {
    Invalid,
    Expired,
    Overflow,
}

/// Milliseconds in the SAME host-wide monotonic domain ([`ProcessOwnership::uptime_millis`]);
/// not wall time or authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceCutoffs {
    operation: u64,
    end: u64,
}
impl SourceCutoffs {
    pub fn from_remaining(
        tick_before: u64,
        operation: Duration,
        total: Duration,
    ) -> Result<Self, ClockError> {
        let operation = u64::try_from(operation.as_millis()).map_err(|_| ClockError::Overflow)?;
        let total = u64::try_from(total.as_millis()).map_err(|_| ClockError::Overflow)?;
        if operation == 0 {
            return Err(ClockError::Expired);
        }
        if operation >= total {
            return Err(ClockError::Invalid);
        }
        Self::new(
            tick_before
                .checked_add(operation)
                .ok_or(ClockError::Overflow)?,
            tick_before.checked_add(total).ok_or(ClockError::Overflow)?,
        )
    }
    pub fn new(operation: u64, end: u64) -> Result<Self, ClockError> {
        if operation == 0 || operation >= end {
            return Err(ClockError::Invalid);
        }
        Ok(Self { operation, end })
    }
    pub fn operation(self) -> u64 {
        self.operation
    }
    pub fn end(self) -> u64 {
        self.end
    }
    pub fn constrained_by(self, other: Self) -> Self {
        Self {
            operation: self.operation.min(other.operation),
            end: self.end.min(other.end),
        }
    }
    pub fn operation_remaining(self, now: u64) -> Duration {
        Duration::from_millis(self.operation.saturating_sub(now))
    }
    pub fn remaining(self, now: u64) -> Duration {
        Duration::from_millis(self.end.saturating_sub(now))
    }
}

/// Owned-source argv markers carrying inherited endpoint values and cutoffs.
pub const CONTROL_ARGUMENT: &str = "--owned-source-control=";
pub const DATA_ARGUMENT: &str = "--owned-source-data=";
pub const OPERATION_ARGUMENT: &str = "--owned-source-operation=";
pub const END_ARGUMENT: &str = "--owned-source-end=";

/// Family resource budget. Memory is committed memory on Windows (Job), address space on POSIX
/// (`RLIMIT_AS`, per process); see [`ProcessCapabilities`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FamilyBudget {
    pub max_committed_bytes: usize,
    pub affinity_mask: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemoryPeaks {
    pub process_peak_working_set_bytes: u64,
    pub process_peak_pagefile_bytes: u64,
    pub process_private_bytes_at_query: u64,
    pub job_peak_memory_bytes: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LaunchState {
    /// Created but not yet inside the owned family (Windows: suspended, before Job assignment).
    UnassignedSuspended,
    AssignedSuspended,
    Running,
}

/// Test-only injected launch faults. Adapters honour the subset meaningful on their OS.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultPoint {
    AttributePreparation,
    AttributeInitialized,
    AfterCreate,
    Assignment,
    AssignmentAndTermination,
    BeforeResume,
    UnexpectedResume,
    LateResume,
    FamilyQuery,
}

/// Opaque child-side endpoint of an owned source channel (inherited by exactly one child).
pub trait SourceChildEndpoint: Send {
    fn as_any(&self) -> &dyn Any;
}

/// Inherited source endpoints and the cutoffs passed to the child on its argv.
pub struct SourceChildren<'a> {
    pub control: &'a dyn SourceChildEndpoint,
    pub data: &'a dyn SourceChildEndpoint,
    pub cutoffs: SourceCutoffs,
}

pub struct SpawnRequest<'a> {
    pub executable: PathBuf,
    pub cwd: PathBuf,
    pub arguments: Vec<OsString>,
    /// None inherits the existing environment. Explicit names currently require ASCII.
    pub environment: Option<BTreeMap<String, OsString>>,
    /// None gives the child a pipe whose write end the owner keeps until `close_stdin`.
    pub stdin: Option<File>,
    pub stdout: File,
    pub stderr: File,
    pub deadline: Deadline,
    pub source: Option<SourceChildren<'a>>,
    pub fault: Option<FaultPoint>,
}

/// One original owned process family. Drop never waits; it releases ownership the way the OS
/// adapter documents (Windows: kill-on-close Job; POSIX: SIGKILL to the group).
pub trait OwnedFamily: Send {
    fn deadline(&self) -> Deadline;
    fn identity(&self) -> &ProcessIdentity;
    fn close_stdin(&mut self) -> io::Result<()>;
    fn poll_child(&mut self) -> io::Result<Option<u32>>;
    fn poll_family(&mut self) -> io::Result<FamilyObservation>;
    fn terminate_owned_family(&mut self) -> io::Result<()>;
    fn state(&self) -> LaunchState;
    /// Whether the child was observed inside a pre-existing outer container (Windows outer Job).
    fn outer_containment_observed(&self) -> bool;
    /// Family accounting without the launch-state shortcut (original container only).
    fn observe_owned_family(&self) -> io::Result<FamilyObservation>;
    fn verify_budget(&self, expected: FamilyBudget) -> io::Result<()>;
    fn memory_peaks(&self) -> io::Result<MemoryPeaks>;
    /// Test-only exit observation of the same original process (never a PID reopen).
    fn exit_probe(&self) -> io::Result<Box<dyn ExitProbe>>;
}

pub trait ExitProbe: Send {
    fn poll(&self) -> io::Result<Option<u32>>;
}

/// An empty family container reserved before spawn (Windows: an empty Job).
pub trait PreparedFamily: Send {
    fn budget(&self) -> FamilyBudget;
    fn check_current(&self) -> io::Result<()>;
    fn into_any(self: Box<Self>) -> Box<dyn Any + Send>;
}

#[must_use = "A post-create failure retains the original owner"]
pub struct SpawnFailure {
    pub stage: &'static str,
    pub error: io::Error,
    pub owner: Option<Box<dyn OwnedFamily>>,
    pub termination_error: Option<io::Error>,
}

/// Completion of one non-blocking pipe operation.
#[derive(Debug, Default)]
pub struct PipeCompletion {
    pub bytes: Vec<u8>,
    pub count: usize,
    pub eof: bool,
    pub cancelled: bool,
}

/// One in-flight non-blocking pipe operation (Windows overlapped I/O, POSIX non-blocking fd).
pub trait PipeOperation: Send {
    /// None while pending.
    fn poll(&mut self) -> Option<io::Result<PipeCompletion>>;
    fn cancel(&mut self) -> io::Result<()>;
    /// The OS no longer references this operation's buffers.
    fn reaped(&self) -> bool;
}

/// Parent- or child-side endpoint of an owned source lane.
pub trait SourcePipe: Send + Sync {
    fn connect(self: Arc<Self>) -> io::Result<Box<dyn PipeOperation>>;
    fn read(self: Arc<Self>, maximum: usize) -> io::Result<Box<dyn PipeOperation>>;
    fn write(self: Arc<Self>, bytes: &[u8]) -> io::Result<Box<dyn PipeOperation>>;
    /// Peer is the expected original endpoint (Windows: pipe client is this process).
    fn verify_peer(&self) -> io::Result<()>;
    fn is_inheritable(&self) -> io::Result<bool>;
    fn as_any(&self) -> &dyn Any;
}

pub struct SourcePipePair {
    pub parent: Arc<dyn SourcePipe>,
    pub child: Box<dyn SourceChildEndpoint>,
    /// Diagnostic name for fixtures (Windows pipe name; None for anonymous socket pairs).
    pub name: Option<String>,
    /// Parent endpoint still needs an asynchronous connect before use.
    pub needs_connect: bool,
}

/// Process-family ownership port.
pub trait ProcessOwnership: Send + Sync {
    fn capabilities(&self) -> ProcessCapabilities;
    /// CPU affinity mask currently allocated to this process; None where the OS exposes none.
    fn current_affinity(&self) -> io::Result<Option<usize>>;
    /// Validates `budget` against the host and reserves an empty family container.
    fn reserve_family(&self, budget: FamilyBudget) -> io::Result<Box<dyn PreparedFamily>>;
    fn spawn(
        &self,
        request: SpawnRequest<'_>,
        prepared: Option<Box<dyn PreparedFamily>>,
    ) -> Result<Box<dyn OwnedFamily>, Box<SpawnFailure>>;
    /// A read-only private duplicate of a regular readable file (for child stdin).
    fn read_only_duplicate(&self, file: &File) -> io::Result<File>;
    /// Host-wide monotonic milliseconds shared by parent and child processes.
    fn uptime_millis(&self) -> u64;
    /// Creates one connected local source lane (Windows named pipe, POSIX socketpair).
    fn source_pipe_pair(&self, maximum: usize) -> io::Result<SourcePipePair>;
    /// Adopts an endpoint value inherited from the owning parent.
    ///
    /// # Safety
    /// `value` must be an original endpoint inherited from `spawn` with a [`SourceChildren`],
    /// transferred exactly once.
    unsafe fn adopt_inherited_source(&self, value: usize) -> io::Result<Arc<dyn SourcePipe>>;
    /// Operations whose buffers were intentionally leaked because the OS never released them.
    fn emergency_retained_operations(&self) -> u64;
}

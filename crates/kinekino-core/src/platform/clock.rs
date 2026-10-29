//! Parent-anchored monotonic counter port (ADR 0008).
//!
//! A parent samples a host-wide monotonic counter together with its own
//! process identity; a worker re-observes the same counter only while that
//! exact original parent (pid + creation stamp) is still its live creator.

/// Counter domain identity shared by parent and worker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CounterIdentity {
    /// Stable domain kind, e.g. `windows-qpc/1` or `posix-monotonic/1`.
    pub kind: &'static str,
    /// Counter ticks per second.
    pub frequency_hz: u64,
    /// Original parent process id.
    pub parent_pid: u32,
    /// OS creation stamp of the original parent (non-zero).
    pub parent_creation: u64,
}

/// One counter observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CounterSample {
    /// Domain the ticks belong to.
    pub identity: CounterIdentity,
    /// Monotonic ticks.
    pub ticks: u64,
}

/// Worker-side handle on the live original creator.
pub trait LiveCreator {
    /// Creator process id observed at connect time.
    fn pid(&self) -> u32;
    /// Samples the counter while the creator is live; `Ok(None)` when it exited.
    fn sample(&self) -> std::io::Result<Option<CounterSample>>;
}

/// Monotonic counter adapter.
pub trait ParentCounter: Send + Sync {
    /// Every domain kind this adapter can produce.
    fn kinds(&self) -> &'static [&'static str];
    /// Samples the counter anchored on the current process.
    fn sample_current(&self) -> std::io::Result<CounterSample>;
    /// Opens the creator (parent) of the current process.
    fn open_creator(&self) -> std::io::Result<Box<dyn LiveCreator>>;
}

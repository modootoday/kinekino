//! POSIX process-family ownership (ADR 0006): the child leads a new session/process group
//! (`setsid`), the family is signalled as a group (SIGTERM, then SIGKILL after a grace period),
//! Linux adds `PR_SET_PDEATHSIG`. Budgets are per-process `RLIMIT_AS` / CPU affinity, so they are
//! reported as [`Enforcement::Advisory`]; cgroup v2 delegation is detected but not yet used.
mod family;
pub(crate) use family::stat_field;
mod pipe;
mod spawn;

use kinekino_core::platform::process::{
    Enforcement, FamilyBudget, OwnedFamily, PreparedFamily, ProcessCapabilities, ProcessOwnership,
    SourcePipe, SourcePipePair, SpawnFailure, SpawnRequest,
};
use std::{fs::File, io, sync::Arc};

/// What a POSIX process group really guarantees (descendants may `setsid` away).
pub const CAPABILITIES: ProcessCapabilities = ProcessCapabilities {
    family_containment: Enforcement::Advisory,
    family_termination: Enforcement::Advisory,
    cleanup_on_owner_death: if cfg!(target_os = "linux") {
        Enforcement::Advisory
    } else {
        Enforcement::Unsupported
    },
    memory_budget: Enforcement::Advisory,
    affinity_budget: if cfg!(target_os = "linux") {
        Enforcement::Advisory
    } else {
        Enforcement::Unsupported
    },
    family_memory_peaks: Enforcement::Advisory,
    exact_family_count: false,
};

/// True when this process sits in a cgroup v2 hierarchy whose `cgroup.subtree_control` it may
/// write (delegation). Reported for diagnostics; enforcement through it is a later step.
pub fn cgroup_v2_delegated() -> bool {
    let Ok(own) = std::fs::read_to_string("/proc/self/cgroup") else {
        return false;
    };
    let Some(path) = own.lines().find_map(|line| line.strip_prefix("0::")) else {
        return false;
    };
    let control = std::path::Path::new("/sys/fs/cgroup")
        .join(path.trim_start_matches('/'))
        .join("cgroup.subtree_control");
    std::fs::OpenOptions::new()
        .write(true)
        .open(control)
        .is_ok()
}

#[derive(Debug, Default)]
pub struct UnixOwnership;

struct UnixPrepared(FamilyBudget);
impl PreparedFamily for UnixPrepared {
    fn budget(&self) -> FamilyBudget {
        self.0
    }
    fn check_current(&self) -> io::Result<()> {
        Ok(())
    }
    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any + Send> {
        self
    }
}

impl ProcessOwnership for UnixOwnership {
    fn capabilities(&self) -> ProcessCapabilities {
        CAPABILITIES
    }
    fn current_affinity(&self) -> io::Result<Option<usize>> {
        spawn::available_affinity()
    }
    fn reserve_family(&self, budget: FamilyBudget) -> io::Result<Box<dyn PreparedFamily>> {
        if budget.max_committed_bytes == 0 || budget.affinity_mask == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "positive memory and affinity required",
            ));
        }
        if let Some(available) = spawn::available_affinity()?
            && budget.affinity_mask & available != budget.affinity_mask
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "affinity exceeds current host allocation",
            ));
        }
        Ok(Box::new(UnixPrepared(budget)))
    }
    fn spawn(
        &self,
        request: SpawnRequest<'_>,
        prepared: Option<Box<dyn PreparedFamily>>,
    ) -> Result<Box<dyn OwnedFamily>, Box<SpawnFailure>> {
        let budget = match prepared {
            None => None,
            Some(prepared) => match prepared.into_any().downcast::<UnixPrepared>() {
                Ok(prepared) => Some(prepared.0),
                Err(_) => {
                    return Err(Box::new(SpawnFailure {
                        stage: "prepared_job",
                        error: io::Error::other("foreign prepared family"),
                        owner: None,
                        termination_error: None,
                    }));
                }
            },
        };
        spawn::spawn(request, budget)
    }
    fn read_only_duplicate(&self, file: &File) -> io::Result<File> {
        spawn::read_only_duplicate(file)
    }
    fn uptime_millis(&self) -> u64 {
        let mut now = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        // SAFETY: valid output pointer; CLOCK_MONOTONIC is system-wide on Linux and macOS.
        unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut now) };
        (now.tv_sec as u64)
            .saturating_mul(1000)
            .saturating_add(now.tv_nsec as u64 / 1_000_000)
    }
    fn source_pipe_pair(&self, _maximum: usize) -> io::Result<SourcePipePair> {
        pipe::pair()
    }
    unsafe fn adopt_inherited_source(&self, value: usize) -> io::Result<Arc<dyn SourcePipe>> {
        // SAFETY: delegated original-inherited-descriptor contract.
        unsafe { pipe::adopt(value) }
    }
    fn emergency_retained_operations(&self) -> u64 {
        // Non-blocking descriptor operations never lend buffers to the kernel.
        0
    }
}

#[cfg(test)]
mod tests;

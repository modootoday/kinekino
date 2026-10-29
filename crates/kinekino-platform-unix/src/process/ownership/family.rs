//! One owned POSIX process group led by the direct child.
use kinekino_core::platform::process::{
    Deadline, ExitProbe, FamilyBudget, FamilyObservation, FaultPoint, LaunchState, MemoryPeaks,
    OwnedFamily, ProcessIdentity,
};
use std::{
    io,
    process::ChildStdin,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// SIGTERM grace before the group is escalated to SIGKILL.
pub(super) const TERMINATE_GRACE: Duration = Duration::from_millis(500);

/// Reap state shared by the owner and test exit probes; only `wait4` on the original pid.
pub(super) struct Reaped {
    pid: libc::pid_t,
    exit: Option<u32>,
    peak_rss_bytes: Option<u64>,
}
impl Reaped {
    pub(super) fn new(pid: libc::pid_t) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            pid,
            exit: None,
            peak_rss_bytes: None,
        }))
    }
    fn poll(&mut self) -> io::Result<Option<u32>> {
        if self.exit.is_some() {
            return Ok(self.exit);
        }
        let mut status = 0;
        // SAFETY: zeroed rusage output; WNOHANG never blocks.
        let mut usage: libc::rusage = unsafe { std::mem::zeroed() };
        let result = unsafe { libc::wait4(self.pid, &mut status, libc::WNOHANG, &mut usage) };
        if result < 0 {
            return Err(io::Error::last_os_error());
        }
        if result == 0 {
            return Ok(None);
        }
        let code = if libc::WIFEXITED(status) {
            libc::WEXITSTATUS(status) as u32
        } else {
            128 + libc::WTERMSIG(status) as u32
        };
        // Linux reports ru_maxrss in KiB, Apple platforms in bytes.
        let scale = if cfg!(target_vendor = "apple") {
            1
        } else {
            1024
        };
        self.peak_rss_bytes = Some((usage.ru_maxrss as u64).saturating_mul(scale));
        self.exit = Some(code);
        Ok(self.exit)
    }
}
pub(super) fn poll_reaped(reaped: &Mutex<Reaped>) -> io::Result<Option<u32>> {
    reaped
        .lock()
        .map_err(|_| io::Error::other("reap state poisoned"))?
        .poll()
}

#[must_use = "Keep the original process-family owner until explicit completion"]
pub(super) struct UnixFamily {
    pub(super) deadline: Deadline,
    pub(super) pgid: libc::pid_t,
    pub(super) reaped: Arc<Mutex<Reaped>>,
    pub(super) input: Option<ChildStdin>,
    pub(super) identity: ProcessIdentity,
    pub(super) budget: Option<FamilyBudget>,
    pub(super) terminate_sent: Option<Instant>,
    pub(super) killed: bool,
    pub(super) fault: Option<FaultPoint>,
}
impl UnixFamily {
    fn signal(&self, signal: libc::c_int) -> io::Result<()> {
        // SAFETY: negative pid targets exactly the original group this owner created.
        if unsafe { libc::kill(-self.pgid, signal) } < 0 {
            let error = io::Error::last_os_error();
            if error.raw_os_error() != Some(libc::ESRCH) {
                return Err(error);
            }
        }
        Ok(())
    }
    fn group_alive(&self) -> io::Result<bool> {
        // SAFETY: signal 0 only probes existence/permission of the original group.
        if unsafe { libc::kill(-self.pgid, 0) } == 0 {
            return Ok(true);
        }
        let error = io::Error::last_os_error();
        match error.raw_os_error() {
            Some(libc::ESRCH) => Ok(false),
            // EPERM: a member exists that we may not signal (escaped privilege change).
            Some(libc::EPERM) => Ok(true),
            _ => Err(error),
        }
    }
    pub(super) fn terminate(&mut self) -> io::Result<()> {
        if self.fault == Some(FaultPoint::AssignmentAndTermination) {
            return Err(io::Error::other("injected termination failure"));
        }
        match self.terminate_sent {
            None => {
                self.terminate_sent = Some(Instant::now());
                self.signal(libc::SIGTERM)
            }
            Some(_) => self.kill(),
        }
    }
    fn kill(&mut self) -> io::Result<()> {
        self.killed = true;
        self.signal(libc::SIGKILL)
    }
    fn count(&self) -> u32 {
        #[cfg(target_os = "linux")]
        if let Ok(entries) = std::fs::read_dir("/proc") {
            let count = entries
                .flatten()
                .filter_map(|entry| entry.file_name().to_str()?.parse::<u32>().ok())
                .filter(|pid| {
                    std::fs::read_to_string(format!("/proc/{pid}/stat"))
                        .ok()
                        .and_then(|stat| stat_field(&stat, 5))
                        == Some(self.pgid as u64)
                })
                .count() as u32;
            return count.max(1);
        }
        1
    }
}
/// 1-based `/proc/<pid>/stat` field, parsed after the parenthesised command name.
pub(crate) fn stat_field(stat: &str, field: usize) -> Option<u64> {
    let rest = &stat[stat.rfind(')')? + 2..];
    rest.split_whitespace().nth(field - 3)?.parse().ok()
}

struct UnixExitProbe(Arc<Mutex<Reaped>>);
impl ExitProbe for UnixExitProbe {
    fn poll(&self) -> io::Result<Option<u32>> {
        poll_reaped(&self.0)
    }
}

impl OwnedFamily for UnixFamily {
    fn deadline(&self) -> Deadline {
        self.deadline
    }
    fn identity(&self) -> &ProcessIdentity {
        &self.identity
    }
    fn close_stdin(&mut self) -> io::Result<()> {
        self.input.take();
        Ok(())
    }
    fn poll_child(&mut self) -> io::Result<Option<u32>> {
        poll_reaped(&self.reaped)
    }
    fn poll_family(&mut self) -> io::Result<FamilyObservation> {
        if self.fault == Some(FaultPoint::FamilyQuery) {
            return Err(io::Error::other("injected family query failure"));
        }
        self.observe_owned_family().and_then(|observation| {
            if observation != FamilyObservation::Empty
                && !self.killed
                && self
                    .terminate_sent
                    .is_some_and(|sent| sent.elapsed() >= TERMINATE_GRACE)
            {
                self.kill()?;
            }
            Ok(observation)
        })
    }
    fn terminate_owned_family(&mut self) -> io::Result<()> {
        self.terminate()
    }
    fn state(&self) -> LaunchState {
        LaunchState::Running
    }
    fn outer_containment_observed(&self) -> bool {
        false
    }
    fn observe_owned_family(&self) -> io::Result<FamilyObservation> {
        // Reap the direct child first so its zombie does not keep the group observable.
        poll_reaped(&self.reaped)?;
        Ok(if self.group_alive()? {
            FamilyObservation::Active(self.count())
        } else {
            FamilyObservation::Empty
        })
    }
    fn verify_budget(&self, expected: FamilyBudget) -> io::Result<()> {
        if self.budget == Some(expected) {
            Ok(())
        } else {
            Err(io::Error::other("family budget readback mismatch"))
        }
    }
    fn memory_peaks(&self) -> io::Result<MemoryPeaks> {
        let reaped = self
            .reaped
            .lock()
            .map_err(|_| io::Error::other("reap state poisoned"))?;
        let peak = match reaped.peak_rss_bytes {
            Some(peak) => peak,
            None => live_peak(reaped.pid)?,
        };
        Ok(MemoryPeaks {
            process_peak_working_set_bytes: peak,
            process_peak_pagefile_bytes: 0,
            process_private_bytes_at_query: 0,
            // Direct child only; descendants are not accounted without cgroups.
            job_peak_memory_bytes: peak,
        })
    }
    fn exit_probe(&self) -> io::Result<Box<dyn ExitProbe>> {
        Ok(Box::new(UnixExitProbe(self.reaped.clone())))
    }
}
fn live_peak(pid: libc::pid_t) -> io::Result<u64> {
    #[cfg(target_os = "linux")]
    {
        let status = std::fs::read_to_string(format!("/proc/{pid}/status"))?;
        if let Some(kib) = status.lines().find_map(|line| {
            line.strip_prefix("VmHWM:")?
                .trim()
                .strip_suffix("kB")?
                .trim()
                .parse::<u64>()
                .ok()
        }) {
            return Ok(kib * 1024);
        }
    }
    let _ = pid;
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "live peak memory unavailable before exit",
    ))
}
impl Drop for UnixFamily {
    fn drop(&mut self) {
        // No wait: SIGKILL the original group, then one non-blocking reap of the direct child.
        if !matches!(self.group_alive(), Ok(false)) {
            let _ = self.signal(libc::SIGKILL);
        }
        let _ = poll_reaped(&self.reaped);
    }
}

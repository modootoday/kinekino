//! Spawns the direct child as leader of a new session/process group.
use super::{
    family::{Reaped, UnixFamily, stat_field},
    pipe::ChildSocket,
};
use kinekino_core::platform::process::{
    CONTROL_ARGUMENT, DATA_ARGUMENT, END_ARGUMENT, FamilyBudget, FaultPoint, OPERATION_ARGUMENT,
    OwnedFamily, ProcessIdentity, SourceChildEndpoint, SourceChildren, SpawnFailure, SpawnRequest,
};
use std::{
    fs::File,
    io,
    os::{
        fd::{AsRawFd, RawFd},
        unix::{fs::MetadataExt, process::CommandExt},
    },
    process::{Command, Stdio},
    time::Instant,
};

fn remaining(deadline: kinekino_core::platform::process::Deadline) -> io::Result<()> {
    if deadline.operation_remaining(Instant::now()).is_zero() {
        return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "spawn operation budget exhausted",
        ));
    }
    Ok(())
}
fn child_fd(endpoint: &dyn SourceChildEndpoint) -> io::Result<RawFd> {
    endpoint
        .as_any()
        .downcast_ref::<ChildSocket>()
        .map(|socket| socket.0.as_raw_fd())
        .ok_or_else(|| io::Error::other("foreign source endpoint"))
}

/// CPU set this process may use, as a mask over the first `usize::BITS` CPUs (Linux only).
pub(super) fn available_affinity() -> io::Result<Option<usize>> {
    #[cfg(target_os = "linux")]
    {
        // SAFETY: zeroed cpu_set_t output of the stated size for this process.
        let mut set: libc::cpu_set_t = unsafe { std::mem::zeroed() };
        if unsafe { libc::sched_getaffinity(0, size_of::<libc::cpu_set_t>(), &mut set) } < 0 {
            return Err(io::Error::last_os_error());
        }
        let mut mask = 0usize;
        for cpu in 0..usize::BITS as usize {
            if unsafe { libc::CPU_ISSET(cpu, &set) } {
                mask |= 1 << cpu;
            }
        }
        Ok(Some(mask))
    }
    #[cfg(not(target_os = "linux"))]
    Ok(None)
}

pub(super) fn read_only_duplicate(file: &File) -> io::Result<File> {
    use std::io::{Read, Seek, SeekFrom};
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Regular input reader required",
        ));
    }
    // SAFETY: F_GETFL only reads the status flags of a descriptor we borrow.
    let flags = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_GETFL) };
    if flags < 0 {
        return Err(io::Error::last_os_error());
    }
    // A reopen must never upgrade a write-only descriptor to read access.
    if flags & libc::O_ACCMODE == libc::O_WRONLY {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "input reader is write-only",
        ));
    }
    // Linux: reopening the descriptor yields a new read-only open file description.
    #[cfg(target_os = "linux")]
    let mut reader = File::open(format!("/proc/self/fd/{}", file.as_raw_fd()))?;
    // Elsewhere POSIX cannot drop rights on a duplicate; the caller's reader is readable only
    // when it was opened read-only (documented Advisory).
    #[cfg(not(target_os = "linux"))]
    let mut reader = file.try_clone()?;
    let reopened = reader.metadata()?;
    if (reopened.dev(), reopened.ino()) != (metadata.dev(), metadata.ino()) {
        return Err(io::Error::other("input reader identity changed"));
    }
    reader.seek(SeekFrom::Start(0))?;
    let _readability_probe_bytes = reader.read(&mut [0; 1])?;
    reader.seek(SeekFrom::Start(0))?;
    Ok(reader)
}

fn creation_ticks(pid: u32) -> Option<u64> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    stat_field(&stat, 22)
}

pub(super) fn spawn(
    request: SpawnRequest<'_>,
    budget: Option<FamilyBudget>,
) -> Result<Box<dyn OwnedFamily>, Box<SpawnFailure>> {
    let before = |stage, error| {
        Box::new(SpawnFailure {
            stage,
            error,
            owner: None,
            termination_error: None,
        })
    };
    let SpawnRequest {
        executable,
        cwd,
        arguments,
        environment,
        stdin,
        stdout,
        stderr,
        deadline,
        source,
        fault,
    } = request;
    let created = (|| -> io::Result<_> {
        remaining(deadline)?;
        if !executable.is_absolute() || !cwd.is_absolute() {
            return Err(io::Error::other("absolute executable and cwd required"));
        }
        let executable = executable.canonicalize()?;
        if !executable.is_file() || !cwd.is_dir() {
            return Err(io::Error::other("executable/cwd unavailable"));
        }
        let mut command = Command::new(&executable);
        command.args(&arguments).current_dir(&cwd);
        if let Some(values) = &environment {
            command.env_clear();
            for (key, value) in values {
                if key.is_empty() || !key.is_ascii() || key.contains(['=', '\0']) {
                    return Err(io::Error::other(
                        "explicit environment names must be nonempty ASCII without = or NUL",
                    ));
                }
                command.env(key, value);
            }
        }
        let mut inherited: Vec<RawFd> = Vec::new();
        if let Some(SourceChildren {
            control,
            data,
            cutoffs,
        }) = source
        {
            let (control, data) = (child_fd(control)?, child_fd(data)?);
            command.args([
                format!("{CONTROL_ARGUMENT}{control}"),
                format!("{DATA_ARGUMENT}{data}"),
                format!("{OPERATION_ARGUMENT}{}", cutoffs.operation()),
                format!("{END_ARGUMENT}{}", cutoffs.end()),
            ]);
            inherited.extend([control, data]);
        }
        let piped_stdin = stdin.is_none();
        command
            .stdin(stdin.map_or_else(Stdio::piped, Stdio::from))
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr));
        if fault == Some(FaultPoint::AttributePreparation) {
            return Err(io::Error::other("injected attribute failure"));
        }
        if fault == Some(FaultPoint::AttributeInitialized) {
            return Err(io::Error::other("injected initialized attribute failure"));
        }
        // SAFETY: getpid is async-signal-safe; captured before fork.
        let parent = unsafe { libc::getpid() };
        let memory = budget.map(|budget| budget.max_committed_bytes as libc::rlim_t);
        let affinity = budget.map(|budget| budget.affinity_mask);
        // SAFETY: the hook only issues async-signal-safe syscalls and allocates nothing.
        unsafe {
            command.pre_exec(move || {
                if libc::setsid() < 0 {
                    return Err(io::Error::last_os_error());
                }
                #[cfg(target_os = "linux")]
                {
                    if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL as libc::c_ulong) < 0 {
                        return Err(io::Error::last_os_error());
                    }
                    // The owner died between fork and prctl: never run unowned.
                    if libc::getppid() != parent {
                        libc::_exit(124);
                    }
                }
                let _ = parent;
                if let Some(bytes) = memory {
                    let limit = libc::rlimit {
                        rlim_cur: bytes,
                        rlim_max: bytes,
                    };
                    if libc::setrlimit(libc::RLIMIT_AS, &limit) < 0 {
                        return Err(io::Error::last_os_error());
                    }
                }
                #[cfg(target_os = "linux")]
                if let Some(mask) = affinity {
                    let mut set: libc::cpu_set_t = std::mem::zeroed();
                    for cpu in 0..usize::BITS as usize {
                        if mask & (1 << cpu) != 0 {
                            libc::CPU_SET(cpu, &mut set);
                        }
                    }
                    if libc::sched_setaffinity(0, size_of::<libc::cpu_set_t>(), &set) < 0 {
                        return Err(io::Error::last_os_error());
                    }
                }
                let _ = affinity;
                for &fd in &inherited {
                    let flags = libc::fcntl(fd, libc::F_GETFD);
                    if flags < 0 || libc::fcntl(fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC) < 0 {
                        return Err(io::Error::last_os_error());
                    }
                }
                Ok(())
            });
        }
        remaining(deadline)?;
        let mut child = command.spawn()?;
        let input = if piped_stdin {
            child.stdin.take()
        } else {
            None
        };
        let pid = child.id();
        // `child` is dropped without waiting; all reaping goes through `wait4` on this pid.
        drop(child);
        Ok(UnixFamily {
            deadline,
            pgid: pid as libc::pid_t,
            reaped: Reaped::new(pid as libc::pid_t),
            input,
            identity: ProcessIdentity {
                pid,
                creation_filetime: creation_ticks(pid),
                executable,
            },
            budget,
            terminate_sent: None,
            killed: false,
            fault,
        })
    })();
    let mut owner = created.map_err(|error| before("prepare_or_create", error))?;
    let admitted = (|| -> io::Result<()> {
        if matches!(
            fault,
            Some(
                FaultPoint::AfterCreate
                    | FaultPoint::Assignment
                    | FaultPoint::AssignmentAndTermination
                    | FaultPoint::BeforeResume
                    | FaultPoint::UnexpectedResume
            )
        ) {
            return Err(io::Error::other("injected post-create failure"));
        }
        if fault == Some(FaultPoint::LateResume) {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "spawn operation budget exhausted",
            ));
        }
        remaining(deadline)
    })();
    if let Err(error) = admitted {
        let termination_error = owner.terminate().err();
        return Err(Box::new(SpawnFailure {
            stage: "assign_verify_resume",
            error,
            owner: Some(Box::new(owner)),
            termination_error,
        }));
    }
    Ok(Box::new(owner))
}

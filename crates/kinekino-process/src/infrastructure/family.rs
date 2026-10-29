//! OS-neutral facade over the central `ProcessOwnership` port (ADR 0006). Windows binds a
//! Job Object; Unix a session/process group. Guarantee levels come from `capabilities()`.
use super::{
    evidence_files::{EvidencePaths, PreparedEvidence},
    ipc::{CONTROL_ARGUMENT, DATA_ARGUMENT, END_ARGUMENT, OPERATION_ARGUMENT},
    os_policy::process,
    source_channel::{PreparedSourceIpc, SourceIpc},
};
use crate::{
    application::supervisor::{ProcessFamily, Supervisor},
    domain::{deadline::Deadline, identity::ProcessIdentity, observation::FamilyObservation},
};
pub use kinekino_core::platform::process::{
    ExitProbe, FamilyBudget as JobBudget, FaultPoint, LaunchState, MemoryPeaks, ProcessCapabilities,
};
use kinekino_core::platform::process::{
    OwnedFamily, PreparedFamily, SourceChildren, SpawnFailure, SpawnRequest,
};
use std::{collections::BTreeMap, ffi::OsString, io, path::PathBuf};

pub fn capabilities() -> ProcessCapabilities {
    process().capabilities()
}

pub struct SpawnSpec {
    pub executable: PathBuf,
    pub cwd: PathBuf,
    pub arguments: Vec<OsString>,
    /// None inherits the existing environment. Explicit names currently require ASCII.
    pub environment: Option<BTreeMap<String, OsString>>,
}
#[must_use = "A post-create failure retains the original owner"]
pub struct LaunchFailure {
    pub stage: &'static str,
    pub error: io::Error,
    pub owner: Option<Box<OwnedProcessFamily>>,
    pub termination_error: Option<io::Error>,
    pub evidence: EvidencePaths,
}
#[must_use = "Retain BOTH original family and IPC endpoints until explicitly closed"]
pub struct SourceFamily {
    pub owner: Supervisor<OwnedProcessFamily>,
    pub ipc: SourceIpc,
}
#[must_use = "Launch failure retains original owner and IPC"]
pub struct SourceLaunchFailure {
    pub launch: Box<LaunchFailure>,
    pub ipc: SourceIpc,
}

/// Budget reserved before spawn; the child is admitted into exactly this family.
pub struct PreparedFamilyJob(Box<dyn PreparedFamily>);
impl PreparedFamilyJob {
    pub fn reserve(budget: JobBudget) -> io::Result<Self> {
        process().reserve_family(budget).map(Self)
    }
    pub fn budget(&self) -> JobBudget {
        self.0.budget()
    }
    pub fn check_current(&self) -> io::Result<()> {
        self.0.check_current()
    }
}

/// Borrowed read-only peak query; it cannot outlive or be used after moving the owner.
///
/// ```compile_fail
/// use kinekino_process::infrastructure::family::{OwnedProcessFamily, ResourceProbe};
/// fn escape(owner: OwnedProcessFamily) -> ResourceProbe<'static> {
///     owner.resource_probe()
/// }
/// ```
/// ```compile_fail
/// use kinekino_process::infrastructure::family::OwnedProcessFamily;
/// fn move_before_query(owner: OwnedProcessFamily) {
///     let probe = owner.resource_probe();
///     drop(owner);
///     let _ = probe.query();
/// }
/// ```
pub struct ResourceProbe<'owner>(&'owner dyn OwnedFamily);
impl ResourceProbe<'_> {
    /// Query failure is an error, never a zero-byte observation or a cleanup acknowledgement.
    pub fn query(&self) -> io::Result<MemoryPeaks> {
        self.0.memory_peaks()
    }
}

#[must_use = "Keep the original process-family owner until explicit completion"]
pub struct OwnedProcessFamily(Box<dyn OwnedFamily>);
impl OwnedProcessFamily {
    pub fn poll_owned_job(&self) -> io::Result<FamilyObservation> {
        self.0.observe_owned_family()
    }
    pub fn verify_job_budget(&self, expected: JobBudget) -> io::Result<()> {
        self.0.verify_budget(expected)
    }
    pub fn state(&self) -> LaunchState {
        self.0.state()
    }
    pub fn outer_job_observed(&self) -> bool {
        self.0.outer_containment_observed()
    }
    pub fn resource_probe(&self) -> ResourceProbe<'_> {
        ResourceProbe(self.0.as_ref())
    }
    pub fn exit_probe(&self) -> io::Result<Box<dyn ExitProbe>> {
        self.0.exit_probe()
    }
}
impl ProcessFamily for OwnedProcessFamily {
    fn deadline(&self) -> Deadline {
        self.0.deadline()
    }
    fn identity(&self) -> &ProcessIdentity {
        self.0.identity()
    }
    fn close_stdin(&mut self) -> io::Result<()> {
        self.0.close_stdin()
    }
    fn poll_child(&mut self) -> io::Result<Option<u32>> {
        self.0.poll_child()
    }
    fn poll_family(&mut self) -> io::Result<FamilyObservation> {
        self.0.poll_family()
    }
    fn terminate_owned_family(&mut self) -> io::Result<()> {
        self.0.terminate_owned_family()
    }
}

type Spawned = Result<(OwnedProcessFamily, EvidencePaths), Box<LaunchFailure>>;

pub fn spawn(spec: SpawnSpec, evidence: PreparedEvidence, deadline: Deadline) -> Spawned {
    spawn_inner(spec, evidence, deadline, None, None, None)
}
pub fn spawn_prepared(
    spec: SpawnSpec,
    evidence: PreparedEvidence,
    deadline: Deadline,
    prepared: PreparedFamilyJob,
) -> Spawned {
    if let Err(error) = prepared.check_current() {
        return Err(Box::new(LaunchFailure {
            stage: "prepared_job",
            error,
            owner: None,
            termination_error: None,
            evidence: evidence.paths.clone(),
        }));
    }
    spawn_inner(spec, evidence, deadline, None, Some(prepared.0), None)
}
/// Deterministic failure injection for the containment contract tests.
#[doc(hidden)]
pub fn spawn_faulted(
    spec: SpawnSpec,
    evidence: PreparedEvidence,
    deadline: Deadline,
    fault: FaultPoint,
) -> Spawned {
    spawn_inner(spec, evidence, deadline, Some(fault), None, None)
}

fn spawn_inner(
    spec: SpawnSpec,
    evidence: PreparedEvidence,
    deadline: Deadline,
    fault: Option<FaultPoint>,
    prepared: Option<Box<dyn PreparedFamily>>,
    source: Option<SourceChildren<'_>>,
) -> Spawned {
    let paths = evidence.paths.clone();
    let request = SpawnRequest {
        executable: spec.executable,
        cwd: spec.cwd,
        arguments: spec.arguments,
        environment: spec.environment,
        stdin: evidence.input,
        stdout: evidence.stdout,
        stderr: evidence.stderr,
        deadline,
        source,
        fault,
    };
    match process().spawn(request, prepared) {
        Ok(owner) => Ok((OwnedProcessFamily(owner), paths)),
        Err(failure) => {
            let SpawnFailure {
                stage,
                error,
                owner,
                termination_error,
            } = *failure;
            Err(Box::new(LaunchFailure {
                stage,
                error,
                owner: owner.map(|owner| Box::new(OwnedProcessFamily(owner))),
                termination_error,
                evidence: paths,
            }))
        }
    }
}

pub fn spawn_source(
    spec: SpawnSpec,
    evidence: PreparedEvidence,
    deadline: Deadline,
    prepared: PreparedSourceIpc,
    raw_limit: u64,
) -> Result<SourceFamily, Box<SourceLaunchFailure>> {
    source_spawn(spec, evidence, deadline, prepared, raw_limit, None)
}
#[doc(hidden)]
pub fn spawn_source_faulted(
    spec: SpawnSpec,
    evidence: PreparedEvidence,
    deadline: Deadline,
    prepared: PreparedSourceIpc,
    raw_limit: u64,
    fault: FaultPoint,
) -> Result<SourceFamily, Box<SourceLaunchFailure>> {
    source_spawn(spec, evidence, deadline, prepared, raw_limit, Some(fault))
}

fn source_spawn(
    spec: SpawnSpec,
    evidence: PreparedEvidence,
    deadline: Deadline,
    mut prepared: PreparedSourceIpc,
    raw_limit: u64,
    fault: Option<FaultPoint>,
) -> Result<SourceFamily, Box<SourceLaunchFailure>> {
    let setup = (|| -> io::Result<_> {
        if raw_limit == 0 {
            return Err(io::Error::other("source raw evidence bound is zero"));
        }
        if spec.arguments.iter().any(|argument| {
            let value = argument.to_string_lossy();
            [
                CONTROL_ARGUMENT,
                DATA_ARGUMENT,
                OPERATION_ARGUMENT,
                END_ARGUMENT,
            ]
            .iter()
            .any(|prefix| value.starts_with(prefix))
        }) {
            return Err(io::Error::other(
                "caller supplied reserved source handles/cutoffs",
            ));
        }
        let deadline = prepared.constrain_deadline(deadline)?;
        if !prepared.poll_connected()? {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "original source pipes still connecting",
            ));
        }
        Ok(deadline)
    })();
    let deadline = match setup {
        Ok(deadline) => deadline,
        Err(error) => {
            drop(prepared.child.take());
            return Err(Box::new(SourceLaunchFailure {
                launch: Box::new(LaunchFailure {
                    stage: "source_prepare",
                    error,
                    owner: None,
                    termination_error: None,
                    evidence: evidence.paths.clone(),
                }),
                ipc: prepared.parent,
            }));
        }
    };
    let cutoffs = prepared.cutoffs();
    let child = prepared.child.take().expect("unconsumed source endpoints");
    let result = spawn_inner(
        spec,
        evidence,
        deadline,
        fault,
        None,
        Some(SourceChildren {
            control: child.control.as_ref(),
            data: child.data.as_ref(),
            cutoffs,
        }),
    );
    // Parent copies of child endpoints close on success AND failure so they cannot
    // suppress the child's EOF.
    drop(child);
    match result {
        Ok((family, paths)) => Ok(SourceFamily {
            owner: Supervisor::new(family, paths, raw_limit),
            ipc: prepared.parent,
        }),
        Err(launch) => Err(Box::new(SourceLaunchFailure {
            launch,
            ipc: prepared.parent,
        })),
    }
}

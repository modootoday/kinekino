//! Windows process-family ownership: one exclusively owned kill-on-close Job Object per family
//! (CREATE_SUSPENDED → AssignProcessToJobObject → ResumeThread), overlapped named-pipe source
//! lanes. Implements `kinekino_core::platform::process::ProcessOwnership` (ADR 0006).
mod handles;
mod identity;
mod job;
mod launch;
mod monotonic;
mod pipe;
use self::{
    handles::{Handle, error},
    job::FamilyJob,
};
use kinekino_core::platform::process::{
    Deadline, Enforcement, ExitProbe, FamilyBudget, FamilyObservation, FaultPoint, LaunchState,
    MemoryPeaks, OwnedFamily, PreparedFamily, ProcessCapabilities, ProcessIdentity,
    ProcessOwnership, SourcePipe, SourcePipePair, SpawnFailure, SpawnRequest,
};
use std::{fs::File, io, sync::Arc};
use windows::Win32::{
    Foundation::{WAIT_OBJECT_0, WAIT_TIMEOUT},
    System::Threading::{GetExitCodeProcess, TerminateProcess, WaitForSingleObject},
};

/// Windows Job Objects contain, budget and kill the whole family (Microsoft Learn: Job Objects).
pub const CAPABILITIES: ProcessCapabilities = ProcessCapabilities {
    family_containment: Enforcement::Enforced,
    family_termination: Enforcement::Enforced,
    cleanup_on_owner_death: Enforcement::Enforced,
    memory_budget: Enforcement::Enforced,
    affinity_budget: Enforcement::Enforced,
    family_memory_peaks: Enforcement::Enforced,
    exact_family_count: true,
};

#[derive(Debug, Default)]
pub struct WindowsOwnership;
impl ProcessOwnership for WindowsOwnership {
    fn capabilities(&self) -> ProcessCapabilities {
        CAPABILITIES
    }
    fn current_affinity(&self) -> io::Result<Option<usize>> {
        job::prepared::current_affinity().map(Some)
    }
    fn reserve_family(&self, budget: FamilyBudget) -> io::Result<Box<dyn PreparedFamily>> {
        Ok(Box::new(job::prepared::PreparedFamilyJob::reserve(budget)?))
    }
    fn spawn(
        &self,
        request: SpawnRequest<'_>,
        prepared: Option<Box<dyn PreparedFamily>>,
    ) -> Result<Box<dyn OwnedFamily>, Box<SpawnFailure>> {
        launch::spawn(request, prepared)
    }
    fn read_only_duplicate(&self, file: &File) -> io::Result<File> {
        Handle::read_only_duplicate(file)
    }
    fn uptime_millis(&self) -> u64 {
        monotonic::uptime_millis()
    }
    fn source_pipe_pair(&self, maximum: usize) -> io::Result<SourcePipePair> {
        pipe::pair(maximum)
    }
    unsafe fn adopt_inherited_source(&self, value: usize) -> io::Result<Arc<dyn SourcePipe>> {
        // SAFETY: delegated to the caller's original-inherited-handle contract.
        unsafe { pipe::adopt(value) }
    }
    fn emergency_retained_operations(&self) -> u64 {
        pipe::operation::emergency_retained_operations()
    }
}

#[must_use = "Keep the original process-family owner until explicit completion"]
pub(crate) struct WindowsFamily {
    deadline: Deadline,
    process: Handle,
    thread: Handle,
    job: FamilyJob,
    input: Option<File>,
    identity: ProcessIdentity,
    state: LaunchState,
    outer_job_observed: bool,
    fault: Option<FaultPoint>,
}
impl WindowsFamily {
    fn terminate(&self) -> io::Result<()> {
        if self.fault == Some(FaultPoint::AssignmentAndTermination) {
            return Err(io::Error::other("injected termination failure"));
        }
        if self.state == LaunchState::UnassignedSuspended {
            // SAFETY: unassigned child has never been resumed; only its original process handle.
            unsafe { TerminateProcess(self.process.raw(), 124).map_err(error) }
        } else {
            self.job.terminate()
        }
    }
}
fn poll_exit(process: &Handle) -> io::Result<Option<u32>> {
    // SAFETY: wait uses zero timeout and original process handle; exit code only after signal.
    match unsafe { WaitForSingleObject(process.raw(), 0) } {
        WAIT_TIMEOUT => Ok(None),
        WAIT_OBJECT_0 => {
            let mut code = 0;
            unsafe {
                GetExitCodeProcess(process.raw(), &mut code).map_err(error)?;
            }
            Ok(Some(code))
        }
        _ => Err(io::Error::last_os_error()),
    }
}
/// Test-only observation of the same original process handle, never a PID reopen.
struct WindowsExitProbe(Handle);
impl ExitProbe for WindowsExitProbe {
    fn poll(&self) -> io::Result<Option<u32>> {
        poll_exit(&self.0)
    }
}
impl Drop for WindowsExitProbe {
    fn drop(&mut self) {
        // Test-owned original process only; fallback contains a failed assertion without waiting.
        if matches!(poll_exit(&self.0), Ok(None)) {
            let _ = unsafe { TerminateProcess(self.0.raw(), 124) };
        }
    }
}
impl OwnedFamily for WindowsFamily {
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
        poll_exit(&self.process)
    }
    fn poll_family(&mut self) -> io::Result<FamilyObservation> {
        if self.fault == Some(FaultPoint::FamilyQuery) {
            return Err(io::Error::other("injected family query failure"));
        }
        if self.state == LaunchState::UnassignedSuspended {
            return Ok(FamilyObservation::Unknown);
        }
        self.job.observe()
    }
    fn terminate_owned_family(&mut self) -> io::Result<()> {
        self.terminate()
    }
    fn state(&self) -> LaunchState {
        self.state
    }
    fn outer_containment_observed(&self) -> bool {
        self.outer_job_observed
    }
    fn observe_owned_family(&self) -> io::Result<FamilyObservation> {
        self.job.observe()
    }
    fn verify_budget(&self, expected: FamilyBudget) -> io::Result<()> {
        self.job.check_budget(expected)
    }
    fn memory_peaks(&self) -> io::Result<MemoryPeaks> {
        job::resources::query(&self.process, &self.job)
    }
    fn exit_probe(&self) -> io::Result<Box<dyn ExitProbe>> {
        Ok(Box::new(WindowsExitProbe(
            self.process.duplicate_private()?,
        )))
    }
}
impl Drop for WindowsFamily {
    fn drop(&mut self) {
        // No wait/read/join/sleep. CloseHandle itself is not a hard real-time guarantee.
        // A failed assignment leaves the suspended child outside the owned Job.
        if self.state == LaunchState::UnassignedSuspended {
            // SAFETY: best effort on the exact retained child, irrespective of injected faults.
            let _ = unsafe { TerminateProcess(self.process.raw(), 124) };
        }
        // Assigned descendants are contained by the exclusively owned kill-on-close Job.
    }
}

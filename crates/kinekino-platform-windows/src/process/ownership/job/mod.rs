pub(super) mod prepared;
pub(super) mod resources;
use super::handles::{Handle, error};
use kinekino_core::platform::process::FamilyObservation;
use std::{io, mem::size_of};
use windows::{
    Win32::{Foundation::HANDLE, System::JobObjects::*},
    core::PCWSTR,
};

pub(super) struct FamilyJob(Handle);
impl FamilyJob {
    pub(super) fn query_handle(&self) -> HANDLE {
        self.0.raw()
    }
    pub(super) fn new() -> io::Result<Self> {
        // SAFETY: unnamed job with no inheritable security attributes; owner immediately wraps it.
        let job = Self(unsafe {
            Handle::from_raw(CreateJobObjectW(None, PCWSTR::null()).map_err(error)?)
        });
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        // SAFETY: correctly sized input/readback structures and exclusively owned job.
        unsafe {
            SetInformationJobObject(
                job.0.raw(),
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
            .map_err(error)?;
            let mut actual = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
            QueryInformationJobObject(
                Some(job.0.raw()),
                JobObjectExtendedLimitInformation,
                (&mut actual as *mut JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                None,
            )
            .map_err(error)?;
            if actual.BasicLimitInformation.LimitFlags != JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE {
                return Err(io::Error::other("owned Job limit readback mismatch"));
            }
        }
        Ok(job)
    }
    pub(super) fn assign(&self, process: HANDLE) -> io::Result<()> {
        // SAFETY: both handles remain owned through assignment and readback.
        unsafe {
            AssignProcessToJobObject(self.0.raw(), process).map_err(error)?;
            let mut member = windows::core::BOOL::default();
            IsProcessInJob(process, Some(self.0.raw()), &mut member).map_err(error)?;
            if !member.as_bool() {
                return Err(io::Error::other("owned Job membership readback failed"));
            }
        }
        Ok(())
    }
    pub(super) fn observe(&self) -> io::Result<FamilyObservation> {
        let mut accounting = JOBOBJECT_BASIC_ACCOUNTING_INFORMATION::default();
        // SAFETY: fixed-size initialized accounting output for this owned job only.
        unsafe {
            QueryInformationJobObject(
                Some(self.0.raw()),
                JobObjectBasicAccountingInformation,
                (&mut accounting as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                size_of::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() as u32,
                None,
            )
            .map_err(error)?;
        }
        Ok(if accounting.ActiveProcesses == 0 {
            FamilyObservation::Empty
        } else {
            FamilyObservation::Active(accounting.ActiveProcesses)
        })
    }
    pub(super) fn terminate(&self) -> io::Result<()> {
        // SAFETY: never receives a caller-supplied/outer Job handle.
        unsafe { TerminateJobObject(self.0.raw(), 124).map_err(error) }
    }
}

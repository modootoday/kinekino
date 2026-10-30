use crate::{cpu_selection::first_two, execution_profile::Profile, windows_topology};
use std::{ffi::c_void, io, mem::size_of};
type Handle = *mut c_void;
const BELOW_NORMAL: u32 = 0x4000;
const EXTENDED_INFORMATION: u32 = 9;

#[repr(C)]
#[derive(Default)]
struct BasicLimits {
    process_time: i64,
    job_time: i64,
    flags: u32,
    minimum_working_set: usize,
    maximum_working_set: usize,
    active_processes: u32,
    affinity: usize,
    priority: u32,
    scheduling: u32,
}
#[repr(C)]
#[derive(Default)]
struct ExtendedLimits {
    basic: BasicLimits,
    io_counters: [u64; 6],
    process_memory: usize,
    job_memory: usize,
    peak_process_memory: usize,
    peak_job_memory: usize,
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentProcess() -> Handle;
    fn CreateJobObjectW(attributes: *const c_void, name: *const u16) -> Handle;
    fn SetInformationJobObject(job: Handle, class: u32, info: *const c_void, length: u32) -> i32;
    fn QueryInformationJobObject(
        job: Handle,
        class: u32,
        info: *mut c_void,
        length: u32,
        returned: *mut u32,
    ) -> i32;
    fn AssignProcessToJobObject(job: Handle, process: Handle) -> i32;
    fn GetProcessAffinityMask(
        process: Handle,
        process_mask: *mut usize,
        system_mask: *mut usize,
    ) -> i32;
    fn GetPriorityClass(process: Handle) -> u32;
    fn IsProcessInJob(process: Handle, job: Handle, result: *mut i32) -> i32;
    fn CloseHandle(handle: Handle) -> i32;
}

pub struct Budget {
    job: Handle,
    mask: usize,
    profile: Profile,
}
impl Budget {
    pub fn permitted_mask() -> io::Result<usize> {
        let mut available = 0;
        let mut system = 0;
        // SAFETY: initialized output masks and current-process pseudo handle.
        unsafe {
            check(GetProcessAffinityMask(
                GetCurrentProcess(),
                &mut available,
                &mut system,
            ))?;
        }
        Ok(available)
    }
    pub fn enter(profile: Profile, selected: usize) -> io::Result<Self> {
        // SAFETY: owned job handle, valid initialized structures and current-process pseudo handle.
        unsafe {
            let process = GetCurrentProcess();
            let mut available = 0;
            let mut system = 0;
            check(GetProcessAffinityMask(process, &mut available, &mut system))?;
            let mask = match profile {
                Profile::Conservative2 => first_two(available),
                Profile::Build4 => {
                    let scope = windows_topology::scope()
                        .map_err(|_| io::Error::other("Permitted CPU scope recheck failed"))?;
                    if selected.count_ones() != 4 || selected & !scope.allowed_mask != 0 {
                        return Err(io::Error::other("Build4 permitted CPU scope changed"));
                    }
                    selected
                }
            };
            if mask == 0 || mask != selected {
                return Err(io::Error::other("Permitted CPU selection changed"));
            }
            let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if job.is_null() {
                return Err(io::Error::last_os_error());
            }
            let guard = Self { job, mask, profile };
            let mut limits = ExtendedLimits::default();
            limits.basic.flags = 0x10 | 0x20; // affinity and priority-class limits, no kill-on-close.
            limits.basic.affinity = mask;
            limits.basic.priority = BELOW_NORMAL;
            check(SetInformationJobObject(
                job,
                EXTENDED_INFORMATION,
                (&limits as *const ExtendedLimits).cast(),
                size_of::<ExtendedLimits>() as u32,
            ))?;
            let mut actual = ExtendedLimits::default();
            check(QueryInformationJobObject(
                job,
                EXTENDED_INFORMATION,
                (&mut actual as *mut ExtendedLimits).cast(),
                size_of::<ExtendedLimits>() as u32,
                std::ptr::null_mut(),
            ))?;
            if actual.basic.affinity != mask
                || actual.basic.priority != BELOW_NORMAL
                || actual.basic.flags & 0x30 != 0x30
            {
                return Err(io::Error::other("Job budget readback mismatch"));
            }
            check(AssignProcessToJobObject(job, process))?;
            guard.verify(process)?;
            Ok(guard)
        }
    }
    pub fn verify(&self, process: Handle) -> io::Result<(usize, u32)> {
        let mut mask = 0;
        let mut system = 0;
        let mut in_job = 0;
        // SAFETY: caller supplies its own live child handle or current-process pseudo handle.
        let priority = unsafe {
            check(GetProcessAffinityMask(process, &mut mask, &mut system))?;
            check(IsProcessInJob(process, self.job, &mut in_job))?;
            GetPriorityClass(process)
        };
        if mask == 0
            || in_job == 0
            || mask & !self.mask != 0
            || mask.count_ones() > self.profile.requested_cpus()
            || (self.profile == Profile::Build4 && mask != self.mask)
            || priority != BELOW_NORMAL
        {
            return Err(io::Error::other(
                "Owned process exceeds the CPU/priority budget",
            ));
        }
        Ok((mask, priority))
    }
}
impl Drop for Budget {
    fn drop(&mut self) {
        // SAFETY: this handle belongs exclusively to this guard; user processes were never assigned.
        unsafe {
            CloseHandle(self.job);
        }
    }
}
fn check(result: i32) -> io::Result<()> {
    if result == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn affinity_is_a_subset_and_never_exceeds_two_logical_cpus() {
        for mask in [0, 1, 2, 3, 0b101010, usize::MAX] {
            let selected = first_two(mask);
            assert_eq!(selected & !mask, 0);
            assert!(selected.count_ones() <= 2);
            assert_eq!(selected == 0, mask == 0);
        }
    }
}

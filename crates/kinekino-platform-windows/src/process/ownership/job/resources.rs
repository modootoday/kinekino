//! Read-only OS peak observations for one original owned process/Job.
use super::{
    super::handles::{Handle, error},
    FamilyJob,
};
use kinekino_core::platform::process::MemoryPeaks;
use std::{io, mem::size_of};
use windows::Win32::{
    Foundation::HANDLE,
    System::{
        JobObjects::{
            JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
            QueryInformationJobObject,
        },
        ProcessStatus::{
            K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX,
        },
    },
};

/// Query after child exit/family empty, before consuming the original owner.
/// Query failure is an error, never a zero-byte observation or a cleanup acknowledgement.
/// Both handles are borrowed from the original owner; no new handles/rights are created.
pub(in crate::process::ownership) fn query(
    process: &Handle,
    job: &FamilyJob,
) -> io::Result<MemoryPeaks> {
    let counters = process_memory(process.raw())?;
    let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
    // SAFETY: borrowed original handles, correctly sized initialized output buffers.
    unsafe {
        QueryInformationJobObject(
            Some(job.query_handle()),
            JobObjectExtendedLimitInformation,
            (&mut limits as *mut JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
            size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            None,
        )
        .map_err(error)?;
    }
    Ok(MemoryPeaks {
        process_peak_working_set_bytes: counters.PeakWorkingSetSize as u64,
        process_peak_pagefile_bytes: counters.PeakPagefileUsage as u64,
        process_private_bytes_at_query: counters.PrivateUsage as u64,
        job_peak_memory_bytes: limits.PeakJobMemoryUsed as u64,
    })
}

fn process_memory(handle: HANDLE) -> io::Result<PROCESS_MEMORY_COUNTERS_EX> {
    let mut process = PROCESS_MEMORY_COUNTERS_EX {
        cb: size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32,
        ..Default::default()
    };
    // SAFETY: the caller retains its original handle, output storage is correctly sized.
    if !unsafe {
        K32GetProcessMemoryInfo(
            handle,
            (&mut process as *mut PROCESS_MEMORY_COUNTERS_EX).cast::<PROCESS_MEMORY_COUNTERS>(),
            size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32,
        )
    }
    .as_bool()
    {
        return Err(io::Error::last_os_error());
    }
    Ok(process)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_original_handle_is_an_error_instead_of_a_zero_peak() {
        assert!(process_memory(HANDLE::default()).is_err());
    }
}

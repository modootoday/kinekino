//! A real empty Job reserved before CreateProcess; never an outer Job handle.
use super::{super::handles::error, FamilyJob};
use kinekino_core::platform::process::{
    FamilyBudget as JobBudget, FamilyObservation, PreparedFamily,
};
use std::io;
use windows::Win32::System::{
    JobObjects::*,
    Threading::{GetCurrentProcess, GetProcessAffinityMask},
};

/// Affinity mask currently allocated to this process.
pub(in crate::process::ownership) fn current_affinity() -> io::Result<usize> {
    let (mut process_mask, mut system_mask) = (0, 0);
    // SAFETY: current-process pseudo handle and initialized output words.
    unsafe {
        GetProcessAffinityMask(GetCurrentProcess(), &mut process_mask, &mut system_mask)
            .map_err(error)?;
    }
    Ok(process_mask)
}

#[must_use = "Retain this actual empty Job until transfer into the original child owner"]
pub(in crate::process::ownership) struct PreparedFamilyJob {
    pub(in crate::process::ownership) job: FamilyJob,
    pub(in crate::process::ownership) budget: JobBudget,
}
impl PreparedFamilyJob {
    pub(in crate::process::ownership) fn reserve(budget: JobBudget) -> io::Result<Self> {
        if budget.max_committed_bytes == 0 || budget.affinity_mask == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "positive memory and affinity required",
            ));
        }
        let (mut process_mask, mut system_mask) = (0, 0);
        // SAFETY: current-process pseudo handle and initialized output words.
        unsafe {
            GetProcessAffinityMask(GetCurrentProcess(), &mut process_mask, &mut system_mask)
                .map_err(error)?;
        }
        if budget.affinity_mask & process_mask != budget.affinity_mask
            || budget.affinity_mask & system_mask != budget.affinity_mask
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Job affinity exceeds current host allocation",
            ));
        }
        let job = FamilyJob::new()?;
        job.set_budget(budget)?;
        let result = Self { job, budget };
        result.check_current()?;
        Ok(result)
    }
}
impl PreparedFamily for PreparedFamilyJob {
    fn budget(&self) -> JobBudget {
        self.budget
    }
    fn check_current(&self) -> io::Result<()> {
        self.job.check_budget(self.budget)?;
        if self.job.observe()? != FamilyObservation::Empty {
            return Err(io::Error::other("Prepared Job is no longer empty"));
        }
        Ok(())
    }
    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any + Send> {
        self
    }
}

impl FamilyJob {
    fn set_budget(&self, budget: JobBudget) -> io::Result<()> {
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
            | JOB_OBJECT_LIMIT_JOB_MEMORY
            | JOB_OBJECT_LIMIT_AFFINITY;
        limits.BasicLimitInformation.Affinity = budget.affinity_mask;
        limits.JobMemoryLimit = budget.max_committed_bytes;
        // SAFETY: exclusively owned unnamed Job and correctly sized initialized structure.
        unsafe {
            SetInformationJobObject(
                self.query_handle(),
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                std::mem::size_of_val(&limits) as u32,
            )
            .map_err(error)?;
        }
        self.check_budget(budget)
    }
    pub(in crate::process::ownership) fn check_budget(&self, budget: JobBudget) -> io::Result<()> {
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        // SAFETY: original owned Job and correctly sized writable output.
        unsafe {
            QueryInformationJobObject(
                Some(self.query_handle()),
                JobObjectExtendedLimitInformation,
                (&mut limits as *mut JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                std::mem::size_of_val(&limits) as u32,
                None,
            )
            .map_err(error)?;
        }
        let expected = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
            | JOB_OBJECT_LIMIT_JOB_MEMORY
            | JOB_OBJECT_LIMIT_AFFINITY;
        if limits.BasicLimitInformation.LimitFlags != expected
            || limits.BasicLimitInformation.Affinity != budget.affinity_mask
            || limits.JobMemoryLimit != budget.max_committed_bytes
        {
            return Err(io::Error::other("Prepared Job budget readback mismatch"));
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_empty_job_has_exact_budget_and_invalid_budget_never_admits() {
        for budget in [
            JobBudget {
                max_committed_bytes: 0,
                affinity_mask: 1,
            },
            JobBudget {
                max_committed_bytes: 1,
                affinity_mask: 0,
            },
        ] {
            assert!(
                matches!(PreparedFamilyJob::reserve(budget), Err(e) if e.kind()==io::ErrorKind::InvalidInput)
            );
        }
        let (mut process_mask, mut system_mask) = (0, 0);
        unsafe {
            GetProcessAffinityMask(GetCurrentProcess(), &mut process_mask, &mut system_mask)
                .unwrap();
        }
        let mask = process_mask.isolate_lowest_one();
        let budget = JobBudget {
            max_committed_bytes: 256 * 1024 * 1024,
            affinity_mask: mask,
        };
        let job = PreparedFamilyJob::reserve(budget).unwrap();
        assert_eq!(job.budget(), budget);
        job.check_current().unwrap();
        assert!(
            job.job
                .check_budget(JobBudget {
                    max_committed_bytes: budget.max_committed_bytes + 1,
                    ..budget
                })
                .is_err()
        );
    }
}

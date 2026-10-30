//! Explicit local review reservation; no engine launch or playback-ready claim.
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        review_playback::{ReviewRuntimeLease, ReviewRuntimeProfile},
    },
    infrastructure::{immutable_storage_paths, mpv::TrustedMpvExecutable},
};
use kinekino_process::infrastructure::family::{JobBudget, PreparedFamilyJob};
use std::{
    fs::File,
    path::{Path, PathBuf},
    sync::Arc,
};

struct ReservationRoot {
    path: PathBuf,
    _guards: Vec<File>,
}
/// One fixed lock namespace must be shared by all local review runtime callers.
/// The central PM still allocates the project-wide heavy slot across other producers.
pub struct ReviewRuntimePool {
    root: Arc<ReservationRoot>,
    tool: Arc<TrustedMpvExecutable>,
    profile: ReviewRuntimeProfile,
    budget: JobBudget,
}
pub struct ReviewJobReservation {
    _root: Arc<ReservationRoot>,
    _exclusive: File,
    tool: Arc<TrustedMpvExecutable>,
    job: PreparedFamilyJob,
    profile: ReviewRuntimeProfile,
}
pub(crate) struct RunningReviewCustody {
    _root: Arc<ReservationRoot>,
    _exclusive: File,
    pub(crate) tool: Arc<TrustedMpvExecutable>,
    pub(crate) profile: ReviewRuntimeProfile,
}
impl ReviewJobReservation {
    pub(crate) fn into_launch_parts(self) -> (PreparedFamilyJob, RunningReviewCustody) {
        (
            self.job,
            RunningReviewCustody {
                _root: self._root,
                _exclusive: self._exclusive,
                tool: self.tool,
                profile: self.profile,
            },
        )
    }
}
impl ReviewRuntimePool {
    pub fn open(
        root: &Path,
        tool: TrustedMpvExecutable,
        profile: ReviewRuntimeProfile,
        budget: JobBudget,
    ) -> Result<Self, ApiError> {
        if profile.engine_sha256 != tool.sha256()
            || profile.max_source_bytes == 0
            || profile.startup_timeout_ms == 0
            || profile.max_job_committed_bytes == 0
            || u64::try_from(budget.max_committed_bytes).ok()
                != Some(profile.max_job_committed_bytes)
            || profile.cpu_threads == 0
            || profile.cpu_threads > 2
            || budget.affinity_mask.count_ones() != profile.cpu_threads
        {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Review runtime profile differs from fixed tool/conservative Job budget",
            ));
        }
        let (path, guards) = immutable_storage_paths::root(root)?;
        Ok(Self {
            root: Arc::new(ReservationRoot {
                path,
                _guards: guards,
            }),
            tool: Arc::new(tool),
            profile,
            budget,
        })
    }
    pub fn reserve_owned(&self) -> Result<ReviewJobReservation, ApiError> {
        let exclusive = immutable_storage_paths::lock(&self.root.path.join("review-runtime.lock"))?;
        let job = PreparedFamilyJob::reserve(self.budget).map_err(resource_error)?;
        let result = ReviewJobReservation {
            _root: self.root.clone(),
            _exclusive: exclusive,
            tool: self.tool.clone(),
            job,
            profile: self.profile.clone(),
        };
        result.check_current()?;
        Ok(result)
    }
}
impl ReviewRuntimeLease for ReviewJobReservation {
    fn profile(&self) -> &ReviewRuntimeProfile {
        &self.profile
    }
    fn check_current(&self) -> Result<(), ApiError> {
        self.job.check_current().map_err(resource_error)?;
        if self.tool.sha256() != self.profile.engine_sha256
            || u64::try_from(self.job.budget().max_committed_bytes).ok()
                != Some(self.profile.max_job_committed_bytes)
            || self.job.budget().affinity_mask.count_ones() != self.profile.cpu_threads
        {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Owned review runtime reservation changed",
            ));
        }
        Ok(())
    }
}
fn resource_error(error: std::io::Error) -> ApiError {
    ApiError::new(
        ErrorCode::ResourceExhausted,
        format!("Review Job reservation failed: {error}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        application::review_playback::{ReviewChildOwnership, ReviewRuntimeBackend},
        infrastructure::content_digest,
    };
    #[test]
    fn actual_job_and_exclusive_lock_survive_factory_drop_then_release() {
        let root = std::env::temp_dir().join(format!(
            "review-runtime-pool-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        // Inert bytes exercise pin custody only. No PE/engine execution is claimed.
        let executable = root.join("mpv.exe");
        std::fs::write(
            &executable,
            b"inert resource custody fixture; never executed",
        )
        .unwrap();
        let sha = content_digest::sha256(&mut File::open(&executable).unwrap(), 4096)
            .unwrap()
            .1;
        let profile = ReviewRuntimeProfile {
            backend: ReviewRuntimeBackend::OwnedLocalMpv,
            engine_sha256: sha.clone(),
            max_source_bytes: 1024,
            max_job_committed_bytes: 256 * 1024 * 1024,
            cpu_threads: 1,
            startup_timeout_ms: 30000,
            child_ownership: ReviewChildOwnership::OwnedJobUntilPhysicalExit,
        };
        let mask = crate::infrastructure::os_policy::process()
            .current_affinity()
            .unwrap()
            .unwrap_or(1);
        let budget = JobBudget {
            max_committed_bytes: 256 * 1024 * 1024,
            affinity_mask: mask.isolate_lowest_one(),
        };
        let pool = ReviewRuntimePool::open(
            &root,
            TrustedMpvExecutable::open(&executable, &sha).unwrap(),
            profile,
            budget,
        )
        .unwrap();
        let permit = pool.reserve_owned().unwrap();
        assert!(matches!(pool.reserve_owned(), Err(e) if e.code == ErrorCode::RevisionConflict));
        drop(pool);
        permit.check_current().unwrap();
        let mandatory = crate::infrastructure::os_policy::has_mandatory_share_modes();
        assert_eq!(
            std::fs::OpenOptions::new()
                .write(true)
                .open(&executable)
                .is_err(),
            mandatory
        );
        match std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(root.join("review-runtime.lock"))
        {
            Err(error) => {
                assert!(mandatory);
                assert!(crate::infrastructure::os_policy::is_sharing_violation(
                    &error
                ));
            }
            // Advisory locking: the second handle opens, but the exclusive lock still excludes it.
            Ok(other) => {
                assert!(!mandatory);
                assert_eq!(
                    crate::infrastructure::os_policy::try_lock_exclusive(&other)
                        .unwrap_err()
                        .kind(),
                    std::io::ErrorKind::WouldBlock
                );
            }
        }
        drop(permit);
        assert!(
            std::fs::OpenOptions::new()
                .write(true)
                .open(&executable)
                .is_ok()
        );
        assert!(
            std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(root.join("review-runtime.lock"))
                .is_ok()
        );
        println!("preserved inert runtime fixture {}", root.display());
    }
}

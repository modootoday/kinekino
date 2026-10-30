use crate::infrastructure::immutable_storage_paths;
#[cfg(test)]
mod capture_tests;
use kinekino_jobs::{
    application::{JobExecutor, JobIdentitySource, JobService, JobsApi},
    domain::JobId,
    infrastructure::{
        JobFiles, RetainingWorkerHost, RootedJobPaths, SubtitleJobExecutor, WorkerHost,
    },
};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use std::{
    fs::File,
    path::{Path, PathBuf},
    sync::Arc,
};
struct NativePaths;
impl RootedJobPaths for NativePaths {
    fn root(&self, path: &Path) -> Result<(PathBuf, Vec<File>), ApiError> {
        immutable_storage_paths::root(path)
    }
    fn directory(&self, path: &Path) -> Result<File, ApiError> {
        immutable_storage_paths::directory(path)
    }
    fn file(&self, path: &Path) -> Result<File, ApiError> {
        immutable_storage_paths::file(path)
    }
    fn lock(&self, path: &Path) -> Result<File, ApiError> {
        immutable_storage_paths::lock(path)
    }
}
struct Identities;
impl JobIdentitySource for Identities {
    fn next(&self) -> Result<JobId, ApiError> {
        let guid = crate::infrastructure::os_policy::fresh_guid_hex().map_err(|error| {
            ApiError::new(
                ErrorCode::EngineFailed,
                format!("Job identity allocation failed: {error}"),
            )
        })?;
        format!("job-{guid}")
            .try_into()
            .map_err(|message| ApiError::new(ErrorCode::Internal, message))
    }
}
pub fn read_only(root: &Path) -> Result<Arc<dyn JobsApi>, ApiError> {
    Ok(Arc::new(JobService::read_only(Arc::new(JobFiles::open(
        root,
        Arc::new(NativePaths),
    )?))))
}
pub fn start_host(root: &Path) -> Result<WorkerHost, ApiError> {
    start_host_with_executor(root, subtitle_executor())
}
pub fn subtitle_executor() -> Arc<dyn JobExecutor> {
    Arc::new(SubtitleJobExecutor::new(Arc::new(NativePaths)))
}
pub fn start_host_with_executor(
    root: &Path,
    executor: Arc<dyn JobExecutor>,
) -> Result<WorkerHost, ApiError> {
    let paths = Arc::new(NativePaths);
    let store = Arc::new(JobFiles::open(root, paths.clone())?);
    WorkerHost::start(store.clone(), executor, store, Arc::new(Identities))
}
pub fn start_retaining_host(root: &Path) -> Result<RetainingWorkerHost, ApiError> {
    start_retaining_host_with_executor(root, subtitle_executor())
}
pub fn start_retaining_host_with_executor(
    root: &Path,
    executor: Arc<dyn JobExecutor>,
) -> Result<RetainingWorkerHost, ApiError> {
    let paths = Arc::new(NativePaths);
    let store = Arc::new(JobFiles::open(root, paths.clone())?);
    WorkerHost::start_retaining(store.clone(), executor, store, Arc::new(Identities))
}

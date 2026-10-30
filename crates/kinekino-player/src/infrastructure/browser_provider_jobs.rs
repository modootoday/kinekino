use crate::{
    application::{
        api::{ApiError, ErrorCode},
        browser_provider::{BrowserCatalogJobs, BrowserEpisodeResolveJobs, BrowserHostState},
        catalog::{CatalogArchive, CatalogCaptureService},
        jobs::{JobExecutor, JobsApi},
    },
    infrastructure::{browser_provider, catalog, immutable_storage_paths as paths, jobs},
};
use kinekino_jobs::infrastructure::{
    JobHostOwner, RetainingWorkerHost, ShutdownReceipt, WorkerHost,
};
use serde::Serialize;
use std::{
    fs::{self, File},
    io::Read,
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};
mod shutdown;
pub use shutdown::{PendingProviderHost, ProviderShutdownComplete, ProviderShutdownOutcome};

pub struct ProviderJobsHost<W: JobHostOwner = WorkerHost> {
    jobs: Option<W>,
    browser: browser_provider::BrowserHost,
    files: Arc<catalog::CatalogFiles>,
    _profile_guards: Vec<File>,
    _profile_lock: File,
    jobs_never_started: bool,
    download_admission_panic: Option<Box<dyn std::any::Any + Send>>,
}
/// Owned-mode assembly cannot discard an incomplete dependency via Result shutdown.
/// ```compile_fail
/// use kinekino_player::infrastructure::browser_provider_jobs::RetainingProviderJobsHost;
/// fn close(owner: RetainingProviderJobsHost) { let _ = owner.shutdown(); }
/// ```
pub type RetainingProviderJobsHost = ProviderJobsHost<RetainingWorkerHost>;
#[must_use = "A partial provider still owns the original browser and profile guards"]
pub struct ProviderStartFailure<W: JobHostOwner> {
    error: ApiError,
    owner: Option<Box<ProviderJobsHost<W>>>,
}
impl<W: JobHostOwner> From<ApiError> for ProviderStartFailure<W> {
    fn from(error: ApiError) -> Self {
        Self { error, owner: None }
    }
}
impl<W: JobHostOwner> ProviderStartFailure<W> {
    /// The original payload remains in the same host through retained closeout.
    pub fn download_admission_panic(&self) -> Option<&(dyn std::any::Any + Send)> {
        self.owner.as_ref()?.download_admission_panic.as_deref()
    }
    pub fn error(&self) -> &ApiError {
        &self.error
    }
    pub(super) fn into_parts(self) -> (ApiError, Option<ProviderJobsHost<W>>) {
        (self.error, self.owner.map(|owner| *owner))
    }
    pub fn into_shutdown(
        self,
        deadline: Instant,
    ) -> (ApiError, Option<Box<PendingProviderHost<W>>>) {
        let (error, owner) = self.into_parts();
        (error, owner.map(|owner| owner.begin_shutdown(deadline)))
    }
    fn into_legacy_error(self) -> ApiError {
        self.error
    }
}
#[derive(Serialize)]
pub struct ProviderShutdownReceipt {
    pub jobs: ShutdownReceipt,
    pub browser_worker_exited: bool,
}
impl<W: JobHostOwner> ProviderJobsHost<W> {
    fn admit_download(
        &mut self,
        factory: &dyn super::download::DownloadHostFactory,
    ) -> Result<Arc<dyn crate::application::download::DownloadPreparation>, ApiError> {
        if self.download_admission_panic.is_some() {
            return Err(ApiError::new(
                ErrorCode::Internal,
                "Original download admission panic remains retained",
            ));
        }
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            factory.admit(self.browser.port(), self.browser.owner())
        })) {
            Ok(result) => result,
            Err(payload) => {
                self.download_admission_panic = Some(payload);
                let mut error =
                    ApiError::new(ErrorCode::Internal, "Download factory admission panicked");
                error.details = serde_json::json!({"original_panic_payload_retained":true});
                Err(error)
            }
        }
    }
    fn start_using(
        job_root: &Path,
        catalog_root: &Path,
        profile_root: &Path,
        start_jobs: fn(&Path, Arc<dyn JobExecutor>) -> Result<W, ApiError>,
        download_factory: Option<&dyn super::download::DownloadHostFactory>,
    ) -> Result<Self, ProviderStartFailure<W>> {
        let (profile, guards) = paths::root(profile_root)?;
        let (job, _) = paths::root(job_root)?;
        let (catalog, _) = paths::root(catalog_root)?;
        let roots = [&profile, &job, &catalog];
        for (index, left) in roots.iter().enumerate() {
            for right in roots.iter().skip(index + 1) {
                if left.starts_with(right) || right.starts_with(left) {
                    return Err(ApiError::new(
                        ErrorCode::InvalidArgument,
                        "Provider profile, jobs and catalog roots must be separate directories",
                    )
                    .into());
                }
            }
        }
        let profile_lock = paths::lock(&profile.join("provider-owner.lock"))?;
        let marker = profile.join("provider-profile.json");
        let marker_bytes = br#"{"profile_schema":"1","engine":"webview2-provider"}"#;
        if !marker.exists() {
            let mut entries =
                fs::read_dir(&profile).map_err(kinekino_core::file_publication::io_error)?;
            while let Some(entry) = entries
                .next()
                .transpose()
                .map_err(kinekino_core::file_publication::io_error)?
            {
                if entry.file_name() != "provider-owner.lock" {
                    return Err(ApiError::new(
                        ErrorCode::NeedsInput,
                        "Use an empty dedicated provider profile directory",
                    )
                    .into());
                }
            }
            kinekino_core::file_publication::publish(
                marker.to_str().ok_or_else(|| {
                    ApiError::new(
                        ErrorCode::InvalidArgument,
                        "Provider profile must be Unicode",
                    )
                })?,
                marker_bytes,
            )?;
        }
        let mut stored = Vec::new();
        paths::file(&marker)?
            .take(512)
            .read_to_end(&mut stored)
            .map_err(kinekino_core::file_publication::io_error)?;
        if stored != marker_bytes {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Provider profile identity mismatch",
            )
            .into());
        }
        let files = Arc::new(catalog::CatalogFiles::open(catalog_root)?);
        let capture: Arc<CatalogCaptureService> =
            Arc::new(catalog::capture_service_with_artifacts(files.clone()));
        let browser = browser_provider::start(profile_root)?;
        let mut owner = Self {
            jobs: None,
            browser,
            files: files.clone(),
            _profile_guards: guards,
            _profile_lock: profile_lock,
            jobs_never_started: true,
            download_admission_panic: None,
        };
        let executor = Arc::new(BrowserCatalogJobs::new(
            owner.browser.port(),
            owner.browser.owner(),
            capture,
            files.clone(),
        ));
        let preparation = match download_factory {
            Some(factory) => match owner.admit_download(factory) {
                Ok(preparation) => Some(preparation),
                Err(error) => {
                    return Err(ProviderStartFailure {
                        error,
                        owner: Some(Box::new(owner)),
                    });
                }
            },
            None => None,
        };
        let registry = match super::download::download_registry(
            vec![
                jobs::subtitle_executor(),
                executor,
                Arc::new(BrowserEpisodeResolveJobs::new(
                    owner.browser.port(),
                    owner.browser.owner(),
                    files.clone(),
                    Arc::new(crate::infrastructure::episode_document::HtmlEpisodeMarkup),
                    Arc::new(browser_provider::BrowserSystemClock),
                    files.clone(),
                )),
            ],
            preparation,
        ) {
            Ok(registry) => registry,
            Err(error) => {
                return Err(ProviderStartFailure {
                    error,
                    owner: Some(Box::new(owner)),
                });
            }
        };
        let jobs = match start_jobs(job_root, registry) {
            Ok(jobs) => jobs,
            Err(error) => {
                return Err(ProviderStartFailure {
                    error,
                    owner: Some(Box::new(owner)),
                });
            }
        };
        owner.jobs = Some(jobs);
        owner.jobs_never_started = false;
        Ok(owner)
    }
    pub fn jobs(&self) -> Arc<dyn JobsApi> {
        self.jobs
            .as_ref()
            .expect("live owned provider host")
            .client()
    }
    pub fn catalog(&self) -> CatalogArchive {
        CatalogArchive::new(self.files.clone(), self.files.clone())
            .with_episode_results(self.files.clone())
    }
    /// The original results/evidence store shared by browser jobs and routed playback.
    pub fn files(&self) -> Arc<catalog::CatalogFiles> {
        self.files.clone()
    }
}
impl ProviderJobsHost<RetainingWorkerHost> {
    pub fn start_retaining(
        job_root: &Path,
        catalog_root: &Path,
        profile_root: &Path,
    ) -> Result<Self, ProviderStartFailure<RetainingWorkerHost>> {
        Self::start_using(
            job_root,
            catalog_root,
            profile_root,
            jobs::start_retaining_host_with_executor,
            None,
        )
    }
    /// Explicit trusted SDK composition; the CLI's default entry remains unchanged.
    pub fn start_retaining_with_downloads(
        job_root: &Path,
        catalog_root: &Path,
        profile_root: &Path,
        factory: &dyn super::download::DownloadHostFactory,
    ) -> Result<Self, ProviderStartFailure<RetainingWorkerHost>> {
        Self::start_using(
            job_root,
            catalog_root,
            profile_root,
            jobs::start_retaining_host_with_executor,
            Some(factory),
        )
    }
}
impl ProviderJobsHost<WorkerHost> {
    pub fn start(
        job_root: &Path,
        catalog_root: &Path,
        profile_root: &Path,
    ) -> Result<Self, ApiError> {
        Self::start_local_with_failure(job_root, catalog_root, profile_root)
            .map_err(ProviderStartFailure::into_legacy_error)
    }
    pub(super) fn start_local_with_failure(
        job_root: &Path,
        catalog_root: &Path,
        profile_root: &Path,
    ) -> Result<Self, ProviderStartFailure<WorkerHost>> {
        Self::start_using(
            job_root,
            catalog_root,
            profile_root,
            jobs::start_host_with_executor,
            None,
        )
    }
    pub fn shutdown(mut self) -> Result<ProviderShutdownReceipt, ApiError> {
        let jobs = self
            .jobs
            .take()
            .ok_or_else(|| ApiError::new(ErrorCode::Internal, "Provider job owner is closed"))?
            .shutdown()?;
        let owner = self.browser.owner();
        owner.request_shutdown()?;
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            match owner.shutdown_snapshot() {
                Ok(snapshot) if snapshot.worker_exited => {
                    if snapshot.state == BrowserHostState::Failed {
                        return Err(ApiError::new(
                            ErrorCode::EngineFailed,
                            "Provider worker exited with failure",
                        ));
                    }
                    return Ok(ProviderShutdownReceipt {
                        jobs,
                        browser_worker_exited: true,
                    });
                }
                Ok(_) => {}
                Err(error) if error.code == ErrorCode::ResourceExhausted => {}
                Err(error) => return Err(error),
            }
            if Instant::now() >= deadline {
                let mut error = ApiError::new(
                    ErrorCode::Timeout,
                    "Provider shutdown was not confirmed within10seconds",
                );
                error.details = serde_json::json!({"browser_worker_exited":false,"jobs":jobs});
                return Err(error);
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
impl<W: JobHostOwner> Drop for ProviderJobsHost<W> {
    fn drop(&mut self) {
        let _ = self.browser.owner().request_shutdown();
    }
}

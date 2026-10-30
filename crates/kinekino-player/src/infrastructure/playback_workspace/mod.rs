//! Common owned browser jobs and native playback assembly.
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        catalog::CatalogArchive,
        jobs::{JobId, JobsApi},
        playback_session::{
            ClosedEpisodePlayback, EpisodePlaybackSessions,
            transition::{EpisodeTransitions, TransitionSlot},
        },
    },
    infrastructure::{
        browser_provider_jobs::{ProviderJobsHost, ProviderShutdownReceipt, ProviderStartFailure},
        catalog::CatalogFiles,
        mpv::{NativeEpisodeEngine, NativePlaybackHost},
    },
};
use kinekino_jobs::infrastructure::{JobHostOwner, RetainingWorkerHost, WorkerHost};
use serde::Serialize;
use std::{path::Path, sync::Arc};
mod shutdown;
pub use shutdown::{PendingPlaybackWorkspace, WorkspaceShutdownComplete, WorkspaceShutdownOutcome};

pub struct PlaybackWorkspaceHost<W: JobHostOwner = WorkerHost> {
    native: NativePlaybackHost,
    provider: Option<ProviderJobsHost<W>>,
    files: Arc<CatalogFiles>,
    transitions: Arc<TransitionSlot>,
    legacy_close: Option<LegacyProviderCloser<W>>,
}
struct LegacyProviderCloser<W: JobHostOwner> {
    close: fn(ProviderJobsHost<W>) -> Result<ProviderShutdownReceipt, ApiError>,
}
type ProviderStarter<W> =
    fn(&Path, &Path, &Path) -> Result<ProviderJobsHost<W>, ProviderStartFailure<W>>;
/// ```compile_fail
/// use kinekino_player::infrastructure::playback_workspace::RetainingPlaybackWorkspaceHost;
/// fn close(owner: RetainingPlaybackWorkspaceHost) { let _ = owner.shutdown(); }
/// ```
pub type RetainingPlaybackWorkspaceHost = PlaybackWorkspaceHost<RetainingWorkerHost>;
#[must_use = "A partial workspace may retain an original provider and native owner"]
pub struct WorkspaceStartFailure<W: JobHostOwner> {
    error: ApiError,
    owner: Option<Box<PlaybackWorkspaceHost<W>>>,
}
impl<W: JobHostOwner> From<ApiError> for WorkspaceStartFailure<W> {
    fn from(error: ApiError) -> Self {
        Self { error, owner: None }
    }
}
impl<W: JobHostOwner> WorkspaceStartFailure<W> {
    pub fn error(&self) -> &ApiError {
        &self.error
    }
    pub fn into_shutdown(
        self,
        deadline: std::time::Instant,
    ) -> (ApiError, Option<Box<PendingPlaybackWorkspace<W>>>) {
        (
            self.error,
            self.owner.map(|owner| owner.begin_shutdown(deadline)),
        )
    }
    fn into_legacy_error(self) -> ApiError {
        self.error
    }
}
pub struct PlaybackWorkspaceConfiguration<'a> {
    pub job_root: &'a Path,
    pub catalog_root: &'a Path,
    pub profile_root: &'a Path,
    pub mpv_executable: &'a Path,
    pub mpv_sha256: &'a str,
    pub parent: Option<u32>,
}
#[derive(Serialize)]
pub struct PlaybackWorkspaceShutdown {
    pub native: Result<Option<ClosedEpisodePlayback>, ApiError>,
    pub pending_transition_job: Result<Option<JobId>, ApiError>,
    pub provider: Result<ProviderShutdownReceipt, ApiError>,
}
impl<W: JobHostOwner> PlaybackWorkspaceHost<W> {
    fn start_using(
        config: PlaybackWorkspaceConfiguration<'_>,
        standalone: bool,
        start_provider: ProviderStarter<W>,
        legacy_close: Option<LegacyProviderCloser<W>>,
    ) -> Result<Self, WorkspaceStartFailure<W>> {
        let (files, native) = if standalone {
            if config.parent.is_some() {
                return Err(ApiError::new(
                    ErrorCode::InvalidArgument,
                    "Guarded standalone workspace cannot have a native parent",
                )
                .into());
            }
            let executable =
                NativePlaybackHost::verify_standalone(config.mpv_executable, config.mpv_sha256)?;
            let files = Arc::new(CatalogFiles::open(config.catalog_root)?);
            let native = NativePlaybackHost::from_verified_standalone(executable, files.clone())?;
            (files, native)
        } else {
            let files = Arc::new(CatalogFiles::open(config.catalog_root)?);
            let native = NativePlaybackHost::start(
                config.mpv_executable,
                config.mpv_sha256,
                config.parent,
                files.clone(),
            )?;
            (files, native)
        };
        let provider =
            match start_provider(config.job_root, config.catalog_root, config.profile_root) {
                Ok(provider) => provider,
                Err(failure) => {
                    let (error, provider) = failure.into_parts();
                    let owner = provider.map(|provider| {
                        Box::new(Self {
                            native,
                            provider: Some(provider),
                            files,
                            transitions: Arc::default(),
                            legacy_close,
                        })
                    });
                    return Err(WorkspaceStartFailure { error, owner });
                }
            };
        Ok(Self {
            native,
            provider: Some(provider),
            files,
            transitions: Arc::default(),
            legacy_close,
        })
    }
    pub fn engine(&self) -> NativeEpisodeEngine {
        self.native.engine()
    }
    pub fn sessions(&self) -> EpisodePlaybackSessions {
        EpisodePlaybackSessions::new(self.files.clone(), Arc::new(self.engine()))
    }
    pub fn files(&self) -> Arc<CatalogFiles> {
        self.files.clone()
    }
    pub fn transitions(&self) -> Arc<TransitionSlot> {
        self.transitions.clone()
    }
    pub fn transition_service(&self) -> Result<EpisodeTransitions, ApiError> {
        Ok(EpisodeTransitions::new(
            self.files.clone(),
            self.files.clone(),
            self.jobs()?,
            Arc::new(self.engine()),
            self.transitions.clone(),
        ))
    }
    pub fn jobs(&self) -> Result<Arc<dyn JobsApi>, ApiError> {
        self.provider
            .as_ref()
            .map(ProviderJobsHost::jobs)
            .ok_or_else(ended)
    }
    pub fn catalog(&self) -> Result<CatalogArchive, ApiError> {
        self.provider
            .as_ref()
            .map(ProviderJobsHost::catalog)
            .ok_or_else(ended)
    }
}
impl PlaybackWorkspaceHost<WorkerHost> {
    pub fn start(config: PlaybackWorkspaceConfiguration<'_>) -> Result<Self, ApiError> {
        Self::start_using(
            config,
            false,
            ProviderJobsHost::start_local_with_failure,
            Some(LegacyProviderCloser {
                close: ProviderJobsHost::shutdown,
            }),
        )
        .map_err(WorkspaceStartFailure::into_legacy_error)
    }
    pub fn start_trusted_standalone(
        config: PlaybackWorkspaceConfiguration<'_>,
    ) -> Result<Self, ApiError> {
        Self::start_using(
            config,
            true,
            ProviderJobsHost::start_local_with_failure,
            Some(LegacyProviderCloser {
                close: ProviderJobsHost::shutdown,
            }),
        )
        .map_err(WorkspaceStartFailure::into_legacy_error)
    }
    pub fn shutdown(mut self) -> Result<PlaybackWorkspaceShutdown, ApiError> {
        // Evaluate every owned shutdown even when an earlier component returns an error.
        let receipt = self.close_owned();
        if receipt.native.is_err()
            || receipt.pending_transition_job.is_err()
            || receipt.provider.is_err()
        {
            let mut error = ApiError::new(
                ErrorCode::EngineFailed,
                "Workspace shutdown was not fully confirmed",
            );
            error.details = serde_json::json!({"shutdown":receipt});
            return Err(error);
        }
        Ok(receipt)
    }
    fn close_owned(&mut self) -> PlaybackWorkspaceShutdown {
        let native = self.native.shutdown();
        let pending_transition_job = self.transitions.end();
        let provider = self
            .provider
            .take()
            .ok_or_else(ended)
            .and_then(ProviderJobsHost::shutdown);
        PlaybackWorkspaceShutdown {
            native,
            pending_transition_job,
            provider,
        }
    }
}
impl PlaybackWorkspaceHost<RetainingWorkerHost> {
    pub fn start_retaining(
        config: PlaybackWorkspaceConfiguration<'_>,
    ) -> Result<Self, WorkspaceStartFailure<RetainingWorkerHost>> {
        Self::start_using(
            config,
            false,
            ProviderJobsHost::<RetainingWorkerHost>::start_retaining,
            None,
        )
    }
    pub fn start_retaining_trusted_standalone(
        config: PlaybackWorkspaceConfiguration<'_>,
    ) -> Result<Self, WorkspaceStartFailure<RetainingWorkerHost>> {
        Self::start_using(
            config,
            true,
            ProviderJobsHost::<RetainingWorkerHost>::start_retaining,
            None,
        )
    }
}
impl<W: JobHostOwner> Drop for PlaybackWorkspaceHost<W> {
    fn drop(&mut self) {
        if self.provider.is_some() {
            let _ = self.native.shutdown();
            let _ = self.transitions.end();
            if let Some(provider) = self.provider.take() {
                if let Some(closer) = &self.legacy_close {
                    let _ = (closer.close)(provider);
                } else {
                    drop(provider);
                }
            }
        }
    }
}
fn ended() -> ApiError {
    ApiError::new(ErrorCode::Cancelled, "Playback workspace host has ended")
}

#[cfg(test)]
mod tests;

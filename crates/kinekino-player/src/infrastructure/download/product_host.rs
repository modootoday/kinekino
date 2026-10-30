//! Optional trusted Rust composition. No wire data, URL or CLI flag admits it.
use crate::application::{
    api::ApiError,
    browser_provider::{BrowserProviderOwner, BrowserProviderPort},
    download::{DownloadJobExecutor, DownloadPreparation},
    jobs::{ExecutorRegistry, JobExecutor},
};
use std::sync::Arc;

/// Called after the original browser host is retained, before registry admission.
/// Admission only connects ports/configuration: no source/thread/process launch.
/// The returned preparation must keep work inert until the original core owns
/// its opener. The already retained browser survives admission failure/unwind.
/// There is no permissive default factory.
pub trait DownloadHostFactory: Send + Sync {
    fn admit(
        &self,
        original_browser: Arc<dyn BrowserProviderPort>,
        original_owner: Arc<dyn BrowserProviderOwner>,
    ) -> Result<Arc<dyn DownloadPreparation>, ApiError>;
}

/// Absence leaves download unregistered. Caller-supplied trusted Rust preparation
/// controls source/tool admission; the common executor retains policy and CAS.
pub fn download_registry(
    mut base: Vec<Arc<dyn JobExecutor>>,
    preparation: Option<Arc<dyn DownloadPreparation>>,
) -> Result<Arc<dyn JobExecutor>, ApiError> {
    if let Some(preparation) = preparation {
        base.push(Arc::new(DownloadJobExecutor::new(preparation)));
    }
    Ok(Arc::new(ExecutorRegistry::new(base)?))
}

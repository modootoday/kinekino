//! Root product SDK composition. CLI only selects addresses, never permissions.
mod executor;
mod inputs;
mod release;
use crate::application::audio_vad::ProductVadHostOptions;
use kinekino_jobs::{application::ExecutorRegistry, infrastructure::RetainingWorkerHost};
use kinekino_media_contracts::errors::ApiError;
use std::path::Path;
use std::sync::Arc;

pub fn start_product_vad_jobs(
    job_root: &Path,
    options: ProductVadHostOptions,
) -> Result<RetainingWorkerHost, ApiError> {
    let plan = options.plan()?;
    // Independent Rust product release must exist before input directories are
    // opened or an execution capability/host/ready endpoint is advertised.
    let approved = Arc::new(release::OpenedProductRelease::open(&plan.release_root)?);
    let factory = Arc::new(inputs::ProductInputFactory::open(plan, approved)?);
    let vad = Arc::new(executor::ProductVadExecutor::new(factory));
    let executors = ExecutorRegistry::new(vec![super::jobs::subtitle_executor(), vad])?;
    super::jobs::start_retaining_host_with_executor(job_root, Arc::new(executors))
}

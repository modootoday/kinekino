//! Inert product assembly candidate; no operation registration or new scope.
use super::{
    vad_job_attempts::VadAttemptConfiguration, vad_job_executor::VadJobExecutor,
    vad_release_inputs::GuardedReleasedVadInputs,
};
use crate::application::audio_vad::product_release::ConfiguredVadRelease;
use kinekino_media_contracts::errors::ApiError;
use kinekino_vad_executor::application::ParentSelectedRelease;
use std::sync::Arc;
pub(super) fn configured_executor(
    inputs: Arc<GuardedReleasedVadInputs>,
    execution: VadAttemptConfiguration,
    release: Arc<ConfiguredVadRelease>,
    selected: Arc<ParentSelectedRelease>,
) -> Result<VadJobExecutor, ApiError> {
    VadJobExecutor::try_from((inputs, release, execution, selected))
}

//! Original Jobs CPU asset attempts; process-owned admission remains disabled.
mod host;
mod inspection;
pub use inspection::{AssetArtifactFiles, RetainedAssetArtifact};
mod owned;
mod publication;
pub use host::OpenedAssetBuildHost;
#[cfg(test)]
mod tests;
pub use crate::application::asset_bundle::BUILD_OPERATION;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        asset_bundle::*,
        asset_projects::AssetPublication,
        jobs::{ExecutionControl, JobExecutor, SourceBlobStore},
        review_source::MutationControl,
    },
    domain::{
        api_values::DecimalU64,
        asset_generation::{AssetFrameOrigin, AssetMaskReview},
    },
    infrastructure::{
        asset_bundle_encoding::RustAssetLayerEncoder,
        asset_inputs::{FixedAssetImage, FixedAssetInputs, FixedAssetMask},
        asset_projects::{AssetProjectFiles, NativeAssetProjectLease},
        asset_raster::RustAssetRaster,
    },
};
use kinekino_jobs::domain::{
    MAX_PARAMETERS_BYTES, MAX_SOURCE_BYTES, PreparedJobInput, VerifiedJobOutput,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};
type NativeOutput = PreparedAssetBundleOutput<NativeAssetProjectLease, FixedAssetInputs>;
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetBuildHostConfig {
    pub producer_sha256: String,
    pub project_root: PathBuf,
    pub artifact_root: PathBuf,
    pub images: BTreeMap<String, FixedAssetImage>,
    pub masks: BTreeMap<String, FixedAssetMask>,
}
#[derive(Clone)]
pub struct AssetBuildJobExecutor {
    config: Arc<AssetBuildHostConfig>,
    host_sha256: String,
    projects: Arc<AssetProjectFiles>,
    inputs: Arc<FixedAssetInputs>,
    artifacts: Arc<publication::AssetArtifacts>,
    _producer_guards: Arc<Vec<std::fs::File>>,
    _host_guards: Arc<Vec<std::fs::File>>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Capture {
    schema_version: String,
    host_sha256: String,
    publication: AssetPublication,
    plan: AssetBundlePlan,
}
struct Observe;
impl MutationControl for Observe {
    fn checkpoint(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
fn error(code: ErrorCode, message: &str) -> ApiError {
    ApiError::new(code, message)
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn local_producer_path(path: &std::path::Path) -> Result<PathBuf, ApiError> {
    use std::path::Component;
    let denied = || {
        error(
            ErrorCode::PermissionDenied,
            "Asset producer must be an ordinary absolute local disk path",
        )
    };
    let normalized =
        crate::infrastructure::os_policy::normalize_local_path(path).ok_or_else(denied)?;
    if !normalized.is_absolute()
        || normalized
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(denied());
    }
    Ok(normalized)
}
fn parse(operation: &str, value: Value) -> Result<AssetBundleRequest, ApiError> {
    if operation != BUILD_OPERATION {
        return Err(error(ErrorCode::Unsupported, "Asset operation unavailable"));
    }
    if serde_json::to_vec(&value)
        .map_err(|_| error(ErrorCode::InvalidArgument, "Invalid asset request"))?
        .len()
        > MAX_PARAMETERS_BYTES
    {
        return Err(error(
            ErrorCode::ResourceExhausted,
            "Asset request byte budget exceeded",
        ));
    }
    serde_json::from_value(value).map_err(|_| {
        error(
            ErrorCode::InvalidArgument,
            "Strict saved asset bundle request required",
        )
    })
}
impl AssetBuildJobExecutor {
    pub fn open(config: AssetBuildHostConfig) -> Result<Self, ApiError> {
        if !crate::domain::identifiers::valid_sha256(&config.producer_sha256) {
            return Err(error(
                ErrorCode::InvalidArgument,
                "Pinned asset producer SHA256 required",
            ));
        }
        let original_producer =
            std::env::current_exe().map_err(kinekino_core::file_publication::io_error)?;
        let producer = local_producer_path(&original_producer)?;
        let (_, mut producer_guards) = crate::infrastructure::immutable_storage_paths::root(
            producer
                .parent()
                .ok_or_else(|| error(ErrorCode::InvalidArgument, "Producer parent absent"))?,
        ).map_err(|mut failure| { failure.details=serde_json::json!({"stage":"asset_producer_parent","producer_path":original_producer,"original_details":failure.details}); failure })?;
        let mut producer_file = crate::infrastructure::immutable_storage_paths::file(&producer)?;
        let (_, actual_producer) =
            crate::infrastructure::content_digest::sha256(&mut producer_file, 512 * 1024 * 1024)?;
        if actual_producer != config.producer_sha256 {
            return Err(error(
                ErrorCode::SourceExpired,
                "Actual asset producer differs from pinned executable",
            ));
        }
        producer_guards.push(producer_file);
        let host_sha256 = hash(&serde_json::to_vec(&serde_json::json!({"config":config,"actual_producer_sha256":actual_producer,"execution_route":"core-cpu/1","renderer_recipe":"asset-raster/1","encoder_recipe":"asset-png/1"})).map_err(|_| {
            error(
                ErrorCode::InvalidArgument,
                "Invalid asset host configuration",
            )
        })?);
        Ok(Self {
            projects: Arc::new(AssetProjectFiles::open(&config.project_root)?),
            inputs: Arc::new(FixedAssetInputs::new(
                config.images.clone(),
                config.masks.clone(),
            )?),
            artifacts: Arc::new(publication::AssetArtifacts::open(&config.artifact_root)?),
            config: Arc::new(config),
            host_sha256,
            _producer_guards: Arc::new(producer_guards),
            _host_guards: Arc::new(Vec::new()),
        })
    }
    fn capture(
        &self,
        request: &AssetBundleRequest,
        control: &dyn MutationControl,
    ) -> Result<CapturedAssetProject<NativeAssetProjectLease>, ApiError> {
        let captured =
            crate::application::asset_bundle::capture(self.projects.as_ref(), request, control)?;
        for unit in &captured.plan().units {
            control.checkpoint()?;
            let selected = captured
                .publication()
                .document
                .frame_request(&unit.frame_id, &unit.layer_id)
                .map_err(|_| error(ErrorCode::SourceExpired, "Captured asset selection changed"))?;
            if !matches!(selected.image.origin, AssetFrameOrigin::StaticPng)
                || !matches!(
                    selected.review,
                    AssetMaskReview::UnreviewedAcknowledged { .. }
                )
            {
                return Err(error(
                    ErrorCode::Unsupported,
                    "Current native asset host requires static PNG and explicit unreviewed acknowledgement",
                ));
            }
            if self
                .config
                .images
                .get(&selected.image.blob.artifact_id)
                .is_none_or(|entry| entry.pin != selected.image)
                || self
                    .config
                    .masks
                    .get(&selected.mask.blob.artifact_id)
                    .is_none_or(|entry| entry.pin != selected.mask)
            {
                return Err(error(
                    ErrorCode::SourceExpired,
                    "Asset pins differ from fixed host catalog",
                ));
            }
        }
        Ok(captured)
    }
    fn snapshot(
        &self,
        captured: &CapturedAssetProject<NativeAssetProjectLease>,
    ) -> Result<String, ApiError> {
        let bytes = serde_json::to_string(&Capture {
            schema_version: "asset-job-capture/1".into(),
            host_sha256: self.host_sha256.clone(),
            publication: captured.publication().clone(),
            plan: captured.plan().clone(),
        })
        .map_err(|_| error(ErrorCode::InvalidArgument, "Invalid asset capture"))?;
        if bytes.len() as u64 > MAX_SOURCE_BYTES {
            return Err(error(
                ErrorCode::ResourceExhausted,
                "Asset capture byte budget exceeded",
            ));
        }
        Ok(bytes)
    }
    fn render(
        &self,
        input: &PreparedJobInput,
        blobs: &dyn SourceBlobStore,
        control: &dyn MutationControl,
    ) -> Result<NativeOutput, ApiError> {
        let request = parse(&input.operation, input.parameters.clone())?;
        let snapshot = blobs.read_utf8(&input.source)?;
        if snapshot.len() as u64 != input.source.bytes.0
            || hash(snapshot.as_bytes()) != input.source.sha256
        {
            return Err(error(
                ErrorCode::SourceExpired,
                "Original asset source blob changed",
            ));
        }
        let captured = self.capture(&request, control)?;
        if self.snapshot(&captured)? != snapshot {
            return Err(error(
                ErrorCode::SourceExpired,
                "Captured asset publication/plan/host changed",
            ));
        }
        let mut builder = AssetBundleOutputBuilder::new(&captured);
        for index in 0..captured.plan().units.len() {
            control.checkpoint()?;
            builder.push(
                captured
                    .render_frame(index, self.inputs.as_ref(), &RustAssetRaster, control)?
                    .encode(&RustAssetLayerEncoder, control)?,
                control,
            )?;
        }
        builder.finish(control)
    }
}
impl JobExecutor for AssetBuildJobExecutor {
    // Process-owned settlement does not admit NotStarted CPU owners. Use the existing CPU route.
    fn owned_executor(
        &self,
        _operation: &str,
    ) -> Option<&kinekino_jobs::application::owned_attempt::OwnedExecutorPort> {
        None
    }
    fn operations(&self) -> Vec<String> {
        vec![BUILD_OPERATION.into()]
    }
    fn supports(&self, operation: &str) -> bool {
        operation == BUILD_OPERATION
    }
    fn supports_batch_capture(&self, operation: &str) -> bool {
        self.supports(operation)
    }
    fn execution_timeout_limit_ms(&self, operation: &str) -> Option<u64> {
        self.supports(operation).then_some(120_000)
    }
    fn normalize_parameters(&self, operation: &str, value: Value) -> Result<Value, ApiError> {
        serde_json::to_value(parse(operation, value)?)
            .map_err(|_| error(ErrorCode::InvalidArgument, "Invalid asset request"))
    }
    fn prepare(
        &self,
        operation: &str,
        parameters: Value,
        expected: Option<&str>,
        blobs: &dyn SourceBlobStore,
    ) -> Result<PreparedJobInput, ApiError> {
        let request = parse(operation, parameters)?;
        let snapshot = self.snapshot(&self.capture(&request, &Observe)?)?;
        if expected.is_some_and(|expected| expected != hash(snapshot.as_bytes())) {
            return Err(error(
                ErrorCode::SourceExpired,
                "Expected asset capture differs",
            ));
        }
        let input = PreparedJobInput {
            operation: BUILD_OPERATION.into(),
            parameters: serde_json::to_value(request)
                .map_err(|_| error(ErrorCode::InvalidArgument, "Invalid asset request"))?,
            source: blobs.put_utf8(&snapshot)?,
            execution_timeout_ms: DecimalU64(120_000),
        };
        input
            .validate()
            .map_err(|message| error(ErrorCode::InvalidArgument, message))?;
        Ok(input)
    }
    fn execute(
        &self,
        input: &PreparedJobInput,
        blobs: &dyn SourceBlobStore,
        control: &dyn ExecutionControl,
    ) -> Result<VerifiedJobOutput, ApiError> {
        input
            .validate()
            .map_err(|message| error(ErrorCode::InvalidArgument, message))?;
        if input.operation != BUILD_OPERATION || input.execution_timeout_ms.0 > 120_000 {
            return Err(error(
                ErrorCode::InvalidArgument,
                "Unsupported CPU asset operation/budget",
            ));
        }
        let identity = control
            .execution_identity()
            .ok_or_else(|| {
                error(
                    ErrorCode::NeedsInput,
                    "Original Jobs CPU attempt identity required",
                )
            })?
            .clone();
        if identity.source_blob_sha256() != input.source.sha256
            || identity.input_sha256()
                != input
                    .sha256()
                    .map_err(|message| error(ErrorCode::InvalidArgument, message))?
        {
            return Err(error(
                ErrorCode::SourceExpired,
                "Original CPU asset attempt input changed",
            ));
        }
        let check = publication::CpuControl(control);
        check.checkpoint()?;
        let output = self.render(input, blobs, &check)?;
        publication::AssetCandidate::execute_cpu(self.artifacts.clone(), output, identity, control)
    }
}

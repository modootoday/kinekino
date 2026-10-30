//! Original Jobs adapter; admission is engine-free and execution retains original leases.
mod host;
mod owned;
mod source_admission;
#[cfg(test)]
mod tests;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        clip_projects::ClipProjectStore,
        clip_render::*,
        review_source::MutationControl,
    },
    domain::{api_values::DecimalU64, clip_document::ClipDocument, identifiers::valid_sha256},
    infrastructure::{
        clip_encoder::FixedClipEncoder,
        clip_presets::FixedClipPresets,
        clip_renderer::{self, ClipArtifactStore, ClipOutputVerifier, DeclaredBt709Limited},
        clip_sources::FfmpegClipSources,
        clip_subtitle_assets::SubtitleAssetFiles,
    },
};
pub use host::OpenedClipRenderHost;
use kinekino_jobs::{
    application::{ExecutionControl, JobExecutor, SourceBlobStore},
    domain::{MAX_PARAMETERS_BYTES, PreparedJobInput, VerifiedJobOutput},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

/// Trusted Rust assembly only. Color declarations remain unverified, never caller JSON.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClipRenderHostConfig {
    pub encoder: PathBuf,
    pub encoder_sha256: String,
    pub encoder_inventory_sha256: String,
    pub probe: PathBuf,
    pub probe_sha256: String,
    pub temporary_root: PathBuf,
    pub artifact_root: PathBuf,
    pub declared_bt709_source_sha256: BTreeSet<String>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Capture {
    schema_version: String,
    host_sha256: String,
    document: ClipDocument,
    preset_definition: crate::application::clip_render::presets::ClipPresetEvidence,
}
#[derive(Clone)]
pub struct ClipRenderJobExecutor {
    projects: Arc<dyn ClipProjectStore>,
    sources: Arc<FfmpegClipSources>,
    config: ClipRenderHostConfig,
    host_sha256: String,
    artifacts: Arc<ClipArtifactStore>,
    verifier: Arc<ClipOutputVerifier>,
    _host_guards: Arc<Vec<std::fs::File>>,
    subtitle_assets: Option<Arc<SubtitleAssetFiles>>,
    presets: Arc<FixedClipPresets>,
}
fn error(code: ErrorCode, message: &str) -> ApiError {
    ApiError::new(code, message)
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn parse(operation: &str, value: Value) -> Result<ClipRenderRequest, ApiError> {
    if operation != RENDER_OPERATION {
        return Err(error(
            ErrorCode::Unsupported,
            "Clip render operation unavailable",
        ));
    }
    if serde_json::to_vec(&value)
        .map_err(|_| error(ErrorCode::InvalidArgument, "Invalid clip request"))?
        .len()
        > MAX_PARAMETERS_BYTES
    {
        return Err(error(
            ErrorCode::ResourceExhausted,
            "Clip request byte limit exceeded",
        ));
    }
    serde_json::from_value(value).map_err(|_| {
        error(
            ErrorCode::InvalidArgument,
            "Expected strict saved clip render request",
        )
    })
}
struct Control<'a>(
    &'a dyn ExecutionControl,
    &'a crate::application::review_source::NativeProcessLedger,
);
struct CaptureControl;
impl MutationControl for CaptureControl {
    fn checkpoint(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
impl MutationControl for Control<'_> {
    fn native_processes(&self) -> Option<&crate::application::review_source::NativeProcessLedger> {
        Some(self.1)
    }
    fn checkpoint(&self) -> Result<(), ApiError> {
        if self.0.cancellation_requested() {
            return Err(error(
                ErrorCode::Cancelled,
                "Clip job cancelled before publication",
            ));
        }
        if Instant::now() >= self.0.deadline() {
            return Err(error(
                ErrorCode::Timeout,
                "Original clip job deadline elapsed",
            ));
        }
        Ok(())
    }
}
impl ClipRenderJobExecutor {
    pub fn open(
        projects: Arc<dyn ClipProjectStore>,
        sources: Arc<FfmpegClipSources>,
        config: ClipRenderHostConfig,
    ) -> Result<Self, ApiError> {
        if [
            &config.encoder_sha256,
            &config.encoder_inventory_sha256,
            &config.probe_sha256,
        ]
        .iter()
        .any(|sha| !valid_sha256(sha))
            || config
                .declared_bt709_source_sha256
                .iter()
                .any(|sha| !valid_sha256(sha))
        {
            return Err(error(
                ErrorCode::InvalidArgument,
                "Host tool, inventory and color source pins required",
            ));
        }
        let host_sha256 = hash(
            &serde_json::to_vec(&config)
                .map_err(|_| error(ErrorCode::InvalidArgument, "Invalid clip host config"))?,
        );
        let artifacts = ClipArtifactStore::open(&config.artifact_root)?;
        let verifier = ClipOutputVerifier::open(
            &config.probe,
            &config.probe_sha256,
            &config.encoder,
            &config.encoder_sha256,
            &config.temporary_root,
            Duration::from_secs(120),
        )?;
        Ok(Self {
            projects,
            sources,
            config,
            host_sha256,
            artifacts: Arc::new(artifacts),
            verifier: Arc::new(verifier),
            _host_guards: Arc::new(Vec::new()),
            subtitle_assets: None,
            presets: Arc::new(FixedClipPresets::open(std::collections::BTreeMap::new())?),
        })
    }
    pub fn with_preset_catalog(mut self, presets: Arc<FixedClipPresets>) -> Result<Self, ApiError> {
        self.host_sha256 = hash(
            &serde_json::to_vec(&[
                self.host_sha256.as_str(),
                presets.catalog_binding_sha256()?.as_str(),
            ])
            .map_err(|_| error(ErrorCode::InvalidArgument, "Invalid preset host binding"))?,
        );
        self.presets = presets;
        Ok(self)
    }
    pub fn with_subtitle_assets(mut self, assets: SubtitleAssetFiles) -> Result<Self, ApiError> {
        self.host_sha256 = hash(
            &serde_json::to_vec(&[
                self.host_sha256.as_str(),
                assets.catalog_binding_sha256()?.as_str(),
            ])
            .map_err(|_| error(ErrorCode::InvalidArgument, "Invalid subtitle host binding"))?,
        );
        self.subtitle_assets = Some(Arc::new(assets));
        Ok(self)
    }
    fn capture(&self, request: &ClipRenderRequest) -> Result<Capture, ApiError> {
        let document = capture_render_document(self.projects.as_ref(), request)?;
        let preset =
            presets::prepare_clip_preset(&document, self.presets.as_ref(), &CaptureControl)?;
        for selection in document
            .segments
            .iter()
            .filter_map(|segment| segment.subtitles.as_ref())
        {
            if !cfg!(feature = "native-proposal-window") {
                return Err(error(
                    ErrorCode::Unsupported,
                    "Common-font subtitle rendering requires native-proposal-window",
                ));
            }
            self.subtitle_assets
                .as_ref()
                .ok_or_else(|| {
                    error(
                        ErrorCode::Unsupported,
                        "Clip host has no configured subtitle assets",
                    )
                })?
                .require_selection_available(selection)?;
        }
        if !document.sources.iter().all(|source| {
            self.config
                .declared_bt709_source_sha256
                .contains(&source.sha256)
        }) {
            return Err(error(
                ErrorCode::NeedsInput,
                "Host has no explicit color interpretation for this source",
            ));
        }
        Ok(Capture {
            schema_version: "clip-job-capture/1".into(),
            host_sha256: self.host_sha256.clone(),
            document,
            preset_definition: preset.evidence().clone(),
        })
    }
}
impl JobExecutor for ClipRenderJobExecutor {
    fn owned_executor(
        &self,
        operation: &str,
    ) -> Option<&kinekino_jobs::application::owned_attempt::OwnedExecutorPort> {
        self.supports(operation)
            .then_some(self as &kinekino_jobs::application::owned_attempt::OwnedExecutorPort)
    }
    fn supports_batch_capture(&self, operation: &str) -> bool {
        self.supports(operation)
    }
    fn operations(&self) -> Vec<String> {
        vec![RENDER_OPERATION.into()]
    }
    fn supports(&self, operation: &str) -> bool {
        operation == RENDER_OPERATION
    }
    fn execution_timeout_limit_ms(&self, operation: &str) -> Option<u64> {
        self.supports(operation).then_some(120_000)
    }
    fn normalize_parameters(&self, operation: &str, parameters: Value) -> Result<Value, ApiError> {
        serde_json::to_value(parse(operation, parameters)?)
            .map_err(|_| error(ErrorCode::InvalidArgument, "Invalid clip request"))
    }
    fn prepare(
        &self,
        operation: &str,
        parameters: Value,
        expected: Option<&str>,
        blobs: &dyn SourceBlobStore,
    ) -> Result<PreparedJobInput, ApiError> {
        let request = parse(operation, parameters)?;
        let snapshot = serde_json::to_string(&self.capture(&request)?)
            .map_err(|_| error(ErrorCode::InvalidArgument, "Invalid clip capture"))?;
        if expected.is_some_and(|pin| pin != hash(snapshot.as_bytes())) {
            return Err(error(
                ErrorCode::SourceExpired,
                "Clip capture differs from expected SHA256",
            ));
        }
        let input = PreparedJobInput {
            operation: RENDER_OPERATION.into(),
            parameters: serde_json::to_value(request)
                .map_err(|_| error(ErrorCode::InvalidArgument, "Invalid clip request"))?,
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
        if input.execution_timeout_ms.0 > 120_000 {
            return Err(error(
                ErrorCode::InvalidArgument,
                "Clip job execution budget exceeds supported limit",
            ));
        }
        let native_processes = crate::application::review_source::NativeProcessLedger::default();
        let check = Control(control, &native_processes);
        check.checkpoint()?;
        let identity = control.execution_identity().ok_or_else(|| {
            error(
                ErrorCode::NeedsInput,
                "Original Jobs attempt identity required for clip publication",
            )
        })?;
        if identity.source_blob_sha256() != input.source.sha256
            || identity.input_sha256()
                != input
                    .sha256()
                    .map_err(|m| error(ErrorCode::InvalidArgument, m))?
        {
            return Err(error(
                ErrorCode::SourceExpired,
                "Original job identity differs from captured input",
            ));
        }
        let verified = self.render_retained(input, blobs, &check)?;
        let observations = native_processes.observations();
        if observations.is_empty() || observations.iter().any(|observation| !observation.closed()) {
            let mut failure = error(
                ErrorCode::Internal,
                "Original clip native owners are not observed closed",
            );
            if let Err(release) = verified.finish_release() {
                failure.details = json!({"working_input_release_error":{"code":release.code,"message":release.message,"details":release.details}});
            }
            return Err(failure);
        }
        let id = format!(
            "{}-attempt-{}",
            identity.job_id().as_str(),
            identity.attempt()
        );
        let retained = match self
            .artifacts
            .publish(&id, &verified.as_verified(), control)
        {
            Ok(retained) => retained,
            Err(mut error) => {
                if let Err(release_error) = verified.finish_release() {
                    if !error.details.is_object() {
                        error.details = json!({"original_details":error.details});
                    }
                    error.details["working_input_release_error"] = json!({"code":release_error.code,"message":release_error.message,"details":release_error.details});
                }
                return Err(error);
            }
        };
        let receipt = retained.receipt();
        // publish already performs post-commit readback with the original deadline and
        // late-cancellation semantics. Never introduce a second reservation or gate.
        let output = VerifiedJobOutput {
            path: self
                .config
                .artifact_root
                .join(format!("clip-{id}"))
                .join("manifest.json")
                .to_str()
                .ok_or_else(|| error(ErrorCode::InvalidArgument, "Non-UTF8 clip artifact root"))?
                .into(),
            sha256: receipt.manifest_sha256.clone(),
            bytes: receipt.manifest_bytes,
            receipt: json!({"schema_version":"clip-job-receipt/1","artifact":receipt,"color_verified":false,"quality_approved":false,"partial_cleanup_supported":false}),
        };
        output
            .validate()
            .map_err(|message| error(ErrorCode::InvalidArgument, message))?;
        verified.finish_release().map_err(|mut error| {
            if !error.details.is_object() {
                error.details = json!({"original_details":error.details});
            }
            error.details["publication_committed"] = json!(true);
            error.details["manifest_sha256"] = json!(output.sha256);
            error.details["stage"] = json!("clip_working_input_release");
            error
        })?;
        Ok(output)
    }
}

impl ClipRenderJobExecutor {
    fn render_retained(
        &self,
        input: &PreparedJobInput,
        blobs: &dyn SourceBlobStore,
        check: &dyn MutationControl,
    ) -> Result<clip_renderer::RetainedClipRenderCandidate, ApiError> {
        let request = parse(&input.operation, input.parameters.clone())?;
        let snapshot = blobs.read_utf8(&input.source)?;
        if hash(snapshot.as_bytes()) != input.source.sha256
            || snapshot.len() as u64 != input.source.bytes.0
        {
            return Err(error(
                ErrorCode::SourceExpired,
                "Stored clip capture changed",
            ));
        }
        let current = serde_json::to_string(&self.capture(&request)?)
            .map_err(|_| error(ErrorCode::InvalidArgument, "Invalid clip capture"))?;
        if current != snapshot {
            return Err(error(
                ErrorCode::SourceExpired,
                "Captured project or host configuration changed",
            ));
        }
        let media = source_admission::prepare_job_media(
            self.projects.as_ref(),
            request.project,
            &serde_json::from_str::<Capture>(&snapshot)
                .map_err(|_| error(ErrorCode::SourceExpired, "Invalid original clip capture"))?
                .document
                .sources,
            self.sources.as_ref(),
            check,
        )?;
        let encoder = prepare_encoder(
            &media.publication().document,
            self.presets.as_ref(),
            &FixedClipEncoder {
                path: self.config.encoder.clone(),
                sha256: self.config.encoder_sha256.clone(),
                inventory_timeout: Duration::from_secs(30),
            },
            check,
        )?;
        if encoder.lease().inventory().inventory_sha256 != self.config.encoder_inventory_sha256 {
            return Err(error(
                ErrorCode::SourceExpired,
                "Actual encoder inventory differs from pinned host inventory",
            ));
        }
        let segment = &media.publication().document.segments[0];
        let source = media
            .sources()
            .iter()
            .find(|source| source.facts().source.media_id == segment.media_id)
            .ok_or_else(|| {
                error(
                    ErrorCode::SourceExpired,
                    "Captured segment source unavailable",
                )
            })?;
        let boundary = source.observe_segment(segment, Duration::from_secs(30), check)?;
        let admitted_subtitles = match &segment.subtitles {
            Some(_) => subtitles::prepare_subtitles(
                &source.facts().source,
                segment,
                self.subtitle_assets.as_deref().ok_or_else(|| {
                    error(
                        ErrorCode::Unsupported,
                        "Clip host has no configured subtitle assets",
                    )
                })?,
                check,
            )?,
            None => None,
        };
        let overlay = admitted_subtitles
            .as_ref()
            .map(|subtitles| {
                clip_renderer::subtitles::prepare_subtitle_overlay(
                    &media,
                    &encoder,
                    &boundary,
                    subtitles,
                    &self.config.temporary_root,
                    check,
                )
            })
            .transpose()?;
        let color = DeclaredBt709Limited {
            source_sha256: source.facts().source.sha256.clone(),
        };
        let candidate = if let Some(overlay) = &overlay {
            clip_renderer::subtitles::encode_candidate_with_subtitles(
                overlay,
                &color,
                Duration::from_secs(120),
                128 * 1024 * 1024,
                check,
            )?
        } else {
            clip_renderer::encode_candidate(
                &media,
                &encoder,
                &boundary,
                &color,
                Duration::from_secs(120),
                128 * 1024 * 1024,
                check,
            )?
        };
        let verified = self.verifier.verify_owned(candidate, check)?;
        let overlay = overlay.map(|overlay| overlay.into_retained());
        let decoded = boundary.into_retained();
        let verified = clip_renderer::RetainedClipRenderCandidate::new(
            verified,
            media,
            encoder,
            decoded,
            admitted_subtitles,
            overlay,
        )?;
        Ok(verified)
    }
}

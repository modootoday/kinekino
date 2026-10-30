//! Per-request immutable media addresses. No global fixed-sample mapping.
mod audio;
mod cache;
use super::super::{
    immutable_storage_paths,
    vad_fixed_inputs::{FixedVadInputs, GuardedFixedVadInputs},
    vad_job_asset::{FixedVadFile, GuardedAsset},
    vad_job_attempts::VadAttemptConfiguration,
    vad_job_executor::VadJobExecutor,
    vad_release_inputs::{FixedVadInputMapping, FixedVadRelease, GuardedReleasedVadInputs},
};
use super::release::OpenedProductRelease;
use crate::application::audio_vad::{ProductVadHostPlan, VadSourceRequest};
use kinekino_jobs::{application::SourceBlobStore, domain::JobId};
use kinekino_media_contracts::{
    api_values::DecimalU64,
    errors::{ApiError, ErrorCode},
};
use kinekino_vad::{
    application::{AnalysisControl, wav_pcm},
    domain::{ArtifactPin, MAX_AUDIO_BYTES, VadInput},
};
use kinekino_vad_executor::{
    application::{SnapshotLease, persist_vad_descriptor},
    domain::OpaqueReference,
};
use std::{
    fs::{self, File},
    sync::Arc,
    time::Duration,
};
pub(super) struct ProductInputFactory {
    plan: ProductVadHostPlan,
    approved: Arc<OpenedProductRelease>,
    _directories: Vec<File>,
}
pub(super) struct BoundProductInput {
    pub executor: VadJobExecutor,
    _preparation: Vec<Arc<dyn SnapshotLease>>,
}
impl ProductInputFactory {
    pub(super) fn open(
        plan: ProductVadHostPlan,
        approved: Arc<OpenedProductRelease>,
    ) -> Result<Self, ApiError> {
        let mut directories = Vec::new();
        for root in [
            &plan.audio_root,
            &plan.output_root,
            &plan.cache_root,
            &plan.evidence_root,
        ] {
            if !root
                .try_exists()
                .map_err(kinekino_core::file_publication::io_error)?
            {
                return Err(ApiError::new(
                    ErrorCode::NeedsInput,
                    "Product VAD workspace requires existing audio/outputs/cache/evidence directories",
                ));
            }
            directories.extend(immutable_storage_paths::root(root)?.1);
        }
        Ok(Self {
            plan,
            approved,
            _directories: directories,
        })
    }
    pub(super) fn bind(
        &self,
        request: &VadSourceRequest,
        blobs: &dyn SourceBlobStore,
        control: Option<&dyn AnalysisControl>,
        job: Option<&JobId>,
    ) -> Result<BoundProductInput, ApiError> {
        let release = &self.approved.release;
        let public = request.parameters();
        if public.profile_id != release.public_profile_id()
            || public.expected_profile_manifest_sha256 != release.manifest().sha256()
        {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Requested release differs from product selection",
            ));
        }
        let audio = &public.audio_artifact_id;
        let destination = &public.destination_id;
        let snapshot = audio::snapshot(
            &self.plan.audio_root,
            audio,
            request.original_audio_sha256(),
            control,
        )?;
        let audio::PreparedAudio {
            path: wav_path,
            source,
            wav,
            pcm: pcm_pin,
            samples,
            bytes: wav_bytes,
            lease: wav_lease,
        } = snapshot;
        let pcm = wav_pcm(&wav_bytes)?;
        let input = VadInput::new(
            wav.clone(),
            pcm_pin.clone(),
            samples,
            DecimalU64(0),
            samples,
        )?;
        VadInput::new(
            wav.clone(),
            pcm_pin.clone(),
            samples,
            public.start_sample,
            public.end_sample,
        )?;
        let (pcm_file, pcm_lease) = cache::publish(
            &self.plan.cache_root,
            "pcm",
            &pcm_pin,
            pcm,
            MAX_AUDIO_BYTES,
            control,
        )?;
        cache::check(control)?;
        // Reuse the existing descriptor codec/publication/readback use case.
        let descriptor = persist_vad_descriptor(release.profile(), &input, blobs)?;
        let descriptor_pin = descriptor.decoded().raw().clone();
        let (descriptor_file, descriptor_lease) = cache::publish(
            &self.plan.cache_root,
            "descriptor",
            &descriptor_pin,
            descriptor.raw().as_bytes(),
            kinekino_vad::protocol::application::MAX_DESCRIPTOR_BYTES,
            control,
        )?;
        let internal_profile = OpaqueReference::new(release.profile().id().into())?;
        let internal_audio = OpaqueReference::new(audio.clone())?;
        let internal_destination = OpaqueReference::new(destination.clone())?;
        let fixed = Arc::new(GuardedFixedVadInputs::new(FixedVadInputs {
            profile_descriptor: descriptor_file,
            audio_id: internal_audio.clone(),
            original_audio: FixedVadFile {
                path: wav_path.clone(),
                pin: source,
            },
            wav: FixedVadFile {
                path: wav_path,
                pin: wav,
            },
            pcm: pcm_file,
            samples,
            destination_id: internal_destination.clone(),
            destination: self
                .plan
                .output_root
                .join(format!("result-{destination}.json")),
        })?);
        let native = release.native_deployment()?;
        let inputs = Arc::new(GuardedReleasedVadInputs::new(
            FixedVadRelease {
                public_profile_id: release.public_profile_id().into(),
                internal_profile,
                private_snapshot: descriptor_pin,
                manifest: FixedVadFile {
                    path: self.approved.root.join("release.manifest.json"),
                    pin: release.manifest().clone(),
                },
                artifacts: native
                    .artifacts
                    .iter()
                    .map(|f| FixedVadFile {
                        path: self.approved.root.join(f.name),
                        pin: f.pin.clone(),
                    })
                    .collect(),
            },
            fixed,
            FixedVadInputMapping {
                public_audio_id: audio.clone(),
                internal_audio,
                public_destination_id: destination.clone(),
                internal_destination,
            },
        )?);
        let evidence_root = match job {
            Some(job) => {
                cache::check(control)?;
                let path = self.plan.evidence_root.join(job.as_str());
                match fs::create_dir(&path) {
                    Ok(()) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                    Err(e) => return Err(kinekino_core::file_publication::io_error(e)),
                };
                path
            }
            None => self.plan.evidence_root.clone(),
        };
        let execution = VadAttemptConfiguration {
            worker: self
                .approved
                .selected
                .native_deployment()
                .ok_or_else(|| {
                    ApiError::new(
                        ErrorCode::NeedsInput,
                        "Product native worker path is not configured",
                    )
                })?
                .worker_path
                .clone()
                .into(),
            worker_pin: release.profile().runtime().clone(),
            evidence_root,
            cleanup_reserve: Duration::from_secs(1),
            environment: Default::default(),
        };
        let executor = super::super::vad_product_composition::configured_executor(
            inputs,
            execution,
            release.clone(),
            self.approved.selected.clone(),
        )?;
        cache::check(control)?;
        Ok(BoundProductInput {
            executor,
            _preparation: vec![wav_lease, pcm_lease, descriptor_lease],
        })
    }
}

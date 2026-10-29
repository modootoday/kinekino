//! Trusted fixed mapping of a public profile to original release files. No registry.
use super::vad_fixed_inputs::GuardedFixedVadInputs;
use super::vad_job_asset::{FixedVadFile, GuardedAsset};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_vad::application::AnalysisControl;
use kinekino_vad_executor::{
    ArtifactPin,
    application::{
        AudioArtifactLookup, AudioSnapshot, DestinationLookup, DestinationSnapshot,
        MAX_RELEASE_MANIFEST_BYTES, PreparedVadAdmission, ProfileLookup, ProfileSnapshot,
        SnapshotLease, decode_release_manifest,
    },
    domain::{ExpectedDigest, OpaqueReference},
};
use std::sync::Arc;

/// Values come from trusted application composition, never caller paths/argv.
pub(super) struct FixedVadRelease {
    pub public_profile_id: String,
    pub internal_profile: OpaqueReference,
    pub private_snapshot: ArtifactPin,
    pub manifest: FixedVadFile,
    pub artifacts: Vec<FixedVadFile>,
}
/// Trusted product identity mapping, separate from the release-file contract.
pub(super) struct FixedVadInputMapping {
    pub public_audio_id: String,
    pub internal_audio: OpaqueReference,
    pub public_destination_id: String,
    pub internal_destination: OpaqueReference,
}
pub(super) struct GuardedReleasedVadInputs {
    public_profile_id: String,
    internal_profile: OpaqueReference,
    private_snapshot: ArtifactPin,
    manifest: GuardedAsset,
    artifacts: Vec<GuardedAsset>,
    fixed: Arc<GuardedFixedVadInputs>,
    mapping: FixedVadInputMapping,
}
struct ReleaseLease {
    _leases: Vec<Arc<dyn SnapshotLease>>,
}
/// Actual guarded bytes from the trusted fixed producer, never request JSON.
pub(super) struct WorkerPayloadSources {
    pub manifest: ArtifactPin,
    pub manifest_bytes: Vec<u8>,
    pub model: Vec<u8>,
    pub license: Vec<u8>,
    pub wav: Vec<u8>,
    pub leases: Vec<Arc<dyn SnapshotLease>>,
}
impl SnapshotLease for ReleaseLease {}
/// Borrows the existing attempt's control only while resolving input snapshots.
pub(super) struct ControlledReleasedVadInputs<'a> {
    inputs: &'a GuardedReleasedVadInputs,
    control: &'a (dyn AnalysisControl + Send + Sync),
}
impl GuardedReleasedVadInputs {
    pub(super) fn controlled<'a>(
        &'a self,
        control: &'a (dyn AnalysisControl + Send + Sync),
    ) -> ControlledReleasedVadInputs<'a> {
        ControlledReleasedVadInputs {
            inputs: self,
            control,
        }
    }
    pub(super) fn new(
        binding: FixedVadRelease,
        fixed: Arc<GuardedFixedVadInputs>,
        mapping: FixedVadInputMapping,
    ) -> Result<Self, ApiError> {
        if !fixed.has_input_ids(&mapping.internal_audio, &mapping.internal_destination) {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Fixed VAD input mapping differs from producer",
            ));
        }
        if binding.artifacts.len() > 35 {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Too many fixed release files",
            ));
        }
        let mut artifacts: Vec<GuardedAsset> = Vec::new();
        for file in binding.artifacts {
            if artifacts.iter().any(|old| old.pin().id() == file.pin.id()) {
                return Err(ApiError::new(
                    ErrorCode::InvalidArgument,
                    "Duplicate fixed release file",
                ));
            }
            artifacts.push(GuardedAsset::new(
                file,
                kinekino_vad::domain::MAX_RUNTIME_BYTES,
            )?);
        }
        Ok(Self {
            public_profile_id: binding.public_profile_id,
            internal_profile: binding.internal_profile,
            private_snapshot: binding.private_snapshot,
            manifest: GuardedAsset::new(binding.manifest, MAX_RELEASE_MANIFEST_BYTES)?,
            artifacts,
            fixed,
            mapping,
        })
    }
    /// Public product identifiers are validated by the SDK. This exact mapping
    /// is configured by the host; no public string becomes a path or private ID.
    pub(super) fn resolve_profile(
        &self,
        public: &str,
        expected: &ExpectedDigest,
    ) -> Result<OpaqueReference, ApiError> {
        if public != self.public_profile_id {
            return Err(ApiError::new(
                ErrorCode::NotFound,
                "Public VAD profile is unavailable",
            ));
        }
        if expected.as_str() != self.manifest.pin().sha256() {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Original release manifest pin differs",
            ));
        }
        Ok(self.internal_profile.clone())
    }
    pub(super) fn resolve_audio(
        &self,
        public: &str,
        original_source: &ExpectedDigest,
    ) -> Result<OpaqueReference, ApiError> {
        if public != self.mapping.public_audio_id {
            return Err(ApiError::new(
                ErrorCode::NotFound,
                "Public VAD audio is unavailable",
            ));
        }
        self.fixed
            .check_audio_binding(&self.mapping.internal_audio, original_source)?;
        Ok(self.mapping.internal_audio.clone())
    }
    pub(super) fn resolve_destination(&self, public: &str) -> Result<OpaqueReference, ApiError> {
        if public != self.mapping.public_destination_id {
            return Err(ApiError::new(
                ErrorCode::NotFound,
                "Public VAD destination is unavailable",
            ));
        }
        Ok(self.mapping.internal_destination.clone())
    }
}
impl GuardedReleasedVadInputs {
    fn profile_snapshot(
        &self,
        id: &OpaqueReference,
        expected: &ExpectedDigest,
        control: Option<&dyn AnalysisControl>,
    ) -> Result<ProfileSnapshot, ApiError> {
        if id != &self.internal_profile || expected.as_str() != self.manifest.pin().sha256() {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Release profile binding differs",
            ));
        }
        let (bytes, manifest_lease) = self.manifest.snapshot_with_control(control)?;
        let release = decode_release_manifest(&bytes, self.manifest.pin())?;
        let private = self.fixed.profile_snapshot(
            id,
            &ExpectedDigest::new(self.private_snapshot.sha256().to_owned())?,
            control,
        )?;
        if release.profile() != &private.profile
            || private.manifest != self.private_snapshot
            || release.artifacts().len() != self.artifacts.len()
        {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Release/private profile or file set differs",
            ));
        }
        let mut leases = vec![manifest_lease, private.lease];
        for expected in release.artifacts() {
            let file = self
                .artifacts
                .iter()
                .find(|file| file.pin() == expected)
                .ok_or_else(|| {
                    ApiError::new(
                        ErrorCode::SourceExpired,
                        "Original release artifact is unavailable",
                    )
                })?;
            let (_, lease) = file.snapshot_with_control(control)?;
            leases.push(lease);
        }
        Ok(ProfileSnapshot {
            profile: private.profile,
            manifest: self.manifest.pin().clone(),
            lease: Arc::new(ReleaseLease { _leases: leases }),
        })
    }
}
impl GuardedReleasedVadInputs {
    pub(super) fn worker_payload_sources(
        &self,
        admission: &PreparedVadAdmission,
        control: &dyn AnalysisControl,
    ) -> Result<WorkerPayloadSources, ApiError> {
        if self.manifest.pin() != admission.profile_manifest() {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Worker release differs from original admission",
            ));
        }
        let (manifest_bytes, manifest_lease) =
            self.manifest.snapshot_with_control(Some(control))?;
        let release = decode_release_manifest(&manifest_bytes, self.manifest.pin())?;
        let profile = admission.descriptor().decoded().profile();
        if release.profile() != profile {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Worker release profile differs from original descriptor",
            ));
        }
        let read = |pin: &ArtifactPin| {
            self.artifacts
                .iter()
                .find(|asset| asset.pin() == pin)
                .ok_or_else(|| {
                    ApiError::new(
                        ErrorCode::SourceExpired,
                        "Original worker asset is unavailable",
                    )
                })?
                .snapshot_with_control(Some(control))
        };
        let (model, model_lease) = read(profile.weights())?;
        let (license, license_lease) = read(profile.license_document())?;
        let (wav, wav_lease) = self
            .fixed
            .worker_wav_snapshot(admission.descriptor().decoded().input().wav(), control)?;
        Ok(WorkerPayloadSources {
            manifest: self.manifest.pin().clone(),
            manifest_bytes,
            model,
            license,
            wav,
            leases: vec![manifest_lease, model_lease, license_lease, wav_lease],
        })
    }
}
impl ProfileLookup for GuardedReleasedVadInputs {
    fn lookup(
        &self,
        id: &OpaqueReference,
        expected: &ExpectedDigest,
    ) -> Result<ProfileSnapshot, ApiError> {
        self.profile_snapshot(id, expected, None)
    }
}
impl ProfileLookup for ControlledReleasedVadInputs<'_> {
    fn lookup(
        &self,
        id: &OpaqueReference,
        expected: &ExpectedDigest,
    ) -> Result<ProfileSnapshot, ApiError> {
        self.inputs
            .profile_snapshot(id, expected, Some(self.control))
    }
}
impl AudioArtifactLookup for ControlledReleasedVadInputs<'_> {
    fn lookup(
        &self,
        id: &OpaqueReference,
        expected: &ExpectedDigest,
    ) -> Result<AudioSnapshot, ApiError> {
        self.inputs
            .fixed
            .audio_snapshot(id, expected, Some(self.control))
    }
}
impl DestinationLookup for ControlledReleasedVadInputs<'_> {
    fn lookup(&self, id: &OpaqueReference) -> Result<DestinationSnapshot, ApiError> {
        DestinationLookup::lookup(self.inputs, id)
    }
}
impl AudioArtifactLookup for GuardedReleasedVadInputs {
    fn lookup(
        &self,
        id: &OpaqueReference,
        expected: &ExpectedDigest,
    ) -> Result<AudioSnapshot, ApiError> {
        AudioArtifactLookup::lookup(self.fixed.as_ref(), id, expected)
    }
}
impl DestinationLookup for GuardedReleasedVadInputs {
    fn lookup(&self, id: &OpaqueReference) -> Result<DestinationSnapshot, ApiError> {
        DestinationLookup::lookup(self.fixed.as_ref(), id)
    }
}

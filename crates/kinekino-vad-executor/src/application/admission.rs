use super::{OutputPublication, SavedVadDescriptor, persist_vad_descriptor};
use crate::domain::{ExpectedDigest, OpaqueReference};
use kinekino_jobs::application::SourceBlobStore;
use kinekino_media_contracts::{
    api_values::DecimalU64,
    errors::{ApiError, ErrorCode},
};
use kinekino_vad::{
    application::ProfileAdmission,
    domain::{ArtifactPin, VadInput, VadProfile},
};
use std::sync::Arc;
/// Typed internal use case, not a registered public JSON/command schema.
pub struct PrepareVadRequest {
    pub profile: OpaqueReference,
    pub profile_manifest: ExpectedDigest,
    pub audio: OpaqueReference,
    pub audio_source: ExpectedDigest,
    pub start: DecimalU64,
    pub end: DecimalU64,
    pub destination: OpaqueReference,
}
/// Trusted concrete implementations retain original immutable storage guards.
pub trait SnapshotLease: Send + Sync {}
pub struct ProfileSnapshot {
    pub profile: VadProfile,
    pub manifest: ArtifactPin,
    pub lease: Arc<dyn SnapshotLease>,
}
pub struct AudioSnapshot {
    pub id: OpaqueReference,
    pub source: ArtifactPin,
    pub wav: ArtifactPin,
    pub pcm: ArtifactPin,
    pub samples: DecimalU64,
    pub lease: Arc<dyn SnapshotLease>,
}
pub struct DestinationSnapshot {
    pub id: OpaqueReference,
    pub publication: Arc<dyn OutputPublication>,
    pub lease: Arc<dyn SnapshotLease>,
}
pub trait ProfileLookup: Send + Sync {
    /// Verify actual immutable manifest bytes/SHA and retain original guards.
    fn lookup(
        &self,
        id: &OpaqueReference,
        expected_manifest: &ExpectedDigest,
    ) -> Result<ProfileSnapshot, ApiError>;
}
pub trait AudioArtifactLookup: Send + Sync {
    /// Original source digest differs from derived WAV/PCM. Verify referenced
    /// bytes and retain immutable guards; a declared JSON pin is insufficient.
    fn lookup(
        &self,
        id: &OpaqueReference,
        expected_source: &ExpectedDigest,
    ) -> Result<AudioSnapshot, ApiError>;
}
pub trait DestinationLookup: Send + Sync {
    fn lookup(&self, id: &OpaqueReference) -> Result<DestinationSnapshot, ApiError>;
}
pub struct PreparedVadAdmission {
    descriptor: SavedVadDescriptor,
    profile_manifest: ArtifactPin,
    audio_source: ArtifactPin,
    destination: DestinationSnapshot,
    _leases: Vec<Arc<dyn SnapshotLease>>,
}
impl PreparedVadAdmission {
    pub fn profile_manifest(&self) -> &ArtifactPin {
        &self.profile_manifest
    }
    pub fn profile_audio_leases(&self) -> &[Arc<dyn SnapshotLease>] {
        &self._leases
    }
    pub fn descriptor(&self) -> &SavedVadDescriptor {
        &self.descriptor
    }
    pub fn audio_source(&self) -> &ArtifactPin {
        &self.audio_source
    }
    pub fn destination(&self) -> &DestinationSnapshot {
        &self.destination
    }
}
pub struct VadAdmissionPorts<'a> {
    pub profiles: &'a dyn ProfileLookup,
    pub audio: &'a dyn AudioArtifactLookup,
    pub destinations: &'a dyn DestinationLookup,
    pub admission: &'a dyn ProfileAdmission,
    pub blobs: &'a dyn SourceBlobStore,
}
pub fn prepare_vad_admission(
    request: &PrepareVadRequest,
    ports: VadAdmissionPorts<'_>,
) -> Result<PreparedVadAdmission, ApiError> {
    let profile = ports
        .profiles
        .lookup(&request.profile, &request.profile_manifest)?;
    if profile.profile.id() != request.profile.as_str()
        || profile.manifest.sha256() != request.profile_manifest.as_str()
    {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Profile lookup did not preserve requested immutable snapshot",
        ));
    };
    ports.admission.authorize(&profile.profile)?;
    let audio = ports.audio.lookup(&request.audio, &request.audio_source)?;
    if audio.id != request.audio || audio.source.sha256() != request.audio_source.as_str() {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Audio lookup did not preserve original source snapshot",
        ));
    };
    let input = VadInput::new(
        audio.wav,
        audio.pcm,
        audio.samples,
        request.start,
        request.end,
    )?;
    let destination = ports.destinations.lookup(&request.destination)?;
    if destination.id != request.destination {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Destination lookup identity changed",
        ));
    };
    let descriptor = persist_vad_descriptor(&profile.profile, &input, ports.blobs)?;
    Ok(PreparedVadAdmission {
        descriptor,
        profile_manifest: profile.manifest,
        audio_source: audio.source,
        destination,
        _leases: vec![profile.lease, audio.lease],
    })
}

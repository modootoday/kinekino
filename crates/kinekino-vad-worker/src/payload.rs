use crate::application::bind_static_worker;
use kinekino_media_contracts::errors::ApiError;
use kinekino_vad::protocol::application::{ValidatedWorkerRequest, decode_descriptor};
use kinekino_vad::{
    application::{AnalysisControl, ArtifactReader, ProfileAdmission, read_verified_artifact},
    domain::{ArtifactPin, MAX_AUDIO_BYTES, MAX_LICENSE_BYTES, MAX_MODEL_BYTES, VadProfile},
};
use std::sync::Arc;

#[derive(Clone, Copy)]
pub(crate) enum PayloadRole {
    Model,
    License,
    Wav,
}
impl PayloadRole {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Model => "model.onnx",
            Self::License => "model-license.txt",
            Self::Wav => "input.wav",
        }
    }
    pub(crate) fn maximum(self) -> u64 {
        match self {
            Self::Model => MAX_MODEL_BYTES,
            Self::License => MAX_LICENSE_BYTES,
            Self::Wav => MAX_AUDIO_BYTES,
        }
    }
}
pub(crate) struct PayloadArtifact {
    pub role: PayloadRole,
    pub pin: ArtifactPin,
    pub bytes: Vec<u8>,
}

/// Parent retains its original admission leases through staging and cleanup.
/// L is the existing owner type, not a new lease/registry/settlement protocol.
pub struct PreparedWorkerPayload<L: Send + Sync + ?Sized> {
    pub(crate) descriptor: Vec<u8>,
    pub(crate) descriptor_pin: ArtifactPin,
    pub(crate) artifacts: [PayloadArtifact; 3],
    pub(crate) original_lease: Arc<L>,
}
pub struct WorkerPayloadPorts<'a> {
    pub admission: &'a dyn ProfileAdmission,
    pub reader: &'a dyn ArtifactReader,
    pub control: &'a dyn AnalysisControl,
}
impl<L: Send + Sync + ?Sized> PreparedWorkerPayload<L> {
    pub fn prepare(
        request: &ValidatedWorkerRequest,
        descriptor_raw: &[u8],
        approved: &VadProfile,
        worker: &ArtifactPin,
        ports: WorkerPayloadPorts<'_>,
        original_lease: &Arc<L>,
    ) -> Result<Self, ApiError> {
        ports.admission.authorize(approved)?;
        bind_static_worker(approved, request.expected().profile(), worker)?;
        let descriptor_pin = request.expected().descriptor().raw().clone();
        decode_descriptor(descriptor_raw, &descriptor_pin)?;
        let input = request.expected().input();
        let read = |role: PayloadRole, pin: &ArtifactPin| {
            Ok::<_, ApiError>(PayloadArtifact {
                role,
                pin: pin.clone(),
                bytes: read_verified_artifact(ports.reader, pin, role.maximum(), ports.control)?,
            })
        };
        let artifacts = [
            read(PayloadRole::Model, approved.weights())?,
            read(PayloadRole::License, approved.license_document())?,
            read(PayloadRole::Wav, input.wav())?,
        ];
        Ok(Self {
            descriptor: descriptor_raw.to_vec(),
            descriptor_pin,
            artifacts,
            original_lease: original_lease.clone(),
        })
    }
}

use kinekino_jobs::{application::SourceBlobStore, domain::SourceBlobRef};
use kinekino_media_contracts::{
    api_values::DecimalU64,
    errors::{ApiError, ErrorCode},
};
use kinekino_vad::domain::{ArtifactPin, VadInput, VadProfile};
use kinekino_vad::protocol::{
    application::{MAX_DESCRIPTOR_BYTES, decode_descriptor},
    domain::DecodedDescriptor,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
pub struct SavedVadDescriptor {
    source: SourceBlobRef,
    decoded: DecodedDescriptor,
    raw: String,
}
impl SavedVadDescriptor {
    pub fn source(&self) -> &SourceBlobRef {
        &self.source
    }
    pub fn decoded(&self) -> &DecodedDescriptor {
        &self.decoded
    }
    pub fn raw(&self) -> &str {
        &self.raw
    }
}
pub(crate) fn snapshot(profile: &VadProfile, input: &VadInput) -> Value {
    let pin =
        |p: &ArtifactPin| json!({"id":p.id(),"sha256":p.sha256(),"bytes":p.bytes().to_string()});
    json!({"schema_version":"1","profile":{"id":profile.id(),"repo":profile.repo(),"revision":profile.revision(),"precision":profile.precision(),"provider":profile.provider(),"intra_threads":profile.intra_threads(),"inter_threads":profile.inter_threads(),"license_id":profile.license_id(),"weights":pin(profile.weights()),"license_document":pin(profile.license_document()),"runtime":pin(profile.runtime())},"input":{"wav":pin(input.wav()),"pcm":pin(input.pcm()),"sample_rate":16000,"channels":1,"bits_per_sample":16,"samples":input.samples().to_string(),"start_sample":input.start().to_string(),"end_sample":input.end().to_string()}})
}
/// Persists checked data, not runtime admission or SubmitJob expected-source semantics.
pub fn persist_vad_descriptor(
    profile: &VadProfile,
    input: &VadInput,
    blobs: &dyn SourceBlobStore,
) -> Result<SavedVadDescriptor, ApiError> {
    let raw = serde_json::to_string(&snapshot(profile, input))
        .map_err(|_| ApiError::new(ErrorCode::Internal, "VAD descriptor serialization failed"))?;
    if raw.len() as u64 > MAX_DESCRIPTOR_BYTES {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "VAD descriptor exceeds wire bound",
        ));
    };
    let pin = ArtifactPin::new(
        "vad-descriptor".into(),
        format!("{:x}", Sha256::digest(raw.as_bytes())),
        DecimalU64(raw.len() as u64),
    )?;
    let decoded = decode_descriptor(raw.as_bytes(), &pin)?;
    let source = blobs.put_utf8(&raw)?;
    if source.sha256 != pin.sha256() || source.bytes.0 != pin.bytes() {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Stored VAD descriptor identity differs from original bytes",
        ));
    };
    if blobs.read_utf8(&source)? != raw {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Stored VAD descriptor reread differs from original bytes",
        ));
    };
    Ok(SavedVadDescriptor {
        source,
        decoded,
        raw,
    })
}

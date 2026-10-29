//! VAD-specific release syntax and immutable identity, not admission authority.
use kinekino_media_contracts::{
    api_values::DecimalU64,
    errors::{ApiError, ErrorCode},
};
use kinekino_vad::domain::{
    ArtifactPin, MAX_RUNTIME_NOTICE_BYTES, MAX_RUNTIME_NOTICE_FILES, VadProfile,
    validate_runtime_notice_sizes,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};

pub const MAX_RELEASE_MANIFEST_BYTES: u64 = 64 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Pin {
    id: String,
    sha256: String,
    bytes: DecimalU64,
}
impl Pin {
    fn checked(self) -> Result<ArtifactPin, ApiError> {
        ArtifactPin::new(self.id, self.sha256, self.bytes)
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Files {
    weights: Pin,
    license_document: Pin,
    runtime: Pin,
    runtime_dependencies: Vec<Pin>,
    runtime_notices: Vec<Pin>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Runtime {
    minimum_version: String,
    provider: String,
    intra_threads: u32,
    inter_threads: u32,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Provenance {
    source_model: String,
    source_revision: String,
    conversion_recipe: String,
    conversion_version: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    schema_version: String,
    task: String,
    profile_id: String,
    repo_id: String,
    immutable_revision: String,
    format: String,
    quantization: String,
    license_id: String,
    files: Files,
    runtime: Runtime,
    provenance: Provenance,
    evaluation_id: Option<String>,
    measured_memory_bytes: Option<DecimalU64>,
}

/// Parsed declared release data. Even complete file pins do not prove license,
/// build provenance, runtime loading, quality or permission to execute.
pub struct VadReleaseManifest {
    profile: VadProfile,
    artifacts: Vec<ArtifactPin>,
    minimum_runtime_version: String,
    provenance: [String; 4],
    evaluation_id: Option<String>,
    measured_memory_bytes: Option<DecimalU64>,
}
impl VadReleaseManifest {
    pub fn profile(&self) -> &VadProfile {
        &self.profile
    }
    pub fn artifacts(&self) -> &[ArtifactPin] {
        &self.artifacts
    }
    pub fn minimum_runtime_version(&self) -> &str {
        &self.minimum_runtime_version
    }
    pub fn declared_provenance(&self) -> &[String; 4] {
        &self.provenance
    }
    pub fn declared_evaluation_id(&self) -> Option<&str> {
        self.evaluation_id.as_deref()
    }
    pub fn declared_memory_bytes(&self) -> Option<DecimalU64> {
        self.measured_memory_bytes
    }
}
fn invalid(message: &str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
fn bounded(value: &str) -> bool {
    !value.is_empty() && value.len() <= 256 && !value.chars().any(char::is_control)
}

pub fn decode_release_manifest(
    raw: &[u8],
    expected: &ArtifactPin,
) -> Result<VadReleaseManifest, ApiError> {
    if expected.bytes() > MAX_RELEASE_MANIFEST_BYTES
        || raw.len() as u64 > MAX_RELEASE_MANIFEST_BYTES
    {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "VAD release manifest exceeds bound",
        ));
    }
    if raw.len() as u64 != expected.bytes()
        || format!("{:x}", Sha256::digest(raw)) != expected.sha256()
    {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Release manifest original bytes/SHA differ",
        ));
    }
    let wire: Wire =
        serde_json::from_slice(raw).map_err(|_| invalid("Invalid VAD release manifest"))?;
    if wire.schema_version != "vad.release/1"
        || wire.task != "audio.vad"
        || wire.format != "onnx"
        || wire.files.runtime_dependencies.len() > 16
        || wire.files.runtime_notices.is_empty()
        || wire.files.runtime_notices.len() > MAX_RUNTIME_NOTICE_FILES
    {
        return Err(invalid("Unsupported VAD release schema or file bounds"));
    }
    let f = wire.files;
    let profile = VadProfile::new(
        wire.profile_id,
        wire.repo_id,
        wire.immutable_revision,
        f.weights.checked()?,
        f.license_document.checked()?,
        f.runtime.checked()?,
    )?;
    if wire.quantization != profile.precision()
        || wire.license_id != profile.license_id()
        || wire.runtime.provider != profile.provider()
        || wire.runtime.intra_threads != profile.intra_threads()
        || wire.runtime.inter_threads != profile.inter_threads()
    {
        return Err(invalid("Release differs from supported P0 profile"));
    }
    let provenance = [
        wire.provenance.source_model,
        wire.provenance.source_revision,
        wire.provenance.conversion_recipe,
        wire.provenance.conversion_version,
    ];
    if !bounded(&wire.runtime.minimum_version)
        || !provenance.iter().all(|value| bounded(value))
        || provenance[1].len() != 40
        || !provenance[1]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        || wire.evaluation_id.as_deref().is_some_and(|id| !bounded(id))
    {
        return Err(invalid("Invalid declared release metadata"));
    }
    let mut artifacts = vec![
        profile.weights().clone(),
        profile.license_document().clone(),
        profile.runtime().clone(),
    ];
    validate_runtime_notice_sizes(f.runtime_notices.iter().map(|pin| pin.bytes.0))?;
    for (pin, maximum) in f
        .runtime_dependencies
        .into_iter()
        .map(|pin| (pin, kinekino_vad::domain::MAX_RUNTIME_BYTES))
        .chain(
            f.runtime_notices
                .into_iter()
                .map(|pin| (pin, MAX_RUNTIME_NOTICE_BYTES)),
        )
    {
        let pin = pin.checked()?;
        if pin.bytes() > maximum || artifacts.iter().any(|old| old.id() == pin.id()) {
            return Err(invalid("Duplicate or oversized release artifact"));
        }
        artifacts.push(pin);
    }
    for (index, artifact) in artifacts.iter().enumerate() {
        if artifacts[..index]
            .iter()
            .any(|old| old.id() == artifact.id())
        {
            return Err(invalid("Duplicate release artifact identity"));
        }
    }
    Ok(VadReleaseManifest {
        profile,
        artifacts,
        minimum_runtime_version: wire.runtime.minimum_version,
        provenance,
        evaluation_id: wire.evaluation_id,
        measured_memory_bytes: wire.measured_memory_bytes,
    })
}

#[cfg(test)]
mod tests;

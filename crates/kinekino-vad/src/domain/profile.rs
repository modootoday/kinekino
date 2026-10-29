use super::invalid;
use kinekino_media_contracts::{api_values::DecimalU64, errors::ApiError};

pub const MAX_MODEL_BYTES: u64 = 4 * 1024 * 1024;
pub const MAX_RUNTIME_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_LICENSE_BYTES: u64 = 64 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactPin {
    id: String,
    sha256: String,
    bytes: DecimalU64,
}
impl ArtifactPin {
    pub fn new(id: String, sha256: String, bytes: DecimalU64) -> Result<Self, ApiError> {
        if id.is_empty()
            || id.len() > 128
            || !id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
            || !hex(&sha256, 64)
            || bytes.0 == 0
        {
            return Err(invalid("Invalid immutable artifact pin"));
        }
        Ok(Self { id, sha256, bytes })
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
    pub fn bytes(&self) -> u64 {
        self.bytes.0
    }
}
fn hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Immutable execution specification, not a quality review or admission credential.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VadProfile {
    id: String,
    repo: String,
    revision: String,
    weights: ArtifactPin,
    license: ArtifactPin,
    runtime: ArtifactPin,
}
impl VadProfile {
    /// The trusted ProfileAdmission port must authorize this complete specification.
    pub fn new(
        id: String,
        repo: String,
        revision: String,
        weights: ArtifactPin,
        license: ArtifactPin,
        runtime: ArtifactPin,
    ) -> Result<Self, ApiError> {
        if id.is_empty()
            || id.len() > 128
            || !id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
            || repo != "onnx-community/silero-vad"
            || !hex(&revision, 40)
            || weights.bytes() > MAX_MODEL_BYTES
            || license.bytes() > MAX_LICENSE_BYTES
            || runtime.bytes() > MAX_RUNTIME_BYTES
        {
            return Err(invalid("Invalid or unsupported VAD profile specification"));
        }
        Ok(Self {
            id,
            repo,
            revision,
            weights,
            license,
            runtime,
        })
    }
    pub fn cached_silero(license: ArtifactPin, runtime: ArtifactPin) -> Result<Self, ApiError> {
        Self::new(
            "silero-vad-fp32-cpu-v1".into(),
            "onnx-community/silero-vad".into(),
            "e71cae966052b992a7eca6b17738916ce0eca4ec".into(),
            ArtifactPin::new(
                "silero-vad-weights".into(),
                "a4a068cd6cf1ea8355b84327595838ca748ec29a25bc91fc82e6c299ccdc5808".into(),
                DecimalU64(2_243_022),
            )?,
            license,
            runtime,
        )
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn repo(&self) -> &str {
        &self.repo
    }
    pub fn revision(&self) -> &str {
        &self.revision
    }
    pub fn weights(&self) -> &ArtifactPin {
        &self.weights
    }
    pub fn license_document(&self) -> &ArtifactPin {
        &self.license
    }
    pub fn license_id(&self) -> &'static str {
        "MIT"
    }
    pub fn runtime(&self) -> &ArtifactPin {
        &self.runtime
    }
    pub fn precision(&self) -> &'static str {
        "fp32"
    }
    pub fn provider(&self) -> &'static str {
        "CPUExecutionProvider"
    }
    pub fn intra_threads(&self) -> u32 {
        2
    }
    pub fn inter_threads(&self) -> u32 {
        1
    }
}

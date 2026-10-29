use kinekino_media_contracts::api_values::DecimalU64;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

pub const MAX_SOURCE_BYTES: u64 = 8 * 1024 * 1024;
pub const MAX_PARAMETERS_BYTES: usize = 64 * 1024;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct JobId(String);
impl JobId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl TryFrom<String> for JobId {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if !value.strip_prefix("job-").is_some_and(|suffix| {
            suffix.len() == 32
                && suffix
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        }) {
            return Err("Invalid opaque job identity");
        }
        Ok(Self(value))
    }
}
impl From<JobId> for String {
    fn from(value: JobId) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceBlobRef {
    pub sha256: String,
    pub bytes: DecimalU64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedJobInput {
    pub operation: String,
    pub parameters: Value,
    pub source: SourceBlobRef,
    pub execution_timeout_ms: DecimalU64,
}
pub fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
impl PreparedJobInput {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.operation.is_empty()
            || self.operation.len() > 80
            || !self
                .operation
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_'))
            || !self.parameters.is_object()
            || !valid_sha256(&self.source.sha256)
            || self.source.bytes.0 == 0
            || self.source.bytes.0 > MAX_SOURCE_BYTES
            || !(1..=86_400_000).contains(&self.execution_timeout_ms.0)
            || serde_json::to_vec(&self.parameters)
                .map_err(|_| "Invalid job parameters")?
                .len()
                > MAX_PARAMETERS_BYTES
        {
            return Err("Invalid prepared job input");
        }
        Ok(())
    }
    pub fn sha256(&self) -> Result<String, &'static str> {
        self.validate()?;
        Ok(format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(self).map_err(|_| "Invalid job input JSON")?)
        ))
    }
}

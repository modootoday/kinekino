use super::*;
use crate::domain::identifiers::valid_sha256;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawEncoding {
    #[default]
    Utf8,
    Hex,
}
fn default_limit() -> usize {
    16_384
}
fn zero() -> DecimalU64 {
    DecimalU64(0)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadCatalogRaw {
    pub artifact_id: String,
    #[serde(default = "zero")]
    pub offset: DecimalU64,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub encoding: RawEncoding,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogRawChunk {
    pub raw_artifact: RawArtifact,
    pub capture: CaptureMetadata,
    pub encoding: RawEncoding,
    pub offset: DecimalU64,
    pub next_offset: Option<DecimalU64>,
    pub chunk_bytes: DecimalU64,
    pub chunk_sha256: String,
    pub text: String,
}
pub struct LoadedCatalogRaw {
    pub body: Vec<u8>,
    pub reference: RawArtifact,
    pub capture: CaptureMetadata,
}
pub trait CatalogRawSource: Send + Sync {
    fn load_raw(&self, artifact_id: &str) -> Result<LoadedCatalogRaw, ApiError>;
}
pub(super) fn read(
    source: &dyn CatalogRawSource,
    input: ReadCatalogRaw,
) -> Result<CatalogRawChunk, ApiError> {
    if !valid_sha256(&input.artifact_id) || !(1..=65536).contains(&input.limit) {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Raw read requires SHA256 and limit1..65536",
        ));
    }
    let raw = source.load_raw(&input.artifact_id)?;
    if raw.body.len() > crate::domain::browser_provider::MAX_CAPTURE_BYTES
        || raw.reference.artifact_id != input.artifact_id
        || raw.reference.body_sha256 != format!("{:x}", Sha256::digest(&raw.body))
        || raw.reference.body_bytes.0 != raw.body.len() as u64
        || raw.capture.body_sha256 != raw.reference.body_sha256
        || raw.capture.body_bytes != raw.reference.body_bytes
    {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Catalog raw identity mismatch",
        ));
    }
    let start = usize::try_from(input.offset.0)
        .ok()
        .filter(|v| *v <= raw.body.len())
        .ok_or_else(|| {
            ApiError::new(
                ErrorCode::InvalidArgument,
                "Raw offset is outside the artifact",
            )
        })?;
    let mut end = start.saturating_add(input.limit).min(raw.body.len());
    let text = match input.encoding {
        RawEncoding::Utf8 => {
            let text = std::str::from_utf8(&raw.body).map_err(|_| {
                ApiError::new(
                    ErrorCode::NeedsInput,
                    "Raw artifact is not UTF8; request encoding=hex",
                )
            })?;
            if !text.is_char_boundary(start) {
                return Err(ApiError::new(
                    ErrorCode::InvalidArgument,
                    "Raw UTF8 offset is inside a codepoint",
                ));
            }
            while !text.is_char_boundary(end) {
                end -= 1;
            }
            if start == end && start < raw.body.len() {
                return Err(ApiError::new(
                    ErrorCode::InvalidArgument,
                    "Raw limit cannot fit the next UTF8 codepoint",
                ));
            }
            text[start..end].into()
        }
        RawEncoding::Hex => raw.body[start..end]
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    };
    Ok(CatalogRawChunk {
        raw_artifact: raw.reference,
        capture: raw.capture,
        encoding: input.encoding,
        offset: input.offset,
        next_offset: (end < raw.body.len()).then_some(DecimalU64(end as u64)),
        chunk_bytes: DecimalU64((end - start) as u64),
        chunk_sha256: format!("{:x}", Sha256::digest(&raw.body[start..end])),
        text,
    })
}

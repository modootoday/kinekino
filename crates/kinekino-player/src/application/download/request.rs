//! Descriptive selections; only a trusted host can admit the original source.
use crate::application::{
    api::{ApiError, ErrorCode},
    jobs::SourceBlobStore,
};
use crate::domain::api_values::DecimalU64;
use kinekino_jobs::domain::PreparedJobInput;
use serde::{Deserialize, Serialize};

pub const DOWNLOAD_OPERATION: &str = "production.download_media";
pub const MAX_DOWNLOAD_BYTES: u64 = 256 * 1024 * 1024;
pub const MAX_DOWNLOAD_REQUEST_BYTES: usize = 4096;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DownloadSourceSelection {
    pub selection_id: String,
    /// Describes the selected source. This hash never grants live request access.
    pub descriptor_sha256: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DownloadRequest {
    pub source: DownloadSourceSelection,
    pub destination_id: String,
    pub maximum_bytes: DecimalU64,
}
impl DownloadRequest {
    pub fn schema() -> serde_json::Value {
        use serde_json::json;
        let id = json!({"type":"string","minLength":1,"maxLength":128,"pattern":"^[A-Za-z0-9._-]+$","not":{"enum":[".",".."]}});
        json!({"type":"object","additionalProperties":false,
        "required":["source","destination_id","maximum_bytes"],
        "properties":{
            "source":{"type":"object","additionalProperties":false,"required":["selection_id","descriptor_sha256"],
                "properties":{"selection_id":id.clone(),"descriptor_sha256":{"type":"string","pattern":"^[a-f0-9]{64}$"}},
                "description":"Descriptive selection only; a live original source requires trusted host admission"},
            "destination_id":id,
            "maximum_bytes":{"type":"string","pattern":"^[0-9]+$","description":"1..268435456 bytes; bounded again by the admitted host"}
        }})
    }
    pub fn validate(&self) -> Result<(), ApiError> {
        let id = |value: &str| {
            !value.is_empty()
                && value.len() <= 128
                && value != "."
                && value != ".."
                && value
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        };
        if !id(&self.source.selection_id)
            || !id(&self.destination_id)
            || !crate::domain::identifiers::valid_sha256(&self.source.descriptor_sha256)
            || !(1..=MAX_DOWNLOAD_BYTES).contains(&self.maximum_bytes.0)
        {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid descriptive download selection or byte limit",
            ));
        }
        Ok(())
    }
    pub fn decode(value: serde_json::Value) -> Result<Self, ApiError> {
        let input: Self = serde_json::from_value(value).map_err(|_| {
            ApiError::new(
                ErrorCode::InvalidArgument,
                "Download requires only source, destination_id and maximum_bytes",
            )
        })?;
        input.validate()?;
        Ok(input)
    }
    pub(super) fn descriptor(&self) -> Result<String, ApiError> {
        self.validate()?;
        let encoded = serde_json::to_string(&serde_json::json!({
            "schema_version":"1", "operation":DOWNLOAD_OPERATION,
            "source_kind":"request_descriptor", "request":self,
        }))
        .map_err(|_| {
            ApiError::new(
                ErrorCode::Internal,
                "Download descriptor serialization failed",
            )
        })?;
        if encoded.len() > MAX_DOWNLOAD_REQUEST_BYTES {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Download descriptor exceeds bound",
            ));
        }
        Ok(encoded)
    }
    pub fn from_prepared(
        input: &PreparedJobInput,
        blobs: &dyn SourceBlobStore,
    ) -> Result<Self, ApiError> {
        use sha2::{Digest, Sha256};
        if input.operation != DOWNLOAD_OPERATION {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Download operation is unavailable",
            ));
        }
        let request = Self::decode(input.parameters.clone())?;
        let descriptor = request.descriptor()?;
        if blobs.read_utf8(&input.source)? != descriptor
            || input.source.sha256 != format!("{:x}", Sha256::digest(descriptor.as_bytes()))
        {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Original download request descriptor changed",
            ));
        }
        Ok(request)
    }
}

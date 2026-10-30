use super::*;
use crate::{
    application::catalog::{CatalogRawSource, LoadedCatalogRaw},
    domain::identifiers::valid_sha256,
};
impl CatalogRawSource for CatalogFiles {
    fn load_raw(&self, artifact_id: &str) -> Result<LoadedCatalogRaw, ApiError> {
        let expired = || {
            ApiError::new(
                ErrorCode::SourceExpired,
                "Catalog raw body or metadata identity mismatch",
            )
        };
        if !valid_sha256(artifact_id) {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid raw artifact ID",
            ));
        }
        let metadata_path = format!("captures/{artifact_id}.json");
        let metadata = self.read_bounded(&metadata_path, 64 * 1024)?;
        if format!("{:x}", Sha256::digest(&metadata)) != artifact_id {
            return Err(expired());
        }
        let capture: CaptureMetadata = serde_json::from_slice(&metadata).map_err(|_| expired())?;
        if !valid_sha256(&capture.body_sha256)
            || capture.body_bytes.0 > crate::domain::browser_provider::MAX_CAPTURE_BYTES as u64
        {
            return Err(expired());
        }
        let body_path = format!("objects/{}.body", capture.body_sha256);
        let body = self.read_bounded(
            &body_path,
            crate::domain::browser_provider::MAX_CAPTURE_BYTES,
        )?;
        if format!("{:x}", Sha256::digest(&body)) != capture.body_sha256
            || body.len() as u64 != capture.body_bytes.0
        {
            return Err(expired());
        }
        let reference = RawArtifact {
            artifact_id: artifact_id.into(),
            body_sha256: capture.body_sha256.clone(),
            body_bytes: capture.body_bytes,
            body_path,
            metadata_path,
        };
        Ok(LoadedCatalogRaw {
            body,
            reference,
            capture,
        })
    }
}

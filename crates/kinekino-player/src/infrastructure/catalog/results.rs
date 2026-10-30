use super::*;
use crate::application::{
    catalog::{CatalogRawSource, CatalogResults, ResultArtifactRef, StoredCatalogResult},
    jobs::VerifiedJobOutput,
};
use crate::domain::{api_values::DecimalU64, identifiers::valid_sha256};
use serde_json::json;
const MAX_RESULT_BYTES: usize = 2 * 1024 * 1024;

impl CatalogFiles {
    pub(super) fn read_bounded(&self, relative: &str, limit: usize) -> Result<Vec<u8>, ApiError> {
        let file = paths::file(&self.root.join(relative))?;
        let mut bytes = Vec::new();
        file.take(limit as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(kinekino_core::file_publication::io_error)?;
        if bytes.len() > limit {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Catalog artifact exceeds its byte limit",
            ));
        }
        Ok(bytes)
    }
}
impl CatalogResults for CatalogFiles {
    fn publish(&self, result: &StoredCatalogResult) -> Result<VerifiedJobOutput, ApiError> {
        result.validate()?;
        let bytes = serde_json::to_vec(result).map_err(|_| {
            ApiError::new(ErrorCode::Internal, "Catalog result serialization failed")
        })?;
        if bytes.len() > MAX_RESULT_BYTES {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Catalog result exceeds2MiB",
            ));
        }
        let sha256 = format!("{:x}", Sha256::digest(&bytes));
        let relative = format!("results/{sha256}.json");
        self.immutable(&relative, &bytes)?;
        let (_, reference) = self.read(&sha256)?;
        Ok(VerifiedJobOutput {
            path: self
                .root
                .join(relative)
                .to_str()
                .ok_or_else(|| {
                    ApiError::new(ErrorCode::InvalidArgument, "Catalog root must be Unicode")
                })?
                .into(),
            sha256,
            bytes: DecimalU64(bytes.len() as u64),
            receipt: json!({"kind":"catalog_result","source_kind":"request_descriptor","result_artifact":reference}),
        })
    }
    fn read(&self, sha256: &str) -> Result<(StoredCatalogResult, ResultArtifactRef), ApiError> {
        let expired = || {
            ApiError::new(
                ErrorCode::SourceExpired,
                "Catalog result or referenced capture identity mismatch",
            )
        };
        if !valid_sha256(sha256) {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid result SHA256",
            ));
        }
        let bytes = self.read_bounded(&format!("results/{sha256}.json"), MAX_RESULT_BYTES)?;
        if format!("{:x}", Sha256::digest(&bytes)) != sha256 {
            return Err(expired());
        }
        let result: StoredCatalogResult = serde_json::from_slice(&bytes).map_err(|_| expired())?;
        result.validate()?;
        let raw = &result.result.raw_artifact;
        let loaded = self.load_raw(&raw.artifact_id)?;
        if loaded.reference != *raw
            || serde_json::to_value(&loaded.capture).map_err(|_| expired())?
                != serde_json::to_value(&result.result.capture).map_err(|_| expired())?
        {
            return Err(expired());
        }
        let reference = ResultArtifactRef {
            sha256: sha256.into(),
            bytes: DecimalU64(bytes.len() as u64),
            kind: "catalog_result".into(),
            operation: result.operation.clone(),
            source_descriptor_sha256: result.source_descriptor_sha256.clone(),
        };
        Ok((result, reference))
    }
}

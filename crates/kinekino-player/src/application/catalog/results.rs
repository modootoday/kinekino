use super::*;
use crate::{application::jobs::VerifiedJobOutput, domain::identifiers::valid_sha256};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadCatalogResult {
    pub result_artifact_sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultArtifactRef {
    pub sha256: String,
    pub bytes: DecimalU64,
    pub kind: String,
    pub operation: String,
    pub source_descriptor_sha256: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredCatalogResult {
    pub schema_version: String,
    pub operation: String,
    pub parameters: Value,
    pub source_descriptor_sha256: String,
    pub result: CatalogResult,
}
impl StoredCatalogResult {
    pub fn validate(&self) -> Result<(), ApiError> {
        let (parameters, location) = normalized_request(&self.operation, self.parameters.clone())?;
        if self.schema_version != "1"
            || parameters != self.parameters
            || !valid_sha256(&self.source_descriptor_sha256)
            || self.source_descriptor_sha256 != descriptor_sha256(&self.operation, &parameters)?
            || self.result.capture.requested_url != location.as_str()
            || self.result.site_verified
            || self.result.result_artifact.is_some()
        {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Stored catalog result identity mismatch",
            ));
        }
        Ok(())
    }
}
pub trait CatalogResults: Send + Sync {
    fn publish(&self, result: &StoredCatalogResult) -> Result<VerifiedJobOutput, ApiError>;
    fn read(&self, sha256: &str) -> Result<(StoredCatalogResult, ResultArtifactRef), ApiError>;
}
pub struct CatalogArchive {
    episode_results: Option<Arc<dyn crate::application::episode_resolve::EpisodeResolutionResults>>,
    results: Arc<dyn CatalogResults>,
    raw: Arc<dyn CatalogRawSource>,
}
impl CatalogArchive {
    pub fn new(results: Arc<dyn CatalogResults>, raw: Arc<dyn CatalogRawSource>) -> Self {
        Self {
            results,
            raw,
            episode_results: None,
        }
    }
    pub fn with_episode_results(
        mut self,
        results: Arc<dyn crate::application::episode_resolve::EpisodeResolutionResults>,
    ) -> Self {
        self.episode_results = Some(results);
        self
    }
}
impl CatalogApi for CatalogArchive {
    fn episode_resolution_read_available(&self) -> bool {
        self.episode_results.is_some()
    }
    fn inspect_episode_resolution(
        &self,
        input: crate::application::episode_resolve::InspectEpisodeResolution,
    ) -> Result<crate::application::episode_resolve::EpisodeResolutionInspection, ApiError> {
        crate::application::episode_resolve::inspect_resolution(
            self.episode_results.as_deref().ok_or_else(unavailable)?,
            input,
        )
    }
    fn available(&self) -> bool {
        true
    }
    fn live_available(&self) -> bool {
        false
    }
    fn result_read_available(&self) -> bool {
        true
    }
    fn raw_read_available(&self) -> bool {
        true
    }
    fn read_raw(&self, input: ReadCatalogRaw) -> Result<CatalogRawChunk, ApiError> {
        super::raw::read(self.raw.as_ref(), input)
    }
    fn read_result(
        &self,
        operation: &str,
        input: ReadCatalogResult,
    ) -> Result<CatalogResult, ApiError> {
        read(self.results.as_ref(), operation, input)
    }
}
pub(super) fn read(
    results: &dyn CatalogResults,
    operation: &str,
    input: ReadCatalogResult,
) -> Result<CatalogResult, ApiError> {
    if !valid_sha256(&input.result_artifact_sha256) {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Invalid catalog result SHA256",
        ));
    }
    let (mut stored, reference) = results.read(&input.result_artifact_sha256)?;
    stored.validate()?;
    if stored.operation != operation {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Result operation does not match requested command",
        ));
    }
    stored.result.result_artifact = Some(reference);
    Ok(stored.result)
}
pub(crate) fn normalized_request(
    operation: &str,
    parameters: Value,
) -> Result<(Value, PageLocation), ApiError> {
    let invalid =
        |error: serde_json::Error| ApiError::new(ErrorCode::InvalidArgument, error.to_string());
    match operation {
        "catalog.search" => {
            let input: SearchCatalog = serde_json::from_value(parameters).map_err(invalid)?;
            let location = input.location()?;
            Ok((serde_json::to_value(input).map_err(invalid)?, location))
        }
        "episodes.list" => {
            let input: ListEpisodes = serde_json::from_value(parameters).map_err(invalid)?;
            let location = input.location()?;
            Ok((serde_json::to_value(input).map_err(invalid)?, location))
        }
        _ => Err(unavailable()),
    }
}
pub(crate) fn descriptor(operation: &str, parameters: &Value) -> Result<String, ApiError> {
    serde_json::to_string(
        &json!({"operation":operation,"parameters":parameters,"source_kind":"request_descriptor"}),
    )
    .map_err(|_| {
        ApiError::new(
            ErrorCode::Internal,
            "Catalog descriptor serialization failed",
        )
    })
}
pub(crate) fn descriptor_sha256(operation: &str, parameters: &Value) -> Result<String, ApiError> {
    Ok(format!(
        "{:x}",
        Sha256::digest(descriptor(operation, parameters)?.as_bytes())
    ))
}

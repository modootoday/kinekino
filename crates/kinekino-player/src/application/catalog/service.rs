use super::*;
use kinekino_core::source::document::{CatalogFetchPort, ErrorKind};
use serde_json::json;
use std::sync::Arc;

pub struct CatalogService {
    fetch: Arc<dyn CatalogFetchPort>,
    capture: CatalogCaptureService,
    results: Option<Arc<dyn CatalogResults>>,
    raw: Option<Arc<dyn CatalogRawSource>>,
}
impl CatalogService {
    pub fn new(
        fetch: Arc<dyn CatalogFetchPort>,
        parser: Arc<dyn CatalogParser>,
        artifacts: Arc<dyn CatalogArtifacts>,
    ) -> Self {
        Self {
            fetch,
            capture: CatalogCaptureService::new(parser, artifacts),
            results: None,
            raw: None,
        }
    }
    pub fn with_results(mut self, results: Arc<dyn CatalogResults>) -> Self {
        self.results = Some(results);
        self
    }
    pub fn with_raw(mut self, raw: Arc<dyn CatalogRawSource>) -> Self {
        self.raw = Some(raw);
        self
    }
    fn read(&self, location: PageLocation) -> Result<CatalogResult, ApiError> {
        let raw = self
            .fetch
            .fetch(&location)
            .map_err(|error| failure(error, None, None))?;
        self.capture.http(&location, &raw)
    }
}
impl CatalogApi for CatalogService {
    fn available(&self) -> bool {
        true
    }
    fn raw_read_available(&self) -> bool {
        self.raw.is_some()
    }
    fn read_raw(&self, input: ReadCatalogRaw) -> Result<CatalogRawChunk, ApiError> {
        super::raw::read(self.raw.as_deref().ok_or_else(unavailable)?, input)
    }
    fn result_read_available(&self) -> bool {
        self.results.is_some()
    }
    fn read_result(
        &self,
        operation: &str,
        input: ReadCatalogResult,
    ) -> Result<CatalogResult, ApiError> {
        super::results::read(
            self.results.as_deref().ok_or_else(unavailable)?,
            operation,
            input,
        )
    }
    fn search(&self, input: SearchCatalog) -> Result<CatalogResult, ApiError> {
        self.read(input.location()?)
    }
    fn episodes(&self, input: ListEpisodes) -> Result<CatalogResult, ApiError> {
        self.read(input.location()?)
    }
}
pub(super) fn failure(
    error: CatalogError,
    artifact: Option<&RawArtifact>,
    capture: Option<&CaptureMetadata>,
) -> ApiError {
    let code = match error.kind {
        ErrorKind::InvalidLocation => ErrorCode::InvalidArgument,
        ErrorKind::Timeout => ErrorCode::Timeout,
        ErrorKind::BodyTooLarge => ErrorCode::ResourceExhausted,
        ErrorKind::AccessRequired
        | ErrorKind::UnrecognizedPage
        | ErrorKind::UnsupportedContent
        | ErrorKind::InvalidEncoding
        | ErrorKind::InvalidDocument => ErrorCode::NeedsInput,
        _ => ErrorCode::EngineFailed,
    };
    let mut result = ApiError::new(code, error.message);
    result.details = json!({"provider_error":error.kind,"transport":error.transport,"raw_artifact":artifact,"capture":capture,"site_verified":false});
    // Preserve the raw reference inside the durable job error budget.
    if serde_json::to_vec(&result.details).is_ok_and(|bytes| bytes.len() > 12_000) {
        result.details["capture"] = serde_json::Value::Null;
        result.details["capture_metadata_in_artifact"] = json!(artifact.is_some());
        result.details["capture_summary"] = capture.map_or(serde_json::Value::Null, |capture| json!({"capture_kind":capture.capture_kind,"status":capture.status,"body_sha256":capture.body_sha256,"body_bytes":capture.body_bytes,"observed_at":capture.observed_at,"parser_revision":capture.parser_revision}));
    }
    result.remediation = "Inspect captured evidence or transport diagnostics; no automatic retry or empty-list fallback".into();
    result
}

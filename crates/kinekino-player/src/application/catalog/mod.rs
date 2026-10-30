#[path = "ports.rs"]
mod ports;
pub use ports::CatalogApi;
mod capture;
mod raw;
mod results;
pub use raw::{CatalogRawChunk, CatalogRawSource, LoadedCatalogRaw, RawEncoding, ReadCatalogRaw};
#[cfg(test)]
mod raw_tests;
mod service;
#[cfg(test)]
mod tests;

pub use capture::{CatalogCaptureService, DomCatalogDocument};
use kinekino_core::source::document::{CatalogError, CatalogPage, RawDocument};
pub use kinekino_core::source::document::{Category, Entry, PageLocation, ResourceKind};
use kinekino_media_contracts::{
    api_values::DecimalU64,
    errors::{ApiError, ErrorCode},
};
pub use results::{
    CatalogArchive, CatalogResults, ReadCatalogResult, ResultArtifactRef, StoredCatalogResult,
};
pub(crate) use results::{descriptor, descriptor_sha256, normalized_request};
use serde::{Deserialize, Serialize};
pub use service::CatalogService;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchCatalog {
    pub category: Category,
    #[serde(default = "first_page")]
    pub page: u32,
}
impl SearchCatalog {
    pub fn location(&self) -> Result<PageLocation, ApiError> {
        if !(1..=10000).contains(&self.page) {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Catalog page must be 1..10000",
            ));
        }
        let category = self.category.location().map_err(|error| {
            let mut api = ApiError::new(ErrorCode::Unsupported, error.message);
            api.remediation =
                "install a document source plugin (see `kinekino plugins list`)".into();
            api
        })?;
        PageLocation::parse(&format!("{}?page={}", category.as_str(), self.page))
            .map_err(|_| ApiError::new(ErrorCode::InvalidArgument, "Invalid catalog page"))
    }
}
fn first_page() -> u32 {
    1
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListEpisodes {
    pub series_url: String,
}
impl ListEpisodes {
    pub fn location(&self) -> Result<PageLocation, ApiError> {
        let location = PageLocation::parse(&self.series_url)
            .map_err(|_| ApiError::new(ErrorCode::InvalidArgument, "Invalid series location"))?;
        if location.kind() != ResourceKind::Series {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "episodes.list requires an allowed series URL",
            ));
        }
        Ok(location)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawArtifact {
    pub artifact_id: String,
    pub body_sha256: String,
    pub body_bytes: DecimalU64,
    pub body_path: String,
    pub metadata_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureMetadata {
    pub requested_url: String,
    pub final_url: String,
    pub redirects: Vec<String>,
    pub observed_at: String,
    pub parser_revision: String,
    #[serde(default)]
    pub capture_kind: CatalogCaptureKind,
    pub status: Option<u16>,
    pub content_type: String,
    pub content_encoding: Option<String>,
    pub body_sha256: String,
    pub body_bytes: DecimalU64,
    #[serde(default)]
    pub browser: Option<BrowserCaptureProvenance>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalogCaptureKind {
    #[default]
    HttpEntity,
    DomSnapshot,
    BrowserResponseBody,
    ManifestPrefix,
    BrowserEventTrace,
    FrameDomSnapshot,
    EpisodeResolution,
    NativeEngineDiagnostic,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrowserFrameCaptureProvenance {
    pub identity: crate::application::browser_provider::FrameDocumentIdentity,
    pub completion: crate::application::browser_provider::FrameNavigationDiagnostic,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrowserCaptureProvenance {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network: Option<BrowserNetworkCaptureProvenance>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame: Option<BrowserFrameCaptureProvenance>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub navigation_diagnostic:
        Option<crate::application::browser_provider::MainNavigationDiagnostic>,
    pub ticket: crate::domain::browser_provider::BrowserTicket,
    pub document_generation: DecimalU64,
    pub navigation_history: Vec<crate::application::browser_provider::BrowserNavigationObservation>,
    pub engine_version: String,
    pub blocking_requested: bool,
    pub blocking_installed: bool,
    pub requested_count: DecimalU64,
    pub blocked_count: DecimalU64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrowserNetworkCaptureProvenance {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_headers: Option<crate::application::browser_network::RequestHeaderEvidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_method: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purpose: Option<crate::application::browser_network::ResponsePurpose>,
    pub session: crate::application::browser_network::ProtocolSession,
    pub scope: crate::application::browser_network::ProtocolDocumentScope,
    pub request_id: String,
    pub attempt: DecimalU64,
    pub response: crate::application::browser_network::NetworkResponse,
    pub referer: Option<String>,
    pub finished_encoded_bytes: DecimalU64,
    pub owner_context_at_body_observation: crate::application::browser_network::BodyOwnerContext,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogResult {
    pub title: String,
    pub series: Option<Entry>,
    pub entries: Vec<Entry>,
    pub next_page: Option<String>,
    pub raw_artifact: RawArtifact,
    pub capture: CaptureMetadata,
    pub site_verified: bool,
    #[serde(default)]
    pub result_artifact: Option<ResultArtifactRef>,
}

pub trait CatalogParser: Send + Sync {
    fn revision(&self) -> &str {
        "unavailable"
    }
    fn parse(&self, raw: &RawDocument) -> Result<CatalogPage, CatalogError>;
    fn parse_html(
        &self,
        _: &PageLocation,
        _: &str,
        _: kinekino_core::source::document::Provenance,
    ) -> Result<CatalogPage, CatalogError> {
        Err(CatalogError::new(
            kinekino_core::source::document::ErrorKind::UnsupportedContent,
            "Rendered HTML parser is unavailable",
        ))
    }
}
pub trait CatalogArtifacts: Send + Sync {
    fn save(&self, raw: &[u8], metadata: &CaptureMetadata) -> Result<RawArtifact, ApiError>;
}
pub struct UnavailableCatalog;
impl CatalogApi for UnavailableCatalog {}
pub fn unavailable() -> ApiError {
    ApiError::new(ErrorCode::Unsupported, "Catalog adapter is unavailable")
}
pub(crate) fn provider_error(error: CatalogError) -> ApiError {
    service::failure(error, None, None)
}

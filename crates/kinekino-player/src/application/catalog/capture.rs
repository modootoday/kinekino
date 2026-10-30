use super::*;
use kinekino_core::source::document::{ErrorKind, Provenance};
use sha2::{Digest, Sha256};
use std::sync::Arc;

pub struct DomCatalogDocument {
    pub location: PageLocation,
    pub body: Arc<[u8]>,
    pub provenance: Provenance,
    pub browser: BrowserCaptureProvenance,
}
pub struct CatalogCaptureService {
    parser: Arc<dyn CatalogParser>,
    artifacts: Arc<dyn CatalogArtifacts>,
}
impl CatalogCaptureService {
    pub fn new(parser: Arc<dyn CatalogParser>, artifacts: Arc<dyn CatalogArtifacts>) -> Self {
        Self { parser, artifacts }
    }
    pub fn parser_revision(&self) -> &str {
        self.parser.revision()
    }
    pub fn http(
        &self,
        location: &PageLocation,
        raw: &RawDocument,
    ) -> Result<CatalogResult, ApiError> {
        validate(location, &raw.body, &raw.provenance)?;
        if !(100..=599).contains(&raw.status)
            || raw.content_type.len() > 1024
            || raw.content_encoding.as_ref().is_some_and(|v| v.len() > 128)
            || raw.provenance.final_url
                != raw
                    .provenance
                    .redirects
                    .last()
                    .map_or(location.as_str(), String::as_str)
        {
            return Err(invalid());
        }
        let capture = metadata(
            &raw.provenance,
            CatalogCaptureKind::HttpEntity,
            Some(raw.status),
            raw.content_type.clone(),
            raw.content_encoding.clone(),
            None,
        )?;
        self.interpret(&raw.body, capture, || self.parser.parse(raw))
    }
    pub fn dom(&self, input: DomCatalogDocument) -> Result<CatalogResult, ApiError> {
        validate(&input.location, &input.body, &input.provenance)?;
        if !input.provenance.redirects.is_empty()
            || input.browser.engine_version.is_empty()
            || input.browser.engine_version.len() > 128
            || input.browser.document_generation.0 == 0
            || input.browser.navigation_history.last().is_none_or(|n| {
                n.document_generation != input.browser.document_generation
                    || n.url != input.provenance.final_url
            })
            || input.browser.navigation_history.len()
                > crate::domain::browser_provider::MAX_NAVIGATIONS
            || input.browser.blocked_count > input.browser.requested_count
            || input.browser.blocking_requested && !input.browser.blocking_installed
        {
            return Err(invalid());
        }
        let capture = metadata(
            &input.provenance,
            CatalogCaptureKind::DomSnapshot,
            None,
            "text/html; charset=utf-8".into(),
            None,
            Some(input.browser),
        )?;
        self.interpret(&input.body, capture, || {
            let html = std::str::from_utf8(&input.body).map_err(|_| {
                CatalogError::new(ErrorKind::InvalidEncoding, "Rendered catalog is not UTF8")
            })?;
            let final_location = PageLocation::parse(&input.provenance.final_url)?;
            self.parser
                .parse_html(&final_location, html, input.provenance)
        })
    }
    fn interpret(
        &self,
        bytes: &[u8],
        capture: CaptureMetadata,
        parse: impl FnOnce() -> Result<CatalogPage, CatalogError>,
    ) -> Result<CatalogResult, ApiError> {
        let artifact = self.artifacts.save(bytes, &capture)?;
        let page = parse()
            .map_err(|error| super::service::failure(error, Some(&artifact), Some(&capture)))?;
        if page.provenance.sha256 != capture.body_sha256
            || page.provenance.bytes as u64 != capture.body_bytes.0
            || page.provenance.final_url != capture.final_url
            || page.provenance.requested_url != capture.requested_url
        {
            return Err(invalid());
        }
        Ok(CatalogResult {
            title: page.title,
            series: page.series,
            entries: page.entries,
            next_page: page.next_page,
            raw_artifact: artifact,
            capture,
            site_verified: false,
            result_artifact: None,
        })
    }
}
fn validate(location: &PageLocation, bytes: &[u8], p: &Provenance) -> Result<(), ApiError> {
    if bytes.len() > crate::domain::browser_provider::MAX_CAPTURE_BYTES
        || p.bytes != bytes.len()
        || p.sha256 != format!("{:x}", Sha256::digest(bytes))
        || p.requested_url != location.as_str()
        || p.redirects.len() > 5
        || p.parser_revision.len() > 128
    {
        return Err(invalid());
    }
    for url in std::iter::once(&p.final_url).chain(p.redirects.iter()) {
        if url.len() > 8192
            || PageLocation::parse(url).map_err(|_| invalid())?.kind() != location.kind()
        {
            return Err(invalid());
        }
    }
    Ok(())
}
fn metadata(
    p: &Provenance,
    capture_kind: CatalogCaptureKind,
    status: Option<u16>,
    content_type: String,
    content_encoding: Option<String>,
    browser: Option<BrowserCaptureProvenance>,
) -> Result<CaptureMetadata, ApiError> {
    let observed_at =
        time::OffsetDateTime::from_unix_timestamp_nanos(i128::from(p.observed_unix_ms) * 1_000_000)
            .map_err(|_| invalid())?
            .format(&time::format_description::well_known::Rfc3339)
            .map_err(|_| invalid())?;
    Ok(CaptureMetadata {
        requested_url: p.requested_url.clone(),
        final_url: p.final_url.clone(),
        redirects: p.redirects.clone(),
        observed_at,
        parser_revision: p.parser_revision.clone(),
        capture_kind,
        status,
        content_type,
        content_encoding,
        body_sha256: p.sha256.clone(),
        body_bytes: DecimalU64(p.bytes as u64),
        browser,
    })
}
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::EngineFailed,
        "Catalog adapter returned invalid capture evidence",
    )
}

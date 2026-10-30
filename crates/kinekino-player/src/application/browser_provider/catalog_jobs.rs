use super::*;
use crate::application::api::ErrorCode;
use crate::application::{
    catalog::{
        self, BrowserCaptureProvenance, CatalogCaptureService, CatalogResults, DomCatalogDocument,
        StoredCatalogResult,
    },
    jobs::{ExecutionControl, JobExecutor, SourceBlobStore, VerifiedJobOutput},
};
use kinekino_jobs::domain::PreparedJobInput;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::time::{Duration, Instant};

pub struct BrowserCatalogJobs {
    browser: Arc<dyn BrowserProviderPort>,
    owner: Arc<dyn BrowserProviderOwner>,
    capture: Arc<CatalogCaptureService>,
    results: Arc<dyn CatalogResults>,
}
impl BrowserCatalogJobs {
    pub fn new(
        browser: Arc<dyn BrowserProviderPort>,
        owner: Arc<dyn BrowserProviderOwner>,
        capture: Arc<CatalogCaptureService>,
        results: Arc<dyn CatalogResults>,
    ) -> Self {
        Self {
            browser,
            owner,
            capture,
            results,
        }
    }
    fn wait_ready(&self, control: &dyn ExecutionControl) -> Result<(), ApiError> {
        loop {
            check(control)?;
            let host = match self.owner.shutdown_snapshot() {
                Ok(host) => host,
                Err(error) if error.code == ErrorCode::ResourceExhausted => {
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                }
                Err(error) => return Err(error),
            };
            match host.state {
                BrowserHostState::Ready => return Ok(()),
                BrowserHostState::Starting => std::thread::sleep(Duration::from_millis(10)),
                _ => {
                    return Err(ApiError::new(
                        ErrorCode::EngineFailed,
                        "Browser host is unavailable",
                    ));
                }
            }
        }
    }
    fn receive(
        &self,
        ticket: &BrowserTicket,
        control: &dyn ExecutionControl,
    ) -> Result<BrowserSnapshot, ApiError> {
        let mut stop = None;
        loop {
            if stop.is_none() {
                stop = if control.cancellation_requested() {
                    Some(ErrorCode::Cancelled)
                } else if Instant::now() >= control.deadline() {
                    Some(ErrorCode::Timeout)
                } else {
                    None
                };
            }
            if stop.is_some() {
                match self.browser.cancel(ticket) {
                    Ok(_) => {}
                    Err(error) if error.code == ErrorCode::ResourceExhausted => {}
                    Err(error) => return Err(error),
                }
            }
            let snapshot = match self.browser.poll(ticket) {
                Ok(snapshot) => snapshot,
                Err(error) if error.code == ErrorCode::ResourceExhausted => {
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                }
                Err(error) => return Err(error),
            };
            if snapshot.state.terminal() {
                if let Some(reason) = stop {
                    return Err(with_snapshot(
                        ApiError::new(reason, "Browser work stopped before result publication"),
                        &snapshot,
                    ));
                }
                return match snapshot.state {
                    BrowserWorkState::Succeeded => Ok(snapshot),
                    BrowserWorkState::Cancelled => Err(with_snapshot(
                        ApiError::new(
                            ErrorCode::Cancelled,
                            "Browser worker acknowledged cancellation",
                        ),
                        &snapshot,
                    )),
                    _ => Err(with_snapshot(
                        snapshot
                            .failure
                            .as_deref()
                            .map(copy_error)
                            .unwrap_or_else(|| {
                                ApiError::new(
                                    ErrorCode::EngineFailed,
                                    "Browser work failed without a diagnostic",
                                )
                            }),
                        &snapshot,
                    )),
                };
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

fn with_snapshot(mut error: ApiError, snapshot: &BrowserSnapshot) -> ApiError {
    // Keep failure receipts bounded: no DOM/media bytes or unbounded navigation history.
    let current_url = snapshot
        .current_url
        .as_deref()
        .filter(|url| url.len() <= 1024);
    error.details = serde_json::json!({"browser_snapshot": {
        "state": format!("{:?}", snapshot.state),
        "engine_version": snapshot.engine_version.as_deref().filter(|version| version.len() <= 128),
        "current_url": current_url,
        "current_url_omitted": snapshot.current_url.is_some() && current_url.is_none(),
        "document_generation": snapshot.document_generation,
        "navigation_pending": snapshot.navigation_pending,
        "navigation_count": snapshot.navigation_history.len(),
        "blocking_requested": snapshot.blocking_requested,
        "blocking_installed": snapshot.blocking_installed,
        "requested_count": snapshot.requested_count,
        "blocked_count": snapshot.blocked_count,
        "document_observed": snapshot.document.is_some(),
        "result_observed": snapshot.result.is_some(),
        "worker_failure_code": snapshot.failure.as_deref().map(|failure| failure.code),
    }});
    error
}

impl JobExecutor for BrowserCatalogJobs {
    fn operations(&self) -> Vec<String> {
        vec!["catalog.search".into(), "episodes.list".into()]
    }
    fn supports(&self, operation: &str) -> bool {
        matches!(operation, "catalog.search" | "episodes.list")
    }
    fn execution_timeout_limit_ms(&self, operation: &str) -> Option<u64> {
        self.supports(operation).then_some(120_000)
    }
    fn normalize_parameters(&self, operation: &str, parameters: Value) -> Result<Value, ApiError> {
        catalog::normalized_request(operation, parameters).map(|(value, _)| value)
    }
    fn prepare(
        &self,
        operation: &str,
        parameters: Value,
        expected: Option<&str>,
        blobs: &dyn SourceBlobStore,
    ) -> Result<PreparedJobInput, ApiError> {
        if expected.is_some() {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Browser jobs do not accept expected_source_sha256; the input blob is a request descriptor, not frozen web content",
            ));
        }
        let (parameters, _) = catalog::normalized_request(operation, parameters)?;
        let descriptor = catalog::descriptor(operation, &parameters)?;
        Ok(PreparedJobInput {
            operation: operation.into(),
            parameters,
            source: blobs.put_utf8(&descriptor)?,
            execution_timeout_ms: DecimalU64(30_000),
        })
    }
    fn execute(
        &self,
        input: &PreparedJobInput,
        blobs: &dyn SourceBlobStore,
        control: &dyn ExecutionControl,
    ) -> Result<VerifiedJobOutput, ApiError> {
        let (parameters, location) =
            catalog::normalized_request(&input.operation, input.parameters.clone())?;
        let descriptor = catalog::descriptor(&input.operation, &parameters)?;
        if blobs.read_utf8(&input.source)? != descriptor
            || input.source.sha256 != catalog::descriptor_sha256(&input.operation, &parameters)?
        {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Catalog request descriptor identity mismatch",
            ));
        }
        self.wait_ready(control)?;
        let timeout = control
            .deadline()
            .saturating_duration_since(Instant::now())
            .as_millis()
            .clamp(1, 120_000) as u64;
        let request = ValidatedBrowserRequest::new(BrowserRequest {
            request_id: format!("catalog-{}", &input.source.sha256[..16]),
            operation: BrowserOperation::ReadCatalog,
            url: location.as_str().into(),
            timeout_ms: DecimalU64(timeout),
            blocking_enabled: true,
        })
        .map_err(|message| ApiError::new(ErrorCode::InvalidArgument, message))?;
        let ticket = loop {
            check(control)?;
            match self.browser.try_start(request.clone()) {
                Ok(ticket) => break ticket,
                Err(error) if error.code == ErrorCode::ResourceExhausted => {
                    std::thread::sleep(Duration::from_millis(10))
                }
                Err(error) => return Err(error),
            }
        };
        let _lease = TicketLease {
            browser: self.browser.clone(),
            ticket: ticket.clone(),
        };
        let snapshot = self.receive(&ticket, control)?;
        check(control)?;
        let document = match snapshot.result.as_deref() {
            Some(BrowserResult::Document(document))
                if document.kind == CaptureKind::DomSnapshot =>
            {
                document
            }
            _ => {
                return Err(ApiError::new(
                    ErrorCode::Unsupported,
                    "Catalog browser jobs currently interpret DOM snapshots only",
                ));
            }
        };
        let browser = BrowserCaptureProvenance {
            network: None,
            frame: None,
            navigation_diagnostic: snapshot.navigation_diagnostic.clone(),
            ticket,
            document_generation: document.document_generation,
            navigation_history: snapshot.navigation_history,
            engine_version: snapshot.engine_version.ok_or_else(|| {
                ApiError::new(
                    ErrorCode::EngineFailed,
                    "Browser engine identity was not observed",
                )
            })?,
            blocking_requested: snapshot.blocking_requested,
            blocking_installed: snapshot.blocking_installed,
            requested_count: snapshot.requested_count,
            blocked_count: snapshot.blocked_count,
        };
        let provenance = kinekino_core::source::document::Provenance {
            requested_url: location.as_str().into(),
            final_url: document.final_url.clone(),
            redirects: vec![],
            sha256: format!("{:x}", Sha256::digest(&document.body)),
            bytes: document.body.len(),
            observed_unix_ms: document.observed_unix_ms.0,
            parser_revision: self.capture.parser_revision().into(),
        };
        let result = self.capture.dom(DomCatalogDocument {
            location,
            body: document.body.clone(),
            provenance,
            browser,
        })?;
        let stored = StoredCatalogResult {
            schema_version: "1".into(),
            operation: input.operation.clone(),
            parameters,
            source_descriptor_sha256: input.source.sha256.clone(),
            result,
        };
        control.reserve_publication()?;
        control.begin_verification()?;
        self.results.publish(&stored)
    }
}
struct TicketLease {
    browser: Arc<dyn BrowserProviderPort>,
    ticket: BrowserTicket,
}
impl Drop for TicketLease {
    fn drop(&mut self) {
        for _ in 0..100 {
            match self.browser.poll(&self.ticket) {
                Ok(snapshot) if snapshot.state.terminal() => {
                    match self.browser.release(&self.ticket) {
                        Ok(()) => return,
                        Err(error) if error.code == ErrorCode::ResourceExhausted => {}
                        Err(_) => return,
                    }
                }
                Ok(_) => {
                    let _ = self.browser.cancel(&self.ticket);
                }
                Err(error) if error.code == ErrorCode::ResourceExhausted => {}
                Err(_) => return,
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
fn check(control: &dyn ExecutionControl) -> Result<(), ApiError> {
    if control.cancellation_requested() {
        return Err(ApiError::new(
            ErrorCode::Cancelled,
            "Catalog job cancelled before publication",
        ));
    }
    if Instant::now() >= control.deadline() {
        return Err(ApiError::new(
            ErrorCode::Timeout,
            "Catalog job deadline expired",
        ));
    }
    Ok(())
}
fn copy_error(error: &ApiError) -> ApiError {
    ApiError {
        code: error.code,
        message: error.message.clone(),
        retryable: error.retryable,
        details: error.details.clone(),
        remediation: error.remediation.clone(),
        request_id: error.request_id.clone(),
    }
}

#[cfg(test)]
mod diagnostics_tests {
    use super::*;

    #[test]
    fn stopped_work_diagnostics_keep_observations_and_omit_large_urls() {
        let request = ValidatedBrowserRequest::new(BrowserRequest {
            request_id: "diagnostic".into(),
            operation: BrowserOperation::ReadCatalog,
            url: "https://catalog.example.test/방영중".into(),
            timeout_ms: DecimalU64(30_000),
            blocking_enabled: true,
        })
        .unwrap();
        let ticket =
            BrowserTicket::new("diagnostic-session".into(), "diagnostic".into(), 1).unwrap();
        let mut snapshot = BrowserWork::new(ticket, request).unwrap().snapshot();
        snapshot.current_url = Some(format!(
            "https://catalog.example.test/c/{}",
            "x".repeat(8000)
        ));
        snapshot.engine_version = Some("test-engine".into());
        snapshot.requested_count = DecimalU64(71);
        let error = with_snapshot(ApiError::new(ErrorCode::Timeout, "deadline"), &snapshot);
        assert_eq!(error.details["browser_snapshot"]["requested_count"], "71");
        assert_eq!(
            error.details["browser_snapshot"]["current_url_omitted"],
            true
        );
        assert!(error.details["browser_snapshot"]["current_url"].is_null());
        assert!(serde_json::to_vec(&error.details).unwrap().len() < 2048);
    }
}

use kinekino_core::source::document::{
    CatalogError, CatalogPage, Entry as PageEntry, ErrorKind, PageLocation, Provenance, RawDocument,
};
use kinekino_player::application::catalog::CatalogParser;
use kinekino_player::application::{
    api::{ApiError, ErrorCode},
    browser_provider::*,
};
use kinekino_player::domain::api_values::DecimalU64;
use std::sync::{Arc, Mutex};

struct Entry {
    work: BrowserWork,
    queued_polls: usize,
}
struct State {
    entries: Vec<Entry>,
    generation: u64,
    host: BrowserHostState,
}
impl Default for State {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            generation: 0,
            host: BrowserHostState::Ready,
        }
    }
}
pub(super) struct SyntheticBrowser {
    state: Mutex<State>,
    queued_polls: usize,
    failure_after_running: Option<ErrorCode>,
}
/// Installs the shared site-neutral fixture layout (`catalog.example.test`, `/c/…` series,
/// `/e/…` episodes); first install wins, so repeats are no-ops.
pub(super) fn install_fixture_rules() {
    kinekino_source_host::plugin_host::install_fixture_rules();
}
/// Site-neutral parser for the synthetic pages above: `<h1>` is the title and every
/// `<a href='…'>…</a>` is one entry resolved against the page location.
pub(super) struct FixtureParser;
impl CatalogParser for FixtureParser {
    fn revision(&self) -> &str {
        "fixture-parser/1"
    }
    fn parse(&self, _: &RawDocument) -> Result<CatalogPage, CatalogError> {
        Err(CatalogError::new(
            ErrorKind::UnsupportedContent,
            "Fixture parser reads DOM snapshots only",
        ))
    }
    fn parse_html(
        &self,
        location: &PageLocation,
        html: &str,
        provenance: Provenance,
    ) -> Result<CatalogPage, CatalogError> {
        let between = |s: &str, open: &str, close: &str| -> Option<(String, usize)> {
            let start = s.find(open)? + open.len();
            let len = s[start..].find(close)?;
            Some((s[start..start + len].to_owned(), start + len + close.len()))
        };
        let title = between(html, "<h1>", "</h1>")
            .map(|(t, _)| t)
            .unwrap_or_default();
        let mut entries = Vec::new();
        let mut rest = html;
        while let Some((href, after)) = between(rest, "<a href='", "'>") {
            let (label, end) = between(&rest[after..], "", "</a>")
                .ok_or_else(|| CatalogError::new(ErrorKind::UnsupportedContent, "open anchor"))?;
            let target = location.resolve(&href)?;
            entries.push(PageEntry {
                id: target.stable_id(),
                title: label,
                url: target.as_str().to_owned(),
            });
            rest = &rest[after + end..];
        }
        Ok(CatalogPage {
            title,
            series: None,
            entries,
            next_page: None,
            provenance,
        })
    }
}
impl Default for SyntheticBrowser {
    fn default() -> Self {
        Self::with_behavior(0, None)
    }
}
impl SyntheticBrowser {
    pub fn with_behavior(queued_polls: usize, failure_after_running: Option<ErrorCode>) -> Self {
        install_fixture_rules();
        Self {
            state: Mutex::new(State::default()),
            queued_polls,
            failure_after_running,
        }
    }
}
fn missing() -> ApiError {
    ApiError::new(ErrorCode::NotFound, "Synthetic browser ticket not found")
}
impl BrowserProviderPort for SyntheticBrowser {
    fn try_start(&self, request: ValidatedBrowserRequest) -> Result<BrowserTicket, ApiError> {
        if request.request().operation != BrowserOperation::ReadCatalog {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Synthetic fixture covers catalog capture only",
            ));
        }
        let mut state = self.state.lock().unwrap();
        if state.host != BrowserHostState::Ready {
            return Err(ApiError::new(
                ErrorCode::NeedsInput,
                "Synthetic browser host is not ready",
            ));
        }
        if state.entries.len() >= MAX_RETAINED_TICKETS
            || state
                .entries
                .iter()
                .any(|entry| !entry.work.snapshot().state.terminal())
        {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Synthetic browser capacity is occupied",
            ));
        }
        let generation = state.generation.checked_add(1).ok_or_else(|| {
            ApiError::new(
                ErrorCode::ResourceExhausted,
                "Synthetic generation exhausted",
            )
        })?;
        let ticket = BrowserTicket::new(
            "synthetic-browser".into(),
            request.request().request_id.clone(),
            generation,
        )
        .unwrap();
        let work = BrowserWork::new(ticket.clone(), request)?;
        state.entries.push(Entry {
            work,
            queued_polls: self.queued_polls,
        });
        state.generation = generation;
        Ok(ticket)
    }
    fn poll(&self, ticket: &BrowserTicket) -> Result<BrowserSnapshot, ApiError> {
        let mut state = self.state.lock().unwrap();
        let entry = state
            .entries
            .iter_mut()
            .find(|entry| entry.work.snapshot().ticket == *ticket)
            .ok_or_else(missing)?;
        let snapshot = entry.work.snapshot();
        match snapshot.state {
            BrowserWorkState::Queued if entry.queued_polls > 0 => {
                entry.queued_polls -= 1;
            }
            BrowserWorkState::Queued => {
                entry.work.apply(
                    ticket,
                    BrowserEvent::Started {
                        engine_version: "synthetic-no-engine".into(),
                        blocking_installed: true,
                    },
                );
                entry.work.apply(
                    ticket,
                    BrowserEvent::Navigated {
                        url: entry.work.request().request().url.clone(),
                        document_generation: 1,
                    },
                );
            }
            BrowserWorkState::Running => {
                if let Some(code) = self.failure_after_running {
                    entry.work.apply(
                        ticket,
                        BrowserEvent::Failed(ApiError::new(
                            code,
                            "Explicit synthetic engine failure",
                        )),
                    );
                } else {
                    let url = snapshot.current_url.unwrap();
                    let html = if url.contains("/c/") {
                        "<h1>Fixture</h1><div class='wr-subject'><a href='/e/two'>2</a><a href='/e/one'>1</a></div>"
                    } else {
                        "<h1>Fixture</h1><div class='wr-subject'><a href='/c/one'>日本語 한글</a></div>"
                    };
                    entry.work.apply(
                        ticket,
                        BrowserEvent::DocumentReady(BrowserDocument {
                            kind: CaptureKind::DomSnapshot,
                            final_url: url,
                            document_generation: DecimalU64(1),
                            observed_unix_ms: DecimalU64(1000),
                            body: Arc::from(html.as_bytes()),
                            response: None,
                        }),
                    );
                }
            }
            BrowserWorkState::Cancelling => {
                entry.work.apply(ticket, BrowserEvent::Cancelled);
            }
            _ => {}
        }
        Ok(entry.work.snapshot())
    }
    fn cancel(&self, ticket: &BrowserTicket) -> Result<CancelDisposition, ApiError> {
        let mut state = self.state.lock().unwrap();
        let entry = state
            .entries
            .iter_mut()
            .find(|entry| entry.work.snapshot().ticket == *ticket)
            .ok_or_else(missing)?;
        Ok(entry.work.cancel())
    }
    fn navigate(&self, ticket: &BrowserTicket, generation: u64, url: &str) -> Result<(), ApiError> {
        let mut state = self.state.lock().unwrap();
        let entry = state
            .entries
            .iter_mut()
            .find(|entry| entry.work.snapshot().ticket == *ticket)
            .ok_or_else(missing)?;
        entry.work.reserve_navigation(generation, url)
    }
    fn release(&self, ticket: &BrowserTicket) -> Result<(), ApiError> {
        let mut state = self.state.lock().unwrap();
        let entry = state
            .entries
            .iter()
            .find(|entry| entry.work.snapshot().ticket == *ticket)
            .ok_or_else(missing)?;
        if !entry.work.snapshot().state.terminal() {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Active synthetic work cannot be released",
            ));
        }
        state
            .entries
            .retain(|entry| entry.work.snapshot().ticket != *ticket);
        Ok(())
    }
}
impl BrowserProviderOwner for SyntheticBrowser {
    fn request_shutdown(&self) -> Result<(), ApiError> {
        let mut state = self.state.lock().unwrap();
        for entry in &mut state.entries {
            let ticket = entry.work.snapshot().ticket;
            entry.work.cancel();
            entry.work.apply(&ticket, BrowserEvent::Cancelled);
        }
        state.host = BrowserHostState::Stopped;
        Ok(())
    }
    fn shutdown_snapshot(&self) -> Result<BrowserHostSnapshot, ApiError> {
        let state = self.state.lock().unwrap();
        Ok(BrowserHostSnapshot {
            state: state.host,
            worker_exited: state.host == BrowserHostState::Stopped,
            failure: None,
        })
    }
}

fn request() -> ValidatedBrowserRequest {
    ValidatedBrowserRequest::new(BrowserRequest {
        request_id: "fixture-request".into(),
        operation: BrowserOperation::ReadCatalog,
        url: "https://catalog.example.test/방영중".into(),
        timeout_ms: DecimalU64(1000),
        blocking_enabled: true,
    })
    .unwrap()
}
#[test]
fn synthetic_ticket_isolation_cancel_ack_and_release_recovery_follow_contract() {
    kinekino_platform::install();
    let browser = SyntheticBrowser::with_behavior(2, None);
    let first = browser.try_start(request()).unwrap();
    assert_eq!(
        browser.try_start(request()).unwrap_err().code,
        ErrorCode::ResourceExhausted
    );
    let foreign = BrowserTicket::new("other-session".into(), "fixture-request".into(), 1).unwrap();
    assert!(matches!(
        browser.poll(&foreign),
        Err(ApiError {
            code: ErrorCode::NotFound,
            ..
        })
    ));
    assert_eq!(
        browser.release(&first).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        browser.cancel(&first).unwrap(),
        CancelDisposition::Requested
    );
    assert_eq!(
        browser.state.lock().unwrap().entries[0]
            .work
            .snapshot()
            .state,
        BrowserWorkState::Cancelling
    );
    assert_eq!(
        browser.poll(&first).unwrap().state,
        BrowserWorkState::Cancelled
    );
    browser.release(&first).unwrap();
    assert!(matches!(
        browser.poll(&first),
        Err(ApiError {
            code: ErrorCode::NotFound,
            ..
        })
    ));
    let next = browser.try_start(request()).unwrap();
    assert_eq!(next.request_generation(), 2);
    browser.request_shutdown().unwrap();
    assert_eq!(
        browser.shutdown_snapshot().unwrap().state,
        BrowserHostState::Stopped
    );
    assert_eq!(
        browser.try_start(request()).unwrap_err().code,
        ErrorCode::NeedsInput
    );
}
#[test]
fn synthetic_latency_and_engine_failure_never_publish_success() {
    kinekino_platform::install();
    let browser = SyntheticBrowser::with_behavior(2, Some(ErrorCode::EngineFailed));
    let ticket = browser.try_start(request()).unwrap();
    assert_eq!(
        browser.poll(&ticket).unwrap().state,
        BrowserWorkState::Queued
    );
    assert_eq!(
        browser.poll(&ticket).unwrap().state,
        BrowserWorkState::Queued
    );
    assert_eq!(
        browser.poll(&ticket).unwrap().state,
        BrowserWorkState::Running
    );
    let failed = browser.poll(&ticket).unwrap();
    assert_eq!(failed.state, BrowserWorkState::Failed);
    assert_eq!(failed.failure.unwrap().code, ErrorCode::EngineFailed);
    assert!(failed.result.is_none());
}
#[test]
fn fixture_parser_reads_site_neutral_entries() {
    install_fixture_rules();
    let location = PageLocation::parse("https://catalog.example.test/c/one").unwrap();
    let provenance = Provenance {
        requested_url: location.as_str().into(),
        final_url: location.as_str().into(),
        redirects: Vec::new(),
        sha256: String::new(),
        bytes: 0,
        observed_unix_ms: 1000,
        parser_revision: FixtureParser.revision().into(),
    };
    let page = FixtureParser
        .parse_html(
            &location,
            "<h1>Fixture</h1><a href='/e/two'>2</a><a href='/e/one'>1</a>",
            provenance,
        )
        .unwrap();
    assert_eq!(page.title, "Fixture");
    let urls: Vec<_> = page.entries.iter().map(|e| e.url.as_str()).collect();
    assert_eq!(
        urls,
        [
            "https://catalog.example.test/e/two",
            "https://catalog.example.test/e/one"
        ]
    );
    assert_eq!(page.entries[1].id, "catalog:/e/one");
    assert!(location.resolve("https://other.example.test/c/x").is_err());
}

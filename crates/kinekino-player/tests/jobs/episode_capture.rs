use super::fixture::{Fixture, request};
use super::observation_guard::{OwnedObservation, browser_call};
use kinekino_core::source::document::{PageLocation, ResourceKind};
use kinekino_player::{
    application::{
        browser_provider::{BrowserDocument, CaptureKind},
        browser_provider::{BrowserHostState, BrowserWorkState, CancelDisposition},
        catalog::{
            BrowserCaptureProvenance, CaptureMetadata, CatalogArtifacts, CatalogCaptureKind,
            CatalogRawSource, CatalogResults,
        },
        episode_resolve::{ResolveEpisode, player_target},
    },
    domain::{
        api_values::DecimalU64,
        browser_provider::{BrowserOperation, BrowserRequest, ValidatedBrowserRequest},
    },
    infrastructure::{
        browser_provider, catalog::CatalogFiles, content_digest,
        episode_document::HtmlEpisodeMarkup,
    },
};
use serde_json::json;
use std::{
    fs,
    path::PathBuf,
    time::{Duration, Instant},
};

#[test]
#[ignore = "Explicit SDK episode DOM observation before CLI resolver implementation; allocated browser run"]
fn actual_episode_dom_is_preserved_before_resolver_implementation() {
    kinekino_platform::install();
    observe_document(false);
}

#[test]
#[ignore = "Explicit SDK player-document navigation from verified episode DOM; not CLI resolve or media playback"]
fn actual_player_document_is_selected_by_rust_from_preserved_episode() {
    kinekino_platform::install();
    observe_document(true);
}

fn observe_document(player_document: bool) {
    let source =
        PathBuf::from(std::env::var_os("EPISODE_SEED_ROOT").expect("Explicit episode seed root"));
    let seed_sha =
        std::env::var("EPISODE_SEED_RESULT_SHA256").expect("Explicit episodes result SHA");
    let source_files = CatalogFiles::open(&source).unwrap();
    let (seed, _) = source_files.read(&seed_sha).unwrap();
    assert_eq!(seed.operation, "episodes.list");
    let episode = seed
        .result
        .entries
        .first()
        .expect("Nonempty verified episodes");
    let location = PageLocation::parse(&episode.url).unwrap();
    assert_eq!(location.kind(), ResourceKind::Episode);
    let (requested_url, operation, parent_artifact) = if player_document {
        let root = PathBuf::from(
            std::env::var_os("EPISODE_DOCUMENT_ROOT")
                .expect("Explicit preserved episode document root"),
        );
        let artifact_id = std::env::var("EPISODE_DOCUMENT_ARTIFACT")
            .expect("Explicit episode document artifact ID");
        let raw = CatalogFiles::open(&root)
            .unwrap()
            .load_raw(&artifact_id)
            .unwrap();
        assert!(matches!(
            raw.capture.capture_kind,
            CatalogCaptureKind::DomSnapshot
        ));
        assert_eq!(raw.capture.observed_at, "2026-10-19T04:28:10.108Z");
        let observed = time::Date::from_calendar_date(2026, time::Month::October, 19)
            .unwrap()
            .with_hms_milli(4, 28, 10, 108)
            .unwrap()
            .assume_utc();
        let input = ResolveEpisode {
            episode_url: location.as_str().into(),
        };
        let document = BrowserDocument {
            final_url: raw.capture.final_url,
            document_generation: raw.capture.browser.unwrap().document_generation,
            observed_unix_ms: DecimalU64(
                u64::try_from(observed.unix_timestamp_nanos() / 1_000_000).unwrap(),
            ),
            kind: CaptureKind::DomSnapshot,
            body: raw.body.into(),
            response: None,
        };
        let target = player_target(&input, &document, &HtmlEpisodeMarkup).unwrap();
        assert_eq!(target.source_body_sha256, raw.reference.body_sha256);
        (
            target.document.as_str().to_owned(),
            BrowserOperation::ReadPlayerDocument,
            Some(artifact_id),
        )
    } else {
        (
            location.as_str().to_owned(),
            BrowserOperation::ResolveEpisode,
            None,
        )
    };
    let fixture = Fixture::named(if player_document {
        "sdk-player-document"
    } else {
        "sdk-episode-dom"
    });
    fs::write(fixture.root.join("seed.json"), serde_json::to_vec_pretty(&json!({"seed_result_sha256":seed_sha,"episode":episode,"parent_episode_artifact":parent_artifact,"fresh_profile":true,"proof_scope":"SDK DOM observation and CLI archive readback; no CLI resolve or playback claim"})).unwrap()).unwrap();
    let profile = fixture.root.join("profile");
    let catalog = fixture.root.join("catalog");
    fs::create_dir(&profile).unwrap();
    fs::create_dir(&catalog).unwrap();
    let files = CatalogFiles::open(&catalog).unwrap();
    let browser = browser_provider::start(&profile).unwrap();
    let owner = browser.owner();
    let port = browser.port();
    let mut cleanup = OwnedObservation::new(
        owner.clone(),
        port.clone(),
        fixture.root.join("panic-cleanup.json"),
    );
    let ready_deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let state = browser_call(ready_deadline, || owner.shutdown_snapshot());
        if state.state != BrowserHostState::Starting {
            assert_eq!(state.state, BrowserHostState::Ready);
            break;
        }
        assert!(Instant::now() < ready_deadline);
        std::thread::sleep(Duration::from_millis(20));
    }
    let browser_request = ValidatedBrowserRequest::new(BrowserRequest {
        request_id: "actual-episode-dom".into(),
        operation,
        url: requested_url.clone(),
        timeout_ms: DecimalU64(30_000),
        blocking_enabled: true,
    })
    .unwrap();
    let ticket = browser_call(ready_deadline, || port.try_start(browser_request.clone()));
    cleanup.ticket = Some(ticket.clone());
    let deadline = Instant::now() + Duration::from_secs(35);
    let snapshot = loop {
        let snapshot = browser_call(deadline, || port.poll(&ticket));
        if snapshot.document.is_some() || snapshot.state.terminal() || Instant::now() >= deadline {
            break snapshot;
        }
        std::thread::sleep(Duration::from_millis(30));
    };
    let captured = snapshot.document.as_ref().map(|document| {
        let (_, sha256) = content_digest::sha256(&mut document.body.as_ref(), 2 * 1024 * 1024).unwrap();
        let metadata = CaptureMetadata {
            requested_url: requested_url.clone(), final_url: document.final_url.clone(), redirects: vec![],
            observed_at: time::OffsetDateTime::from_unix_timestamp_nanos(i128::from(document.observed_unix_ms.0) * 1_000_000).unwrap().format(&time::format_description::well_known::Rfc3339).unwrap(),
            parser_revision: "browser-dom-uninterpreted-v1".into(), capture_kind: CatalogCaptureKind::DomSnapshot,
            status: None, content_type: "text/html; charset=utf-8".into(), content_encoding: None,
            body_sha256: sha256, body_bytes: DecimalU64(document.body.len() as u64),
            browser: Some(BrowserCaptureProvenance {
                network: None,
                frame: None,
                navigation_diagnostic: snapshot.navigation_diagnostic.clone(),
                ticket: ticket.clone(), document_generation: document.document_generation,
                navigation_history: snapshot.navigation_history.clone(), engine_version: snapshot.engine_version.clone().unwrap(),
                blocking_requested: snapshot.blocking_requested, blocking_installed: snapshot.blocking_installed,
                requested_count: snapshot.requested_count, blocked_count: snapshot.blocked_count,
            }),
        };
        let artifact = files.save(&document.body, &metadata).unwrap();
        fs::write(fixture.root.join("capture.json"), serde_json::to_vec_pretty(&json!({"raw_artifact":artifact,"capture":metadata,"provider_state":format!("{:?}",snapshot.state),"media_resolved":false})).unwrap()).unwrap();
        (artifact, document.body.clone())
    });
    fs::write(fixture.root.join("observation.json"), serde_json::to_vec_pretty(&json!({"state":format!("{:?}",snapshot.state),"document_observed":snapshot.document.is_some(),"result_observed":snapshot.result.is_some(),"current_url":snapshot.current_url,"document_generation":snapshot.document_generation,"failure":snapshot.failure.as_deref()})).unwrap()).unwrap();
    let cancel_deadline = Instant::now() + Duration::from_secs(3);
    let cancellation = browser_call(cancel_deadline, || port.cancel(&ticket));
    let cancelled = loop {
        let state = browser_call(cancel_deadline, || port.poll(&ticket)).state;
        if state.terminal() || Instant::now() >= cancel_deadline {
            break state;
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    let stop_deadline = Instant::now() + Duration::from_secs(10);
    browser_call(stop_deadline, || owner.request_shutdown());
    let stopped = loop {
        let stopped = browser_call(stop_deadline, || owner.shutdown_snapshot());
        if stopped.worker_exited || Instant::now() >= stop_deadline {
            break stopped;
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    fs::write(fixture.root.join("stop.json"), serde_json::to_vec_pretty(&json!({"cancel_disposition":format!("{cancellation:?}"),"cancel_state":format!("{cancelled:?}"),"host_state":format!("{:?}",stopped.state),"worker_exited":stopped.worker_exited})).unwrap()).unwrap();
    assert!(stopped.worker_exited);
    cleanup.finished = true;
    let (artifact, body) = captured.expect("Actual episode DOM was not observed");
    match cancellation {
        CancelDisposition::Requested | CancelDisposition::AlreadyRequested => {
            assert_eq!(cancelled, BrowserWorkState::Cancelled)
        }
        CancelDisposition::AlreadyTerminal => assert!(cancelled.terminal()),
    }
    let mut restored = Vec::new();
    let mut offset = 0u64;
    for index in 0..128 {
        let chunk = fixture.call_catalog(&format!("raw-{index}"), &request("catalog.read_raw", json!({"artifact_id":artifact.artifact_id,"offset":offset.to_string(),"limit":65536})), &catalog, 0);
        restored.extend_from_slice(chunk["data"]["text"].as_str().unwrap().as_bytes());
        let Some(next) = chunk["data"]["next_offset"].as_str() else {
            break;
        };
        let next = next.parse::<u64>().unwrap();
        assert!(next > offset);
        offset = next;
    }
    assert_eq!(restored, body.as_ref());
    source_files.read(&seed_sha).unwrap();
}

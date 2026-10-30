use super::fixture::{Fixture, request};
use kinekino_core::source::document::{PageLocation, Provenance};
use kinekino_player::infrastructure::catalog::document_plugin;
use kinekino_player::{
    application::catalog::{CatalogRawSource, CatalogResults},
    infrastructure::catalog::CatalogFiles,
};
use serde_json::json;
use std::{fs, path::PathBuf};

#[test]
#[ignore = "Explicit preserved episodes result read; no browser, network or playback"]
fn saved_episode_result_is_reopened_by_fresh_cli_without_network() {
    kinekino_platform::install();
    let fixture = Fixture::named("episode-result-replay");
    let root = PathBuf::from(
        std::env::var_os("CATALOG_EPISODE_SOURCE_ROOT")
            .expect("Explicit copied episode archive root"),
    );
    let result_sha = "b1a2af4afa4491ef5743f2a1095d7b9973b6bc4268f1edf70dd9482446bea3bc";
    let files = CatalogFiles::open(&root).unwrap();
    let (stored, _) = files.read(result_sha).unwrap();
    assert_eq!(stored.operation, "episodes.list");
    assert_eq!(stored.result.entries.len(), 4);
    let artifact = "cf48a62cb39a8c34f6224370c973f33982a026432eafd080b8fc275be31c3550";
    let original = files.load_raw(artifact).unwrap();
    assert_eq!(original.body.len(), 134681);
    assert_eq!(
        original.reference.body_sha256,
        "f0c73cacbfc4526bffada0012912c6e01429c376a8ef496f89251ed34f01d8c3"
    );
    let response = fixture.call_catalog(
        "fresh-episode-archive",
        &request(
            "episodes.list",
            json!({"result_artifact_sha256":result_sha}),
        ),
        &root,
        0,
    );
    assert_eq!(
        response["data"]["entries"],
        serde_json::to_value(&stored.result.entries).unwrap()
    );
    for entry in &stored.result.entries {
        assert_eq!(
            PageLocation::parse(&entry.url).unwrap().kind(),
            kinekino_core::source::document::ResourceKind::Episode
        );
    }
    assert_eq!(files.load_raw(artifact).unwrap().body, original.body);
    assert_eq!(
        serde_json::to_value(files.read(result_sha).unwrap().0).unwrap(),
        serde_json::to_value(stored).unwrap()
    );
    fs::write(fixture.root.join("scope.json"), serde_json::to_vec_pretty(&json!({"source_result_sha256":result_sha,"entries":4,"new_network_requests":0,"scope":"historical_archive_read_only","current_site_verified":false,"playback_verified":false})).unwrap()).unwrap();
}

#[test]
#[ignore = "Explicit offline interpretation of preserved completed/movie DOMs; no network request"]
fn saved_library_category_doms_are_reparsed_without_network() {
    kinekino_platform::install();
    let fixture = Fixture::named("library-category-replay");
    let source = PathBuf::from(
        std::env::var_os("CATALOG_CATEGORY_CAPTURE_ROOT").expect("Explicit category capture root"),
    );
    let files = CatalogFiles::open(&source).unwrap();
    for (category, artifact_id, expected_bytes, second, milli) in [
        (
            "completed",
            "646d64e22463c02d789ac4109b359321cba91c50a2014f312acef1bdf661088b",
            221453,
            18,
            859,
        ),
        (
            "movies",
            "4c1900b11d5989033d93212da513dbcc90de8ee908e34f5a1322d0018b9fba97",
            205480,
            36,
            701,
        ),
    ] {
        let raw = files.load_raw(artifact_id).unwrap();
        assert_eq!(raw.body.len(), expected_bytes);
        let expected_time = format!("2026-10-19T03:54:{second:02}.{milli:03}Z");
        assert_eq!(raw.capture.observed_at, expected_time);
        let observed = time::Date::from_calendar_date(2026, time::Month::October, 19)
            .unwrap()
            .with_hms_milli(3, 54, second, milli)
            .unwrap()
            .assume_utc();
        let location = PageLocation::parse(&raw.capture.final_url).unwrap();
        let page = document_plugin()
            .unwrap()
            .parse_html(
                &location,
                std::str::from_utf8(&raw.body).unwrap(),
                Provenance {
                    requested_url: raw.capture.requested_url.clone(),
                    final_url: raw.capture.final_url.clone(),
                    redirects: vec![],
                    sha256: raw.reference.body_sha256.clone(),
                    bytes: raw.body.len(),
                    observed_unix_ms: u64::try_from(observed.unix_timestamp_nanos() / 1_000_000)
                        .unwrap(),
                    parser_revision: document_plugin().unwrap().revision().to_owned(),
                },
            )
            .unwrap();
        assert!(!page.entries.is_empty());
        assert_eq!(files.load_raw(artifact_id).unwrap().body, raw.body);
        fs::write(
            fixture.root.join(format!("{category}-interpretation.json")),
            serde_json::to_vec_pretty(&json!({
                "source_capture_id":artifact_id,"source_body_sha256":raw.reference.body_sha256,
                "source_capture_observed_at":raw.capture.observed_at,
                "source_capture_parser_revision":raw.capture.parser_revision,
                "interpretation_parser_revision":document_plugin().unwrap().revision(),
                "entry_count":page.entries.len(),"next_page":page.next_page,"entries":page.entries,
                "new_network_requests":0,
            }))
            .unwrap(),
        )
        .unwrap();
    }
}

#[test]
#[ignore = "Explicit offline interpretation of the preserved actual DOM; no network request"]
fn saved_actual_dom_is_reparsed_without_mutating_original_capture() {
    kinekino_platform::install();
    let fixture = Fixture::named("catalog-replay");
    let source = PathBuf::from(
        std::env::var_os("CATALOG_SOURCE_ROOT").expect("Explicit saved capture root"),
    );
    let artifact_id = "fae1aed877f01dc026148158593a8974231a412e213b7319b3bccc1623f568ca";
    let files = CatalogFiles::open(&source).unwrap();
    let raw = files.load_raw(artifact_id).unwrap();
    assert_eq!(raw.body.len(), 101495);
    assert_eq!(
        raw.reference.body_sha256,
        "56b3430f96481e8d6106ed8a39ec6f2693f1ad3f8c7bc848ef5523c9d670e109"
    );
    let html = std::str::from_utf8(&raw.body).unwrap();
    let location = PageLocation::parse(&raw.capture.final_url).unwrap();
    assert_eq!(raw.capture.observed_at, "2026-10-19T03:27:24.887Z");
    let observed = time::Date::from_calendar_date(2026, time::Month::October, 19)
        .unwrap()
        .with_hms_milli(3, 27, 24, 887)
        .unwrap()
        .assume_utc();
    let original_revision = raw.capture.parser_revision.clone();
    let page = document_plugin()
        .unwrap()
        .parse_html(
            &location,
            html,
            Provenance {
                requested_url: raw.capture.requested_url.clone(),
                final_url: raw.capture.final_url.clone(),
                redirects: vec![],
                sha256: raw.reference.body_sha256.clone(),
                bytes: raw.body.len(),
                observed_unix_ms: u64::try_from(observed.unix_timestamp_nanos() / 1_000_000)
                    .unwrap(),
                parser_revision: document_plugin().unwrap().revision().to_owned(),
            },
        )
        .unwrap();
    assert!(!page.entries.is_empty());
    assert_ne!(original_revision, document_plugin().unwrap().revision());
    fs::write(fixture.root.join("interpretation.json"), serde_json::to_vec_pretty(&json!({"source_capture_id":artifact_id,"source_body_sha256":raw.reference.body_sha256,"source_capture_parser_revision":original_revision,"interpretation_parser_revision":document_plugin().unwrap().revision(),"entry_count":page.entries.len(),"next_page":page.next_page,"entries":page.entries,"new_network_requests":0})).unwrap()).unwrap();
    let mut reconstructed = Vec::new();
    let mut offset = 0u64;
    let mut chunk = 0usize;
    loop {
        let result = fixture.call_catalog(&format!("raw-chunk-{chunk}"), &request("catalog.read_raw", json!({"artifact_id":artifact_id,"offset":offset.to_string(),"limit":65536,"encoding":"utf8"})), &source, 0);
        assert_eq!(
            result["data"]["capture"]["parser_revision"],
            original_revision
        );
        reconstructed.extend_from_slice(result["data"]["text"].as_str().unwrap().as_bytes());
        let Some(next) = result["data"]["next_offset"].as_str() else {
            break;
        };
        let next: u64 = next.parse().unwrap();
        assert!(next > offset);
        offset = next;
        chunk += 1;
        assert!(chunk < 128);
    }
    assert_eq!(reconstructed, raw.body);
    assert_eq!(
        files.load_raw(artifact_id).unwrap().capture.parser_revision,
        original_revision
    );
}

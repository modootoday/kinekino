use super::{
    fixture::{Fixture, Host, request},
    provider::terminal,
};
use kinekino_core::source::document::{PageLocation, ResourceKind};
use kinekino_player::{
    application::{
        catalog::CatalogResults,
        jobs::{JobId, JobState},
    },
    infrastructure::catalog::CatalogFiles,
};
use serde_json::{Value, json};
use std::{fs, path::PathBuf};

#[test]
#[ignore = "Explicit actual episodes.list from a verified immutable category result; allocated provider run"]
fn actual_cli_series_episodes_preserves_dom_and_stops_owned_host() {
    kinekino_platform::install();
    let source =
        PathBuf::from(std::env::var_os("CATALOG_SEED_ROOT").expect("Explicit seed catalog root"));
    let seed_sha =
        std::env::var("CATALOG_SEED_RESULT_SHA256").expect("Explicit seed result SHA256");
    let seed_files = CatalogFiles::open(&source).unwrap();
    let (seed, _) = seed_files.read(&seed_sha).unwrap();
    assert_eq!(seed.operation, "catalog.search");
    let selected = seed
        .result
        .entries
        .first()
        .expect("Nonempty category result");
    let location = PageLocation::parse(&selected.url).unwrap();
    assert_eq!(location.kind(), ResourceKind::Series);
    let fixture = Fixture::named("provider-series");
    fs::write(fixture.root.join("seed.json"), serde_json::to_vec_pretty(&json!({
        "seed_result_sha256":seed_sha,"selected_entry":selected,"selection":"first entry in verified preserved category order",
    })).unwrap()).unwrap();
    let catalog_root = fixture.root.join("catalog");
    let profile = fixture.root.join("profile");
    fs::create_dir(&catalog_root).unwrap();
    fs::create_dir(&profile).unwrap();
    let host = Host::start_provider(
        &fixture,
        "series",
        &fixture.root.join("store"),
        &catalog_root,
        &profile,
    );
    let client = fixture.endpoint(&host);
    let mut submit = request(
        "jobs.submit",
        json!({"operation":"episodes.list","parameters":{"series_url":location.as_str()},"execution_timeout_ms":"30000"}),
    );
    submit.context.idempotency_key = Some("actual-series-first".into());
    let accepted = fixture.call("submit", &submit, Some(&host.endpoint), None, 0);
    let id: JobId = serde_json::from_value(accepted["data"]["job"]["job_id"].clone()).unwrap();
    let job = terminal(&client, &id);
    fs::write(
        fixture.root.join("observation.json"),
        serde_json::to_vec_pretty(&job).unwrap(),
    )
    .unwrap();
    fixture.call(
        "terminal",
        &request("jobs.get", json!({"job_id":id})),
        Some(&host.endpoint),
        None,
        0,
    );
    // Retain the failure and release owned resources before the functional success assertion.
    host.stop();
    let (raw, result) = if job.state == JobState::Completed {
        let output = job.output.as_ref().unwrap();
        let result = fixture.call_catalog(
            "fresh-episodes",
            &request(
                "episodes.list",
                json!({"result_artifact_sha256":output.sha256}),
            ),
            &catalog_root,
            0,
        );
        (result["data"]["raw_artifact"].clone(), Some(result))
    } else {
        (
            job.failure
                .as_ref()
                .map(|failure| failure.details["raw_artifact"].clone())
                .unwrap_or(Value::Null),
            None,
        )
    };
    if let Some(artifact_id) = raw["artifact_id"].as_str() {
        let mut offset = 0u64;
        let mut restored = Vec::new();
        for index in 0..128 {
            let chunk = fixture.call_catalog(
                &format!("raw-{index}"),
                &request(
                    "catalog.read_raw",
                    json!({"artifact_id":artifact_id,"offset":offset.to_string(),"limit":65536}),
                ),
                &catalog_root,
                0,
            );
            assert_eq!(chunk["data"]["capture"]["capture_kind"], "dom_snapshot");
            assert!(chunk["data"]["capture"]["status"].is_null());
            restored.extend_from_slice(chunk["data"]["text"].as_str().unwrap().as_bytes());
            let Some(next) = chunk["data"]["next_offset"].as_str() else {
                break;
            };
            let next = next.parse::<u64>().unwrap();
            assert!(next > offset);
            offset = next;
        }
        assert_eq!(
            restored,
            fs::read(catalog_root.join(raw["body_path"].as_str().unwrap())).unwrap()
        );
    }
    seed_files.read(&seed_sha).unwrap();
    assert_eq!(
        job.state,
        JobState::Completed,
        "Actual series did not complete: {:?}",
        job.failure
    );
    let result = result.unwrap();
    assert!(!result["data"]["entries"].as_array().unwrap().is_empty());
    assert_eq!(result["data"]["series"]["url"], location.as_str());
    for episode in result["data"]["entries"].as_array().unwrap() {
        assert_eq!(
            PageLocation::parse(episode["url"].as_str().unwrap())
                .unwrap()
                .kind(),
            ResourceKind::Episode
        );
    }
}

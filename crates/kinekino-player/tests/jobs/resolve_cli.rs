use super::fixture::{Fixture, Host, request};
use kinekino_player::application::api::{EpisodeClient, JobClient};
use kinekino_player::{
    application::{
        api::RequestContext,
        catalog::{CatalogResults, RawArtifact},
        episode_resolve::{EpisodeResolutionResults, ResolveEpisode},
        jobs::{GetJob, JobId, JobState},
    },
    domain::api_values::DecimalU64,
    infrastructure::catalog::CatalogFiles,
};
use serde_json::json;
use std::{
    fs,
    path::PathBuf,
    time::{Duration, Instant},
};

#[test]
#[ignore = "Explicit actual CLI episode activation and private evidence; no verified media or playback claim"]
pub(super) fn actual_cli_resolve_preserves_observed_player_script_response() {
    kinekino_platform::install();
    let source =
        PathBuf::from(std::env::var_os("EPISODE_SEED_ROOT").expect("Explicit episodes seed root"));
    let sha = std::env::var("EPISODE_SEED_RESULT_SHA256").expect("Explicit episodes seed SHA");
    let source_files = CatalogFiles::open(&source).unwrap();
    let (seed, _) = CatalogResults::read(&source_files, &sha).unwrap();
    assert_eq!(seed.operation, "episodes.list");
    let episode = seed.result.entries.first().unwrap();
    let fixture = Fixture::named("resolve-watch-cli");
    let catalog = fixture.root.join("catalog");
    let profile = fixture.root.join("profile");
    fs::create_dir(&catalog).unwrap();
    fs::create_dir(&profile).unwrap();
    let host = Host::start_provider(
        &fixture,
        "watch",
        &fixture.root.join("store"),
        &catalog,
        &profile,
    );
    let client = fixture.endpoint(&host);
    let schema = fixture.call(
        "schema",
        &request("schema", json!({"command":"jobs.submit"})),
        Some(&host.endpoint),
        None,
        0,
    );
    assert_eq!(schema["data"]["episode_resolution_can_complete"], true);
    assert_eq!(schema["data"]["episode_media_decode_available"], false);
    let mut submit = request(
        "jobs.submit",
        json!({"operation":"episodes.resolve","parameters":{"episode_url":episode.url},"execution_timeout_ms":"30000"}),
    );
    submit.context.idempotency_key = Some("actual-watch-resolution".into());
    let submitted = fixture.call("submit", &submit, Some(&host.endpoint), None, 0);
    let id: JobId = serde_json::from_value(submitted["data"]["job"]["job_id"].clone()).unwrap();
    let context = RequestContext {
        request_id: "typed-watch-submission".into(),
        idempotency_key: Some("actual-watch-resolution".into()),
        ..Default::default()
    };
    let repeated = client
        .submit_episode_resolution(
            context,
            ResolveEpisode {
                episode_url: episode.url.clone(),
            },
            Some(DecimalU64(30_000)),
        )
        .unwrap();
    assert!(!repeated.created);
    assert_eq!(repeated.job.job_id, id);
    let deadline = Instant::now() + Duration::from_secs(60);
    let job = loop {
        let job = client
            .get_job(Default::default(), GetJob { job_id: id.clone() })
            .unwrap();
        if job.state.terminal() {
            break job;
        }
        assert!(Instant::now() < deadline, "Episode job did not terminate");
        std::thread::sleep(Duration::from_millis(100));
    };
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
    host.stop();
    let evidence = job
        .output
        .as_ref()
        .map(|output| &output.receipt)
        .or_else(|| job.failure.as_ref().map(|failure| &failure.details))
        .expect("Resolution evidence receipt");
    let mut references = evidence["raw_artifacts"]
        .as_array()
        .expect("Episode evidence references")
        .clone();
    if evidence["resolution_artifact"].is_object() {
        references.push(evidence["resolution_artifact"].clone());
    }
    let mut dom_count = 0;
    let mut frame_dom_count = 0;
    let mut response_body_count = 0;
    let mut trace_observed = false;
    let mut runtime_samples = 0;
    let mut response_statuses = Vec::new();
    for (artifact_index, artifact) in references.iter().enumerate() {
        let mut bytes = Vec::new();
        let mut offset = 0u64;
        for index in 0..128 {
            let raw=fixture.call_catalog(&format!("raw-{artifact_index}-{index}"),&request("catalog.read_raw",json!({"artifact_id":artifact["artifact_id"],"offset":offset.to_string(),"limit":65536})),&catalog,0);
            if raw["data"]["capture"]["capture_kind"] == "browser_response_body" {
                let status = raw["data"]["capture"]["status"].as_u64().unwrap();
                response_statuses.push(status);
                if index == 0 {
                    response_body_count += 1;
                    assert!(raw["data"]["capture"]["browser"]["network"]["request_id"].is_string());
                    assert!(
                        raw["data"]["capture"]["browser"]["network"]["scope"]["loader_id"]
                            .is_string()
                    );
                }
            } else {
                assert!(raw["data"]["capture"]["status"].is_null());
            }
            if index == 0 && raw["data"]["capture"]["capture_kind"] == "dom_snapshot" {
                dom_count += 1;
            }
            if index == 0 && raw["data"]["capture"]["capture_kind"] == "frame_dom_snapshot" {
                frame_dom_count += 1;
                let frame = &raw["data"]["capture"]["browser"]["frame"];
                assert!(frame["identity"]["frame_id"].is_string());
                assert!(frame["completion"].is_object());
            }
            bytes.extend_from_slice(raw["data"]["text"].as_str().unwrap().as_bytes());
            let Some(next) = raw["data"]["next_offset"].as_str() else {
                break;
            };
            let next = next.parse::<u64>().unwrap();
            assert!(next > offset);
            offset = next;
        }
        assert!(
            bytes == fs::read(catalog.join(artifact["body_path"].as_str().unwrap())).unwrap(),
            "CLI raw bytes differ from the preserved artifact"
        );
        if let Ok(trace) = serde_json::from_slice::<serde_json::Value>(&bytes)
            && trace["representation"] == "browser_event_trace"
        {
            trace_observed = true;
            assert!(!trace["activation_history"].as_array().unwrap().is_empty());
            assert!(trace["main_navigation"].is_object());
            assert_eq!(trace["candidate_is_media"], false);
            assert_eq!(trace["observation_end"], "manifest_verified");
            assert!(
                trace["network"]["events"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|event| event["received_unix_ms"].is_string())
            );
            for frame in trace["frame_documents"].as_array().unwrap() {
                let samples = frame["runtime_observations"].as_array().unwrap();
                runtime_samples += samples.len();
                for sample in samples {
                    assert!(sample["snapshot"]["document_ready_state"].is_string());
                    assert!(sample["snapshot"]["video_count"].is_number());
                    assert!(sample["snapshot"]["play_overlay"]["present"].is_boolean());
                }
            }
        }
    }
    CatalogResults::read(&source_files, &sha).unwrap();
    assert_eq!(
        job.state,
        JobState::Completed,
        "Actual episode resolution did not verify and persist its manifest"
    );
    assert!(!references.is_empty());
    assert_eq!(dom_count, 2);
    assert!(frame_dom_count >= 1);
    assert!(response_body_count >= 3);
    assert!(
        response_statuses
            .iter()
            .all(|status| (100..=599).contains(status))
    );
    assert!(trace_observed);
    assert!(runtime_samples >= 8);
    let output = job.output.as_ref().unwrap();
    assert!(job.publication_started);
    assert_eq!(output.receipt["manifest_verified"], true);
    assert_eq!(output.receipt["media_verified"], false);
    assert_eq!(output.receipt["native_playback"], false);
    let result_ref: RawArtifact =
        serde_json::from_value(output.receipt["resolution_artifact"].clone()).unwrap();
    let files = CatalogFiles::open(&catalog).unwrap();
    let resolved = EpisodeResolutionResults::read(&files, &result_ref.artifact_id).unwrap();
    assert_eq!(resolved.schema_version, "2");
    assert_eq!(resolved.episode_url, episode.url);
    assert!(resolved.manifest.summary.resource_references > 0);
    {
        use kinekino_player::application::catalog::CatalogRawSource;
        let manifest = files
            .load_raw(&resolved.manifest.artifact.artifact_id)
            .unwrap();
        let network = manifest.capture.browser.unwrap().network.unwrap();
        let headers = network
            .request_headers
            .expect("Actual ExtraInfo request headers");
        assert!(headers.received_unix_ms.is_some());
        assert_eq!(headers.referer, resolved.manifest.referer);
    }
    assert_eq!(output.sha256, result_ref.body_sha256);
    let inspected = fixture.call_catalog(
        "inspect-resolution-fresh-cli",
        &request(
            "episodes.inspect_resolution",
            json!({"resolution_artifact_id":result_ref.artifact_id}),
        ),
        &catalog,
        0,
    );
    assert_eq!(inspected["data"]["manifest_verified"], true);
    assert_eq!(inspected["data"]["media_decoded"], false);
    assert_eq!(inspected["data"]["source_uris_redacted"], true);
    assert_eq!(
        inspected["data"]["source_descriptor_sha256"],
        resolved.source_descriptor_sha256
    );
    let tampered = fixture.root.join("tampered-resolution");
    for reference in resolved
        .raw_artifacts
        .iter()
        .chain(std::iter::once(&result_ref))
    {
        for relative in [&reference.body_path, &reference.metadata_path] {
            let destination = tampered.join(relative);
            fs::create_dir_all(destination.parent().unwrap()).unwrap();
            fs::copy(catalog.join(relative), destination).unwrap();
        }
    }
    let manifest_copy = tampered.join(&resolved.manifest.artifact.body_path);
    let mut bytes = fs::read(&manifest_copy).unwrap();
    bytes[0] ^= 1;
    fs::write(manifest_copy, bytes).unwrap();
    assert_eq!(
        EpisodeResolutionResults::read(
            &CatalogFiles::open(&tampered).unwrap(),
            &result_ref.artifact_id
        )
        .unwrap_err()
        .code,
        kinekino_player::application::api::ErrorCode::SourceExpired
    );
    EpisodeResolutionResults::read(&files, &result_ref.artifact_id).unwrap();
    let rejected = fixture.call_catalog(
        "inspect-tampered-fresh-cli",
        &request(
            "episodes.inspect_resolution",
            json!({"resolution_artifact_id":result_ref.artifact_id}),
        ),
        &tampered,
        8,
    );
    assert_eq!(rejected["error"]["code"], "source_expired");
    fixture.call_catalog(
        "inspect-original-after-tamper",
        &request(
            "episodes.inspect_resolution",
            json!({"resolution_artifact_id":result_ref.artifact_id}),
        ),
        &catalog,
        0,
    );
    // Explicit synthetic schema-version fixture; never presented as a historical valid site result.
    {
        use kinekino_player::application::catalog::{
            CaptureMetadata, CatalogArtifacts, CatalogCaptureKind,
        };
        use sha2::{Digest, Sha256};
        let legacy_root = fixture.root.join("legacy-schema-fixture");
        fs::create_dir(&legacy_root).unwrap();
        let legacy = CatalogFiles::open(&legacy_root).unwrap();
        let bytes = b"{\"schema_version\":\"1\",\"fixture\":\"synthetic_version_boundary\"}";
        let raw = legacy
            .save(
                bytes,
                &CaptureMetadata {
                    requested_url: "https://catalog.example.test/e/fixture".into(),
                    final_url: "https://catalog.example.test/e/fixture".into(),
                    redirects: vec![],
                    observed_at: "2026-10-19T00:00:00Z".into(),
                    parser_revision: "synthetic-legacy-episode-result-v1".into(),
                    capture_kind: CatalogCaptureKind::EpisodeResolution,
                    status: None,
                    content_type: "application/json".into(),
                    content_encoding: None,
                    body_sha256: format!("{:x}", Sha256::digest(bytes)),
                    body_bytes: DecimalU64(bytes.len() as u64),
                    browser: None,
                },
            )
            .unwrap();
        let rejected = fixture.call_catalog(
            "inspect-legacy-version",
            &request(
                "episodes.inspect_resolution",
                json!({"resolution_artifact_id":raw.artifact_id}),
            ),
            &legacy_root,
            3,
        );
        assert_eq!(rejected["error"]["code"], "unsupported");
        let raw = fixture.call_catalog(
            "read-legacy-version-raw",
            &request(
                "catalog.read_raw",
                json!({"artifact_id":raw.artifact_id,"offset":"0","limit":65536}),
            ),
            &legacy_root,
            0,
        );
        assert_eq!(raw["data"]["text"], std::str::from_utf8(bytes).unwrap());
    }
}

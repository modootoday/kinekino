use super::fixture::{Fixture, Host, request};
use super::synthetic_browser::{FixtureParser, SyntheticBrowser};
use kinekino_player::application::api::JobClient;
#[path = "provider/observation_trace.rs"]
mod observation_trace;
use kinekino_player::{
    application::{
        api::{ApplicationClient, PlayerSdk, RequestContext},
        browser_provider::BrowserCatalogJobs,
        catalog::{CatalogArchive, CatalogCaptureService},
        jobs::{ExecutorRegistry, GetJob, JobExecutor, JobId, JobRecord, JobState},
    },
    infrastructure::{
        catalog::CatalogFiles,
        jobs,
        local_host::{EndpointClient, LocalHost, TransportOptions},
        subtitle_files::SubtitleFiles,
    },
};
use serde_json::json;
use std::{
    fs,
    sync::Arc,
    time::{Duration, Instant},
};

pub(super) fn terminal(client: &dyn ApplicationClient, id: &JobId) -> JobRecord {
    let deadline = Instant::now()
        .checked_add(Duration::from_secs(45))
        .expect("Job observation deadline overflow");
    terminal_until(client, id, deadline)
}
pub(super) fn try_terminal_until(
    endpoint: &str,
    id: &JobId,
    original: kinekino_process::domain::deadline::Deadline,
    cap: Duration,
    root: &std::path::Path,
) -> Result<JobRecord, Box<super::kinekino_process::CliFailure>> {
    use super::kinekino_process::{CliFailure, phase_deadline, sdk_failure, transport_at};
    let failed = |stage, error| CliFailure::not_spawned(stage, error, root);
    let deadline =
        phase_deadline(original, cap).map_err(|error| failed("job observation budget", error))?;
    let mut trace = observation_trace::ObservationTrace::new();
    let result = (|| {
        loop {
            let client = transport_at(endpoint, deadline)
                .map_err(|error| failed("job transport budget", error))?;
            trace.begin_read();
            let job = match client.get_job(RequestContext::default(), GetJob { job_id: id.clone() })
            {
                Ok(job) => {
                    trace.observed(job.state);
                    job
                }
                Err(error) => {
                    trace.failed(&error);
                    return Err(sdk_failure("job observation", error, root));
                }
            };
            if deadline.operation_remaining(Instant::now()).is_zero() {
                return Err(failed(
                    "job observation deadline",
                    std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        "Original job observation cutoff exceeded",
                    ),
                ));
            }
            if job.state.terminal() {
                return Ok(job);
            }
            std::thread::sleep(
                Duration::from_millis(100).min(deadline.operation_remaining(Instant::now())),
            );
        }
    })();
    trace.finish(result, root, id)
}
// Observation timeout does not cancel the original job or replace its owner.
pub(super) fn terminal_until(
    client: &dyn ApplicationClient,
    id: &JobId,
    deadline: Instant,
) -> JobRecord {
    loop {
        let job = client
            .get_job(RequestContext::default(), GetJob { job_id: id.clone() })
            .unwrap();
        if job.state.terminal() {
            return job;
        }
        assert!(
            Instant::now() < deadline,
            "browser job did not reach terminal state"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}
#[test]
fn synthetic_browser_job_uses_actual_files_and_cli_reopens_verified_result() {
    kinekino_platform::install();
    let fixture = Fixture::named("provider-synthetic");
    let catalog_root = fixture.root.join("catalog");
    fs::create_dir(&catalog_root).unwrap();
    let files = Arc::new(CatalogFiles::open(&catalog_root).unwrap());
    let browser = Arc::new(SyntheticBrowser::default());
    let browser_jobs: Arc<dyn JobExecutor> = Arc::new(BrowserCatalogJobs::new(
        browser.clone(),
        browser,
        Arc::new(CatalogCaptureService::new(
            Arc::new(FixtureParser),
            files.clone(),
        )),
        files.clone(),
    ));
    let registry = ExecutorRegistry::new(vec![jobs::subtitle_executor(), browser_jobs]).unwrap();
    let expected_operations: std::collections::BTreeSet<String> =
        registry.operations().into_iter().collect();
    assert_eq!(
        expected_operations,
        [
            "subtitles.convert",
            "subtitles.export_range",
            "subtitles.from_transcript",
            "catalog.search",
            "episodes.list"
        ]
        .into_iter()
        .map(str::to_owned)
        .collect(),
    );
    let worker =
        jobs::start_host_with_executor(&fixture.root.join("store"), Arc::new(registry)).unwrap();
    let sdk = Arc::new(
        PlayerSdk::new(SubtitleFiles)
            .with_jobs(worker.client())
            .with_catalog(CatalogArchive::new(files.clone(), files)),
    );
    let local = LocalHost::start(sdk.clone(), TransportOptions::default()).unwrap();
    let caps = fixture.call(
        "provider-capabilities",
        &request("capabilities", json!({})),
        Some(local.endpoint()),
        None,
        0,
    );
    assert_eq!(caps["data"]["catalog"]["live_read_available"], false);
    assert_eq!(caps["data"]["catalog"]["result_read_available"], true);
    let schema = fixture.call(
        "provider-submit-schema",
        &request("schema", json!({"command":"jobs.submit"})),
        Some(local.endpoint()),
        None,
        0,
    );
    let variants = schema["data"]["request"]["properties"]["payload"]["oneOf"]
        .as_array()
        .unwrap();
    let schema_operations: std::collections::BTreeSet<String> = variants
        .iter()
        .map(|variant| {
            variant["properties"]["operation"]["const"]
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect();
    assert_eq!(
        variants.len(),
        schema_operations.len(),
        "Duplicate operation schema"
    );
    assert_eq!(schema_operations, expected_operations);
    let mut submit = request(
        "jobs.submit",
        json!({"operation":"catalog.search","parameters":{"category":"airing"},"execution_timeout_ms":"5000"}),
    );
    submit.context.idempotency_key = Some("catalog-job".into());
    let submitted = fixture.call("provider-submit", &submit, Some(local.endpoint()), None, 0);
    let id: JobId = serde_json::from_value(submitted["data"]["job"]["job_id"].clone()).unwrap();
    let job = terminal(sdk.as_ref(), &id);
    assert_eq!(job.state, JobState::Completed, "{job:?}");
    let output = job.output.unwrap();
    fixture.call(
        "provider-terminal",
        &request("jobs.get", json!({"job_id":id})),
        Some(local.endpoint()),
        None,
        0,
    );
    drop(local);
    worker.shutdown().unwrap();
    let read = request(
        "catalog.search",
        json!({"result_artifact_sha256":output.sha256}),
    );
    let result = fixture.call_catalog("provider-fresh-cli-read", &read, &catalog_root, 0);
    assert_eq!(result["data"]["capture"]["capture_kind"], "dom_snapshot");
    assert!(result["data"]["capture"]["status"].is_null());
    assert_eq!(
        result["data"]["capture"]["browser"]["engine_version"],
        "synthetic-no-engine"
    );
    assert_eq!(result["data"]["entries"][0]["title"], "日本語 한글");
    assert_eq!(
        result["data"]["result_artifact"]["source_descriptor_sha256"],
        job.input.source.sha256
    );
    let raw_request = request(
        "catalog.read_raw",
        json!({"artifact_id":result["data"]["raw_artifact"]["artifact_id"],"limit":65536}),
    );
    let raw = fixture.call_catalog("provider-fresh-cli-raw", &raw_request, &catalog_root, 0);
    assert!(
        raw["data"]["text"]
            .as_str()
            .unwrap()
            .contains("日本語 한글")
    );
    assert_eq!(raw["data"]["capture"]["capture_kind"], "dom_snapshot");
    assert!(raw["data"]["next_offset"].is_null());
    fixture.call_catalog(
        "provider-invalid-raw-offset",
        &request(
            "catalog.read_raw",
            json!({"artifact_id":result["data"]["raw_artifact"]["artifact_id"],"offset":"9999999"}),
        ),
        &catalog_root,
        2,
    );
    fixture.call_catalog(
        "provider-wrong-operation",
        &request("episodes.list", read.payload.clone()),
        &catalog_root,
        2,
    );
    let body = catalog_root.join(
        result["data"]["raw_artifact"]["body_path"]
            .as_str()
            .unwrap(),
    );
    fs::write(body, b"corrupt").unwrap();
    fixture.call_catalog("provider-corrupt-body", &read, &catalog_root, 8);
    fixture.call_catalog("provider-corrupt-raw", &raw_request, &catalog_root, 8);
}

#[test]
#[ignore = "Explicit real WebView2/network observation; requires centrally allocated CLI provider run"]
fn actual_provider_cli_single_category_and_eof_shutdown() {
    kinekino_platform::install();
    let fixture = Fixture::named("provider-actual");
    let catalog_root = fixture.root.join("catalog");
    let profile = fixture.root.join("profile");
    fs::create_dir(&catalog_root).unwrap();
    fs::create_dir(&profile).unwrap();
    let host = Host::start_provider(
        &fixture,
        "provider",
        &fixture.root.join("store"),
        &catalog_root,
        &profile,
    );
    let client = EndpointClient::new(&host.endpoint, TransportOptions::default()).unwrap();
    let mut submit = request(
        "jobs.submit",
        json!({"operation":"catalog.search","parameters":{"category":"airing"},"execution_timeout_ms":"30000"}),
    );
    submit.context.idempotency_key = Some("actual-category-one".into());
    let submitted = fixture.call(
        "actual-category-submit",
        &submit,
        Some(&host.endpoint),
        None,
        0,
    );
    let id: JobId = serde_json::from_value(submitted["data"]["job"]["job_id"].clone()).unwrap();
    let job = terminal(&client, &id);
    fixture.call(
        "actual-category-terminal",
        &request("jobs.get", json!({"job_id":id})),
        Some(&host.endpoint),
        None,
        0,
    );
    fs::write(
        fixture.root.join("observation.json"),
        serde_json::to_vec_pretty(&job).unwrap(),
    )
    .unwrap();
    if job.state == JobState::Completed {
        let output = job.output.as_ref().unwrap();
        let read = request(
            "catalog.search",
            json!({"result_artifact_sha256":output.sha256}),
        );
        let result = fixture.call("actual-category-read", &read, Some(&host.endpoint), None, 0);
        assert_eq!(result["data"]["capture"]["capture_kind"], "dom_snapshot");
        assert!(result["data"]["capture"]["status"].is_null());
        host.stop();
        fixture.call_catalog("actual-category-fresh-cli-read", &read, &catalog_root, 0);
    } else {
        assert!(job.output.is_none());
        assert!(job.failure.is_some());
        host.stop();
    }
}

#[test]
#[ignore = "Explicit actual three-category validation; requires centrally allocated browser CLI run"]
fn actual_provider_cli_three_categories_and_eof_shutdown() {
    kinekino_platform::install();
    let fixture = Fixture::named("provider-three-categories");
    let catalog_root = fixture.root.join("catalog");
    let profile = fixture.root.join("profile");
    fs::create_dir(&catalog_root).unwrap();
    fs::create_dir(&profile).unwrap();
    let host = Host::start_provider(
        &fixture,
        "provider",
        &fixture.root.join("store"),
        &catalog_root,
        &profile,
    );
    let client = EndpointClient::new(&host.endpoint, TransportOptions::default()).unwrap();
    let mut completed = Vec::new();
    let mut failures = Vec::new();
    for category in ["airing", "completed", "movies"] {
        let mut submit = request(
            "jobs.submit",
            json!({"operation":"catalog.search","parameters":{"category":category},"execution_timeout_ms":"30000"}),
        );
        submit.context.idempotency_key = Some(format!("actual-{category}"));
        let submitted = fixture.call(
            &format!("{category}-submit"),
            &submit,
            Some(&host.endpoint),
            None,
            0,
        );
        let id: JobId = serde_json::from_value(submitted["data"]["job"]["job_id"].clone()).unwrap();
        let job = terminal(&client, &id);
        fixture.call(
            &format!("{category}-terminal"),
            &request("jobs.get", json!({"job_id":id})),
            Some(&host.endpoint),
            None,
            0,
        );
        fs::write(
            fixture.root.join(format!("{category}-observation.json")),
            serde_json::to_vec_pretty(&job).unwrap(),
        )
        .unwrap();
        if job.state == JobState::Completed {
            let output = job.output.as_ref().unwrap();
            let read = request(
                "catalog.search",
                json!({"result_artifact_sha256":output.sha256}),
            );
            let result = fixture.call(
                &format!("{category}-read"),
                &read,
                Some(&host.endpoint),
                None,
                0,
            );
            assert!(result["data"]["entries"].is_array());
            assert_eq!(result["data"]["capture"]["capture_kind"], "dom_snapshot");
            assert!(result["data"]["capture"]["status"].is_null());
            completed.push((category, read, result));
        } else {
            failures.push(format!("{category}: {:?}", job.failure));
        }
    }
    let original_endpoint = host.endpoint.clone();
    host.stop();
    let closed = client.execute(request("version", json!({})));
    fs::write(
        fixture.root.join("logs/closed-endpoint.json"),
        serde_json::to_vec_pretty(&closed).unwrap(),
    )
    .unwrap();
    assert!(!closed.ok);
    let recovered = Host::start_provider(
        &fixture,
        "provider-recovered",
        &fixture.root.join("store"),
        &catalog_root,
        &profile,
    );
    assert_ne!(recovered.endpoint, original_endpoint);
    recovered.stop();
    for (category, read, result) in completed {
        fixture.call_catalog(&format!("{category}-fresh-read"), &read, &catalog_root, 0);
        let artifact_id = result["data"]["raw_artifact"]["artifact_id"]
            .as_str()
            .unwrap();
        let mut offset = 0u64;
        let mut bytes = Vec::new();
        let mut index = 0;
        loop {
            let raw = fixture.call_catalog(
                &format!("{category}-raw-{index}"),
                &request(
                    "catalog.read_raw",
                    json!({"artifact_id":artifact_id,"offset":offset.to_string(),"limit":65536}),
                ),
                &catalog_root,
                0,
            );
            bytes.extend_from_slice(raw["data"]["text"].as_str().unwrap().as_bytes());
            let Some(next) = raw["data"]["next_offset"].as_str() else {
                break;
            };
            let next = next.parse::<u64>().unwrap();
            assert!(next > offset);
            offset = next;
            index += 1;
            assert!(index < 128);
        }
        assert_eq!(
            bytes,
            fs::read(
                catalog_root.join(
                    result["data"]["raw_artifact"]["body_path"]
                        .as_str()
                        .unwrap()
                )
            )
            .unwrap()
        );
    }
    assert!(
        failures.is_empty(),
        "Actual categories did not all complete: {failures:?}"
    );
}

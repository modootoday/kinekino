//! Original category chain over a borrowed endpoint and caller's unchanged original deadline.
use super::evidence::provider_links;
#[path = "chain/adjacent.rs"]
mod adjacent;
use crate::{
    fixture::{Fixture, request},
    kinekino_process::CliFailure,
    playback_observation::{
        try_capture_frame as capture_frame, try_wait_for_progress as wait_for_progress,
    },
    provider::try_terminal_until as terminal_until,
};
use kinekino_core::source::document::{PageLocation, ResourceKind};
use kinekino_player::{
    application::{
        api::Request,
        catalog::{CatalogRawSource, CatalogResults},
        episode_resolve::EpisodeResolutionResults,
        jobs::{JobId, JobRecord, JobState},
        playback_session::{ClosedEpisodePlayback, OpenedEpisodePlayback},
    },
    infrastructure::catalog::CatalogFiles,
};
use kinekino_process::domain::deadline::Deadline;
use serde_json::{Value, json};
use std::{fs, time::Duration};

/// This value conveys no GUI/parent ownership or readiness authority. Caller validates its owner.
pub(crate) struct EndpointInput<'a> {
    pub endpoint: &'a str,
    pub original: Deadline,
}
pub(crate) struct ChainBudgets {
    pub execution_ms: u64,
    pub observation: Duration,
    pub request: Duration,
}

fn named(label: &str, command: &str, payload: Value) -> Request {
    let mut value = request(command, payload);
    value.context.request_id = label.into();
    value
}

fn call(
    fixture: &Fixture,
    host: &EndpointInput<'_>,
    label: &str,
    command: &str,
    payload: Value,
    budgets: &ChainBudgets,
) -> Result<Value, Box<CliFailure>> {
    let endpoint = host.endpoint;
    fs::write(
        fixture.root.join(format!("logs/{label}.route.json")),
        serde_json::to_vec_pretty(&json!({"endpoint":endpoint,"label":label,"command":command}))
            .unwrap(),
    )
    .unwrap();
    fixture.try_call_until(
        label,
        &named(label, command, payload),
        endpoint,
        host.original,
        budgets.request,
        0,
    )
}
fn submit(
    fixture: &Fixture,
    host: &EndpointInput<'_>,
    label: &str,
    operation: &str,
    parameters: Value,
    budgets: &ChainBudgets,
) -> Result<JobRecord, Box<CliFailure>> {
    let endpoint = host.endpoint;
    let mut request = named(
        label,
        "jobs.submit",
        json!({"operation":operation,"parameters":parameters,"execution_timeout_ms":budgets.execution_ms.to_string()}),
    );
    request.context.idempotency_key = Some(label.into());
    fs::write(
        fixture.root.join(format!("logs/{label}.route.json")),
        serde_json::to_vec_pretty(
            &json!({"endpoint":endpoint,"operation":operation,"parameters":parameters}),
        )
        .unwrap(),
    )
    .unwrap();
    let accepted =
        fixture.try_call_until(label, &request, endpoint, host.original, budgets.request, 0)?;
    let id: JobId = serde_json::from_value(accepted["data"]["job"]["job_id"].clone()).unwrap();
    let terminal = terminal_until(
        endpoint,
        &id,
        host.original,
        budgets.observation,
        &fixture.root,
    )?;
    fs::write(
        fixture.root.join(format!("{label}-terminal.json")),
        serde_json::to_vec_pretty(&terminal).unwrap(),
    )
    .unwrap();
    let actual = call(
        fixture,
        host,
        &format!("{label}-get"),
        "jobs.get",
        json!({"job_id":id}),
        budgets,
    )?;
    assert_eq!(
        actual["data"]["job_id"],
        serde_json::to_value(&terminal.job_id).unwrap()
    );
    assert_eq!(
        terminal.state,
        JobState::Completed,
        "{label}: {:?}",
        terminal.failure
    );
    Ok(terminal)
}

fn unsupported_selection(
    fixture: &Fixture,
    category: &str,
    result_sha: &str,
    reason: &'static str,
) -> Box<CliFailure> {
    let mut failure = CliFailure::not_spawned(
        "category scenario unsupported",
        std::io::Error::new(std::io::ErrorKind::Unsupported, reason),
        &fixture.root,
    );
    let evidence = serde_json::json!({"category":category,"scenario_supported":false,
        "reason":reason,"observed_result_artifact_sha256":result_sha,
        "scope":"this episode-list/playback verification chain; not a fabricated product SDK response",
        "substitute_episode_created":false});
    if let Err(error) = serde_json::to_vec_pretty(&evidence)
        .map_err(std::io::Error::other)
        .and_then(|bytes| {
            fs::write(
                fixture
                    .root
                    .join(format!("{category}-scenario-unsupported.json")),
                bytes,
            )
        })
    {
        failure
            .secondary
            .push(format!("Unsupported-scenario evidence: {error}"));
    }
    failure
}

pub(crate) fn run_category(
    fixture: &Fixture,
    host: &EndpointInput<'_>,
    files: &CatalogFiles,
    category: &str,
    budgets: &ChainBudgets,
) -> Result<Value, Box<CliFailure>> {
    run_category_scenario(fixture, host, files, category, budgets, false)
}
pub(crate) fn run_category_with_adjacent(
    fixture: &Fixture,
    host: &EndpointInput<'_>,
    files: &CatalogFiles,
    category: &str,
    budgets: &ChainBudgets,
) -> Result<Value, Box<CliFailure>> {
    run_category_scenario(fixture, host, files, category, budgets, true)
}
fn run_category_scenario(
    fixture: &Fixture,
    host: &EndpointInput<'_>,
    files: &CatalogFiles,
    category: &str,
    budgets: &ChainBudgets,
    adjacent: bool,
) -> Result<Value, Box<CliFailure>> {
    let _: kinekino_player::application::catalog::Category =
        serde_json::from_value(json!(category)).map_err(|error| {
            CliFailure::not_spawned(
                "category input",
                std::io::Error::new(std::io::ErrorKind::InvalidInput, error),
                &fixture.root,
            )
        })?;
    let endpoint = host.endpoint;
    let listing = submit(
        fixture,
        host,
        &format!("{category}-category"),
        "catalog.search",
        json!({"category":category}),
        budgets,
    )?;
    let category_sha = &listing.output.as_ref().unwrap().sha256;
    let category_read = call(
        fixture,
        host,
        &format!("{category}-listing"),
        "catalog.search",
        json!({"result_artifact_sha256":category_sha}),
        budgets,
    )?;
    let (category_result, _) = CatalogResults::read(files, category_sha).unwrap();
    assert_eq!(
        category_read["data"]["entries"],
        serde_json::to_value(&category_result.result.entries).unwrap()
    );
    assert_eq!(
        category_read["data"]["result_artifact"]["sha256"],
        category_sha.as_str()
    );
    let category_capture = files
        .load_raw(&category_result.result.raw_artifact.artifact_id)
        .unwrap()
        .capture;
    assert_eq!(category_result.operation, "catalog.search");
    assert_eq!(category_result.parameters["category"], category);
    let (index, series) = category_result
        .result
        .entries
        .iter()
        .enumerate()
        .find(|(_, entry)| {
            PageLocation::parse(&entry.url)
                .is_ok_and(|location| location.kind() == ResourceKind::Series)
        })
        .ok_or_else(|| {
            unsupported_selection(
                fixture,
                category,
                category_sha,
                "Verified category has no Series entry; this episode-list scenario cannot continue",
            )
        })?;
    let episodes = submit(
        fixture,
        host,
        &format!("{category}-episodes"),
        "episodes.list",
        json!({"series_url":series.url}),
        budgets,
    )?;
    let episodes_sha = &episodes.output.as_ref().unwrap().sha256;
    let episode_read = call(
        fixture,
        host,
        &format!("{category}-episode-list"),
        "episodes.list",
        json!({"result_artifact_sha256":episodes_sha}),
        budgets,
    )?;
    let (episode_result, _) = CatalogResults::read(files, episodes_sha).unwrap();
    assert_eq!(
        episode_read["data"]["entries"],
        serde_json::to_value(&episode_result.result.entries).unwrap()
    );
    assert_eq!(
        episode_read["data"]["result_artifact"]["sha256"],
        episodes_sha.as_str()
    );
    assert_eq!(episode_result.operation, "episodes.list");
    assert_eq!(episode_result.parameters["series_url"], series.url);
    let episode = episode_result
        .result
        .entries
        .first()
        .ok_or_else(|| unsupported_selection(fixture, category, episodes_sha,
            "Verified episode list is empty; this playback scenario cannot create a substitute episode"))?;
    assert_eq!(
        PageLocation::parse(&episode.url).unwrap().kind(),
        ResourceKind::Episode
    );
    fs::write(fixture.root.join(format!("{category}-selection.json")),serde_json::to_vec_pretty(&json!({"endpoint":endpoint,"category_sha":category_sha,"series_index":index,"series":series,"episodes_sha":episodes_sha,"episode_index":0,"episode":episode})).unwrap()).unwrap();
    let resolution = submit(
        fixture,
        host,
        &format!("{category}-resolve"),
        "episodes.resolve",
        json!({"episode_url":episode.url}),
        budgets,
    )?;
    let output = resolution.output.as_ref().unwrap();
    let artifact = output.receipt["resolution_artifact"]["artifact_id"]
        .as_str()
        .unwrap();
    assert_eq!(output.receipt["manifest_verified"], true);
    assert_eq!(output.receipt["media_verified"], false);
    assert_eq!(output.receipt["native_playback"], false);
    assert_eq!(
        output.sha256,
        output.receipt["resolution_artifact"]["body_sha256"]
    );
    let verified = EpisodeResolutionResults::read(files, artifact).unwrap();
    assert_eq!(verified.episode_url, episode.url);
    let mut provider = provider_links(
        &verified.episode_url,
        &verified.player_document_url,
        &verified.source_descriptor_sha256,
        &verified.raw_artifacts,
    );
    provider["manifest_artifact"] = serde_json::to_value(&verified.manifest.artifact).unwrap();
    fs::write(
        fixture.root.join(format!("{category}-provider.json")),
        serde_json::to_vec_pretty(&provider).unwrap(),
    )
    .unwrap();
    let inspected = call(
        fixture,
        host,
        &format!("{category}-inspect"),
        "episodes.inspect_resolution",
        json!({"resolution_artifact_id":artifact}),
        budgets,
    )?;
    assert_eq!(
        inspected["data"]["source_descriptor_sha256"],
        verified.source_descriptor_sha256
    );
    assert_eq!(inspected["data"]["manifest_verified"], true);
    assert_eq!(inspected["data"]["media_decoded"], false);
    let opened = call(
        fixture,
        host,
        &format!("{category}-open"),
        "playback.open",
        json!({"resolution_artifact_id":artifact,"paused":false,"volume_percent":0.}),
        budgets,
    )?;
    let opened: OpenedEpisodePlayback = serde_json::from_value(opened["data"].clone()).unwrap();
    assert_ne!(opened.engine_instance_id.0, 0);
    assert_eq!(opened.resolution_artifact_id, artifact);
    wait_for_progress(fixture, endpoint, &opened, category, 0, host.original)?;
    let (frame, frame_sha) = capture_frame(
        fixture,
        endpoint,
        &opened,
        category,
        host.original,
        budgets.request,
    )?;
    let initial_instance = opened.engine_instance_id;
    let (opened, transitions) = if adjacent {
        adjacent::round_trip(
            fixture,
            host,
            files,
            category,
            budgets,
            episodes_sha,
            opened,
        )?
    } else {
        (opened, Vec::<Value>::new())
    };
    let closed = call(
        fixture,
        host,
        &format!("{category}-close"),
        "playback.close",
        json!({"engine_instance_id":opened.engine_instance_id}),
        budgets,
    )?;
    let closed: ClosedEpisodePlayback = serde_json::from_value(closed["data"].clone()).unwrap();
    assert_eq!(closed.engine_instance_id, opened.engine_instance_id);
    assert!(
        closed.engine_exited
            && closed.ipc_worker_exited
            && closed.stdout_worker_exited
            && closed.stderr_worker_exited
    );
    assert!(
        closed.diagnostics.complete
            && !closed.diagnostics.truncated
            && !closed.diagnostics.read_failed
    );
    let diagnostic = call(
        fixture,
        host,
        &format!("{category}-diagnostic"),
        "catalog.read_raw",
        json!({"artifact_id":closed.diagnostics.artifact.artifact_id,"offset":"0","limit":65536}),
        budgets,
    )?;
    assert_eq!(
        diagnostic["data"]["capture"]["capture_kind"],
        "native_engine_diagnostic"
    );
    let absent = fixture.try_call_until(
        &format!("{category}-closed-observation"),
        &named(
            &format!("{category}-closed-observation"),
            "playback.engine",
            json!({}),
        ),
        endpoint,
        host.original,
        budgets.request,
        4,
    )?;
    assert_eq!(absent["error"]["code"], "not_found");
    Ok(
        json!({"category":category,"endpoint":endpoint,"category_sha":category_sha,"episodes_sha":episodes_sha,
            "initial_resolution_artifact_id":artifact,"initial_source_descriptor_sha256":verified.source_descriptor_sha256,
            "initial_engine_instance_id":initial_instance,"initial_provider_evidence":format!("{category}-provider.json"),
            "resolution_artifact_id":opened.resolution_artifact_id,"engine_instance_id":opened.engine_instance_id,
            "initial_frame":frame,"initial_frame_sha256":frame_sha,"closed":closed,
            "filter_capture_metadata":category_capture,"transitions":transitions}),
    )
}

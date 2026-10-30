//! Existing adjacent transition API, original endpoint/deadline, no close/open replacement.
use super::*;
use kinekino_player::application::{
    episode_queue::EpisodeQueue,
    playback_session::{
        EpisodePlaybackObservation,
        transition::{ReplacedEpisode, SubmittedTransition, TransitionObservation},
    },
};
use kinekino_player::domain::episodes::{Episode, EpisodePage};
use std::{
    io,
    time::{Duration, Instant},
};

fn reject(fixture: &Fixture, label: &'static str, message: &str) -> Box<CliFailure> {
    CliFailure::not_spawned(label, io::Error::other(message), &fixture.root)
}
fn ids(value: &Value) -> Vec<String> {
    assert!(
        value["data"]["next_cursor"].is_null(),
        "Fixture job list must be complete"
    );
    let mut ids: Vec<_> = value["data"]["jobs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|job| job["job_id"].as_str().unwrap().to_owned())
        .collect();
    ids.sort();
    ids
}
fn observe(
    fixture: &Fixture,
    host: &EndpointInput<'_>,
    label: &str,
    budgets: &ChainBudgets,
    current: &OpenedEpisodePlayback,
) -> Result<(), Box<CliFailure>> {
    let value = call(fixture, host, label, "playback.engine", json!({}), budgets)?;
    let observed: EpisodePlaybackObservation =
        serde_json::from_value(value["data"].clone()).unwrap();
    assert_eq!(observed.engine_instance_id, current.engine_instance_id);
    assert_eq!(
        observed.resolution_artifact_id,
        current.resolution_artifact_id
    );
    assert!(!observed.playback.has_error);
    Ok(())
}
fn expected_target(
    files: &CatalogFiles,
    list_sha: &str,
    current: &OpenedEpisodePlayback,
    delta: isize,
) -> String {
    let (stored, _) = CatalogResults::read(files, list_sha).unwrap();
    assert_eq!(stored.operation, "episodes.list");
    let resolution =
        EpisodeResolutionResults::read(files, &current.resolution_artifact_id).unwrap();
    let mut queue = EpisodeQueue::default();
    let page = EpisodePage {
        page: resolution.episode_url.clone(),
        series: stored.parameters["series_url"].as_str().unwrap().into(),
        title: stored.result.title,
        entries: stored
            .result
            .entries
            .into_iter()
            .map(|entry| Episode {
                title: entry.title,
                url: entry.url,
            })
            .collect(),
    };
    assert!(queue.observe(page, &resolution.episode_url));
    queue
        .adjacent(delta)
        .expect("Actual adjacent scenario requires a distinct next episode")
}

pub(super) fn round_trip(
    fixture: &Fixture,
    host: &EndpointInput<'_>,
    files: &CatalogFiles,
    category: &str,
    budgets: &ChainBudgets,
    list_sha: &str,
    mut current: OpenedEpisodePlayback,
) -> Result<(OpenedEpisodePlayback, Vec<Value>), Box<CliFailure>> {
    let mut receipts = Vec::new();
    let original_episode = EpisodeResolutionResults::read(files, &current.resolution_artifact_id)
        .unwrap()
        .episode_url;
    for (command, suffix, delta) in [
        ("playback.next", "next", 1),
        ("playback.previous", "previous", -1),
    ] {
        let target = expected_target(files, list_sha, &current, delta);
        if delta == -1 {
            assert_eq!(target, original_episode);
        }
        let prefix = format!("{category}-{suffix}");
        let before = call(
            fixture,
            host,
            &format!("{prefix}-jobs-before"),
            "jobs.list",
            json!({"cursor":null,"limit":100}),
            budgets,
        )?;
        let begun = call(
            fixture,
            host,
            &format!("{prefix}-begin"),
            command,
            json!({
            "expected_engine_instance_id":current.engine_instance_id,"catalog_result_sha256":list_sha,
            "resolution_submission_key":format!("{prefix}-resolve"),"paused":false,"volume_percent":0.,
            "execution_timeout_ms":budgets.execution_ms.to_string()}),
            budgets,
        )?;
        let begun: SubmittedTransition = serde_json::from_value(begun["data"].clone()).unwrap();
        assert!(!begun.engine_replaced);
        let after = call(
            fixture,
            host,
            &format!("{prefix}-jobs-after"),
            "jobs.list",
            json!({"cursor":null,"limit":100}),
            budgets,
        )?;
        let before_ids = ids(&before);
        let after_ids = ids(&after);
        assert_eq!(after_ids.len(), before_ids.len() + 1);
        let new_ids: Vec<_> = after_ids
            .iter()
            .filter(|id| !before_ids.contains(id))
            .cloned()
            .collect();
        assert_eq!(
            new_ids,
            vec![
                serde_json::to_value(&begun.resolution_job_id)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_owned()
            ]
        );
        let duplicate = fixture.try_call_until(&format!("{prefix}-duplicate"),
            &named(&format!("{prefix}-duplicate"), command, json!({
                "expected_engine_instance_id":current.engine_instance_id,"catalog_result_sha256":list_sha,
                "resolution_submission_key":format!("{prefix}-duplicate-resolve"),"paused":false,"volume_percent":0.,
                "execution_timeout_ms":budgets.execution_ms.to_string()})),
            host.endpoint, host.original, budgets.request, 5)?;
        assert_eq!(duplicate["error"]["code"], "revision_conflict");
        let after_duplicate = call(
            fixture,
            host,
            &format!("{prefix}-jobs-duplicate"),
            "jobs.list",
            json!({"cursor":null,"limit":100}),
            budgets,
        )?;
        assert_eq!(ids(&after_duplicate), after_ids);
        let cutoff = crate::kinekino_process::phase_deadline(host.original, budgets.observation)
            .map_err(|error| CliFailure::not_spawned("transition cutoff", error, &fixture.root))?;
        let mut polling = Vec::new();
        let mut charged = 2usize;
        loop {
            if cutoff.operation_remaining(Instant::now()).is_zero() {
                return Err(reject(
                    fixture,
                    "transition observation",
                    "Original transition budget expired",
                ));
            }
            let phase_host = EndpointInput {
                endpoint: host.endpoint,
                original: cutoff,
            };
            let value = call(
                fixture,
                &phase_host,
                &format!("{prefix}-observe-{}", polling.len()),
                "playback.transition.observe",
                json!({"transition_id":begun.transition_id}),
                budgets,
            )?;
            let sample: TransitionObservation =
                serde_json::from_value(value["data"].clone()).unwrap();
            assert_eq!(sample.transition_id, begun.transition_id);
            assert_eq!(sample.resolution_job_id, begun.resolution_job_id);
            assert!(!sample.engine_replaced);
            let completed = sample.job_state == JobState::Completed;
            let failed = sample.job_state.terminal() && !completed;
            charged = charged
                .checked_add(serde_json::to_vec(&sample).unwrap().len() + 1)
                .filter(|bytes| *bytes <= 128 * 1024)
                .ok_or_else(|| {
                    reject(
                        fixture,
                        "transition observations",
                        "Original sample byte budget exceeded",
                    )
                })?;
            polling.push(value);
            if polling.len() > 128 {
                return Err(reject(
                    fixture,
                    "transition observations",
                    "Bounded sample capacity exceeded",
                ));
            }
            fs::write(
                fixture.root.join(format!("{prefix}-observations.json")),
                serde_json::to_vec_pretty(&polling).unwrap(),
            )
            .unwrap();
            observe(
                fixture,
                &phase_host,
                &format!("{prefix}-old-{}", polling.len()),
                budgets,
                &current,
            )?;
            if failed {
                return Err(reject(
                    fixture,
                    "transition resolve",
                    "Original resolve failed; current remains unchanged",
                ));
            }
            if completed {
                break;
            }
            std::thread::sleep(
                Duration::from_millis(100).min(cutoff.operation_remaining(Instant::now())),
            );
        }
        let job = call(
            fixture,
            host,
            &format!("{prefix}-resolve-job"),
            "jobs.get",
            json!({"job_id":begun.resolution_job_id}),
            budgets,
        )?;
        let job: JobRecord = serde_json::from_value(job["data"].clone()).unwrap();
        assert_eq!(job.state, JobState::Completed);
        assert_eq!(job.input.operation, "episodes.resolve");
        assert_eq!(job.input.parameters, json!({"episode_url":target}));
        let committed = call(
            fixture,
            host,
            &format!("{prefix}-commit"),
            "playback.transition.commit",
            json!({"transition_id":begun.transition_id}),
            budgets,
        )?;
        let replaced: ReplacedEpisode = serde_json::from_value(committed["data"].clone()).unwrap();
        assert_eq!(
            replaced.previous.engine_instance_id,
            current.engine_instance_id
        );
        assert!(
            replaced.previous.engine_exited
                && replaced.previous.ipc_worker_exited
                && replaced.previous.stdout_worker_exited
                && replaced.previous.stderr_worker_exited
        );
        assert!(
            replaced.previous.diagnostics.complete
                && !replaced.previous.diagnostics.truncated
                && !replaced.previous.diagnostics.read_failed
        );
        assert_ne!(
            replaced.current.engine_instance_id,
            current.engine_instance_id
        );
        let actual =
            EpisodeResolutionResults::read(files, &replaced.current.resolution_artifact_id)
                .unwrap();
        assert_eq!(actual.episode_url, target);
        wait_for_progress(
            fixture,
            host.endpoint,
            &replaced.current,
            &prefix,
            0,
            host.original,
        )?;
        let (frame, frame_sha) = capture_frame(
            fixture,
            host.endpoint,
            &replaced.current,
            &prefix,
            host.original,
            budgets.request,
        )?;
        for (label, command) in [
            ("commit-again", "playback.transition.commit"),
            ("consumed", "playback.transition.observe"),
        ] {
            let rejected = fixture.try_call_until(
                &format!("{prefix}-{label}"),
                &named(
                    &format!("{prefix}-{label}"),
                    command,
                    json!({"transition_id":begun.transition_id}),
                ),
                host.endpoint,
                host.original,
                budgets.request,
                4,
            )?;
            assert_eq!(rejected["error"]["code"], "not_found");
        }
        let rejected = fixture.try_call_until(
            &format!("{prefix}-late-instance"),
            &named(
                &format!("{prefix}-late-instance"),
                "playback.pause",
                json!({"engine_instance_id":current.engine_instance_id}),
            ),
            host.endpoint,
            host.original,
            budgets.request,
            5,
        )?;
        assert_eq!(rejected["error"]["code"], "revision_conflict");
        observe(
            fixture,
            host,
            &format!("{prefix}-current"),
            budgets,
            &replaced.current,
        )?;
        receipts.push(json!({"direction":suffix,"submitted":begun,"job":job,
            "replacement":committed["data"],"frame":frame,"frame_sha256":frame_sha}));
        fs::write(
            fixture.root.join(format!("{category}-transitions.json")),
            serde_json::to_vec_pretty(&receipts).unwrap(),
        )
        .unwrap();
        current = replaced.current;
    }
    Ok((current, receipts))
}

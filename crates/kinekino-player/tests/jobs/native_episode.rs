use kinekino_player::application::api::EpisodeClient;
// Actual CLI gate; source interval is fixed separately from the initial native checkpoint.
use super::{
    fixture::{Fixture, request},
    playback_observation::{capture_frame, observe_until, wait_for_progress},
    playback_owner::NativeOwner,
};
#[path = "native_episode/options.rs"]
mod options;
#[path = "native_episode/pointer.rs"]
mod pointer;
#[path = "native_episode/transition.rs"]
mod transition;
use kinekino_player::{
    application::{
        api::{ApplicationClient, RequestContext},
        playback_session::{
            ClosedEpisodePlayback, EpisodePlaybackObservation, OpenEpisodePlayback,
            OpenedEpisodePlayback,
        },
    },
    domain::api_values::DecimalU64,
    infrastructure::local_host::{EndpointClient, TransportOptions},
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

#[test]
#[ignore = "Explicit actual resolve and owned trusted mpv; requires central native execution slot"]
fn actual_resolve_and_owned_native_lifetime() {
    kinekino_platform::install();
    let evidence =
        PathBuf::from(std::env::var_os("JOB_EVIDENCE_DIR").expect("Explicit fresh evidence root"));
    assert!(evidence.is_absolute());
    super::resolve_cli::actual_cli_resolve_preserves_observed_player_script_response();
    let resolution_root = evidence.join("resolve-watch-cli");
    let catalog = resolution_root.join("catalog");
    let job: Value =
        serde_json::from_slice(&fs::read(resolution_root.join("observation.json")).unwrap())
            .unwrap();
    assert_eq!(job["state"], "completed");
    let artifact = job["output"]["receipt"]["resolution_artifact"]["artifact_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let executable =
        PathBuf::from(std::env::var_os("NATIVE_MPV_EXE").expect("Explicit trusted native engine"));
    let sha256 = std::env::var("NATIVE_MPV_SHA256").expect("Explicit native engine SHA");
    let fixture = Fixture::named("native-episode-cli");
    let host = NativeOwner::start(&fixture, &catalog, &executable, &sha256);
    let client = EndpointClient::new(&host.endpoint, TransportOptions::default()).unwrap();
    let opened = fixture.call(
        "native-open",
        &request(
            "playback.open",
            json!({"resolution_artifact_id":artifact,"paused":false,"volume_percent":0.}),
        ),
        Some(&host.endpoint),
        None,
        0,
    );
    let opened: OpenedEpisodePlayback = serde_json::from_value(opened["data"].clone()).unwrap();
    fs::write(
        fixture.root.join("open.json"),
        serde_json::to_vec_pretty(&opened).unwrap(),
    )
    .unwrap();
    assert_ne!(opened.engine_instance_id.0, 0);
    assert_eq!(opened.resolution_artifact_id, artifact);
    let repeated = fixture.call(
        "open-existing-refused",
        &request(
            "playback.open",
            json!({"resolution_artifact_id":artifact,"paused":false,"volume_percent":0.}),
        ),
        Some(&host.endpoint),
        None,
        5,
    );
    assert_eq!(repeated["error"]["code"], "revision_conflict");
    wait_for_progress(&fixture, &client, &opened, "native", 60_000_000);
    if std::env::var_os("NATIVE_POINTER_EVIDENCE").is_some() {
        pointer::exercise(&fixture, &executable);
    }
    let (frame_path, frame_sha) = capture_frame(&fixture, &host.endpoint, &opened, "native");
    let overwritten = fixture.call(
        "native-capture-no-overwrite",
        &request(
            "production.capture_frame",
            json!({"engine_instance_id":opened.engine_instance_id,"output_path":frame_path}),
        ),
        Some(&host.endpoint),
        None,
        5,
    );
    assert_eq!(overwritten["error"]["code"], "revision_conflict");
    let (_, after_sha) = kinekino_player::infrastructure::content_digest::sha256(
        &mut fs::File::open(&frame_path).unwrap(),
        64 * 1024 * 1024,
    )
    .unwrap();
    assert_eq!(after_sha, frame_sha);
    let (list_sha, next_descriptor, previous_descriptor) = transition_targets(&catalog, &opened);
    let opened = transition::exercise(
        &fixture,
        &client,
        &host.endpoint,
        &list_sha,
        &opened,
        transition::ActualTransitionLeg {
            direction:
                kinekino_player::application::playback_session::transition::EpisodeDirection::Next,
            expected_target_descriptor_sha: &next_descriptor,
        },
    );
    let opened = transition::exercise(
        &fixture,
        &client,
        &host.endpoint,
        &list_sha,
        &opened,
        transition::ActualTransitionLeg {
            direction: kinekino_player::application::playback_session::transition::EpisodeDirection::Previous,
            expected_target_descriptor_sha: &previous_descriptor,
        },
    );
    let artifact = opened.resolution_artifact_id.clone();
    options::exercise(&fixture, &client, &host.endpoint, opened.engine_instance_id);
    let stale = opened.engine_instance_id.0.checked_add(1).unwrap();
    let rejected = fixture.call(
        "stale-pause",
        &request(
            "playback.pause",
            json!({"engine_instance_id":stale.to_string()}),
        ),
        Some(&host.endpoint),
        None,
        5,
    );
    assert_eq!(rejected["error"]["code"], "revision_conflict");
    let receipt = fixture.call(
        "native-pause",
        &request(
            "playback.pause",
            json!({"engine_instance_id":opened.engine_instance_id}),
        ),
        Some(&host.endpoint),
        None,
        0,
    );
    assert_eq!(
        receipt["data"]["engine_instance_id"],
        serde_json::json!(opened.engine_instance_id)
    );
    assert_eq!(receipt["data"]["completion"], "unobserved");
    let pause_deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let observed: EpisodePlaybackObservation = client
            .observe_episode_playback(RequestContext::default())
            .unwrap();
        if observed.playback.paused == Some(true) {
            break;
        }
        assert!(
            Instant::now() < pause_deadline,
            "Pause application not observed"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
    fixture.call(
        "native-seek",
        &request(
            "playback.seek",
            json!({"engine_instance_id":opened.engine_instance_id,"position_us":"10000000"}),
        ),
        Some(&host.endpoint),
        None,
        0,
    );
    observe_until(
        &fixture,
        &client,
        "seek",
        opened.engine_instance_id.0,
        |sample| {
            sample.playback.paused == Some(true)
                && sample
                    .playback
                    .position_us
                    .is_some_and(|position| position.0.abs_diff(10_000_000) <= 1_000_000)
        },
    );
    let mute_receipt = fixture.call(
        "native-mute",
        &request(
            "playback.set_muted",
            json!({"engine_instance_id":opened.engine_instance_id,"muted":true}),
        ),
        Some(&host.endpoint),
        None,
        0,
    );
    assert_eq!(mute_receipt["data"]["acceptance"], "accepted");
    assert_eq!(mute_receipt["data"]["completion"], "unobserved");
    observe_until(
        &fixture,
        &client,
        "mute",
        opened.engine_instance_id.0,
        |sample| sample.playback.muted == Some(true),
    );
    let invalid_mute = fixture.call(
        "native-mute-invalid",
        &request(
            "playback.set_muted",
            json!({"engine_instance_id":opened.engine_instance_id,"muted":"false"}),
        ),
        Some(&host.endpoint),
        None,
        2,
    );
    assert_eq!(invalid_mute["error"]["code"], "invalid_argument");
    let stale_mute = fixture.call(
        "native-mute-stale",
        &request(
            "playback.set_muted",
            json!({"engine_instance_id":stale.to_string(),"muted":false}),
        ),
        Some(&host.endpoint),
        None,
        5,
    );
    assert_eq!(stale_mute["error"]["code"], "revision_conflict");
    fixture.call(
        "native-volume",
        &request(
            "playback.set_volume",
            json!({"engine_instance_id":opened.engine_instance_id,"volume_percent":13.25}),
        ),
        Some(&host.endpoint),
        None,
        0,
    );
    observe_until(
        &fixture,
        &client,
        "volume",
        opened.engine_instance_id.0,
        |sample| {
            sample.playback.muted == Some(true)
                && sample
                    .playback
                    .volume_percent
                    .is_some_and(|volume| (volume - 13.25).abs() < 0.1)
        },
    );
    fixture.call(
        "native-volume-restore",
        &request(
            "playback.set_volume",
            json!({"engine_instance_id":opened.engine_instance_id,"volume_percent":0.}),
        ),
        Some(&host.endpoint),
        None,
        0,
    );
    observe_until(
        &fixture,
        &client,
        "volume-restore",
        opened.engine_instance_id.0,
        |sample| sample.playback.volume_percent == Some(0.) && sample.playback.muted == Some(true),
    );
    fixture.call(
        "native-unmute",
        &request(
            "playback.set_muted",
            json!({"engine_instance_id":opened.engine_instance_id,"muted":false}),
        ),
        Some(&host.endpoint),
        None,
        0,
    );
    let before_play = observe_until(
        &fixture,
        &client,
        "unmute-at-zero-volume",
        opened.engine_instance_id.0,
        |sample| sample.playback.muted == Some(false) && sample.playback.volume_percent == Some(0.),
    );
    fixture.call(
        "native-play",
        &request(
            "playback.play",
            json!({"engine_instance_id":opened.engine_instance_id}),
        ),
        Some(&host.endpoint),
        None,
        0,
    );
    let before_play = before_play.playback.position_us.unwrap().0;
    observe_until(
        &fixture,
        &client,
        "play",
        opened.engine_instance_id.0,
        |sample| {
            sample.playback.paused == Some(false)
                && sample
                    .playback
                    .position_us
                    .is_some_and(|position| position.0 > before_play + 500_000)
        },
    );
    fixture.call(
        "native-pause-before-close",
        &request(
            "playback.pause",
            json!({"engine_instance_id":opened.engine_instance_id}),
        ),
        Some(&host.endpoint),
        None,
        0,
    );
    observe_until(
        &fixture,
        &client,
        "pause-before-close",
        opened.engine_instance_id.0,
        |sample| sample.playback.paused == Some(true),
    );
    let closed = fixture.call(
        "native-close",
        &request(
            "playback.close",
            json!({"engine_instance_id":opened.engine_instance_id}),
        ),
        Some(&host.endpoint),
        None,
        0,
    );
    let closed: ClosedEpisodePlayback = serde_json::from_value(closed["data"].clone()).unwrap();
    assert!(
        closed.engine_exited
            && closed.ipc_worker_exited
            && closed.stderr_worker_exited
            && closed.stdout_worker_exited
    );
    assert!(
        closed.diagnostics.complete
            && !closed.diagnostics.truncated
            && !closed.diagnostics.read_failed
    );
    let raw = fixture.call_catalog("native-stderr-raw", &request("catalog.read_raw", json!({"artifact_id":closed.diagnostics.artifact.artifact_id,"offset":"0","limit":65536})), &catalog, 0);
    assert_eq!(
        raw["data"]["capture"]["capture_kind"],
        "native_engine_diagnostic"
    );
    assert!(raw["data"]["capture"]["status"].is_null());
    fs::write(
        fixture.root.join("closed.json"),
        serde_json::to_vec_pretty(&closed).unwrap(),
    )
    .unwrap();
    fixture.call(
        "after-close",
        &request("playback.engine", json!({})),
        Some(&host.endpoint),
        None,
        4,
    );
    let reopened = client
        .open_episode_playback(
            RequestContext::default(),
            OpenEpisodePlayback {
                resolution_artifact_id: artifact,
                paused: true,
                volume_percent: 0.,
            },
        )
        .unwrap();
    assert_ne!(reopened.engine_instance_id.0, opened.engine_instance_id.0);
    fixture.call(
        "old-instance-cannot-close-new",
        &request(
            "playback.close",
            json!({"engine_instance_id":DecimalU64(opened.engine_instance_id.0)}),
        ),
        Some(&host.endpoint),
        None,
        5,
    );
    assert_eq!(
        client
            .observe_episode_playback(RequestContext::default())
            .unwrap()
            .engine_instance_id
            .0,
        reopened.engine_instance_id.0
    );
    host.stop(true);
    {
        use kinekino_player::{
            application::{
                api::{ErrorCode, PlayerSdk},
                playback_session::EpisodePlaybackSessions,
            },
            infrastructure::{catalog::CatalogFiles, mpv::NativePlaybackHost},
        };
        use std::sync::Arc;
        let files = Arc::new(CatalogFiles::open(&catalog).unwrap());
        let ended = NativePlaybackHost::start(&executable, &sha256, None, files.clone()).unwrap();
        let sdk = PlayerSdk::default().with_episode_playback(EpisodePlaybackSessions::new(
            files,
            Arc::new(ended.engine()),
        ));
        assert!(ended.shutdown().unwrap().is_none());
        assert_eq!(
            sdk.open_episode_playback(
                RequestContext::default(),
                OpenEpisodePlayback {
                    resolution_artifact_id: opened.resolution_artifact_id,
                    paused: true,
                    volume_percent: 0.
                }
            )
            .unwrap_err()
            .code,
            ErrorCode::Cancelled
        );
    }
}

fn transition_targets(catalog: &Path, opened: &OpenedEpisodePlayback) -> (String, String, String) {
    use kinekino_player::{
        application::{
            catalog::{CatalogArtifacts, CatalogRawSource, CatalogResults},
            episode_queue::EpisodeQueue,
            episode_resolve::EpisodeResolutionResults,
        },
        domain::episodes::{Episode, EpisodePage},
        infrastructure::catalog::CatalogFiles,
    };
    use sha2::{Digest, Sha256};
    let source = PathBuf::from(std::env::var_os("EPISODE_SEED_ROOT").unwrap());
    let sha = std::env::var("EPISODE_SEED_RESULT_SHA256").unwrap();
    let source_files = CatalogFiles::open(&source).unwrap();
    let files = CatalogFiles::open(catalog).unwrap();
    let (seed, _) = CatalogResults::read(&source_files, &sha).unwrap();
    assert_eq!(seed.operation, "episodes.list");
    let raw = source_files
        .load_raw(&seed.result.raw_artifact.artifact_id)
        .unwrap();
    let copied = files.save(&raw.body, &raw.capture).unwrap();
    assert_eq!(copied, seed.result.raw_artifact);
    let published = CatalogResults::publish(&files, &seed).unwrap();
    assert_eq!(published.sha256, sha);
    let current = EpisodeResolutionResults::read(&files, &opened.resolution_artifact_id).unwrap();
    let mut queue = EpisodeQueue::default();
    let series = seed.parameters["series_url"].as_str().unwrap().to_owned();
    assert!(
        queue.observe(
            EpisodePage {
                page: current.episode_url.clone(),
                series,
                title: seed.result.title,
                entries: seed
                    .result
                    .entries
                    .into_iter()
                    .map(|entry| Episode {
                        title: entry.title,
                        url: entry.url
                    })
                    .collect(),
            },
            &current.episode_url
        )
    );
    let next = queue
        .adjacent(1)
        .expect("Actual fixture requires a distinct next episode");
    assert_ne!(next, current.episode_url);
    let descriptor = json!({"operation":"episodes.resolve","parameters":{"episode_url":next},"source_kind":"request_descriptor"});
    let next_sha = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&descriptor).unwrap())
    );
    (sha, next_sha, current.source_descriptor_sha256)
}

use super::{Fixture, observe_until, request};
use kinekino_player::application::api::EpisodeClient;
use kinekino_player::{
    application::{
        api::RequestContext,
        playback_session::{EpisodePlaybackObservation, SelectEpisodeTrack},
    },
    domain::{
        api_values::DecimalU64,
        playback_options::{ObservedTrackChoice, TrackKind, TrackSelection},
    },
    infrastructure::local_host::EndpointClient,
};
use serde_json::{Value, json};
use std::fs;

fn revision(observation: &EpisodePlaybackObservation) -> DecimalU64 {
    observation
        .options
        .as_ref()
        .unwrap()
        .tracks
        .as_ref()
        .unwrap()
        .revision
}
fn selected(observation: &EpisodePlaybackObservation, kind: TrackKind) -> u64 {
    let options = observation.options.as_ref().unwrap();
    let choice = match kind {
        TrackKind::Audio => options.audio_selection,
        TrackKind::Video => options.video_selection,
        TrackKind::Subtitle => options.subtitle_selection,
    };
    let Some(ObservedTrackChoice::Selected { track_id }) = choice else {
        panic!("No actual effective {kind:?} track was observed");
    };
    track_id.get()
}
fn choice(
    observation: &EpisodePlaybackObservation,
    kind: TrackKind,
) -> Option<ObservedTrackChoice> {
    let options = observation.options.as_ref()?;
    match kind {
        TrackKind::Audio => options.audio_selection,
        TrackKind::Video => options.video_selection,
        TrackKind::Subtitle => options.subtitle_selection,
    }
}
fn send(
    fixture: &Fixture,
    endpoint: &str,
    name: &str,
    input: SelectEpisodeTrack,
    exit: i32,
) -> Value {
    fixture.call(
        name,
        &request(
            "playback.select_track",
            serde_json::to_value(input).unwrap(),
        ),
        Some(endpoint),
        None,
        exit,
    )
}
fn restore_kind(
    fixture: &Fixture,
    client: &EndpointClient,
    endpoint: &str,
    instance: DecimalU64,
    kind: TrackKind,
    id: u64,
    name: &str,
) {
    let before = client
        .observe_episode_playback(RequestContext::default())
        .unwrap();
    let original_revision = revision(&before);
    let receipt = send(
        fixture,
        endpoint,
        &format!("{name}-disable"),
        SelectEpisodeTrack {
            engine_instance_id: instance,
            expected_tracks_revision: original_revision,
            track_kind: kind,
            selection: TrackSelection::Disabled,
        },
        0,
    );
    assert_eq!(receipt["data"]["completion"], "unobserved");
    let disabled = observe_until(
        fixture,
        client,
        &format!("{name}-disabled"),
        instance.0,
        |sample| {
            choice(sample, kind) == Some(ObservedTrackChoice::Disabled)
                && revision(sample) != original_revision
        },
    );
    let stale = send(
        fixture,
        endpoint,
        &format!("{name}-stale-revision"),
        SelectEpisodeTrack {
            engine_instance_id: instance,
            expected_tracks_revision: original_revision,
            track_kind: kind,
            selection: TrackSelection::Disabled,
        },
        5,
    );
    assert_eq!(stale["error"]["code"], "revision_conflict");
    let disabled_revision = revision(&disabled);
    let receipt = send(
        fixture,
        endpoint,
        &format!("{name}-restore"),
        SelectEpisodeTrack {
            engine_instance_id: instance,
            expected_tracks_revision: disabled_revision,
            track_kind: kind,
            selection: TrackSelection::Id {
                track_id: kinekino_player::domain::playback_options::TrackId::new(id).unwrap(),
            },
        },
        0,
    );
    assert_eq!(
        receipt["data"]["validated_tracks_revision"],
        json!(disabled_revision)
    );
    observe_until(
        fixture,
        client,
        &format!("{name}-restored"),
        instance.0,
        |sample| {
            choice(sample, kind)
                == Some(ObservedTrackChoice::Selected {
                    track_id: kinekino_player::domain::playback_options::TrackId::new(id).unwrap(),
                })
                && revision(sample) != disabled_revision
                && (kind != TrackKind::Video || sample.playback.video_size.is_some())
        },
    );
}

pub(super) fn exercise(
    fixture: &Fixture,
    client: &EndpointClient,
    endpoint: &str,
    instance: DecimalU64,
) {
    fixture.call(
        "options-rate-schema",
        &request("schema", json!({"command":"playback.set_rate"})),
        Some(endpoint),
        None,
        0,
    );
    fixture.call(
        "options-track-schema",
        &request("schema", json!({"command":"playback.select_track"})),
        Some(endpoint),
        None,
        0,
    );
    let rate = fixture.call(
        "options-rate-125",
        &request(
            "playback.set_rate",
            json!({"engine_instance_id":instance,"rate":1.25}),
        ),
        Some(endpoint),
        None,
        0,
    );
    assert_eq!(rate["data"]["completion"], "unobserved");
    let at_rate = observe_until(fixture, client, "rate-125", instance.0, |sample| {
        sample
            .options
            .as_ref()
            .is_some_and(|options| options.rate == Some(1.25))
    });
    let start = at_rate.playback.position_us.unwrap().0;
    observe_until(fixture, client, "rate-125-progress", instance.0, |sample| {
        sample
            .options
            .as_ref()
            .is_some_and(|options| options.rate == Some(1.25))
            && sample
                .playback
                .position_us
                .is_some_and(|time| time.0 > start + 500_000)
    });
    fixture.call(
        "options-rate-restore",
        &request(
            "playback.set_rate",
            json!({"engine_instance_id":instance,"rate":1.0}),
        ),
        Some(endpoint),
        None,
        0,
    );
    observe_until(fixture, client, "rate-restored", instance.0, |sample| {
        sample
            .options
            .as_ref()
            .is_some_and(|options| options.rate == Some(1.0))
    });
    fixture.call(
        "options-rate-invalid",
        &request(
            "playback.set_rate",
            json!({"engine_instance_id":instance,"rate":0.0}),
        ),
        Some(endpoint),
        None,
        2,
    );
    fixture.call(
        "options-rate-wrong-instance",
        &request(
            "playback.set_rate",
            json!({"engine_instance_id":DecimalU64(instance.0+1),"rate":1.0}),
        ),
        Some(endpoint),
        None,
        5,
    );
    fixture.call(
        "options-pause-for-tracks",
        &request("playback.pause", json!({"engine_instance_id":instance})),
        Some(endpoint),
        None,
        0,
    );
    let initial = observe_until(
        fixture,
        client,
        "options-track-inventory",
        instance.0,
        |sample| {
            sample.playback.paused == Some(true)
                && sample.options.as_ref().is_some_and(|options| {
                    options.tracks.is_some()
                        && matches!(
                            options.audio_selection,
                            Some(ObservedTrackChoice::Selected { .. })
                        )
                        && matches!(
                            options.video_selection,
                            Some(ObservedTrackChoice::Selected { .. })
                        )
                })
        },
    );
    fs::write(
        fixture.root.join("options-initial.json"),
        serde_json::to_vec_pretty(&initial).unwrap(),
    )
    .unwrap();
    let audio_id = selected(&initial, TrackKind::Audio);
    let video_id = selected(&initial, TrackKind::Video);
    let tracks = initial.options.as_ref().unwrap().tracks.as_ref().unwrap();
    let unknown = (1..=65)
        .find(|candidate| {
            !tracks
                .entries
                .iter()
                .any(|track| track.kind == TrackKind::Audio && track.id.get() == *candidate)
        })
        .unwrap();
    let unknown = send(
        fixture,
        endpoint,
        "options-nonexistent-track",
        SelectEpisodeTrack {
            engine_instance_id: instance,
            expected_tracks_revision: revision(&initial),
            track_kind: TrackKind::Audio,
            selection: TrackSelection::Id {
                track_id: kinekino_player::domain::playback_options::TrackId::new(unknown).unwrap(),
            },
        },
        4,
    );
    assert_eq!(unknown["error"]["code"], "not_found");
    if !tracks
        .entries
        .iter()
        .any(|track| track.kind == TrackKind::Subtitle)
    {
        let wrong_kind = send(
            fixture,
            endpoint,
            "options-audio-id-is-not-subtitle",
            SelectEpisodeTrack {
                engine_instance_id: instance,
                expected_tracks_revision: revision(&initial),
                track_kind: TrackKind::Subtitle,
                selection: TrackSelection::Id {
                    track_id: kinekino_player::domain::playback_options::TrackId::new(audio_id)
                        .unwrap(),
                },
            },
            4,
        );
        assert_eq!(wrong_kind["error"]["code"], "not_found");
    }
    restore_kind(
        fixture,
        client,
        endpoint,
        instance,
        TrackKind::Audio,
        audio_id,
        "audio",
    );
    restore_kind(
        fixture,
        client,
        endpoint,
        instance,
        TrackKind::Video,
        video_id,
        "video",
    );
    let path = fixture.root.join("native-restored-frame.png");
    let frame = fixture.call(
        "options-restored-frame",
        &request(
            "production.capture_frame",
            json!({"engine_instance_id":instance,"output_path":path}),
        ),
        Some(endpoint),
        None,
        0,
    );
    let (_, sha) = kinekino_player::infrastructure::content_digest::sha256(
        &mut fs::File::open(path).unwrap(),
        64 * 1024 * 1024,
    )
    .unwrap();
    assert_eq!(frame["data"]["output_sha256"], sha);
    fixture.call(
        "options-resume",
        &request("playback.play", json!({"engine_instance_id":instance})),
        Some(endpoint),
        None,
        0,
    );
    observe_until(fixture, client, "options-resumed", instance.0, |sample| {
        sample.playback.paused == Some(false)
    });
}

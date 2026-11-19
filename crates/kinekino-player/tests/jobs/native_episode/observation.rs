//! Windows native-episode observations; the `try_` variants in `playback_observation` are portable.
use super::super::fixture::{Fixture, request};
use super::super::playback_observation::{progress_step, verify_capture};
use kinekino_player::application::api::EpisodeClient;
use kinekino_player::application::{
    api::RequestContext,
    playback_session::{EpisodePlaybackObservation, OpenedEpisodePlayback},
};
use kinekino_player::infrastructure::local_host::EndpointClient;
use serde_json::json;
use std::{
    fs,
    path::PathBuf,
    time::{Duration, Instant},
};
pub(super) fn observe_until(
    fixture: &Fixture,
    client: &EndpointClient,
    name: &str,
    instance: u64,
    predicate: impl Fn(&EpisodePlaybackObservation) -> bool,
) -> EpisodePlaybackObservation {
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut samples = Vec::new();
    loop {
        let sample = client
            .observe_episode_playback(RequestContext::default())
            .unwrap();
        let done = predicate(&sample);
        let failed = sample.playback.has_error;
        assert_eq!(sample.engine_instance_id.0, instance);
        samples.push(sample.clone());
        fs::write(
            fixture.root.join(format!("logs/{name}-observations.json")),
            serde_json::to_vec_pretty(&samples).unwrap(),
        )
        .unwrap();
        assert!(
            !failed,
            "Native engine failed during {name}; inspect preserved observations"
        );
        if done {
            return sample;
        }
        assert!(
            Instant::now() < deadline,
            "Native application not observed for {name}"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

pub(super) fn wait_for_progress(
    fixture: &Fixture,
    client: &EndpointClient,
    opened: &OpenedEpisodePlayback,
    name: &str,
    minimum_duration_us: u64,
) {
    let mut samples = Vec::new();
    let mut first_position = None;
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let observation = client
            .observe_episode_playback(RequestContext::default())
            .unwrap();
        let (progressed, failed) = progress_step(
            &observation,
            opened,
            minimum_duration_us,
            &mut first_position,
        );
        samples.push(observation);
        fs::write(
            fixture.root.join(format!("{name}-observations.json")),
            serde_json::to_vec_pretty(&samples).unwrap(),
        )
        .unwrap();
        assert!(
            !failed,
            "Actual native load/decoder error; inspect preserved observations"
        );
        if progressed {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "Actual native progress was not observed"
        );
        std::thread::sleep(Duration::from_millis(250));
    }
}

pub(super) fn capture_frame(
    fixture: &Fixture,
    endpoint: &str,
    opened: &OpenedEpisodePlayback,
    name: &str,
) -> (PathBuf, String) {
    let frame_path = fixture.root.join(format!("{name}-frame.png"));
    let captured = fixture.call(
        &format!("{name}-capture"),
        &request(
            "production.capture_frame",
            json!({"engine_instance_id":opened.engine_instance_id,"output_path":frame_path}),
        ),
        Some(endpoint),
        None,
        0,
    );
    verify_capture(fixture, opened, name, captured)
}

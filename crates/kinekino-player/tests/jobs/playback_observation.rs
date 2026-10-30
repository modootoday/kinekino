//! Observations and capture proofs shared by actual ignored CLI gates.
use super::fixture::{Fixture, request};
use kinekino_player::application::api::EpisodeClient;
use kinekino_player::{
    application::{
        api::RequestContext,
        playback_session::{EpisodePlaybackObservation, OpenedEpisodePlayback},
    },
    infrastructure::local_host::EndpointClient,
};
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

fn progress_step(
    observation: &EpisodePlaybackObservation,
    opened: &OpenedEpisodePlayback,
    minimum_duration_us: u64,
    first_position: &mut Option<u64>,
) -> (bool, bool) {
    assert_eq!(observation.engine_instance_id, opened.engine_instance_id);
    assert_eq!(
        observation.resolution_artifact_id,
        opened.resolution_artifact_id
    );
    let state = &observation.playback;
    let position = state.position_us.map(|value| value.0);
    let ready = state.connected
        && state.idle == Some(false)
        && state.video_size.is_some()
        && state.audio_channels.is_some()
        && state
            .duration_us
            .is_some_and(|value| value.0 > minimum_duration_us);
    if ready && first_position.is_none() {
        *first_position = position;
    }
    (
        ready
            && position
                .zip(*first_position)
                .is_some_and(|(current, first)| current > first + 1_000_000),
        state.has_error,
    )
}
pub(super) fn try_wait_for_progress(
    fixture: &Fixture,
    endpoint: &str,
    opened: &OpenedEpisodePlayback,
    name: &str,
    minimum_duration_us: u64,
    original: kinekino_process::domain::deadline::Deadline,
) -> Result<(), Box<super::kinekino_process::CliFailure>> {
    use super::kinekino_process::{CliFailure, phase_deadline, sdk_failure, transport_at};
    let failed = |stage, error| CliFailure::not_spawned(stage, error, &fixture.root);
    let deadline = phase_deadline(original, Duration::from_secs(30))
        .map_err(|error| failed("progress budget", error))?;
    let mut samples = Vec::new();
    let mut first = None;
    let mut charged = 2usize;
    loop {
        let client = transport_at(endpoint, deadline)
            .map_err(|error| failed("progress transport", error))?;
        let observation = client
            .observe_episode_playback(RequestContext::default())
            .map_err(|error| sdk_failure("progress observation", error, &fixture.root))?;
        let encoded = serde_json::to_vec(&observation)
            .map_err(|error| failed("progress encode", std::io::Error::other(error)))?;
        charged = charged.checked_add(encoded.len() + 1).ok_or_else(|| {
            failed(
                "progress bytes",
                std::io::Error::other("Progress byte counter overflow"),
            )
        })?;
        if charged > 8 * 1024 * 1024 || samples.len() >= 256 {
            return Err(failed(
                "progress evidence bound",
                std::io::Error::other("Progress evidence limit exceeded"),
            ));
        }
        let (progressed, has_error) =
            progress_step(&observation, opened, minimum_duration_us, &mut first);
        samples.push(observation);
        fs::write(
            fixture.root.join(format!("{name}-observations.json")),
            serde_json::to_vec(&samples)
                .map_err(|error| failed("progress encode", std::io::Error::other(error)))?,
        )
        .map_err(|error| failed("progress evidence", error))?;
        if has_error {
            return Err(failed(
                "progress native error",
                std::io::Error::other("Actual native load/decoder error; observations preserved"),
            ));
        }
        if deadline.operation_remaining(Instant::now()).is_zero() {
            return Err(failed(
                "progress deadline",
                std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Original native progress cutoff exceeded",
                ),
            ));
        }
        if progressed {
            return Ok(());
        }
        std::thread::sleep(
            Duration::from_millis(250).min(deadline.operation_remaining(Instant::now())),
        );
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
pub(super) fn try_capture_frame(
    fixture: &Fixture,
    endpoint: &str,
    opened: &OpenedEpisodePlayback,
    name: &str,
    original: kinekino_process::domain::deadline::Deadline,
    cap: Duration,
) -> Result<(PathBuf, String), Box<super::kinekino_process::CliFailure>> {
    let original = super::kinekino_process::phase_deadline(original, cap).map_err(|error| {
        super::kinekino_process::CliFailure::not_spawned(
            "capture phase budget",
            error,
            &fixture.root,
        )
    })?;
    let frame_path = fixture.root.join(format!("{name}-frame.png"));
    let captured = fixture.try_call_until(
        &format!("{name}-capture"),
        &request(
            "production.capture_frame",
            json!({"engine_instance_id":opened.engine_instance_id,"output_path":frame_path}),
        ),
        endpoint,
        original,
        cap,
        0,
    )?;
    let verified = verify_capture(fixture, opened, name, captured);
    if original.operation_remaining(Instant::now()).is_zero() {
        return Err(super::kinekino_process::CliFailure::not_spawned(
            "capture verification deadline",
            std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Original operation deadline elapsed",
            ),
            &fixture.root,
        ));
    }
    Ok(verified)
}
fn verify_capture(
    fixture: &Fixture,
    opened: &OpenedEpisodePlayback,
    name: &str,
    captured: serde_json::Value,
) -> (PathBuf, String) {
    let frame_path = fixture.root.join(format!("{name}-frame.png"));
    assert_eq!(
        captured["data"]["engine_instance_id"],
        json!(opened.engine_instance_id)
    );
    assert_eq!(
        captured["data"]["resolution_artifact_id"],
        opened.resolution_artifact_id
    );
    assert_eq!(captured["data"]["frame_time_exact"], false);
    assert_eq!(captured["data"]["color_parity_verified"], false);
    let (frame_bytes, frame_sha) = kinekino_player::infrastructure::content_digest::sha256(
        &mut fs::File::open(&frame_path).unwrap(),
        64 * 1024 * 1024,
    )
    .unwrap();
    assert_eq!(captured["data"]["output_sha256"], frame_sha);
    assert_eq!(captured["data"]["output_bytes"], frame_bytes.to_string());
    let mut decoded = png::Decoder::new(std::io::BufReader::new(
        fs::File::open(&frame_path).unwrap(),
    ))
    .read_info()
    .unwrap();
    assert_eq!(
        u64::from(decoded.info().width),
        captured["data"]["width"].as_u64().unwrap()
    );
    assert_eq!(
        u64::from(decoded.info().height),
        captured["data"]["height"].as_u64().unwrap()
    );
    let mut pixels = vec![0; decoded.output_buffer_size().unwrap()];
    let frame = decoded.next_frame(&mut pixels).unwrap();
    assert!(frame.width > 0 && frame.height > 0 && frame.buffer_size() > 0);
    decoded.finish().unwrap();
    (frame_path, frame_sha)
}

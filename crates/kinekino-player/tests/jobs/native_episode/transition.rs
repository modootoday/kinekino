//! Actual adjacent transitions through a combined owned host.
use super::*;
use kinekino_player::application::api::EpisodeClient;
use kinekino_player::application::{api::Request, playback_session::transition::EpisodeDirection};

pub struct ActualTransitionLeg<'a> {
    pub direction: EpisodeDirection,
    pub expected_target_descriptor_sha: &'a str,
}

pub fn exercise(
    fixture: &Fixture,
    client: &EndpointClient,
    endpoint: &str,
    list_sha: &str,
    current: &OpenedEpisodePlayback,
    leg: ActualTransitionLeg<'_>,
) -> OpenedEpisodePlayback {
    let (command, prefix) = match leg.direction {
        EpisodeDirection::Next => ("playback.next", "next"),
        EpisodeDirection::Previous => ("playback.previous", "previous"),
    };
    let call = |label: &str, input: &Request, exit: i32| {
        fixture.call(
            &format!("{prefix}-{label}"),
            input,
            Some(endpoint),
            None,
            exit,
        )
    };
    let begun = call(
        "begin",
        &request(
            command,
            json!({
                "expected_engine_instance_id":current.engine_instance_id,
                "catalog_result_sha256":list_sha,
                "resolution_submission_key":format!("actual-native-{prefix}-resolve"),
                "paused":false,"volume_percent":0.,"execution_timeout_ms":"30000"
            }),
        ),
        0,
    );
    assert_eq!(begun["data"]["engine_replaced"], false);
    let handle = begun["data"]["transition_id"].as_str().unwrap().to_owned();
    let before_duplicate = call(
        "jobs-before-duplicate",
        &request("jobs.list", json!({"cursor":null,"limit":100})),
        0,
    );
    let duplicate = call(
        "duplicate-begin-rejected",
        &request(
            command,
            json!({
                "expected_engine_instance_id":current.engine_instance_id,
                "catalog_result_sha256":list_sha,
                "resolution_submission_key":format!("actual-native-{prefix}-duplicate"),
                "paused":false,"volume_percent":0.,"execution_timeout_ms":"30000"
            }),
        ),
        5,
    );
    assert_eq!(duplicate["error"]["code"], "revision_conflict");
    let after_duplicate = call(
        "jobs-after-duplicate",
        &request("jobs.list", json!({"cursor":null,"limit":100})),
        0,
    );
    let job_ids = |wire: &Value| {
        assert!(
            wire["data"]["next_cursor"].is_null(),
            "Actual fixture job list must be complete"
        );
        let mut ids: Vec<_> = wire["data"]["jobs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|job| job["job_id"].as_str().unwrap().to_owned())
            .collect();
        ids.sort();
        ids
    };
    assert_eq!(job_ids(&before_duplicate), job_ids(&after_duplicate));
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut samples = Vec::new();
    loop {
        let response = client.execute(request(
            "playback.transition.observe",
            json!({"transition_id":handle}),
        ));
        assert!(response.ok);
        let data = response.data.unwrap();
        assert_eq!(data["engine_replaced"], false);
        let state = data["job_state"].as_str().unwrap().to_owned();
        samples.push(data);
        fs::write(
            fixture
                .root
                .join(format!("logs/{prefix}-job-observations.json")),
            serde_json::to_vec_pretty(&samples).unwrap(),
        )
        .unwrap();
        assert!(
            !matches!(
                state.as_str(),
                "failed" | "cancelled" | "interrupted" | "needs_input"
            ),
            "{prefix} resolve failed; original observations preserved"
        );
        let old = client
            .observe_episode_playback(RequestContext::default())
            .unwrap();
        assert_eq!(old.engine_instance_id, current.engine_instance_id);
        assert_eq!(old.resolution_artifact_id, current.resolution_artifact_id);
        assert!(!old.playback.has_error);
        if state == "completed" {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "{prefix} resolution did not complete"
        );
        std::thread::sleep(Duration::from_millis(250));
    }
    let replaced = call(
        "commit",
        &request(
            "playback.transition.commit",
            json!({"transition_id":handle}),
        ),
        0,
    );
    let previous: ClosedEpisodePlayback =
        serde_json::from_value(replaced["data"]["previous"].clone()).unwrap();
    assert_eq!(previous.engine_instance_id, current.engine_instance_id);
    assert!(
        previous.engine_exited
            && previous.ipc_worker_exited
            && previous.stdout_worker_exited
            && previous.stderr_worker_exited
    );
    let next: OpenedEpisodePlayback =
        serde_json::from_value(replaced["data"]["current"].clone()).unwrap();
    assert_ne!(next.engine_instance_id, current.engine_instance_id);
    assert_ne!(next.resolution_artifact_id, current.resolution_artifact_id);
    let inspected = call(
        "inspect-resolution",
        &request(
            "episodes.inspect_resolution",
            json!({"resolution_artifact_id":next.resolution_artifact_id}),
        ),
        0,
    );
    assert_eq!(
        inspected["data"]["source_descriptor_sha256"],
        leg.expected_target_descriptor_sha
    );
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut first = None;
    let mut observations = Vec::new();
    loop {
        let sample = client
            .observe_episode_playback(RequestContext::default())
            .unwrap();
        assert_eq!(sample.engine_instance_id, next.engine_instance_id);
        assert_eq!(sample.resolution_artifact_id, next.resolution_artifact_id);
        assert!(!sample.playback.has_error);
        let ready = sample.playback.connected
            && sample.playback.idle == Some(false)
            && sample.playback.video_size.is_some()
            && sample.playback.audio_channels.is_some();
        if ready && first.is_none() {
            first = sample.playback.position_us;
        }
        let progressed = ready
            && sample
                .playback
                .position_us
                .zip(first)
                .is_some_and(|(now, before)| now.0 > before.0 + 1_000_000);
        observations.push(sample);
        fs::write(
            fixture
                .root
                .join(format!("logs/{prefix}-native-observations.json")),
            serde_json::to_vec_pretty(&observations).unwrap(),
        )
        .unwrap();
        if progressed {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "{prefix} native decode/time progress not observed"
        );
        std::thread::sleep(Duration::from_millis(250));
    }
    let path = fixture.root.join(format!("{prefix}-native-frame.png"));
    let frame = call(
        "capture",
        &request(
            "production.capture_frame",
            json!({"engine_instance_id":next.engine_instance_id,"output_path":path}),
        ),
        0,
    );
    let (bytes, sha) = kinekino_player::infrastructure::content_digest::sha256(
        &mut fs::File::open(&path).unwrap(),
        64 * 1024 * 1024,
    )
    .unwrap();
    assert_eq!(frame["data"]["output_sha256"], sha);
    assert_eq!(frame["data"]["output_bytes"], bytes.to_string());
    assert_eq!(
        frame["data"]["engine_instance_id"],
        json!(next.engine_instance_id)
    );
    assert_eq!(
        frame["data"]["resolution_artifact_id"],
        next.resolution_artifact_id
    );
    assert_eq!(frame["data"]["frame_time_exact"], false);
    assert_eq!(frame["data"]["color_parity_verified"], false);
    let mut decoded = png::Decoder::new(std::io::BufReader::new(fs::File::open(&path).unwrap()))
        .read_info()
        .unwrap();
    let width = decoded.info().width;
    let height = decoded.info().height;
    assert!(width > 0 && height > 0);
    assert!(
        u64::from(width) * u64::from(height)
            <= kinekino_player::domain::frame_capture::MAX_FRAME_PIXELS
    );
    assert_eq!(frame["data"]["width"], width);
    assert_eq!(frame["data"]["height"], height);
    let size = decoded.output_buffer_size().unwrap();
    assert!(size <= kinekino_player::domain::frame_capture::MAX_FRAME_PIXELS as usize * 4);
    let mut pixels = vec![0; size];
    assert!(decoded.next_frame(&mut pixels).unwrap().buffer_size() > 0);
    decoded.finish().unwrap();
    for (label, command) in [
        ("commit-replay-rejected", "playback.transition.commit"),
        ("consumed-handle-rejected", "playback.transition.observe"),
    ] {
        let rejected = call(label, &request(command, json!({"transition_id":handle})), 4);
        assert_eq!(rejected["error"]["code"], "not_found");
    }
    call(
        "old-instance-rejected",
        &request(
            "playback.pause",
            json!({"engine_instance_id":current.engine_instance_id}),
        ),
        5,
    );
    assert_eq!(
        client
            .observe_episode_playback(RequestContext::default())
            .unwrap()
            .engine_instance_id,
        next.engine_instance_id
    );
    next
}

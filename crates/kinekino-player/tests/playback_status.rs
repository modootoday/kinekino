use kinekino_player::application::api::PlaybackClient;
#[path = "support/cli_exe.rs"]
mod cli_exe;
use kinekino_player::{
    application::{
        api::{ApplicationClientHandle, ErrorCode, PlayerSdk, Request, RequestContext},
        frame_capture::UnavailableFrameCapture,
        native_playback::NativePlayback,
        playback_status::{PlaybackStatusPort, native_status_channel},
    },
    infrastructure::subtitle_files::SubtitleFiles,
};
use serde_json::{Value, json};
use std::{
    io::Write,
    process::{Command, Stdio},
    sync::Arc,
};

#[test]
fn absent_provider_stays_unsupported_in_actual_cli_and_existing_constructors() {
    kinekino_platform::install();
    let request = Request::new("playback.status");
    let sdk = PlayerSdk::new(SubtitleFiles).with_frame_capture(UnavailableFrameCapture);
    let expected = serde_json::to_value(sdk.execute(request.clone())).unwrap();
    let mut child = Command::new(crate::cli_exe::kinekino_cli_exe())
        .args(["--input", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(&request).unwrap())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(3));
    assert!(output.stderr.is_empty());
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        expected
    );
    assert_eq!(expected["data"], Value::Null);
    let _: PlayerSdk = PlayerSdk::default();
    let _: PlayerSdk<SubtitleFiles, UnavailableFrameCapture> = sdk;
}

#[test]
fn observations_are_optional_sanitized_and_shared_with_object_safe_client() {
    kinekino_platform::install();
    let (publisher, reader) = native_status_channel();
    let client: ApplicationClientHandle = Arc::new(
        PlayerSdk::new(SubtitleFiles)
            .with_playback_status(reader.clone())
            .with_frame_capture(UnavailableFrameCapture),
    );
    assert_eq!(
        client
            .playback_status(RequestContext::default())
            .unwrap_err()
            .code,
        ErrorCode::NotFound
    );
    let mut state = NativePlayback {
        connected: true,
        time: Some(12.25),
        duration: Some(60.),
        volume: Some(75.5),
        width: Some(1920),
        height: Some(1080),
        audio_channels: Some(2),
        error: Some("https://private.example/TEST_SECRET?token=TEST_SECRET".into()),
        ..Default::default()
    };
    publisher.publish(&state).unwrap();
    let status =
        std::thread::spawn(move || client.playback_status(RequestContext::default()).unwrap())
            .join()
            .unwrap();
    assert!(status.connected && status.has_error);
    assert_eq!(status.position_us.unwrap().0, 12_250_000);
    assert_eq!(status.duration_us.unwrap().0, 60_000_000);
    assert_eq!(status.volume_percent, Some(75.5));
    assert_eq!(status.video_size.unwrap().width, 1920);
    assert_eq!(status.audio_channels, Some(2));
    assert_eq!(status.paused, None);
    assert_eq!(status.idle, None);
    assert!(
        !serde_json::to_string(&status)
            .unwrap()
            .contains("TEST_SECRET")
    );
    let mut unexpected = serde_json::to_value(&status).unwrap();
    unexpected["source_url"] = json!("TEST_SECRET");
    assert!(
        serde_json::from_value::<kinekino_player::application::playback_status::PlaybackStatus>(
            unexpected
        )
        .is_err()
    );
    state.time = Some(f64::NAN);
    state.duration = Some(-1.);
    state.volume = Some(f64::INFINITY);
    state.width = Some(u64::MAX);
    state.audio_channels = Some(0);
    state.pause_observed = true;
    state.idle_observed = true;
    publisher.publish(&state).unwrap();
    let status = reader.status().unwrap();
    assert!(
        status.position_us.is_none()
            && status.duration_us.is_none()
            && status.volume_percent.is_none()
            && status.video_size.is_none()
            && status.audio_channels.is_none()
    );
    assert_eq!(status.paused, Some(false));
    assert_eq!(status.idle, Some(false));
    state.time = Some(u64::MAX as f64);
    publisher.publish(&state).unwrap();
    assert!(reader.status().unwrap().position_us.is_none());
    publisher.clear().unwrap();
    assert_eq!(reader.status().unwrap_err().code, ErrorCode::NotFound);
    publisher.publish(&NativePlayback::default()).unwrap();
    drop(publisher);
    assert_eq!(reader.status().unwrap_err().code, ErrorCode::NotFound);
}

#[test]
fn configured_discovery_schema_and_strict_read_context_agree() {
    kinekino_platform::install();
    let (publisher, reader) = native_status_channel();
    let sdk = PlayerSdk::new(SubtitleFiles)
        .with_frame_capture(UnavailableFrameCapture)
        .with_playback_status(reader);
    let capabilities = sdk.execute(Request::new("capabilities")).data.unwrap();
    assert_eq!(
        capabilities["commands"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["command"] == "playback.status")
            .unwrap()["status"],
        "supported"
    );
    assert_eq!(capabilities["local_host_transport"], "unsupported");
    let mut request = Request::new("schema");
    request.payload = json!({"command":"playback.status"});
    let schema = sdk.execute(request).data.unwrap();
    assert_eq!(schema["status"], "supported");
    assert_eq!(
        schema["request"]["properties"]["payload"]["maxProperties"],
        0
    );
    assert_eq!(
        schema["response"]["properties"]["data"]["anyOf"][0]["properties"]["origin"]["const"],
        "native_observation"
    );
    publisher.publish(&NativePlayback::default()).unwrap();
    for payload in [json!(null), json!([]), json!({"url":"TEST_SECRET"})] {
        let mut request = Request::new("playback.status");
        request.payload = payload;
        let response = sdk.execute(request);
        assert_eq!(
            response.error.as_ref().unwrap().code,
            ErrorCode::InvalidArgument
        );
        assert!(
            !serde_json::to_string(&response)
                .unwrap()
                .contains("TEST_SECRET")
        );
    }
    for field in ["timeout_ms", "expected_revision", "idempotency_key"] {
        let mut request = Request::new("playback.status");
        match field {
            "timeout_ms" => request.context.timeout_ms = Some(1),
            "expected_revision" => request.context.expected_revision = Some("1".into()),
            _ => request.context.idempotency_key = Some("key".into()),
        }
        assert_eq!(
            sdk.execute(request).error.unwrap().code,
            ErrorCode::Unsupported
        );
    }
    for command in [
        "playback.play",
        "playback.open",
        "playback.pause",
        "playback.seek",
    ] {
        assert_eq!(
            sdk.execute(Request::new(command)).error.unwrap().code,
            ErrorCode::Unsupported
        );
    }
}

use kinekino_player::application::api::PlaybackClient;
use kinekino_player::{
    application::{
        api::{ApiError, ErrorCode, PlayerSdk, Request, RequestContext},
        playback_control::{ControlReceipt, PlaybackControlPort, parse},
    },
    domain::playback_control::PlaybackControl,
};
use serde_json::json;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

struct Port(Arc<AtomicUsize>);
impl PlaybackControlPort for Port {
    fn submit(&self, _: PlaybackControl) -> Result<ControlReceipt, ApiError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(ControlReceipt::enqueued(42))
    }
}
#[test]
fn strict_payloads_and_numeric_boundaries() {
    kinekino_platform::install();
    for (name, payload) in [
        ("playback.play", json!(null)),
        ("playback.pause", json!({"command":"sensitive"})),
        ("playback.seek", json!({"position_us":1})),
        ("playback.seek", json!({"position_us":"-1"})),
        (
            "playback.seek",
            json!({"position_us":"18446744073709551616"}),
        ),
        ("playback.seek", json!({"position_us":"9007199254740992"})),
        ("playback.set_volume", json!({"volume_percent":-1})),
        ("playback.set_volume", json!({"volume_percent":100.01})),
        ("playback.set_volume", json!({"volume_percent":null})),
    ] {
        assert_eq!(
            parse(name, &payload).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
    }
    for position in ["0", "9007199254740991"] {
        assert!(parse("playback.seek", &json!({"position_us":position})).is_ok());
    }
    for volume in [0., 100.] {
        assert!(parse("playback.set_volume", &json!({"volume_percent":volume})).is_ok());
    }
    for volume in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(
            PlaybackControl::SetVolume {
                volume_percent: volume
            }
            .validate()
            .is_err()
        );
    }
}
#[test]
fn absent_provider_is_unsupported_and_injected_provider_is_conditional() {
    kinekino_platform::install();
    let calls = Arc::new(AtomicUsize::new(0));
    let sdk = PlayerSdk::default().with_playback_control(Port(calls.clone()));
    for command in [
        "playback.play",
        "playback.pause",
        "playback.seek",
        "playback.set_volume",
    ] {
        let mut request = Request::new(command);
        request.payload = match command {
            "playback.seek" => json!({"position_us":"1"}),
            "playback.set_volume" => json!({"volume_percent":50}),
            _ => json!({}),
        };
        assert_eq!(
            PlayerSdk::default()
                .execute(request.clone())
                .error
                .unwrap()
                .code,
            ErrorCode::Unsupported
        );
        let response = sdk.execute(request);
        assert!(response.ok);
        let data = response.data.unwrap();
        assert_eq!(data["acceptance"], "accepted");
        assert_eq!(data["completion"], "unobserved");
        assert_eq!(data["engine_instance_id"], "42");
    }
    assert_eq!(calls.load(Ordering::SeqCst), 4);
}
#[test]
fn invalid_payload_and_unimplemented_context_do_not_reach_adapter() {
    kinekino_platform::install();
    let calls = Arc::new(AtomicUsize::new(0));
    let sdk = PlayerSdk::default().with_playback_control(Port(calls.clone()));
    let mut request = Request::new("playback.play");
    request.payload = json!({"secret":"file:///private"});
    let result = sdk.execute(request);
    assert_eq!(
        result.error.as_ref().unwrap().code,
        ErrorCode::InvalidArgument
    );
    assert!(!serde_json::to_string(&result).unwrap().contains("private"));
    let context = RequestContext {
        timeout_ms: Some(1),
        ..RequestContext::default()
    };
    assert_eq!(
        sdk.playback_control(context, PlaybackControl::Play)
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[test]
#[cfg(windows)]
fn provider_capability_and_schema_agree_without_claiming_engine_readiness() {
    kinekino_platform::install();
    let (_, native) =
        kinekino_player::infrastructure::native_playback_control::native_control_channel();
    let sdk = PlayerSdk::default().with_playback_control(native);
    let caps = sdk.execute(Request::new("capabilities")).data.unwrap();
    for command in [
        "playback.play",
        "playback.pause",
        "playback.seek",
        "playback.set_volume",
    ] {
        let entry = caps["commands"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["command"] == command)
            .unwrap();
        assert_eq!(entry["status"], "supported");
        let mut request = Request::new("schema");
        request.payload = json!({"command":command});
        let schema = sdk.execute(request.clone()).data.unwrap();
        assert_eq!(schema["status"], "supported");
        assert_eq!(
            schema["response"]["properties"]["data"]["anyOf"][0]["properties"]["completion"]["const"],
            "unobserved"
        );
        assert_eq!(
            PlayerSdk::default().execute(request).data.unwrap()["status"],
            "unsupported"
        );
    }
    assert_eq!(
        sdk.playback_control(RequestContext::default(), PlaybackControl::Play)
            .unwrap_err()
            .code,
        ErrorCode::NotFound
    );
}

use super::{ApiError, ErrorCode, Request};
use crate::application::playback_control::PlaybackControlPort;
use serde_json::{Value, json};

pub fn execute(request: &Request, port: &impl PlaybackControlPort) -> Result<Value, ApiError> {
    if !port.audio_handoff_available() {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Native audio handoff observations are unavailable",
        ));
    }
    if request
        .payload
        .as_object()
        .is_none_or(|fields| !fields.is_empty())
    {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Audio handoff observation accepts an empty object payload only",
        ));
    }
    if request.context.expected_revision.is_some()
        || request.context.idempotency_key.is_some()
        || request.context.timeout_ms.is_some()
    {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Audio handoff observation does not support revision, idempotency or timeout options",
        ));
    }
    port.audio_handoff_status()
}
pub fn capabilities(mut value: Value, available: bool) -> Value {
    if let Some(commands) = value["commands"].as_array_mut() {
        for command in commands {
            if command["command"] == "playback.audio_handoff" {
                command["status"] = json!(if available {
                    "supported"
                } else {
                    "unsupported"
                });
            }
        }
    }
    value
}
pub fn schema(request: &Request, available: bool) -> Result<Value, ApiError> {
    let mut value = super::schema::schema(&request.payload, false, false, false, false, false)?;
    value["status"] = json!(if available {
        "supported"
    } else {
        "unsupported"
    });
    value["request"]["properties"]["payload"] =
        json!({"type":"object","additionalProperties":false,"maxProperties":0});
    for field in ["expected_revision", "idempotency_key", "timeout_ms"] {
        value["request"]["properties"]["context"]["properties"][field] = json!({"type":"null"});
    }
    value["observation_semantics"] = json!({"source":"common_application_coordinator","readiness":"observed_native_mute_volume_pause_with_owned_webview_silence_when_present","freshness":"published_age_ms","queue_acceptance_is_completion":false,"historical_after_close":true});
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        application::{api::PlayerSdk, playback_control::ControlReceipt},
        domain::playback_control::PlaybackControl,
    };
    struct Observations;
    impl PlaybackControlPort for Observations {
        fn audio_handoff_available(&self) -> bool {
            true
        }
        fn audio_handoff_status(&self) -> Result<Value, ApiError> {
            Ok(json!({"ready":false,"published_age_ms":"20"}))
        }
        fn submit(&self, _: PlaybackControl) -> Result<ControlReceipt, ApiError> {
            panic!("Read-only observation submitted control")
        }
    }
    #[test]
    fn sdk_observation_discovery_and_validation_use_same_port() {
        let sdk = PlayerSdk::default().with_playback_control(Observations);
        let response = sdk.execute(Request::new("playback.audio_handoff"));
        assert!(response.ok);
        assert_eq!(response.data.unwrap()["ready"], false);
        let discovery = sdk.execute(Request::new("capabilities")).data.unwrap();
        assert_eq!(
            discovery["commands"]
                .as_array()
                .unwrap()
                .iter()
                .find(|v| v["command"] == "playback.audio_handoff")
                .unwrap()["status"],
            "supported"
        );
        let mut request = Request::new("schema");
        request.payload = json!({"command":"playback.audio_handoff"});
        assert_eq!(sdk.execute(request).data.unwrap()["status"], "supported");
        let mut request = Request::new("playback.audio_handoff");
        request.payload = json!({"engine_instance_id":"7"});
        assert_eq!(
            sdk.execute(request).error.unwrap().code,
            ErrorCode::InvalidArgument
        );
        let mut request = Request::new("playback.audio_handoff");
        request.context.timeout_ms = Some(10);
        assert_eq!(
            sdk.execute(request).error.unwrap().code,
            ErrorCode::Unsupported
        );
    }
}

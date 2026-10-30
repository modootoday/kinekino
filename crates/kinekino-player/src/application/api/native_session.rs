use super::{ApiError, ErrorCode, Request};
use crate::application::playback_session::{
    CaptureEpisodeFrame, CloseEpisodePlayback, ControlEpisodePlayback, EpisodePlaybackSessions,
    OpenEpisodePlayback, SelectEpisodeTrack, SetEpisodeRate,
};
use serde_json::{Value, json};

pub(super) fn is_command(command: &str) -> bool {
    matches!(
        command,
        "playback.open"
            | "playback.engine"
            | "playback.status"
            | "playback.close"
            | "production.capture_frame"
            | "playback.set_rate"
            | "playback.select_track"
    ) || crate::application::playback_control::is_command(command)
}
pub(super) fn available(command: &str, sessions: &EpisodePlaybackSessions) -> bool {
    match command {
        "production.capture_frame" => sessions.capture_frame_available(),
        "playback.set_rate" | "playback.select_track" => sessions.options_available(),
        _ => is_command(command),
    }
}
pub(super) fn execute(
    request: &Request,
    sessions: &EpisodePlaybackSessions,
) -> Result<Value, ApiError> {
    if request.context.expected_revision.is_some()
        || request.context.idempotency_key.is_some()
        || request.context.timeout_ms.is_some()
    {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Native lifetime revision, idempotency and completion timeout are unavailable",
        ));
    }
    let invalid = || ApiError::new(ErrorCode::InvalidArgument, "Invalid native episode payload");
    match request.command.as_str() {
        "playback.set_rate" => encode(
            sessions.set_rate(
                serde_json::from_value::<SetEpisodeRate>(request.payload.clone())
                    .map_err(|_| invalid())?,
            )?,
        ),
        "playback.select_track" => encode(
            sessions.select_track(
                serde_json::from_value::<SelectEpisodeTrack>(request.payload.clone())
                    .map_err(|_| invalid())?,
            )?,
        ),
        "production.capture_frame" => {
            let input: CaptureEpisodeFrame =
                serde_json::from_value(request.payload.clone()).map_err(|_| invalid())?;
            encode(sessions.capture_frame(input)?)
        }
        "playback.open" => {
            let input: OpenEpisodePlayback =
                serde_json::from_value(request.payload.clone()).map_err(|_| invalid())?;
            encode(sessions.open(input)?)
        }
        "playback.close" => {
            let input: CloseEpisodePlayback =
                serde_json::from_value(request.payload.clone()).map_err(|_| invalid())?;
            encode(sessions.close(input)?)
        }
        "playback.engine" | "playback.status" => {
            if request
                .payload
                .as_object()
                .is_none_or(|value| !value.is_empty())
            {
                return Err(invalid());
            }
            let observation = sessions.observe()?;
            if request.command == "playback.status" {
                encode(observation.playback)
            } else {
                encode(observation)
            }
        }
        command if crate::application::playback_control::is_command(command) => {
            let mut payload = request.payload.clone();
            let id = payload
                .as_object_mut()
                .and_then(|value| value.remove("engine_instance_id"))
                .ok_or_else(invalid)?;
            let engine_instance_id = serde_json::from_value(id).map_err(|_| invalid())?;
            let control = crate::application::playback_control::parse(command, &payload)?;
            encode(sessions.control(ControlEpisodePlayback {
                engine_instance_id,
                control,
            })?)
        }
        _ => Err(invalid()),
    }
}
fn encode(value: impl serde::Serialize) -> Result<Value, ApiError> {
    serde_json::to_value(value)
        .map_err(|_| ApiError::new(ErrorCode::Internal, "Native receipt serialization failed"))
}

pub(super) fn schema(command: &str) -> Value {
    let instance = json!({"type":"string","pattern":"^[1-9][0-9]*$","description":"Actual current native engine instance ID; compared atomically by owner"});
    let payload = match command {
        "playback.set_rate" => {
            json!({"type":"object","additionalProperties":false,"required":["engine_instance_id","rate"],"properties":{"engine_instance_id":instance,"rate":{"type":"number","minimum":crate::domain::playback_options::PlaybackRate::MIN,"maximum":crate::domain::playback_options::PlaybackRate::MAX}}})
        }
        "playback.select_track" => {
            json!({"type":"object","additionalProperties":false,"required":["engine_instance_id","expected_tracks_revision","track_kind","selection"],"properties":{"engine_instance_id":instance,"expected_tracks_revision":instance,"track_kind":{"enum":["audio","video","subtitle"]},"selection":{"oneOf":[{"type":"object","additionalProperties":false,"required":["kind"],"properties":{"kind":{"const":"disabled"}}},{"type":"object","additionalProperties":false,"required":["kind","track_id"],"properties":{"kind":{"const":"id"},"track_id":{"type":"string","pattern":"^[1-9][0-9]*$","maxLength":10,"description":"Observed track ID within the requested kind; nonzero u32 range checked by domain"}}}]}}})
        }
        "production.capture_frame" => {
            json!({"type":"object","additionalProperties":false,"required":["engine_instance_id","output_path"],"properties":{"engine_instance_id":instance,"output_path":{"type":"string","description":"New absolute ordinary local PNG path; never overwritten. Current engine frame, not source PTS. Capture deadline 4 seconds; no automatic retry."}}})
        }
        "playback.open" => {
            json!({"type":"object","additionalProperties":false,"required":["resolution_artifact_id","paused","volume_percent"],"properties":{"resolution_artifact_id":{"type":"string","pattern":"^[0-9a-f]{64}$"},"paused":{"type":"boolean"},"volume_percent":{"type":"number","minimum":0,"maximum":100}}})
        }
        "playback.close" => {
            json!({"type":"object","additionalProperties":false,"required":["engine_instance_id"],"properties":{"engine_instance_id":instance}})
        }
        name if crate::application::playback_control::is_command(name) => {
            let mut payload = crate::application::playback_control::payload_schema(name);
            payload.as_object_mut().unwrap().remove("maxProperties");
            payload["properties"]["engine_instance_id"] = instance;
            if payload["required"].is_null() {
                payload["required"] = json!([]);
            }
            payload["required"]
                .as_array_mut()
                .unwrap()
                .push(json!("engine_instance_id"));
            payload
        }
        _ => json!({"type":"object","additionalProperties":false,"maxProperties":0}),
    };
    let mut context = super::schema::context_schema();
    for field in ["timeout_ms", "idempotency_key", "expected_revision"] {
        context["properties"][field] = json!({"type":"null"});
    }
    json!({"schema_version":"1","command":command,"status":"experimental","request":{"type":"object","additionalProperties":false,"required":["context","command","payload"],"properties":{"context":context,"command":{"const":command},"payload":payload}},"response":super::native_response_schema::response(command),"semantics":"Explicit owned native lifetime. Open is process creation only; control acceptance is unobserved completion. Stored manifest evidence does not prove live URL, referenced bytes, decode or playback. Close waits for owned engine, IPC and stderr workers. Native capture verifies a current frame PNG; not source PTS or color/output parity. No automatic retry, startup, replacement or source reconstruction."})
}
pub(super) fn capabilities(value: &mut Value, capture: bool, options: bool) {
    if let Some(commands) = value["commands"].as_array_mut() {
        for command in commands {
            if command["command"].as_str().is_some_and(|name| {
                is_command(name)
                    && match name {
                        "production.capture_frame" => capture,
                        "playback.set_rate" | "playback.select_track" => options,
                        _ => true,
                    }
            }) {
                command["status"] = json!("experimental");
            }
        }
    }
    value["native_episode_playback"] = json!({"available":true,"explicit_host":true,"instance_bound_control":true,"open_means":"process_started; load and decode unobserved","control_completion":"unobserved","source":"reverified stored resolution; current URL validity unobserved","close_means":"owned engine and IPC worker exited"});
    value["native_episode_playback"]["current_frame_capture_available"] = json!(capture);
    value["native_episode_playback"]["rate_and_tracks_available"] = json!(options);
    value["native_episode_playback"]["track_selection_validation"] = json!(
        "engine instance and latest observed catalog revision; queue acceptance is not an engine transaction"
    );
}

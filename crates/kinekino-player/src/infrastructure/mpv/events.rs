use crate::application::native_playback::NativePlayback;
use serde_json::Value;
pub(super) fn apply(state: &mut NativePlayback, message: &Value) {
    if message.get("request_id").is_some() {
        if message["error"] == "success" {
            state.acknowledgements += 1;
        } else if let Some(error) = message["error"].as_str() {
            state.error = Some(error.to_owned());
        }
    }
    if message["event"] == "end-file" && message["reason"] == "error" {
        state.error = Some("mpv media load/decoding failed".into());
    }
    if message["event"] != "property-change" {
        return;
    }
    let data = &message["data"];
    match message["name"].as_str().unwrap_or("") {
        "time-pos" => state.time = data.as_f64(),
        "duration" => state.duration = data.as_f64(),
        "pause" => {
            state.pause_observed = data.is_boolean();
            state.paused = data.as_bool().unwrap_or(false);
        }
        "fullscreen" => state.fullscreen = data.as_bool().unwrap_or(false),
        "idle-active" => {
            state.idle_observed = data.is_boolean();
            state.idle = data.as_bool().unwrap_or(true);
        }
        "volume" => state.volume = data.as_f64(),
        "mute" => state.muted = data.as_bool(),
        "video-params" => {
            state.width = data["w"].as_u64();
            state.height = data["h"].as_u64();
        }
        "audio-params" => state.audio_channels = data["channel-count"].as_u64(),
        _ => (),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn mute_is_independent_real_property_observation_not_request_ack_or_zero_volume() {
        use crate::application::playback_status::{PlaybackStatusPort, native_status_channel};
        let (publisher, reader) = native_status_channel();
        let mut state = NativePlayback::default();
        apply(&mut state, &json!({"request_id":1000,"error":"success"}));
        assert_eq!(state.muted, None);
        publisher.publish(&state).unwrap();
        assert_eq!(reader.status().unwrap().muted, None);
        apply(
            &mut state,
            &json!({"event":"property-change","name":"volume","data":0}),
        );
        assert_eq!(state.muted, None);
        apply(
            &mut state,
            &json!({"event":"property-change","name":"mute","data":true}),
        );
        assert_eq!(state.muted, Some(true));
        publisher.publish(&state).unwrap();
        assert_eq!(reader.status().unwrap().muted, Some(true));
        apply(
            &mut state,
            &json!({"event":"property-change","name":"mute","data":false}),
        );
        assert_eq!(state.muted, Some(false));
        publisher.publish(&state).unwrap();
        assert_eq!(reader.status().unwrap().muted, Some(false));
        apply(
            &mut state,
            &json!({"event":"property-change","name":"mute","data":null}),
        );
        assert_eq!(state.muted, None);
        publisher.publish(&state).unwrap();
        assert_eq!(reader.status().unwrap().muted, None);
    }
    #[test]
    fn requires_real_progress_and_preserves_load_errors() {
        let mut state = NativePlayback::default();
        apply(&mut state, &json!({"request_id":1000,"error":"success"}));
        assert_eq!(state.time, None);
        apply(
            &mut state,
            &json!({"event":"property-change","name":"time-pos","data":31.25}),
        );
        assert_eq!(state.time, Some(31.25));
        apply(&mut state, &json!({"event":"end-file","reason":"error"}));
        assert!(state.error.is_some());
    }

    #[test]
    fn status_distinguishes_missing_and_observed_false_properties() {
        use crate::application::playback_status::{PlaybackStatusPort, native_status_channel};
        let (publisher, reader) = native_status_channel();
        let mut state = NativePlayback {
            connected: true,
            ..Default::default()
        };
        publisher.publish(&state).unwrap();
        assert_eq!(reader.status().unwrap().paused, None);
        for name in ["pause", "idle-active"] {
            apply(
                &mut state,
                &json!({"event":"property-change","name":name,"data":false}),
            );
        }
        publisher.publish(&state).unwrap();
        let status = reader.status().unwrap();
        assert_eq!(status.paused, Some(false));
        assert_eq!(status.idle, Some(false));
        apply(
            &mut state,
            &json!({"event":"property-change","name":"pause","data":null}),
        );
        publisher.publish(&state).unwrap();
        assert_eq!(reader.status().unwrap().paused, None);
    }
}

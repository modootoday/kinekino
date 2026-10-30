use serde_json::{Value, json};
fn object(properties: Value) -> Value {
    let required: Vec<_> = properties.as_object().unwrap().keys().cloned().collect();
    json!({"type":"object","additionalProperties":false,"required":required,"properties":properties})
}
fn options() -> Value {
    let id = json!({"type":"string","pattern":"^[1-9][0-9]*$"});
    let track = object(
        json!({"id":id,"kind":{"enum":["audio","video","subtitle"]},"language":{"type":["string","null"],"maxLength":64},"codec":{"type":["string","null"],"maxLength":128},"selected":{"type":["boolean","null"]},"external":{"type":["boolean","null"]}}),
    );
    let tracks = object(
        json!({"revision":id,"entries":{"type":"array","maxItems":crate::domain::playback_options::MAX_TRACKS,"items":track}}),
    );
    let choice = json!({"anyOf":[object(json!({"state":{"enum":["disabled","unresolved"]}})),object(json!({"state":{"const":"selected"},"track_id":id})),{"type":"null"}]});
    object(
        json!({"rate":{"type":["number","null"],"exclusiveMinimum":0},"tracks":{"anyOf":[tracks,{"type":"null"}]},"tracks_unavailable_reason":{"anyOf":[{"enum":crate::domain::playback_options::TrackCatalogError::ALL},{"type":"null"}]},"audio_selection":choice,"video_selection":choice,"subtitle_selection":choice}),
    )
}
pub(super) fn response(command: &str) -> Value {
    let decimal = json!({"type":"string","pattern":"^[0-9]+$"});
    let optional_decimal = json!({"anyOf":[decimal,{"type":"null"}]});
    let instance = json!({"type":"string","pattern":"^[1-9][0-9]*$"});
    let sha = json!({"type":"string","pattern":"^[0-9a-f]{64}$"});
    let artifact = object(
        json!({"artifact_id":sha,"body_sha256":sha,"body_bytes":decimal,"body_path":{"type":"string"},"metadata_path":{"type":"string"}}),
    );
    let diagnostic = object(
        json!({"artifact":artifact,"stderr_sha256":sha,"stderr_bytes":decimal,"stdout_sha256":sha,"stdout_bytes":decimal,"complete":{"type":"boolean"},"truncated":{"type":"boolean"},"read_failed":{"type":"boolean"}}),
    );
    let data = match command {
        "playback.open" => object(
            json!({"engine_instance_id":instance,"resolution_artifact_id":sha,"completion":{"const":"unobserved"}}),
        ),
        "playback.close" => object(
            json!({"engine_instance_id":instance,"engine_exited":{"const":true},"ipc_worker_exited":{"const":true},"stderr_worker_exited":{"const":true},"stdout_worker_exited":{"const":true},"diagnostics":diagnostic}),
        ),
        "playback.engine" => object(
            json!({"engine_instance_id":instance,"resolution_artifact_id":sha,"playback":crate::application::playback_status::response_schema(),"diagnostics":{"anyOf":[diagnostic,{"type":"null"}]},"options":{"anyOf":[options(),{"type":"null"}]}}),
        ),
        "playback.status" => crate::application::playback_status::response_schema(),
        "playback.select_track" => object(
            json!({"acceptance":{"const":"accepted"},"completion":{"const":"unobserved"},"engine_instance_id":instance,"validated_tracks_revision":instance}),
        ),
        "production.capture_frame" => object(
            json!({"engine_instance_id":instance,"resolution_artifact_id":sha,"output_path":{"type":"string"},"output_sha256":sha,"output_bytes":decimal,"width":{"type":"integer","minimum":1},"height":{"type":"integer","minimum":1},"position_before_us":optional_decimal,"position_after_us":optional_decimal,"frame_time_exact":{"const":false},"subtitles_included":{"const":false},"window_capture":{"const":false},"color_parity_verified":{"const":false}}),
        ),
        _ => crate::application::playback_control::response_schema(),
    };
    let error = object(
        json!({"code":{"enum":super::ErrorCode::ALL},"message":{"type":"string"},"retryable":{"type":"boolean"},"details":{"type":"object"},"remediation":{"type":"string"},"request_id":{"type":"string"}}),
    );
    object(
        json!({"api_version":{"const":"1"},"request_id":{"type":"string"},"ok":{"type":"boolean"},"data":{"anyOf":[data,{"type":"null"}]},"error":{"anyOf":[error,{"type":"null"}]}}),
    )
}

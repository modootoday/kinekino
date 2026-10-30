use super::{ApiError, ErrorCode, Request, capabilities};
use serde_json::{Value, json};

pub fn schema(
    payload: &Value,
    files_available: bool,
    frame_available: bool,
    playback_available: bool,
    control_available: bool,
    review_available: bool,
) -> Result<Value, ApiError> {
    let command = payload
        .get("command")
        .and_then(Value::as_str)
        .ok_or_else(|| ApiError::new(ErrorCode::InvalidArgument, "schema requires command"))?;
    if payload.as_object().is_none_or(|v| v.len() != 1)
        || !capabilities::COMMANDS.contains(&command)
    {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Unknown schema command or field",
        ));
    }
    let supported = capabilities::supported_with_reviews(
        command,
        files_available,
        frame_available,
        playback_available,
        control_available,
        review_available,
    );
    let mut example = Request::new(command);
    let payload_schema = match command {
        command if crate::application::model_review::is_command(command) => {
            example.payload = crate::application::model_review::schema::example(command);
            crate::application::model_review::schema::payload(command)
        }
        command if crate::application::playback_control::is_command(command) => {
            example.payload = match command {
                "playback.seek" => json!({"position_us":"1000000"}),
                "playback.set_volume" => json!({"volume_percent":75}),
                "playback.set_muted" => json!({"muted":true}),
                _ => json!({}),
            };
            crate::application::playback_control::payload_schema(command)
        }
        "playback.status" => crate::application::playback_status::payload_schema(),
        "production.capture_frame" => {
            example.payload = json!({"source_path":"D:/media/source.mp4","source_pts_us":"1000000","output_path":"D:/exports/frame.png"});
            crate::application::frame_capture::schema()
        }
        "schema" => {
            example.payload = json!({"command":"version"});
            json!({"type":"object","additionalProperties":false,"required":["command"],"properties":{"command":{"enum":capabilities::COMMANDS}}})
        }
        "subtitles.convert" => {
            example.payload = json!({"input_path":"input.srt","output_path":"output.vtt","input_format":"srt","output_format":"vtt","allow_loss":false,"speaker_policy":"omit"});
            crate::application::subtitles::convert::schema()
        }
        "subtitles.export_range" => {
            example.payload = json!({"input_path":"input.srt","output_path":"output.vtt","input_format":"srt","output_format":"vtt","range":{"in_us":"1000000","out_us":"3000000"},"allow_loss":false,"speaker_policy":"omit"});
            crate::application::subtitles::export_range::schema()
        }
        "subtitles.from_transcript" => {
            example.payload = json!({"input_path":"transcript.json","output_path":"output.ass","output_format":"ass","layout":{"max_chars":42},"ass_style":{"font_name":"Noto Sans KR","font_size":56},"allow_loss":false});
            crate::application::subtitles::from_transcript::schema()
        }
        _ if supported => json!({"type":"object","maxProperties":0}),
        _ => {
            json!({"type":"object","description":"No executable payload contract available; returns unsupported"})
        }
    };
    let mut context = context_schema();
    if command == "production.capture_frame" {
        for field in ["expected_revision", "idempotency_key"] {
            context["properties"][field] = json!({"type":"null"});
        }
        context["properties"]["timeout_ms"] =
            json!({"type":["integer","null"],"minimum":1,"maximum":120000,"default":30000});
    }
    if matches!(
        command,
        "subtitles.convert"
            | "subtitles.export_range"
            | "subtitles.from_transcript"
            | "playback.status"
    ) || crate::application::playback_control::is_command(command)
        || crate::application::model_review::is_command(command)
    {
        for field in ["expected_revision", "idempotency_key", "timeout_ms"] {
            context["properties"][field] =
                json!({"type":"null","description":"Unavailable for this synchronous operation"});
        }
    }
    let required = if crate::application::model_review::is_command(command)
        || matches!(
            command,
            "production.capture_frame"
                | "subtitles.convert"
                | "subtitles.export_range"
                | "subtitles.from_transcript"
                | "playback.seek"
                | "playback.set_volume"
                | "playback.set_muted"
        ) {
        vec!["context", "command", "payload"]
    } else {
        vec!["context", "command"]
    };
    let request = json!({"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","additionalProperties":false,"required":required,"properties":{"context":context,"command":{"const":command},"payload":payload_schema}});
    Ok(
        json!({"schema_version":"1","command":command,"status":if supported {"supported"} else {"unsupported"},"request":request,"response":{"type":"object","required":["api_version","request_id","ok","data","error"],"properties":{"api_version":{"const":"1"},"request_id":{"type":"string"},"ok":{"type":"boolean"},"data":{},"error":{"type":["object","null"],"required":["code","message","retryable","details","remediation","request_id"]}}},"error_codes":ErrorCode::ALL,"example":example}),
    ).map(|mut schema| {
        if crate::application::model_review::is_command(command){schema["response"]["properties"]["data"]=json!({"anyOf":[crate::application::model_review::schema::response(command),{"type":"null"}]});schema["review_package_schema"]=crate::application::model_review::schema::package();}
        if command == "playback.status" {
            schema["response"]["properties"]["data"] = json!({"anyOf":[crate::application::playback_status::response_schema(), {"type":"null"}]});
        }
        if crate::application::playback_control::is_command(command){schema["response"]["properties"]["data"]=json!({"anyOf":[crate::application::playback_control::response_schema(),{"type":"null"}]});}
        schema
    })
}

pub(crate) fn context_schema() -> Value {
    json!({"type":"object","additionalProperties":false,"required":["request_id","api_version"],"properties":{"request_id":{"type":"string","minLength":1,"maxLength":super::commands::MAX_CONTEXT_TEXT_LENGTH,"not":{"pattern":super::commands::REQUEST_ID_CONTROL_PATTERN},"description":"1..128 Unicode code points, no C0/C1 control characters; public correlation identifier, not secret input"},"api_version":{"const":"1"},"expected_revision":{"type":["string","null"],"pattern":"^[0-9]+$","description":"Decimal u64; range checked by application validation"},"idempotency_key":{"type":["string","null"],"minLength":1,"maxLength":super::commands::MAX_CONTEXT_TEXT_LENGTH,"description":"1..128 Unicode code points; command support is separate"},"timeout_ms":{"type":["integer","null"],"minimum":1}}})
}

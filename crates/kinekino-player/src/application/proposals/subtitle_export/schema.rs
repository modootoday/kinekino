use super::COMMAND;
use crate::application::api::{ApiError, ErrorCode};
use serde_json::{Value, json};

fn object(properties: Value, required: &[&str]) -> Value {
    json!({"type":"object","additionalProperties":false,"properties":properties,"required":required})
}
pub(crate) fn describe(input: &Value, available: bool) -> Result<Value, ApiError> {
    if input.as_object().is_none_or(|fields| fields.len() != 1) || input["command"] != COMMAND {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Export schema accepts its command only",
        ));
    }
    let target = super::super::output_review::schema::target();
    let payload = object(
        json!({"target":target,"output_path":{"type":"string","minLength":1},"output_format":{"enum":["srt","vtt"]},"allow_loss":{"type":"boolean","default":false}}),
        &["target", "output_path", "output_format"],
    );
    let mut context = crate::application::api::schema::context_schema();
    for field in ["expected_revision", "idempotency_key", "timeout_ms"] {
        context["properties"][field] = json!({"type":"null"});
    }
    let data = object(
        json!({"target":super::super::output_review::schema::target(),"output_path":{"type":"string"},"output_format":{"enum":["srt","vtt"]},"output_sha256":{"type":"string","pattern":"^[a-f0-9]{64}$"},"output_bytes":{"type":"string","pattern":"^[0-9]+$"},"cue_count":{"type":"integer","minimum":1},"losses":{"type":"array","items":object(json!({"code":{"type":"string"},"message":{"type":"string"}}), &["code","message"])},"encoding":{"const":"utf-8"},"overwrite":{"const":false}}),
        &[
            "target",
            "output_path",
            "output_format",
            "output_sha256",
            "output_bytes",
            "cue_count",
            "losses",
            "encoding",
            "overwrite",
        ],
    );
    Ok(
        json!({"schema_version":"1","command":COMMAND,"status":if available{"supported"}else{"unsupported"},"request":object(json!({"context":context,"command":{"const":COMMAND},"payload":payload}), &["context","command","payload"]),"response":object(json!({"api_version":{"const":"1"},"request_id":{"type":"string"},"ok":{"type":"boolean"},"data":{"anyOf":[data,{"type":"null"}]},"error":{"type":["object","null"]}}), &["api_version","request_id","ok","data","error"]),"error_codes":ErrorCode::ALL,"accepted_output_mutated":false,"automatic_acceptance":false,"publication":"explicit_new_file","output_binding":"selected_accepted_target_and_rendered_bytes_sha256","timestamp_precision":"existing_range_precision_checks_and_explicit_loss_acknowledgement"}),
    )
}
pub(crate) fn capabilities(value: &mut Value, available: bool) {
    if let Some(commands) = value["commands"].as_array_mut() {
        for command in commands {
            if command["command"] == COMMAND {
                command["status"] = json!(if available {
                    "supported"
                } else {
                    "unsupported"
                });
            }
        }
    }
}

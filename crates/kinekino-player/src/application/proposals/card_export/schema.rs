use super::*;
fn object(properties: Value, required: &[&str]) -> Value {
    json!({"type":"object","additionalProperties":false,"properties":properties,"required":required})
}
pub(crate) fn describe(request: &Request, available: bool) -> Result<Value, ApiError> {
    if request.payload.as_object().is_none_or(|p| p.len() != 1) {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Card review schema accepts command only",
        ));
    }
    let sha = json!({"type":"string","pattern":"^[a-f0-9]{64}$"});
    let text = json!({"type":"string","minLength":1});
    let template = object(
        json!({"id":text,"version":text,"declaration_sha256":sha}),
        &["id", "version", "declaration_sha256"],
    );
    let payload = object(
        json!({"target":super::super::output_review::schema::target(),"position":{"type":"string","pattern":"^(0|[1-9][0-9]*)$"},"frame_path":text,"frame_sha256":sha,"template":template,"output_path":text,"review_artifact_only":{"const":true}}),
        &[
            "target",
            "position",
            "frame_path",
            "frame_sha256",
            "template",
            "output_path",
            "review_artifact_only",
        ],
    );
    let mut context = crate::application::api::schema::context_schema();
    for name in ["expected_revision", "idempotency_key", "timeout_ms"] {
        context["properties"][name] = json!({"type":"null","description":"Unavailable for synchronous unapproved review PNG"});
    }
    let receipt = object(
        json!({"target":super::super::output_review::schema::target(),"position":{"type":"string"},"source_refs":{"type":"array","items":{"type":"string"}},"output_path":text,"output_sha256":sha,"output_bytes":{"type":"string"},"state":{"const":"review_artifact"},"quality_approved":{"const":false},"template_approved":{"const":false},"overwrite":{"const":false},"limitations":{"type":"array","items":{"enum":LIMITATIONS}},"cancellation_supported":{"const":false},"timeout_supported":{"const":false},"provenance":{"type":"object"}}),
        &[
            "target",
            "position",
            "source_refs",
            "output_path",
            "output_sha256",
            "output_bytes",
            "state",
            "quality_approved",
            "template_approved",
            "overwrite",
            "limitations",
            "cancellation_supported",
            "timeout_supported",
            "provenance",
        ],
    );
    let response = object(
        json!({"api_version":{"const":"1"},"request_id":{"type":"string"},"ok":{"type":"boolean"},"data":{"anyOf":[receipt,{"type":"null"}]},"error":{"type":["object","null"]}}),
        &["api_version", "request_id", "ok", "data", "error"],
    );
    Ok(
        json!({"schema_version":"1","command":COMMAND,"status":if available{"available_unverified"}else{"unsupported"},"request":object(json!({"command":{"const":COMMAND},"context":context,"payload":payload}),&["command","context","payload"]),"response":response,"result_state":"review_artifact","quality_approved":false,"template_approved":false,"support":support(available),"error_codes":ErrorCode::ALL,"verification":"source_candidate_not_actual_CLI_verified"}),
    )
}
pub(crate) fn capabilities(value: &mut Value, available: bool) {
    if let Some(commands) = value["commands"].as_array_mut() {
        for command in commands {
            if command["command"] == COMMAND {
                command["status"] = json!(if available {
                    "available_unverified"
                } else {
                    "unsupported"
                });
            }
        }
    }
    value["card_review_png"] = support(available);
    value["card_review_png"]["verification"] = json!("not_actual_CLI_verified");
}

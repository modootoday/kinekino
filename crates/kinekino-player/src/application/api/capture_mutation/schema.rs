use super::{ApiError, ErrorCode, is_command};
use serde_json::{Value, json};

fn object(properties: Value, required: &[&str]) -> Value {
    json!({"type":"object","additionalProperties":false,"properties":properties,"required":required})
}
fn decimal() -> Value {
    json!({"type":"string","pattern":"^[0-9]+$","description":"Positive decimal u64; range checked"})
}
fn sha() -> Value {
    json!({"type":"string","pattern":"^[a-f0-9]{64}$"})
}
fn id() -> Value {
    json!({"type":"string","minLength":1,"maxLength":80,"pattern":"^[A-Za-z0-9_.-]+$","not":{"enum":[".",".."]}})
}
fn nullable(value: Value) -> Value {
    json!({"anyOf":[value,{"type":"null"}]})
}
fn expectation() -> Value {
    object(
        json!({"draft_revision":decimal(),"state_sha256":sha()}),
        &["draft_revision", "state_sha256"],
    )
}
fn payload(command: &str) -> Value {
    let anchor = object(
        json!({"project_id":id(),"project_revision":decimal(),"project_revision_sha256":sha(),"proposal_id":id(),"capture_sha256":sha()}),
        &[
            "project_id",
            "project_revision",
            "project_revision_sha256",
            "proposal_id",
            "capture_sha256",
        ],
    );
    let mut properties = json!({"anchor":anchor,"expected":nullable(expectation())});
    let field = if command == "proposals.edit_capture" {
        properties["draft"] = crate::application::proposals::schema::payload_type();
        "draft"
    } else {
        properties["target_source_revision"] = decimal();
        "target_source_revision"
    };
    object(properties, &["anchor", field])
}
fn result() -> Value {
    object(
        json!({
            "state":crate::application::proposals::schema::response_type("proposals.inspect"),
            "continuation":expectation(),
            "acceptance":nullable(crate::application::proposals::schema::response_type("proposals.accept")),
            "reused_acceptance":{"type":"boolean"}
        }),
        &["state", "continuation", "acceptance", "reused_acceptance"],
    )
}
pub(in crate::application::api) fn describe(
    input: &Value,
    available: bool,
) -> Result<Value, ApiError> {
    let command = input["command"]
        .as_str()
        .filter(|command| is_command(command))
        .filter(|_| input.as_object().is_some_and(|object| object.len() == 1))
        .ok_or_else(|| {
            ApiError::new(
                ErrorCode::InvalidArgument,
                "Schema requires one capture mutation command",
            )
        })?;
    let mut context = crate::application::api::schema::context_schema();
    for field in ["expected_revision", "idempotency_key", "timeout_ms"] {
        context["properties"][field] = json!({"type":"null"});
    }
    Ok(json!({
        "schema_version":"1","command":command,"status":if available {"supported"} else {"unsupported"},
        "request":object(json!({"context":context,"command":{"const":command},"payload":payload(command)}), &["context","command","payload"]),
        "response":object(json!({"api_version":{"const":"1"},"request_id":{"type":"string"},"ok":{"type":"boolean"},"data":nullable(result()),"error":{"type":["object","null"]}}), &["api_version","request_id","ok","data","error"]),
        "error_codes":ErrorCode::ALL,
        "constraints":{"anchor_from_configured_project_store":true,"project_revision_immutable":true,"project_latest_atomic":false,"origin_verified_under_writer":true,"legacy_capture_read_only":true,"continuation_is_cas_expectation":true,"automatic_refresh":false,"accept_retry_requires_prestate_and_exact_poststate":true,"project_capture_automatically_updated":false},
        "automatic_application_enabled":false
    }))
}
pub(in crate::application::api) fn capabilities(value: &mut Value, available: bool) {
    if let Some(commands) = value["commands"].as_array_mut() {
        for command in ["proposals.edit_capture", "proposals.accept_capture"] {
            commands.push(
                json!({"command":command,"status":if available {"supported"} else {"unsupported"}}),
            );
        }
    }
    value["capture_mutation"] = json!({"backend_available":available,"project_revision_immutable":true,"project_latest_atomic":false,"continuation_is_cas_expectation":true,"legacy_capture_read_only":true,"automatic_refresh":false,"automatic_application_enabled":false});
}

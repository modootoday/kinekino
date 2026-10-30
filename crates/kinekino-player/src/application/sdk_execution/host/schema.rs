//! This host's discovery is separate from the in-process SDK command catalog.
use super::*;
use serde_json::{Value, json};
pub const COMMANDS: &[&str] = &[
    "version",
    "capabilities",
    "schema",
    "execution.submit",
    "execution.submission.inspect",
    "execution.list",
    "execution.peek",
    "execution.cancel",
    "execution.retry_receipt",
    "execution.acknowledge",
];
pub fn capabilities() -> Value {
    json!({"host_protocol":"execution-host/1","commands":COMMANDS.iter().map(|command|json!({"command":command,"status":"supported"})).collect::<Vec<_>>(),"operation_support":"inner SDK result determines support; admission does not guarantee success","submission_records_max":128,"submission_replay":"explicit matching identity and full request digest in original live host only; acknowledged IDs never reexecute","automatic_retry":false,"automatic_ack":false,"forced_cancellation":false,"receipt_retry_max_generations":16,"scope_lifetime":"original live host only","request_context":"outer expected_revision/idempotency_key/timeout_ms must be null; inner SDK context remains separate"})
}
fn submission() -> Value {
    object(
        json!({"host_scope":{"type":"string","pattern":"^[0-9a-f]{32}$"},"submission_id":{"type":"string","minLength":1,"maxLength":128,"not":{"pattern":crate::application::api::commands::REQUEST_ID_CONTROL_PATTERN}}}),
        &["host_scope", "submission_id"],
    )
}
fn object(properties: Value, required: &[&str]) -> Value {
    json!({"type":"object","additionalProperties":false,"properties":properties,"required":required})
}
fn reference() -> Value {
    object(
        json!({"host_scope":{"type":"string","pattern":"^[0-9a-f]{32}$"},"ordinal":{"type":"string","pattern":"^[1-9][0-9]{0,19}$","description":"Canonical positive decimal within u64; must resolve to a retained operation in this host"}}),
        &["host_scope", "ordinal"],
    )
}
fn context() -> Value {
    object(
        json!({"api_version":{"const":"1"},"request_id":{"type":"string","minLength":1,"maxLength":128,"not":{"pattern":crate::application::api::commands::REQUEST_ID_CONTROL_PATTERN}},"expected_revision":{"type":"null"},"idempotency_key":{"type":"null"},"timeout_ms":{"type":"null"}}),
        &["api_version", "request_id"],
    )
}
pub fn command(command: &str) -> Result<Value, ApiError> {
    if !COMMANDS.contains(&command) {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Unknown execution host schema command",
        ));
    }
    let (payload, example) = match command {
        "schema" => (
            object(json!({"command":{"enum":COMMANDS}}), &["command"]),
            json!({"command":"execution.peek"}),
        ),
        "execution.submit" => {
            let mut inner_context = context();
            inner_context["properties"]["expected_revision"] = json!({"type":["string","null"],"pattern":"^[0-9]+$","description":"Decimal u64; exact SDK admission performs numeric range validation"});
            inner_context["properties"]["idempotency_key"] =
                json!({"type":["string","null"],"minLength":1,"maxLength":128});
            inner_context["properties"]["timeout_ms"] =
                json!({"type":["integer","null"],"minimum":1,"maximum":u64::MAX});
            let inner = object(
                json!({"context":inner_context,"command":{"type":"string","not":{"pattern":"^execution\\."},"description":"SDK support is observed in the original terminal"},"payload":{"type":"object"}}),
                &["context", "command"],
            );
            (
                object(
                    json!({"request":inner,"submission":{"anyOf":[submission(),{"type":"null"}]}}),
                    &["request"],
                ),
                json!({"request":Request::new("version")}),
            )
        }
        "execution.submission.inspect" => (
            object(json!({"submission":submission()}), &["submission"]),
            json!({"submission":{"host_scope":"0".repeat(32),"submission_id":"explicit-submit-1"}}),
        ),
        "execution.peek" | "execution.cancel" | "execution.acknowledge" => (
            object(json!({"operation":reference()}), &["operation"]),
            json!({"operation":{"host_scope":"0".repeat(32),"ordinal":"1"}}),
        ),
        "execution.retry_receipt" => (
            object(
                json!({"operation":reference(),"expected_generation":{"type":"integer","minimum":0,"maximum":255,"description":"Must equal the current observed generation; only generations1..16 can exist"}}),
                &["operation", "expected_generation"],
            ),
            json!({"operation":{"host_scope":"0".repeat(32),"ordinal":"1"},"expected_generation":1}),
        ),
        _ => (object(json!({}), &[]), json!({})),
    };
    let mut request = Request::new(command);
    request.payload = example;
    Ok(
        json!({"schema_version":"1","host_protocol":"execution-host/1","command":command,"status":"supported","request":{"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","additionalProperties":false,"required":["context","command","payload"],"properties":{"context":context(),"command":{"const":command},"payload":payload}},"response":{"type":"object","required":["api_version","request_id","ok","data","error"],"properties":{"api_version":{"const":"1"},"request_id":{"type":"string"},"ok":{"type":"boolean"},"data":{},"error":{"type":["object","null"]}}},"example":request,"semantics":{"submit":"admitted only","retry":"unobserved request disposition, not operation success","terminal":"original response retained; response_available/envelope_valid remain separate","ack":"explicit terminal retention release","automatic_retry":false,"automatic_ack":false},"error_codes":ErrorCode::ALL}),
    )
}

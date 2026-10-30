use crate::application::api::{ApiError, ErrorCode};
use serde_json::{Value, json};
fn object(properties: Value, required: &[&str]) -> Value {
    json!({"type":"object","additionalProperties":false,"properties":properties,"required":required})
}
fn decimal() -> Value {
    json!({"type":"string","pattern":"^[0-9]+$","description":"Decimal u64 string; checked range"})
}
fn id() -> Value {
    json!({"type":"string","minLength":1,"maxLength":80,"pattern":"^[A-Za-z0-9_.-]+$","not":{"enum":[".",".."]}})
}
fn sha() -> Value {
    json!({"type":"string","pattern":"^[a-f0-9]{64}$"})
}
fn task() -> Value {
    json!({"enum":["correction","cards","highlights"]})
}
fn nullable(value: Value) -> Value {
    json!({"anyOf":[value,{"type":"null"}]})
}
fn text(maximum: usize) -> Value {
    json!({"type":"string","minLength":1,"maxLength":maximum,"description":"Runtime limits are UTF-8 byte bounds; nonblank and control characters are validated"})
}
fn historical() -> Value {
    object(
        json!({"schema_version":{"const":"1"},"reference":object(json!({"sha256":sha(),"bytes":{"type":"integer","minimum":1,"maximum":262144},"item_id":id()}),&["sha256","bytes","item_id"])}),
        &["schema_version", "reference"],
    )
}
pub(crate) fn target() -> Value {
    object(
        json!({"schema_version":{"const":"1"},"kind":{"const":"accepted_output"},"proposal_id":id(),"task":task(),"draft_revision":decimal(),"output_revision":decimal(),"hash_format":{"const":"accepted_typed_output_v1"},"output_sha256":sha(),"source_id":id(),"source_revision":decimal(),"source_sha256":sha(),"generated_raw_sha256":sha(),"generated_raw_bytes":decimal(),"historical":nullable(historical())}),
        &[
            "schema_version",
            "kind",
            "proposal_id",
            "task",
            "draft_revision",
            "output_revision",
            "hash_format",
            "output_sha256",
            "source_id",
            "source_revision",
            "source_sha256",
            "generated_raw_sha256",
            "generated_raw_bytes",
        ],
    )
}
fn criteria_identity() -> Value {
    object(
        json!({"criteria_id":id(),"version":id(),"task":task(),"purpose":text(4096),"content_sha256":sha()}),
        &[
            "criteria_id",
            "version",
            "task",
            "purpose",
            "content_sha256",
        ],
    )
}
fn criteria_document() -> Value {
    object(
        json!({"schema_version":{"const":"1"},"criteria_id":id(),"version":id(),"task":task(),"purpose":text(4096),"clauses":{"type":"array","minItems":1,"maxItems":32,"items":object(json!({"criterion_id":id(),"how_to_review":text(4096),"expected":text(4096)}),&["criterion_id","how_to_review","expected"])}}),
        &[
            "schema_version",
            "criteria_id",
            "version",
            "task",
            "purpose",
            "clauses",
        ],
    )
}
fn decision_input() -> Value {
    object(
        json!({"decision_id":id(),"target":target(),"criteria":criteria_identity(),"verdict":{"enum":["usable","needs_changes","deferred"]},"note":text(16384),"declared_reviewer":object(json!({"label":text(256),"reviewer_id":nullable(id())}),&["label"]),"claimed_review_unix_ms":nullable(decimal())}),
        &[
            "decision_id",
            "target",
            "criteria",
            "verdict",
            "note",
            "declared_reviewer",
        ],
    )
}
fn stored() -> Value {
    object(
        json!({"decision_sha256":sha(),"decision":object(json!({"schema_version":{"const":"1"},"input":decision_input(),"criteria_document":criteria_document(),"observed_record_time":object(json!({"unix_ms":decimal(),"clock":{"const":"local_system_unix_ms"}}),&["unix_ms","clock"]),"scope":{"const":"accepted_output_only"},"automatic_application_enabled":{"const":false},"counts_as_model_sample":{"const":false}}),&["schema_version","input","criteria_document","observed_record_time","scope","automatic_application_enabled","counts_as_model_sample"])}),
        &["decision_sha256", "decision"],
    )
}
fn payload(command: &str) -> Value {
    if command == "proposals.record_output_decision" {
        object(
            json!({"proposal_id":id(),"decision":decision_input()}),
            &["proposal_id", "decision"],
        )
    } else {
        object(json!({"proposal_id":id()}), &["proposal_id"])
    }
}
fn response(command: &str) -> Value {
    match command {
        "proposals.inspect_output_review_target" => object(
            json!({"schema_version":{"const":"1"},"state":crate::application::proposals::schema::response_type("proposals.inspect"),"target":target(),"criteria_document":criteria_document(),"criteria_identity":criteria_identity(),"evidence_observation":{"enum":["not_historically_bound","historical_binding_reverified_in_session"]}}),
            &[
                "schema_version",
                "state",
                "target",
                "criteria_document",
                "criteria_identity",
                "evidence_observation",
            ],
        ),
        "proposals.record_output_decision" => stored(),
        _ => object(
            json!({"records":{"type":"array","maxItems":256,"items":stored()},"current_evidence":{"const":"not_checked"}}),
            &["records", "current_evidence"],
        ),
    }
}
pub fn describe(input: &Value, available: bool) -> Result<Value, ApiError> {
    if input.as_object().is_none_or(|payload| payload.len() != 1) {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Output review schema payload must contain command only",
        ));
    }
    let command = input
        .get("command")
        .and_then(Value::as_str)
        .filter(|command| super::is_command(command))
        .ok_or_else(|| ApiError::new(ErrorCode::InvalidArgument, "Unknown output review schema"))?;
    let mut context = crate::application::api::schema::context_schema();
    context["properties"]["idempotency_key"] = json!({"type":"null"});
    context["properties"]["timeout_ms"] = json!({"type":"null"});
    if command == "proposals.record_output_decision" {
        context["required"] = json!(["request_id", "api_version", "expected_revision"]);
        context["properties"]["expected_revision"] = decimal();
    } else {
        context["properties"]["expected_revision"] = json!({"type":"null"});
    }
    Ok(
        json!({"schema_version":"1","command":command,"status":if available{"supported"}else{"unsupported"},"request":object(json!({"context":context,"command":{"const":command},"payload":payload(command)}),&["context","command","payload"]),"response":object(json!({"api_version":{"const":"1"},"request_id":{"type":"string"},"ok":{"type":"boolean"},"data":nullable(response(command)),"error":{"type":["object","null"]}}),&["api_version","request_id","ok","data","error"]),"error_codes":ErrorCode::ALL,"scope":"accepted_output_only","technical_acceptance_is_quality_approval":false,"automatic_application_enabled":false,"counts_as_model_sample":false,"record_retry":"current target/criteria/evidence first; identical decision_id/request returns original body and observed time","historical_evidence":"bound inspect and record require explicitly configured pinned verifier; history does not reverify current evidence","time_scope":"declared reviewer/claimed time and adapter local observed time are distinct; neither authenticates identity or time"}),
    )
}
pub fn capabilities(value: &mut Value, available: bool) {
    if let Some(commands) = value["commands"].as_array_mut() {
        for command in commands {
            if command["command"].as_str().is_some_and(super::is_command) {
                command["status"] = json!(if available {
                    "supported"
                } else {
                    "unsupported"
                });
            }
        }
    }
    value["output_reviews"] = json!({"available":available,"schema_version":"1","target_kind":"accepted_output","hash_format":"accepted_typed_output_v1","explicit_decision_required":true,"record_expected_revision":"current_output_revision","history_current_evidence":"not_checked","automatic_application_enabled":false,"counts_as_model_sample":false});
}

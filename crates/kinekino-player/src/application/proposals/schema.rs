use crate::application::api::{ApiError, ErrorCode};
use crate::domain::proposals::{
    DOCUMENT_VERSIONS, HISTORICAL_BINDING_VERSION, HISTORICAL_DOCUMENT_VERSION,
    LEGACY_DOCUMENT_VERSIONS,
};
use serde_json::{Value, json};
fn object(properties: Value, required: &[&str]) -> Value {
    json!({"type":"object","additionalProperties":false,"properties":properties,"required":required})
}
fn decimal() -> Value {
    json!({"type":"string","pattern":"^[0-9]+$","description":"Decimal u64 string; range checked"})
}
fn text() -> Value {
    json!({"type":"string","minLength":1,"maxLength":65536})
}
fn identifier() -> Value {
    json!({"type":"string","minLength":1,"maxLength":80,"pattern":"^[A-Za-z0-9_.-]+$","not":{"enum":[".",".."]}})
}
fn generated_type() -> Value {
    object(
        json!({"task":{"enum":["correction","cards","highlights"]},"raw_text":{"type":"string","maxLength":262144}}),
        &["task", "raw_text"],
    )
}
fn companion_reference() -> Value {
    object(
        json!({"sha256":{"type":"string","pattern":"^[a-f0-9]{64}$"},"bytes":{"type":"integer","minimum":1,"maximum":262144},"item_id":identifier()}),
        &["sha256", "bytes", "item_id"],
    )
}
fn historical_binding() -> Value {
    object(
        json!({"schema_version":{"const":HISTORICAL_BINDING_VERSION},"reference":companion_reference()}),
        &["schema_version", "reference"],
    )
}
pub(crate) fn response_type(command: &str) -> Value {
    if command == super::source_context::COMMAND {
        return super::source_context::response_schema();
    }
    if matches!(command, "proposals.inspect" | "proposals.import_historical") {
        let receipt = json!({"anyOf":[response_type("proposals.accept"),{"type":"null"}]});
        let mut document = response_type("proposals.edit");
        document["properties"]["schema_version"] = json!({"const":HISTORICAL_DOCUMENT_VERSION});
        let bound = object(
            json!({"document":document,"acceptance":receipt,"historical":historical_binding()}),
            &["document", "acceptance", "historical"],
        );
        if command == "proposals.import_historical" {
            return bound;
        }
        let legacy = object(
            json!({"document":response_type("proposals.create"),"acceptance":receipt,"historical":{"type":"null"}}),
            &["document", "acceptance"],
        );
        return json!({"oneOf":[legacy,bound]});
    }
    if command == "proposals.accept" {
        object(
            json!({"proposal_id":identifier(),"draft_revision":decimal(),"source_id":identifier(),"source_revision":decimal(),"source_sha256":{"type":"string","pattern":"^[a-f0-9]{64}$"},"output_revision":decimal(),"output":payload_type(),"automatic_application_enabled":{"const":false}}),
            &[
                "proposal_id",
                "draft_revision",
                "source_id",
                "source_revision",
                "source_sha256",
                "output_revision",
                "output",
                "automatic_application_enabled",
            ],
        )
    } else {
        let mut document = object(
            json!({"schema_version":{"enum":LEGACY_DOCUMENT_VERSIONS},"proposal_id":identifier(),"source":source_type(),"generated":generated_type(),"draft_revision":decimal(),"state":{"enum":["editable","needs_repair"]},"draft":{"anyOf":[payload_type(),{"type":"null"}]},"import_error":{"type":["string","null"]}}),
            &[
                "schema_version",
                "proposal_id",
                "source",
                "generated",
                "draft_revision",
                "state",
                "draft",
                "import_error",
            ],
        );
        if command == "proposals.edit" {
            document["properties"]["schema_version"] = json!({"enum":DOCUMENT_VERSIONS});
        }
        document
    }
}
pub fn payload_type() -> Value {
    let refs = json!({"type":"array","minItems":1,"maxItems":4096,"uniqueItems":true,"items":{"type":"string"}});
    let card = object(
        json!({"front":text(),"back":text(),"source_refs":refs}),
        &["front", "back", "source_refs"],
    );
    let highlight = object(
        json!({"title":text(),"source_refs":refs}),
        &["title", "source_refs"],
    );
    let mut variants = [
        ("correction", "edits", cue_edit_type()),
        ("cards", "cards", card),
        ("highlights", "highlights", highlight),
    ]
    .into_iter()
    .map(|(task, key, items)| {
        let mut p = json!({"task":{"const":task}});
        p[key] = json!({"type":"array","minItems":1,"maxItems":4096,"items":items});
        object(p, &["task", key])
    })
    .collect::<Vec<_>>();
    let edit = object(
        json!({"source_ref":document_reference_type(),"text":text()}),
        &["source_ref", "text"],
    );
    variants.push(object(json!({"task":{"const":"document_correction"},"edits":{"type":"array","minItems":1,"maxItems":4096,"items":edit}}),&["task","edits"]));
    json!({"oneOf":variants})
}
fn cue_edit_type() -> Value {
    object(
        json!({"cue_id":identifier(),"text":text()}),
        &["cue_id", "text"],
    )
}
pub(crate) fn cue_edits_type() -> Value {
    json!({"type":"array","minItems":1,"maxItems":4096,"items":cue_edit_type()})
}
fn cue_source_type() -> Value {
    let nullable_time = json!({"anyOf":[decimal(),{"type":"null"}]});
    let cue = object(
        json!({"cue_id":identifier(),"text":text(),"start_us":nullable_time,"end_us":nullable_time}),
        &["cue_id", "text", "start_us", "end_us"],
    );
    object(
        json!({"source_id":identifier(),"source_revision":decimal(),"source_sha256":{"type":"string","pattern":"^[a-f0-9]{64}$"},"cues":{"type":"array","minItems":1,"maxItems":4096,"items":cue}}),
        &["source_id", "source_revision", "source_sha256", "cues"],
    )
}
fn document_reference_type() -> Value {
    object(
        json!({"source_asset_sha256":{"type":"string","pattern":"^[a-f0-9]{64}$"},"outer_pointer":{"const":"/messages/0/content"},"extraction":{"const":"asr_request_after_no_think_lf_v1"},"inner_pointer":{"type":"string","pattern":"^/(0|[1-9][0-9]*)$"}}),
        &[
            "source_asset_sha256",
            "outer_pointer",
            "extraction",
            "inner_pointer",
        ],
    )
}
pub fn source_type() -> Value {
    let selector = object(
        json!({"outer_pointer":{"const":"/messages/0/content"},"extraction":{"const":"asr_request_after_no_think_lf_v1"}}),
        &["outer_pointer", "extraction"],
    );
    let item = object(
        json!({"inner_pointer":{"type":"string","pattern":"^/(0|[1-9][0-9]*)$"},"text":text()}),
        &["inner_pointer", "text"],
    );
    let document = object(
        json!({"schema_version":{"const":"2"},"source_id":identifier(),"source_revision":decimal(),"source_sha256":{"type":"string","pattern":"^[a-f0-9]{64}$"},"asset_utf8":{"type":"string","minLength":1,"maxLength":crate::domain::proposals::MAX_DOCUMENT_BYTES,"description":"Exact UTF8 asset; SHA256 is verified before publication; byte bound enforced"},"selector":selector,"texts":{"type":"array","minItems":1,"maxItems":4096,"items":item}}),
        &[
            "schema_version",
            "source_id",
            "source_revision",
            "source_sha256",
            "asset_utf8",
            "selector",
            "texts",
        ],
    );
    json!({"oneOf":[cue_source_type(),document]})
}
pub fn describe(input: &Value, available: bool) -> Result<Value, ApiError> {
    describe_with_list(input, available, false)
}
pub fn describe_with_list(
    input: &Value,
    available: bool,
    list_available: bool,
) -> Result<Value, ApiError> {
    if input.as_object().is_none_or(|v| v.len() != 1) {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "schema requires only command",
        ));
    }
    let command = input["command"]
        .as_str()
        .filter(|s| super::is_command(s))
        .ok_or_else(|| ApiError::new(ErrorCode::InvalidArgument, "Unknown proposal schema"))?;
    if command == super::source_context::COMMAND {
        return super::source_context::describe(input, false);
    }
    if command == "proposals.list" {
        let mut context = crate::application::api::schema::context_schema();
        for key in ["expected_revision", "idempotency_key", "timeout_ms"] {
            context["properties"][key] = json!({"type":"null"});
        }
        return Ok(
            json!({"schema_version":"1","command":command,"status":if available && list_available {"supported"}else{"unsupported"},
            "request":object(json!({"context":context,"command":{"const":command},"payload":super::list::request_schema()}), &["context","command","payload"]),
            "response":object(json!({"api_version":{"const":"1"},"request_id":{"type":"string"},"ok":{"type":"boolean"},"data":{"anyOf":[super::list::response_schema(),{"type":"null"}]},"error":{"type":["object","null"]}}), &["api_version","request_id","ok","data","error"]),
            "error_codes":ErrorCode::ALL,"snapshot":"bounded_full_manifest; concurrent modification invalidates continuation","current_evidence":"not_checked","selection_authority":"none","in_flight_cancel":false,"sdk_execution_queued_cancel":true,"response_max_bytes":super::list::MAX_RESPONSE_BYTES}),
        );
    }
    let id = identifier();
    let mut properties = json!({"proposal_id":id});
    let mut required = vec!["proposal_id"];
    match command {
        "proposals.import_historical" => {
            properties["source_id"] = identifier();
            properties["source_revision"] = decimal();
            properties["reference"] = companion_reference();
            required.extend(["source_id", "source_revision", "reference"]);
        }
        "proposals.create" => {
            properties["source"] = source_type();
            properties["generated"] = object(
                json!({"task":{"enum":["correction","cards","highlights"]},"raw_text":{"type":"string","maxLength":262144}}),
                &["task", "raw_text"],
            );
            required.extend(["source", "generated"]);
        }
        "proposals.edit" => {
            properties["draft"] = payload_type();
            required.push("draft");
        }
        "proposals.accept" => {
            properties["target_source_revision"] = decimal();
            required.push("target_source_revision");
        }
        _ => {}
    }
    let mut context = crate::application::api::commands::RequestContext::default();
    if matches!(command, "proposals.edit" | "proposals.accept") {
        context.expected_revision = Some("1".into());
    }
    let mut context_type = crate::application::api::schema::context_schema();
    context_type["properties"]["idempotency_key"] = json!({"type":"null"});
    context_type["properties"]["timeout_ms"] = json!({"type":"null"});
    if matches!(command, "proposals.edit" | "proposals.accept") {
        context_type["required"] = json!(["request_id", "api_version", "expected_revision"]);
        context_type["properties"]["expected_revision"] = decimal();
    } else {
        context_type["properties"]["expected_revision"] = json!({"type":"null"});
    }
    Ok(
        json!({"schema_version":"1","command":command,"status":if available{"supported"}else{"unsupported"},"request":{"type":"object","additionalProperties":false,"required":["context","command","payload"],"properties":{"context":context_type,"command":{"const":command},"payload":object(properties,&required)}},"response":{"type":"object","required":["api_version","request_id","ok","data","error"],"properties":{"api_version":{"const":"1"},"request_id":{"type":"string"},"ok":{"type":"boolean"},"data":{"anyOf":[response_type(command),{"type":"null"}]},"error":{"type":["object","null"]}}},"error_codes":ErrorCode::ALL,"proposal_payload_schema":payload_type(),"source_snapshot_schema":source_type(),"context_example":context,"output_scope":"immutable_typed_output_revision; no subtitle/card rendering or project overwrite","accept_retry":"same draft revision and source revision return existing receipt while it remains current; otherwise revision_conflict","automatic_application_enabled":false}),
    )
}
pub fn capabilities(value: &mut Value, available: bool, historical: bool) {
    capabilities_with_list(value, available, historical, false)
}
pub fn capabilities_with_list(
    value: &mut Value,
    available: bool,
    historical: bool,
    list_available: bool,
) {
    if let Some(commands) = value["commands"].as_array_mut() {
        for command in commands {
            if command["command"].as_str().is_some_and(super::is_command) {
                command["status"] = json!(if available
                    && (command["command"] != "proposals.list" || list_available)
                    && (command["command"] != "proposals.import_historical" || historical)
                    && command["command"] != super::source_context::COMMAND
                {
                    "supported"
                } else {
                    "unsupported"
                });
            }
        }
    }
    value["proposals"] = json!({"backend_available":available,"schema_version":"1","document_versions":DOCUMENT_VERSIONS,"source_profiles":["cue_snapshot_v1","asr_request_after_no_think_lf_v1"],"tasks":["correction","cards","highlights"],"draft_types":["correction","document_correction","cards","highlights"],"immutable_source_and_generation":true,"explicit_accept_required":true,"model_quality_approval_required":false,"automatic_application_enabled":false,"output_scope":"immutable_typed_output_revision","historical":{"available":available && historical,"document_version":HISTORICAL_DOCUMENT_VERSION,"binding_version":HISTORICAL_BINDING_VERSION,"explicit_pinned_verifier_required":true,"synchronous_verification":true,"quality_approved":false}});
    value["proposals"]["listing"] = json!({"available":available && list_available,"current_evidence":"not_checked","selection_authority":"none","in_flight_cancel":false,"sdk_execution_queued_cancel":true,"snapshot":"bounded_full_manifest","response_max_bytes":super::list::MAX_RESPONSE_BYTES});
}

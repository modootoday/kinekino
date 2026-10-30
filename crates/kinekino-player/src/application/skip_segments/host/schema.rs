use super::{SkipHostCapability, is_command};
use crate::application::api::{ApiError, ErrorCode};
use serde_json::{Value, json};
fn object(properties: Value, required: &[&str]) -> Value {
    json!({"type":"object","additionalProperties":false,"properties":properties,"required":required})
}
fn decimal() -> Value {
    json!({"type":"string","pattern":"^[0-9]+$","description":"Decimal u64; checked by application"})
}
fn id() -> Value {
    json!({"type":"string","minLength":1,"maxLength":80,"pattern":"^[A-Za-z0-9_.-]+$","not":{"enum":[".",".."]}})
}
fn binding() -> Value {
    object(
        json!({"engine_instance_id":decimal(),"resolution_artifact_id":id()}),
        &["engine_instance_id", "resolution_artifact_id"],
    )
}
fn reference() -> Value {
    object(
        json!({"skip_session_id":id(),"binding":binding()}),
        &["skip_session_id", "binding"],
    )
}
fn policy() -> Value {
    let mode = json!({"enum":["off","manual","auto"]});
    object(
        json!({"opening":mode,"ending":mode}),
        &["opening", "ending"],
    )
}
fn intent() -> Value {
    json!({"enum":["manual","automatic"]})
}
fn input(command: &str) -> Value {
    match command {
        "segments.edit" => object(
            json!({"segment_id":id(),"kind":{"enum":["opening","ending"]},"in_us":decimal(),"out_us":decimal()}),
            &["segment_id", "kind", "in_us", "out_us"],
        ),
        "segments.accept_skip" => object(
            json!({"segment_id":id(),"annotation_revision":decimal()}),
            &["segment_id", "annotation_revision"],
        ),
        "segments.set_skip_policy" => object(json!({"policy":policy()}), &["policy"]),
        "segments.evaluate" => object(
            json!({"segment_id":id(),"intent":intent()}),
            &["segment_id", "intent"],
        ),
        "segments.skip" => object(
            json!({"segment_id":id(),"intent":intent(),"cancelled":{"type":"boolean","default":false}}),
            &["segment_id", "intent"],
        ),
        _ => {
            json!({"oneOf":[object(json!({"action":{"const":"cancel"},"segment_id":id()}), &["action","segment_id"]),object(json!({"action":{"enum":["manual_seek","begin_new_pass"]}}), &["action"])]})
        }
    }
}
fn payload(command: &str) -> Value {
    match command {
        super::SKIP_EXPORT => object(
            json!({"issuer_scope":export_scope(),"session":export_session(),"input":input("segments.skip")}),
            &["issuer_scope", "session", "input"],
        ),
        super::EXPORT_LIST => object(
            json!({"issuer_scope":export_scope(),"origin":export_origin()}),
            &["issuer_scope", "origin"],
        ),
        super::EXPORT_PEEK | super::EXPORT_CONSUME => object(
            json!({"issuer_scope":export_scope(),"origin":export_origin(),"reference":export_ref()}),
            &["issuer_scope", "origin", "reference"],
        ),
        "segments.open" => object(json!({"binding":binding()}), &["binding"]),
        "segments.inspect" | "segments.close" => reference(),
        _ => object(
            json!({"session":reference(),"input":input(command)}),
            &["session", "input"],
        ),
    }
}
fn snapshot() -> Value {
    let segment = object(
        json!({"id":id(),"revision":decimal(),"kind":{"enum":["opening","ending"]},"in_us":decimal(),"out_us":decimal(),"origin":{"const":"manual"},"timebase":{"const":"engine_playback"},"verified_revision":{"anyOf":[decimal(),{"type":"null"}]}}),
        &[
            "id",
            "revision",
            "kind",
            "in_us",
            "out_us",
            "origin",
            "timebase",
            "verified_revision",
        ],
    );
    object(
        json!({"binding":binding(),"revision":decimal(),"pass_id":decimal(),"policy":policy(),"segments":{"type":"array","items":segment},"durable":{"const":false},"detection_supported":{"const":false}}),
        &[
            "binding",
            "revision",
            "pass_id",
            "policy",
            "segments",
            "durable",
            "detection_supported",
        ],
    )
}
fn decision() -> Value {
    json!({"oneOf":[object(json!({"state":{"const":"blocked"},"reason":{"enum":["invalid_annotation","unverified","overlap","off","manual_only","observation_unknown","seek_unknown","not_seekable","duration_unknown","position_unknown","outside_duration","paused_or_unknown","outside_segment","overridden","already_attempted"]}}), &["state","reason"]),object(json!({"state":{"const":"notice"},"remaining_us":decimal()}), &["state","remaining_us"]),object(json!({"state":{"const":"eligible"},"target_us":decimal()}), &["state","target_us"])]})
}
fn cancel_status() -> Value {
    json!({"enum":["before_admission","too_late_or_unobserved"]})
}
fn response(command: &str) -> Value {
    match command {
        super::SKIP_EXPORT => export_submit_response(),
        super::EXPORT_LIST => object(
            json!({"scope":export_scope(),"items":{"type":"array","maxItems":64,"items":export_item()}}),
            &["scope", "items"],
        ),
        super::EXPORT_PEEK | super::EXPORT_CONSUME => object(
            json!({"item":export_item(),"write_committed":{"type":"boolean"},"failure":{"anyOf":[export_failure(false),{"type":"null"}]}}),
            &["item", "write_committed", "failure"],
        ),
        "segments.open" => object(
            json!({"session":reference(),"snapshot":snapshot(),"reused":{"type":"boolean"}}),
            &["session", "snapshot", "reused"],
        ),
        "segments.evaluate" => object(
            json!({"snapshot":snapshot(),"decision":decision()}),
            &["snapshot", "decision"],
        ),
        "segments.skip" => {
            let result = json!({"oneOf":[object(json!({"state":{"const":"not_submitted"},"decision":decision()}), &["state","decision"]),object(json!({"state":{"const":"cancelled"},"session_revision":decimal(),"status":cancel_status()}), &["state","session_revision","status"]),object(json!({"state":{"const":"admitted"},"session_revision":decimal(),"receipt":crate::application::playback_control::response_schema()}), &["state","session_revision","receipt"])]});
            object(
                json!({"snapshot":snapshot(),"result":result}),
                &["snapshot", "result"],
            )
        }
        "segments.override" => object(
            json!({"snapshot":snapshot(),"cancel_status":{"anyOf":[cancel_status(),{"type":"null"}]}}),
            &["snapshot", "cancel_status"],
        ),
        "segments.close" => object(
            json!({"skip_session_id":id(),"closed":{"const":true},"playback_closed":{"const":false}}),
            &["skip_session_id", "closed", "playback_closed"],
        ),
        _ => snapshot(),
    }
}
pub fn supported(command: &str, capability: SkipHostCapability) -> bool {
    !super::is_export_command(command)
        && is_command(command)
        && capability.sessions
        && (command != "segments.skip" || capability.seek_submission)
}
pub fn capabilities(value: &mut Value, capability: SkipHostCapability) {
    if let Some(commands) = value["commands"].as_array_mut() {
        for command in commands {
            if let Some(name) = command["command"].as_str().filter(|name| is_command(name)) {
                command["status"] = json!(if supported(name, capability) {
                    "supported"
                } else {
                    "unsupported"
                });
            }
        }
    }
    value["skip_sessions"] = json!({"available":capability.sessions,"seek_submission_available":capability.seek_submission,"durable":false,"detection_supported":false,"automatic_host_tick_supported":false,"seek_landing_observed":false,"inspection_scope":"stored ephemeral state; not proof of current native playback","lifetime":"same explicit live endpoint host only; restart invalidates session ids"});
}
pub(crate) fn capabilities_with_export(
    value: &mut Value,
    capability: SkipHostCapability,
    export: super::SkipExportCapability,
) {
    capabilities(value, capability);
    let available = capability.sessions && export.available;
    if let Some(commands) = value["commands"].as_array_mut() {
        for command in commands {
            if command["command"]
                .as_str()
                .is_some_and(super::is_export_command)
            {
                command["status"] = json!(if available {
                    "supported"
                } else {
                    "unsupported"
                });
            }
        }
    }
    value["skip_exports"] = json!({"available":available,"issuer_scope":export.issuer_scope,"json_bytes":crate::application::playback_session::v1::NATIVE_EXPORT_JSON_BYTES,"wire_bytes":crate::application::playback_session::v1::NATIVE_EXPORT_WIRE_BYTES,"activation":if available {"explicit original Sessions issuer and bounded transport"} else {"requires explicit producer and transport integration acceptance"}});
}
pub(crate) fn describe_with_export(
    payload: &Value,
    capability: SkipHostCapability,
    export: super::SkipExportCapability,
) -> Result<Value, ApiError> {
    let mut value = describe(payload, capability)?;
    if payload["command"]
        .as_str()
        .is_some_and(super::is_export_command)
    {
        value["status"] = json!(if capability.sessions && export.available {
            "supported"
        } else {
            "unsupported"
        });
    }
    Ok(value)
}
pub fn describe(payload: &Value, capability: SkipHostCapability) -> Result<Value, ApiError> {
    let command = payload
        .get("command")
        .and_then(Value::as_str)
        .filter(|command| is_command(command))
        .ok_or_else(|| ApiError::new(ErrorCode::InvalidArgument, "Unknown skip schema command"))?;
    if payload.as_object().is_none_or(|fields| fields.len() != 1) {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Skip schema accepts command only",
        ));
    }
    let mut context = crate::application::api::schema::context_schema();
    context["properties"]["idempotency_key"] = json!({"type":"null"});
    context["properties"]["timeout_ms"] = json!({"type":"null"});
    if matches!(
        command,
        "segments.open"
            | "segments.inspect"
            | "segments.evaluate"
            | super::EXPORT_LIST
            | super::EXPORT_PEEK
            | super::EXPORT_CONSUME
    ) {
        context["properties"]["expected_revision"] = json!({"type":"null"});
    } else {
        context["required"] = json!(["request_id", "api_version", "expected_revision"]);
        context["properties"]["expected_revision"] = if command == super::SKIP_EXPORT {
            positive_decimal()
        } else {
            decimal()
        };
    }
    let request = object(
        json!({"context":context,"command":{"const":command},"payload":self::payload(command)}),
        &["context", "command", "payload"],
    );
    let envelope = object(
        json!({"api_version":{"const":"1"},"request_id":{"type":"string"},"ok":{"type":"boolean"},"data":{"anyOf":[response(command),{"type":"null"}]},"error":{"type":["object","null"]}}),
        &["api_version", "request_id", "ok", "data", "error"],
    );
    Ok(
        json!({"schema_version":"1","command":command,"status":if supported(command,capability){"supported"}else{"unsupported"},"request":request,"response":envelope,"error_codes":ErrorCode::ALL,"durable":false,"detection_supported":false,"automatic_host_tick_supported":false,"completion":"existing admission receipt only; no observed landing"}),
    )
}

fn positive_decimal() -> Value {
    json!({"type":"string","pattern":"^[0-9]+$","not":{"pattern":"^0+$"},"maxLength":20,"description":"Positive decimal u64 checked by application"})
}
fn export_scope() -> Value {
    json!({"type":"string","minLength":37,"maxLength":37,"pattern":"^nsi1\\.[0-9a-f]{32}$","not":{"const":"nsi1.00000000000000000000000000000000"},"description":"SDK strict v1 nonzero nonce codec"})
}
fn export_ref() -> Value {
    json!({"type":"string","minLength":54,"maxLength":54,"pattern":"^nsr1\\.[0-9a-f]{32}\\.[0-9a-f]{16}$","not":{"anyOf":[{"pattern":"^nsr1\\.0{32}\\."},{"pattern":"\\.0{16}$"}]},"description":"SDK strict v1 nonzero nonce and row codec"})
}
fn export_session() -> Value {
    let binding = object(
        json!({"engine_instance_id":positive_decimal(),"resolution_artifact_id":digest()}),
        &["engine_instance_id", "resolution_artifact_id"],
    );
    object(
        json!({"skip_session_id":id(),"binding":binding}),
        &["skip_session_id", "binding"],
    )
}
fn digest() -> Value {
    json!({"type":"string","minLength":64,"maxLength":64,"pattern":"^[0-9a-f]{64}$"})
}
fn export_origin() -> Value {
    object(
        json!({"session":export_session(),"pass_id":positive_decimal(),"request_revision":positive_decimal(),"segment_id":id()}),
        &["session", "pass_id", "request_revision", "segment_id"],
    )
}
fn export_metadata() -> Value {
    let origin = object(
        json!({"registry_generation":{"type":"string","maxLength":40},"session":id(),"pass":positive_decimal(),"segment":id(),"request_revision":positive_decimal(),"engine_instance_id":positive_decimal(),"resolution_artifact_id":digest()}),
        &[
            "registry_generation",
            "session",
            "pass",
            "segment",
            "request_revision",
            "engine_instance_id",
            "resolution_artifact_id",
        ],
    );
    let annotation = object(
        json!({"annotation_revision":positive_decimal(),"verified_revision":positive_decimal(),"timebase":{"const":"engine_playback"}}),
        &["annotation_revision", "verified_revision", "timebase"],
    );
    object(
        json!({"origin":origin,"annotation":annotation,"actual_target_us":decimal()}),
        &["origin", "annotation", "actual_target_us"],
    )
}
fn export_item() -> Value {
    let mut schema = object(
        json!({"reference":export_ref(),"metadata":export_metadata(),"state":{"enum":["queued","write_committed","written_awaiting_ack","command_accepted","rejected_before_write","failed","outcome_unobserved"]},"read_mode":{"enum":["standalone","routed","unavailable"]}}),
        &["reference", "metadata", "state", "read_mode"],
    );
    schema["x-max-serialized-bytes"] = json!(1024);
    schema
}
fn export_failure(core: bool) -> Value {
    let mut fields = json!({"code":{"enum":ErrorCode::ALL},"message_prefix":{"type":"string","x-max-utf8-bytes":1024},"original_message_bytes":decimal(),"sha256":digest(),"truncated":{"type":"boolean"}});
    let mut required = vec![
        "code",
        "message_prefix",
        "original_message_bytes",
        "sha256",
        "truncated",
    ];
    if core {
        fields["retryable"] = json!({"type":"boolean"});
        fields["details_omitted"] = json!({"const":true});
        fields["remediation_omitted"] = json!({"const":true});
        required.extend(["retryable", "details_omitted", "remediation_omitted"]);
    } else {
        fields["stage"] =
            json!({"enum":["before_write","write","acknowledgement","connection","deadline"]});
        fields["raw_os_code"] =
            json!({"type":["integer","null"],"minimum":i32::MIN,"maximum":i32::MAX});
        fields["kind"] = json!({"anyOf":[{"type":"string","minLength":1,"maxLength":48,"pattern":"^[A-Za-z]+$"},{"type":"null"}]});
        required.extend(["stage", "raw_os_code", "kind"]);
    }
    object(fields, &required)
}
fn export_tag(state: &str, mut properties: Value, required: &[&str]) -> Value {
    properties["state"] = json!({"const":state});
    let mut fields = required.to_vec();
    fields.push("state");
    object(properties, &fields)
}
fn export_submit_response() -> Value {
    let admitted = json!({"receipt":crate::application::playback_control::response_schema(),"reference":export_ref(),"metadata":export_metadata()});
    let mut with_error = admitted.clone();
    with_error["error"] = export_failure(true);
    let result = json!({"oneOf":[
        export_tag("not_submitted",json!({"decision":decision()}), &["decision"]),
        export_tag("cancelled",json!({"status":cancel_status()}), &["status"]),
        export_tag("admitted",admitted, &["receipt","reference","metadata"]),
        export_tag("admitted_with_error",with_error, &["receipt","reference","metadata","error"]),
        export_tag("no_submission_returned",json!({"error":export_failure(true),"origin":export_origin()}), &["error","origin"])
    ]});
    let state = object(
        json!({"session":export_session(),"revision":positive_decimal(),"pass_id":positive_decimal()}),
        &["session", "revision", "pass_id"],
    );
    object(
        json!({"session_state":state,"result":result}),
        &["session_state", "result"],
    )
}

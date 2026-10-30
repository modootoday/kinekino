//! Request/output schemas share the exact typed session boundary; no runtime settings in input.
use super::{api, sessions::ReviewPlaybackSessions};
use crate::application::api::{ApiError, ErrorCode};
use serde_json::{Value, json};
fn object(properties: Value, required: &[&str]) -> Value {
    json!({"type":"object","additionalProperties":false,"properties":properties,"required":required})
}
fn id() -> Value {
    json!({"type":"string","minLength":1,"maxLength":128,"pattern":"^[A-Za-z0-9_-]+$"})
}
fn decimal() -> Value {
    json!({"type":"string","pattern":"^[0-9]+$","description":"Checked decimal u64 greater than zero"})
}
fn nullable(value: Value) -> Value {
    json!({"anyOf":[value,{"type":"null"}]})
}
fn selection() -> Value {
    let sha = json!({"type":"string","pattern":"^[a-f0-9]{64}$"});
    let case = object(
        json!({"package_sha256":sha,"item_id":id(),"source_request_sha256":sha}),
        &["package_sha256", "item_id", "source_request_sha256"],
    );
    object(
        json!({"case":case,"association_id":id(),"association_revision":decimal()}),
        &["case", "association_id", "association_revision"],
    )
}
fn reference() -> Value {
    object(
        json!({"host_scope":id(),"ordinal":decimal(),"selection":selection()}),
        &["host_scope", "ordinal", "selection"],
    )
}
fn view() -> Value {
    let boolean = json!({"type":"boolean"});
    let failure = object(
        json!({"code":{"enum":ErrorCode::ALL},"message":{"type":"string"}}),
        &["code", "message"],
    );
    let observation = object(
        json!({"process_created":boolean,"connected":boolean,"closing":boolean,"physical_closed":boolean,"family_empty":boolean,"ipc_exited":boolean,"owner_finished":boolean,"engine_instance_id":nullable(decimal()),"has_error":boolean,"failure":nullable(failure),"position_us":nullable(clock()),"duration_us":nullable(clock()),"paused":nullable(boolean.clone()),"volume_percent":nullable(json!({"type":"number"})),"muted":nullable(boolean.clone())}),
        &[
            "process_created",
            "connected",
            "closing",
            "physical_closed",
            "family_empty",
            "ipc_exited",
            "owner_finished",
            "engine_instance_id",
            "has_error",
            "failure",
            "position_us",
            "duration_us",
            "paused",
            "volume_percent",
            "muted",
        ],
    );
    object(
        json!({"session":reference(),"close_requested":boolean,"observation":observation}),
        &["session", "close_requested", "observation"],
    )
}
fn clock() -> Value {
    json!({"type":"string","pattern":"^[0-9]+$","description":"Observed nonnegative engine microseconds; not source PTS"})
}
fn control() -> Value {
    json!({"oneOf":[
        object(json!({"action":{"enum":["play","pause"]}}), &["action"]),
        object(json!({"action":{"const":"seek"},"position_us":clock()}), &["action","position_us"]),
        object(json!({"action":{"const":"set_volume"},"volume_percent":{"type":"number","minimum":0,"maximum":100}}), &["action","volume_percent"]),
        object(json!({"action":{"const":"set_muted"},"muted":{"type":"boolean"}}), &["action","muted"])
    ]})
}
pub fn describe(
    query: &Value,
    available: bool,
    controls_available: bool,
) -> Result<Value, ApiError> {
    let command = query["command"]
        .as_str()
        .filter(|command| api::is_command(command))
        .ok_or_else(|| {
            ApiError::new(
                ErrorCode::InvalidArgument,
                "Unknown review playback schema command",
            )
        })?;
    if query.as_object().is_none_or(|object| object.len() != 1) {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Invalid review playback schema query",
        ));
    }
    let mut context = crate::application::api::schema::context_schema();
    context["properties"]["expected_revision"] = json!({"type":"null"});
    let payload = if command == "reviews.playback.open" {
        context["properties"]["idempotency_key"] = id();
        context["required"]
            .as_array_mut()
            .unwrap()
            .push(json!("idempotency_key"));
        context["properties"]["timeout_ms"] =
            json!({"type":["integer","null"],"minimum":1,"maximum":30000,"default":30000});
        object(json!({"selection":selection()}), &["selection"])
    } else {
        context["properties"]["idempotency_key"] = json!({"type":"null"});
        context["properties"]["timeout_ms"] = json!({"type":"null"});
        if command == "reviews.playback.control" {
            object(
                json!({"session":reference(),"engine_instance_id":decimal(),"control":control()}),
                &["session", "engine_instance_id", "control"],
            )
        } else {
            object(json!({"session":reference()}), &["session"])
        }
    };
    let response = if command == "reviews.playback.control" {
        object(
            json!({"session":reference(),"receipt":object(json!({"acceptance":{"const":"accepted"},"completion":{"const":"unobserved"},"engine_instance_id":decimal()}), &["acceptance","completion","engine_instance_id"])}),
            &["session", "receipt"],
        )
    } else {
        view()
    };
    let supported = available && (command != "reviews.playback.control" || controls_available);
    Ok(
        json!({"schema_version":"1","command":command,"status":if supported {"supported"} else {"unsupported"},
        "request":object(json!({"context":context,"command":{"const":command},"payload":payload}), &["context","command","payload"]),
        "response":object(json!({"api_version":{"const":"1"},"request_id":{"type":"string"},"ok":{"type":"boolean"},"data":nullable(response),"error":{"type":["object","null"]}}), &["api_version","request_id","ok","data","error"]),
        "error_codes":ErrorCode::ALL,"open_completion":"retained_owner_only_poll_status","close_completion":"request_only_poll_physical_exit","replay":"same_host_case_key_returns_original_session_and_current_observation","timeout_scope":"new_owner_admission_only_never_extends_runtime","frame_exact":false,"user_quality_approved":false}),
    )
}
pub fn capabilities(value: &mut Value, sessions: Option<&ReviewPlaybackSessions>) {
    if let Some(commands) = value["commands"].as_array_mut() {
        for command in commands {
            if let Some(name) = command["command"]
                .as_str()
                .filter(|name| api::is_command(name))
            {
                command["status"] = json!(if sessions.is_some()
                    && (name != "reviews.playback.control"
                        || sessions.is_some_and(|sessions| sessions.controls_available()))
                {
                    "supported"
                } else {
                    "unsupported"
                });
            }
        }
    }
    value["review_playback"] = json!({"available":sessions.is_some(),"control_available":sessions.is_some_and(|sessions| sessions.controls_available()),"max_history":sessions.map(|sessions| sessions.max_history()),"max_open_timeout_ms":30000,"control_timeout_ms":2000,"control_completion":"unobserved","play_window":"retained_coverage_observed_clock","boundary_pause":"poll_observed_not_frame_exact","replay_scope":"host_lifetime","physical_close_observation_required":true,"frame_exact":false,"quality_approved":false});
}

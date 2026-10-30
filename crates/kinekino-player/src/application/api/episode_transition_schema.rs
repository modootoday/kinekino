//! Transition request and response schemas derived from shared contracts.
use super::{ApiError, ErrorCode};
use crate::application::jobs::JobState;
use serde_json::{Value, json};

pub fn schema(command: &str, available: bool) -> Result<Value, ApiError> {
    let id = json!({"type":"string","pattern":"^[1-9][0-9]*$","description":"Nonzero u64 decimal string"});
    let job = json!({"type":"string","pattern":"^job-[0-9a-f]{32}$"});
    let handle = object(json!({"transition_id":id}));
    let start = object(json!({
        "expected_engine_instance_id":id,
        "catalog_result_sha256":{"type":"string","pattern":"^[0-9a-f]{64}$"},
        "resolution_submission_key":{"type":"string","description":"Existing resolve job submission key; JobsApi validates 1..128 characters and no NUL. Reusing a key can return its historical completed resolve. This is not whole-transition idempotency."},
        "paused":{"type":"boolean"},"volume_percent":{"type":"number","minimum":0,"maximum":100},
        "execution_timeout_ms":{"type":"string","pattern":"^[1-9][0-9]*$","description":"Resolve execution budget 1..120000ms; application validates range. Not engine replacement completion timeout."}
    }));
    let (payload, data) = match command {
        "playback.previous" | "playback.next" => (
            start,
            object(
                json!({"transition_id":id,"resolution_job_id":job,"engine_replaced":{"const":false}}),
            ),
        ),
        "playback.transition.observe" => (
            handle,
            object(
                json!({"transition_id":id,"resolution_job_id":job,"attempt":id,"job_state":{"enum":[JobState::Queued,JobState::Running,JobState::Cancelling,JobState::Verifying,JobState::Completed,JobState::NeedsInput,JobState::Failed,JobState::Cancelled,JobState::Interrupted]},"engine_replaced":{"const":false}}),
            ),
        ),
        "playback.transition.release" => (
            handle,
            object(json!({"resolution_job_id":job,"cancellation_confirmed":{"const":false}})),
        ),
        "playback.transition.commit" => (
            handle,
            object(
                json!({"previous":native_data("playback.close")?,"current":native_data("playback.open")?}),
            ),
        ),
        _ => {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Unknown transition schema command",
            ));
        }
    };
    let mut context = super::schema::context_schema();
    for key in ["expected_revision", "idempotency_key", "timeout_ms"] {
        context["properties"][key] = json!({"type":"null"});
    }
    let mut response = super::native_response_schema::response("playback.open");
    response["properties"]["data"]["anyOf"][0] = data;
    Ok(json!({
        "schema_version":"1","command":command,"status":if available {"experimental"} else {"unsupported"},
        "request":object(json!({"context":context,"command":{"const":command},"payload":payload})),
        "response":response,"error_codes":ErrorCode::ALL,
        "scope":"verified stored-list adjacency -> existing durable resolve -> explicit verified owner commit",
        "process_started_is_media_loaded":false,"automatic_commit":false,"automatic_retry":false,
        "release_is_job_cancelled":false,"host_restart_resumes_transition":false,
        "observe_after_commit":"handle consumed; use playback.engine with the returned new instance identity",
        "live_playback_verification":"Independently observe native state/time progress and verify captured frame; completed resolve alone is not playback"
    }))
}
fn object(properties: Value) -> Value {
    let required: Vec<_> = properties.as_object().unwrap().keys().cloned().collect();
    json!({"type":"object","additionalProperties":false,"required":required,"properties":properties})
}
fn native_data(command: &str) -> Result<Value, ApiError> {
    super::native_response_schema::response(command)
        .pointer("/properties/data/anyOf/0")
        .filter(|schema| schema.is_object())
        .cloned()
        .ok_or_else(|| {
            ApiError::new(
                ErrorCode::Internal,
                "Native response schema data contract unavailable",
            )
        })
}

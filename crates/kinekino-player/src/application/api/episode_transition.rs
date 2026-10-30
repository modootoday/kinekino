//! Request adapter; application owns transition policy and engine effects.
use crate::{
    application::{
        api::{ApiError, ErrorCode, Request},
        playback_session::transition::{
            BeginAdjacentResolution, EpisodeDirection, EpisodeTransitions,
        },
    },
    domain::api_values::DecimalU64,
};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Start {
    expected_engine_instance_id: DecimalU64,
    catalog_result_sha256: String,
    resolution_submission_key: String,
    paused: bool,
    volume_percent: f64,
    execution_timeout_ms: DecimalU64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Handle {
    transition_id: DecimalU64,
}

pub const COMMANDS: &[&str] = &[
    "playback.previous",
    "playback.next",
    "playback.transition.observe",
    "playback.transition.commit",
    "playback.transition.release",
];
pub fn is_command(command: &str) -> bool {
    COMMANDS.contains(&command)
}
pub fn capabilities(value: &mut Value, available: bool) {
    if let Some(commands) = value["commands"].as_array_mut() {
        for command in commands {
            if command["command"].as_str().is_some_and(is_command) {
                command["status"] = serde_json::json!(if available {
                    "experimental"
                } else {
                    "unsupported"
                });
            }
        }
    }
    value["episode_transitions"] = serde_json::json!({
        "available":available,"explicit_combined_host":true,"max_pending":1,
        "whole_transition_idempotency":false,"automatic_commit":false,
        "automatic_retry":false,"restart_resumption":false,
        "resolve_completed_is_playback":false,"release_is_cancelled":false,
        "commit_means":"old owned engine exit confirmed and new process created; media readiness observed separately"
    });
}
pub fn execute(request: &Request, transitions: &EpisodeTransitions) -> Result<Value, ApiError> {
    if request.context.expected_revision.is_some()
        || request.context.idempotency_key.is_some()
        || request.context.timeout_ms.is_some()
    {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Whole-transition revision, idempotency and completion timeout are unavailable",
        ));
    }
    match request.command.as_str() {
        "playback.previous" | "playback.next" => {
            let input: Start =
                serde_json::from_value(request.payload.clone()).map_err(|_| invalid())?;
            // Existing JobsApi validates its submission key; no second key policy here.
            let direction = if request.command == "playback.previous" {
                EpisodeDirection::Previous
            } else {
                EpisodeDirection::Next
            };
            encode(transitions.begin(
                BeginAdjacentResolution {
                    expected_instance_id: input.expected_engine_instance_id,
                    catalog_result_sha256: input.catalog_result_sha256,
                    direction,
                    paused: input.paused,
                    volume_percent: input.volume_percent,
                    execution_timeout_ms: input.execution_timeout_ms,
                },
                &input.resolution_submission_key,
            )?)
        }
        "playback.transition.observe"
        | "playback.transition.commit"
        | "playback.transition.release" => {
            let input: Handle =
                serde_json::from_value(request.payload.clone()).map_err(|_| invalid())?;
            if input.transition_id.0 == 0 {
                return Err(invalid());
            }
            match request.command.as_str() {
                "playback.transition.observe" => encode(transitions.observe(input.transition_id)?),
                "playback.transition.commit" => encode(transitions.commit(input.transition_id)?),
                _ => encode(transitions.release(input.transition_id)?),
            }
        }
        _ => Err(ApiError::new(
            ErrorCode::Unsupported,
            "Unknown transition command",
        )),
    }
}
fn encode(value: impl serde::Serialize) -> Result<Value, ApiError> {
    serde_json::to_value(value).map_err(|_| {
        ApiError::new(
            ErrorCode::Internal,
            "Transition response serialization failed",
        )
    })
}
fn invalid() -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, "Invalid transition payload")
}

//! Thin transport binding to the common retained session service.
use super::sessions::{ReviewPlaybackSessions, Selection, SessionRef};
use crate::application::{
    api::{ApiError, ErrorCode, Request},
    review_source::MutationControl,
};
use crate::domain::{api_values::DecimalU64, playback_control::PlaybackControl};
use serde::Deserialize;
use serde_json::Value;
use std::time::{Duration, Instant};
pub const COMMANDS: &[&str] = &[
    "reviews.playback.open",
    "reviews.playback.status",
    "reviews.playback.close",
    "reviews.playback.control",
];
pub fn is_command(command: &str) -> bool {
    COMMANDS.contains(&command)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Open {
    selection: Selection,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Existing {
    session: SessionRef,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Control {
    session: SessionRef,
    engine_instance_id: DecimalU64,
    control: PlaybackControl,
}
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::InvalidArgument,
        "Invalid review playback payload or operation context",
    )
}
struct Deadline(Instant);
impl MutationControl for Deadline {
    fn checkpoint(&self) -> Result<(), ApiError> {
        if Instant::now() >= self.0 {
            Err(ApiError::new(
                ErrorCode::Timeout,
                "Review open deadline elapsed before owner creation",
            ))
        } else {
            Ok(())
        }
    }
}
pub fn execute(sessions: &ReviewPlaybackSessions, request: &Request) -> Result<Value, ApiError> {
    request.validate()?;
    if request.context.expected_revision.is_some() {
        return Err(invalid());
    }
    if request.command == "reviews.playback.control" {
        if request.context.idempotency_key.is_some() || request.context.timeout_ms.is_some() {
            return Err(invalid());
        }
        let input: Control =
            serde_json::from_value(request.payload.clone()).map_err(|_| invalid())?;
        let receipt = sessions.control(&input.session, input.engine_instance_id, input.control)?;
        return Ok(serde_json::json!({"session":input.session,"receipt":receipt}));
    }
    let view = match request.command.as_str() {
        "reviews.playback.open" => {
            let input: Open =
                serde_json::from_value(request.payload.clone()).map_err(|_| invalid())?;
            let key = request
                .context
                .idempotency_key
                .as_deref()
                .ok_or_else(invalid)?;
            let timeout = request.context.timeout_ms.unwrap_or(30000);
            if !(1..=30000).contains(&timeout) {
                return Err(invalid());
            }
            let deadline = Instant::now()
                .checked_add(Duration::from_millis(timeout))
                .ok_or_else(invalid)?;
            sessions.open(input.selection, key, &Deadline(deadline))?
        }
        "reviews.playback.status" | "reviews.playback.close" => {
            if request.context.idempotency_key.is_some() || request.context.timeout_ms.is_some() {
                return Err(invalid());
            }
            let input: Existing =
                serde_json::from_value(request.payload.clone()).map_err(|_| invalid())?;
            if request.command == "reviews.playback.close" {
                sessions.close(&input.session)?
            } else {
                sessions.status(&input.session)?
            }
        }
        _ => return Err(invalid()),
    };
    serde_json::to_value(view).map_err(|_| {
        ApiError::new(
            ErrorCode::Internal,
            "Review session response encoding failed",
        )
    })
}

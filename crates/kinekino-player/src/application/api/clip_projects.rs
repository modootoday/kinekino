mod schema;
use super::{ApiError, ErrorCode, Request};
use crate::{
    application::clip_projects::{self, ClipProjectInput, ClipProjectStore, InspectClipProject},
    domain::api_values::DecimalU64,
};
pub(super) use schema::source_schema;
pub(super) use schema::{capabilities, describe};
use serde_json::Value;

pub(super) fn is_command(command: &str) -> bool {
    matches!(
        command,
        "clip_projects.create" | "clip_projects.inspect" | "clip_projects.save_revision"
    )
}
fn invalid(message: &str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
fn input<T: serde::de::DeserializeOwned>(payload: &Value) -> Result<T, ApiError> {
    serde_json::from_value(payload.clone())
        .map_err(|_| invalid("Expected strict clip project payload"))
}
pub(super) fn execute(
    request: &Request,
    store: Option<&dyn ClipProjectStore>,
) -> Result<Value, ApiError> {
    if request.context.timeout_ms.is_some() || request.context.idempotency_key.is_some() {
        return Err(invalid(
            "Clip storage has no timeout or idempotency context support",
        ));
    }
    let expected = if request.command == "clip_projects.save_revision" {
        Some(DecimalU64(
            request
                .context
                .expected_revision
                .as_deref()
                .and_then(|value| value.parse::<u64>().ok())
                .filter(|value| *value > 0)
                .ok_or_else(|| invalid("Positive expected publication revision is required"))?,
        ))
    } else {
        if request.context.expected_revision.is_some() {
            return Err(invalid("Only clip project save accepts expected_revision"));
        }
        None
    };
    let document = if request.command != "clip_projects.inspect" {
        let parsed: ClipProjectInput = input(&request.payload)?;
        parsed.document.validate().map_err(invalid)?;
        Some(parsed.document)
    } else {
        None
    };
    let inspect = if request.command == "clip_projects.inspect" {
        Some(input::<InspectClipProject>(&request.payload)?)
    } else {
        None
    };
    let store = store.ok_or_else(|| {
        ApiError::new(
            ErrorCode::Unsupported,
            "Explicit clip project store is unavailable",
        )
    })?;
    let result = match request.command.as_str() {
        "clip_projects.create" => {
            clip_projects::create(store, document.as_ref().expect("parsed document"))?
        }
        "clip_projects.save_revision" => clip_projects::save(
            store,
            expected.expect("save revision"),
            document.as_ref().expect("parsed document"),
        )?,
        "clip_projects.inspect" => {
            let inspect = inspect.expect("parsed inspection");
            clip_projects::inspect(store, &inspect.document_id, inspect.publication_revision)?
        }
        _ => return Err(invalid("Unknown clip project command")),
    };
    serde_json::to_value(result)
        .map_err(|_| ApiError::new(ErrorCode::Internal, "Invalid clip project response"))
}

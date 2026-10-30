mod schema;
use super::{ApiError, ErrorCode, Request};
use crate::{
    application::card_projects::{self, CardProjectInput, CardProjectStore, InspectCardProject},
    domain::api_values::DecimalU64,
};
pub(super) use schema::{capabilities, describe};
use serde_json::Value;

pub(super) fn is_command(command: &str) -> bool {
    matches!(
        command,
        "card_projects.create" | "card_projects.inspect" | "card_projects.save_revision"
    )
}
fn invalid(message: &str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
fn input<T: serde::de::DeserializeOwned>(payload: &Value) -> Result<T, ApiError> {
    serde_json::from_value(payload.clone())
        .map_err(|_| invalid("Expected strict card project payload"))
}
pub(super) fn execute(
    request: &Request,
    store: Option<&dyn CardProjectStore>,
) -> Result<Value, ApiError> {
    if request.context.timeout_ms.is_some() || request.context.idempotency_key.is_some() {
        return Err(invalid(
            "Card project storage has no timeout or idempotency context support",
        ));
    }
    let expected = if request.command == "card_projects.save_revision" {
        Some(DecimalU64(
            request
                .context
                .expected_revision
                .as_deref()
                .and_then(|s| s.parse::<u64>().ok())
                .filter(|n| *n > 0)
                .ok_or_else(|| invalid("Positive expected publication revision is required"))?,
        ))
    } else {
        if request.context.expected_revision.is_some() {
            return Err(invalid("Only card project save accepts expected_revision"));
        }
        None
    };
    // Parse before availability so malformed input never becomes an executable contract.
    let document = if request.command != "card_projects.inspect" {
        let parsed: CardProjectInput = input(&request.payload)?;
        parsed
            .document
            .validate()
            .map_err(|_| invalid("Invalid card document"))?;
        Some(parsed.document)
    } else {
        None
    };
    let inspect = if request.command == "card_projects.inspect" {
        Some(input::<InspectCardProject>(&request.payload)?)
    } else {
        None
    };
    let store = store.ok_or_else(|| {
        ApiError::new(
            ErrorCode::Unsupported,
            "Explicit card project store is unavailable",
        )
    })?;
    let result = match request.command.as_str() {
        "card_projects.create" => {
            card_projects::create(store, document.as_ref().expect("parsed document"))?
        }
        "card_projects.save_revision" => card_projects::save(
            store,
            expected.expect("save revision"),
            document.as_ref().expect("parsed document"),
        )?,
        "card_projects.inspect" => {
            let inspect = inspect.expect("parsed inspection");
            card_projects::inspect(store, &inspect.document_id, inspect.publication_revision)?
        }
        _ => return Err(invalid("Unknown card project command")),
    };
    serde_json::to_value(result)
        .map_err(|_| ApiError::new(ErrorCode::Internal, "Invalid card project response"))
}

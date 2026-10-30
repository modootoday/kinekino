pub(super) mod schema;
use super::{ApiError, ErrorCode, Request};
use crate::{
    application::asset_projects::{
        self, AssetProjectInput, AssetProjectStore, InspectAssetProject,
    },
    domain::api_values::DecimalU64,
};
pub(super) use schema::{capabilities, describe};
use serde_json::Value;

pub(super) fn is_command(command: &str) -> bool {
    matches!(
        command,
        "asset_projects.create" | "asset_projects.inspect" | "asset_projects.save_revision"
    )
}
fn invalid(message: &str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
fn input<T: serde::de::DeserializeOwned>(payload: &Value) -> Result<T, ApiError> {
    serde_json::from_value(payload.clone())
        .map_err(|_| invalid("Expected strict asset project payload"))
}
pub(super) fn execute(
    request: &Request,
    store: Option<&dyn AssetProjectStore>,
) -> Result<Value, ApiError> {
    if request.context.timeout_ms.is_some() || request.context.idempotency_key.is_some() {
        return Err(invalid(
            "Asset storage has no timeout or idempotency context support",
        ));
    }
    let expected = if request.command == "asset_projects.save_revision" {
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
            return Err(invalid("Only asset project save accepts expected_revision"));
        }
        None
    };
    let document = if request.command != "asset_projects.inspect" {
        let parsed: AssetProjectInput = input(&request.payload)?;
        parsed.document.validate().map_err(invalid)?;
        Some(parsed.document)
    } else {
        None
    };
    let inspect = if request.command == "asset_projects.inspect" {
        Some(input::<InspectAssetProject>(&request.payload)?)
    } else {
        None
    };
    let store = store.ok_or_else(|| {
        ApiError::new(
            ErrorCode::Unsupported,
            "Explicit asset project store is unavailable",
        )
    })?;
    let result = match request.command.as_str() {
        "asset_projects.create" => {
            asset_projects::create(store, document.as_ref().expect("parsed document"))?
        }
        "asset_projects.save_revision" => asset_projects::save(
            store,
            expected.expect("save revision"),
            document.as_ref().expect("parsed document"),
        )?,
        "asset_projects.inspect" => {
            let inspect = inspect.expect("parsed inspection");
            asset_projects::inspect(store, &inspect.document_id, inspect.publication_revision)?
        }
        _ => return Err(invalid("Unknown asset project command")),
    };
    serde_json::to_value(result)
        .map_err(|_| ApiError::new(ErrorCode::Internal, "Invalid asset project response"))
}

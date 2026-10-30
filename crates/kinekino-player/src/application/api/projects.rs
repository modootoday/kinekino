use super::{ApiError, ErrorCode, Request};
use crate::application::{
    projects::{self, ProjectStore},
    proposals::ProposalStore,
};
use serde_json::Value;
pub fn is_command(command: &str) -> bool {
    matches!(
        command,
        "projects.create"
            | "projects.inspect"
            | "projects.save_revision"
            | "projects.reconnect_source"
    )
}
pub fn execute(
    request: &Request,
    store: &dyn ProjectStore,
    proposals: &dyn ProposalStore,
) -> Result<Value, ApiError> {
    if !store.available() {
        return Err(projects::unavailable());
    }
    let invalid = |message| ApiError::new(ErrorCode::InvalidArgument, message);
    if request.context.timeout_ms.is_some() || request.context.idempotency_key.is_some() {
        return Err(invalid(
            "Project commands do not accept timeout_ms or idempotency_key",
        ));
    }
    if !matches!(
        request.command.as_str(),
        "projects.save_revision" | "projects.reconnect_source"
    ) && request.context.expected_revision.is_some()
    {
        return Err(invalid(
            "Only project save/reconnect accepts expected_revision",
        ));
    }
    fn input<T: serde::de::DeserializeOwned>(value: &Value) -> Result<T, ApiError> {
        serde_json::from_value(value.clone())
            .map_err(|error| ApiError::new(ErrorCode::InvalidArgument, error.to_string()))
    }
    let result = match request.command.as_str() {
        "projects.reconnect_source" => {
            let expected = request
                .context
                .expected_revision
                .as_deref()
                .and_then(|v| v.parse::<u64>().ok())
                .filter(|n| *n > 0)
                .ok_or_else(|| {
                    invalid("Positive expected_revision is required for source reconnection")
                })?;
            return serde_json::to_value(projects::reconnect_source(
                store,
                proposals,
                expected,
                input(&request.payload)?,
            )?)
            .map_err(|_| {
                ApiError::new(
                    ErrorCode::Internal,
                    "Project reconnection serialization failed",
                )
            });
        }
        "projects.create" => projects::create(store, proposals, input(&request.payload)?)?,
        "projects.inspect" => {
            projects::inspect_with_proposals(store, proposals, input(&request.payload)?)?
        }
        "projects.save_revision" => {
            let expected = request
                .context
                .expected_revision
                .as_deref()
                .and_then(|value| value.parse::<u64>().ok())
                .filter(|n| *n > 0)
                .ok_or_else(|| {
                    invalid("Positive expected_revision is required for project save")
                })?;
            projects::save(store, proposals, expected, input(&request.payload)?)?
        }
        _ => return Err(projects::unavailable()),
    };
    serde_json::to_value(result)
        .map_err(|_| ApiError::new(ErrorCode::Internal, "Project serialization failed"))
}

mod schema;
use super::{ApiError, ErrorCode, Request};
use crate::application::{
    projects::ProjectStore,
    proposals::{
        ProposalStore,
        guarded::{self, AcceptCapturedProposal, EditCapturedProposal},
    },
};
pub(super) use schema::{capabilities, describe};
use serde_json::Value;

pub(super) fn is_command(command: &str) -> bool {
    matches!(
        command,
        "proposals.edit_capture" | "proposals.accept_capture"
    )
}
pub(super) fn available(projects: &dyn ProjectStore, proposals: &dyn ProposalStore) -> bool {
    projects.available() && proposals.available() && proposals.capture_mutation_available()
}
pub(super) fn execute(
    request: &Request,
    projects: &dyn ProjectStore,
    proposals: &dyn ProposalStore,
) -> Result<Value, ApiError> {
    if !available(projects, proposals) {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Capture mutation backend unavailable",
        ));
    }
    if request.context.expected_revision.is_some()
        || request.context.idempotency_key.is_some()
        || request.context.timeout_ms.is_some()
    {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Capture commands require payload expected state; context revision, idempotency and timeout are unsupported",
        ));
    }
    fn input<T: serde::de::DeserializeOwned>(value: &Value) -> Result<T, ApiError> {
        serde_json::from_value(value.clone())
            .map_err(|error| ApiError::new(ErrorCode::InvalidArgument, error.to_string()))
    }
    let result = match request.command.as_str() {
        "proposals.edit_capture" => {
            let input: EditCapturedProposal = input(&request.payload)?;
            guarded::edit(
                projects,
                proposals,
                &input.anchor,
                input.expected.as_ref(),
                input.draft,
            )?
        }
        "proposals.accept_capture" => {
            let input: AcceptCapturedProposal = input(&request.payload)?;
            guarded::accept(
                projects,
                proposals,
                &input.anchor,
                input.expected.as_ref(),
                input.target_source_revision,
            )?
        }
        _ => {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Unknown capture mutation command",
            ));
        }
    };
    serde_json::to_value(result).map_err(|_| {
        ApiError::new(
            ErrorCode::Internal,
            "Capture mutation response serialization failed",
        )
    })
}

#[cfg(test)]
mod tests;

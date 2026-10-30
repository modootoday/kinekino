use super::{ApiError, ErrorCode, Request};
use crate::application::jobs::*;
use serde_json::Value;
pub fn is_command(command: &str) -> bool {
    matches!(
        command,
        "jobs.get" | "jobs.list" | "jobs.watch" | "jobs.submit" | "jobs.cancel" | "jobs.retry"
    )
}
pub fn is_write(command: &str) -> bool {
    matches!(command, "jobs.submit" | "jobs.cancel" | "jobs.retry")
}
pub fn execute(request: &Request, jobs: &dyn JobsApi) -> Result<Value, ApiError> {
    let cap = jobs.capability();
    if if is_write(&request.command) {
        !cap.execution_available
    } else {
        !cap.read_available
    } {
        return Err(unavailable());
    }
    let invalid = |message| ApiError::new(ErrorCode::InvalidArgument, message);
    if request.context.timeout_ms.is_some() {
        return Err(invalid(
            "Job RPC context timeout is unsupported; use execution_timeout_ms in the submitted task",
        ));
    }
    if request.command != "jobs.submit" && request.context.idempotency_key.is_some() {
        return Err(invalid("Only jobs.submit accepts idempotency_key"));
    }
    if request.command != "jobs.retry" && request.context.expected_revision.is_some() {
        return Err(invalid(
            "Only jobs.retry accepts expected_revision; cancel uses expected_attempt",
        ));
    }
    fn input<T: serde::de::DeserializeOwned>(value: &Value) -> Result<T, ApiError> {
        serde_json::from_value(value.clone())
            .map_err(|error| ApiError::new(ErrorCode::InvalidArgument, error.to_string()))
    }
    let result = match request.command.as_str() {
        "jobs.get" => serde_json::to_value(jobs.get(input(&request.payload)?)?),
        "jobs.list" => serde_json::to_value(jobs.list(input(&request.payload)?)?),
        "jobs.watch" => serde_json::to_value(jobs.watch(input(&request.payload)?)?),
        "jobs.submit" => serde_json::to_value(
            jobs.submit(
                input(&request.payload)?,
                request
                    .context
                    .idempotency_key
                    .as_deref()
                    .ok_or_else(|| invalid("jobs.submit requires idempotency_key"))?,
            )?,
        ),
        "jobs.cancel" => serde_json::to_value(jobs.cancel(input(&request.payload)?)?),
        "jobs.retry" => {
            let expected = request
                .context
                .expected_revision
                .as_deref()
                .and_then(|value| value.parse::<u64>().ok())
                .filter(|value| *value > 0)
                .ok_or_else(|| invalid("jobs.retry requires positive expected_revision"))?;
            serde_json::to_value(jobs.retry(input(&request.payload)?, expected)?)
        }
        _ => return Err(unavailable()),
    };
    result.map_err(|_| ApiError::new(ErrorCode::Internal, "Job response serialization failed"))
}

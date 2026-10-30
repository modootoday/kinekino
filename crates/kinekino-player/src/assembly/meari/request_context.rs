//! Preserve explicit SDK request options while deriving document ticket correlation.
use crate::application::api::{ApiError, ErrorCode, RequestContext};
use meari::{application::plugins::PluginTicket, domain::build::ContentHash};

/// Preserve the explicit caller context; scope only its request correlation identity.
pub fn scoped_context(
    context: &RequestContext,
    ticket: &PluginTicket,
) -> Result<RequestContext, ApiError> {
    context.validate()?;
    let mut next = context.clone();
    let correlation = serde_json::to_vec(&(&context.request_id, ticket)).map_err(|_| {
        ApiError::new(
            ErrorCode::Internal,
            "Plugin request correlation encoding failed",
        )
    })?;
    next.request_id = format!("ui-{}", ContentHash::of(&correlation).as_str());
    Ok(next)
}

use super::{API_VERSION, ApiError, ErrorCode};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const MAX_CONTEXT_TEXT_LENGTH: usize = 128;
pub const REQUEST_ID_CONTROL_PATTERN: &str = r"[\u0000-\u001f\u007f-\u009f]";

fn bounded_context_text(value: &str) -> bool {
    !value.is_empty() && value.chars().count() <= MAX_CONTEXT_TEXT_LENGTH
}

pub fn valid_request_id(value: &str) -> bool {
    crate::domain::identifiers::valid_request_id(value)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RequestContext {
    pub request_id: String,
    pub api_version: String,
    pub expected_revision: Option<String>,
    pub idempotency_key: Option<String>,
    pub timeout_ms: Option<u64>,
}

impl Default for RequestContext {
    fn default() -> Self {
        Self {
            request_id: "cli".into(),
            api_version: API_VERSION.into(),
            expected_revision: None,
            idempotency_key: None,
            timeout_ms: None,
        }
    }
}

impl RequestContext {
    /// Validate caller context before a transport derives a correlation ID.
    pub fn validate(&self) -> Result<(), ApiError> {
        let invalid = self.api_version != API_VERSION
            || !valid_request_id(&self.request_id)
            || self.expected_revision.as_ref().is_some_and(|value| {
                value.is_empty()
                    || !value.bytes().all(|byte| byte.is_ascii_digit())
                    || value.parse::<u64>().is_err()
            })
            || self
                .idempotency_key
                .as_ref()
                .is_some_and(|value| !bounded_context_text(value))
            || self.timeout_ms == Some(0);
        if invalid {
            return Err(invalid_request_error(self));
        }
        Ok(())
    }
}

fn invalid_request_error(context: &RequestContext) -> ApiError {
    let mut error = ApiError::new(
        ErrorCode::InvalidArgument,
        "Invalid request context or payload",
    );
    if valid_request_id(&context.request_id) {
        error.request_id = context.request_id.clone();
    }
    error
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub context: RequestContext,
    pub command: String,
    #[serde(default = "empty_payload")]
    pub payload: Value,
}

fn empty_payload() -> Value {
    json!({})
}

impl Request {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            context: RequestContext::default(),
            command: command.into(),
            payload: empty_payload(),
        }
    }
    pub fn validate(&self) -> Result<(), ApiError> {
        self.context.validate()?;
        if !self.payload.is_object() {
            return Err(invalid_request_error(&self.context));
        }
        Ok(())
    }
}

use kinekino_core::port::{ErrorCode, ToPortError};
use std::fmt;

/// Typed inference failures; HTTP adapters map status codes onto these.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InferenceError {
    Unsupported(String),
    InvalidInput(String),
    /// 401/403 or missing token.
    Auth(String),
    /// 402 or local budget refusal (`retryable=false`).
    Budget(String),
    /// 429; retryable after backoff.
    RateLimited(String),
    /// 5xx, connect failure, offline.
    Unavailable(String),
    Timeout(String),
    Cancelled,
    /// Upload consent missing or withdrawn.
    ConsentRequired(String),
    /// Unparseable response.
    Protocol(String),
}

impl fmt::Display for InferenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported(t) => write!(f, "task not supported: {t}"),
            Self::InvalidInput(m) => write!(f, "invalid input: {m}"),
            Self::Auth(m) => write!(f, "authentication failed: {m}"),
            Self::Budget(m) => write!(f, "budget refused: {m}"),
            Self::RateLimited(m) => write!(f, "rate limited: {m}"),
            Self::Unavailable(m) => write!(f, "backend unavailable: {m}"),
            Self::Timeout(m) => write!(f, "timed out: {m}"),
            Self::Cancelled => f.write_str("cancelled"),
            Self::ConsentRequired(m) => write!(f, "upload consent required: {m}"),
            Self::Protocol(m) => write!(f, "bad response: {m}"),
        }
    }
}

impl std::error::Error for InferenceError {}

impl ToPortError for InferenceError {
    fn error_code(&self) -> ErrorCode {
        match self {
            Self::Unsupported(_) => ErrorCode::Unsupported,
            Self::InvalidInput(_) | Self::ConsentRequired(_) => ErrorCode::InvalidInput,
            Self::Auth(_) => ErrorCode::Auth,
            Self::Budget(_) | Self::RateLimited(_) => ErrorCode::Quota,
            Self::Unavailable(_) => ErrorCode::Unavailable,
            Self::Timeout(_) => ErrorCode::Timeout,
            Self::Cancelled => ErrorCode::Cancelled,
            Self::Protocol(_) => ErrorCode::Internal,
        }
    }
    fn retryable(&self) -> bool {
        matches!(
            self,
            Self::RateLimited(_) | Self::Unavailable(_) | Self::Timeout(_)
        )
    }
}

impl InferenceError {
    /// Every variant once (contract harness samples).
    pub fn samples() -> Vec<Self> {
        vec![
            Self::Unsupported("x".into()),
            Self::InvalidInput("x".into()),
            Self::Auth("x".into()),
            Self::Budget("x".into()),
            Self::RateLimited("x".into()),
            Self::Unavailable("x".into()),
            Self::Timeout("x".into()),
            Self::Cancelled,
            Self::ConsentRequired("x".into()),
            Self::Protocol("x".into()),
        ]
    }
}

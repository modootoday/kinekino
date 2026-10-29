use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidArgument,
    Unsupported,
    NeedsInput,
    NotFound,
    RevisionConflict,
    PermissionDenied,
    ResourceExhausted,
    SourceExpired,
    EngineFailed,
    Cancelled,
    Internal,
    Timeout,
}

impl ErrorCode {
    pub const ALL: [Self; 12] = [
        Self::InvalidArgument,
        Self::Unsupported,
        Self::NeedsInput,
        Self::NotFound,
        Self::RevisionConflict,
        Self::PermissionDenied,
        Self::ResourceExhausted,
        Self::SourceExpired,
        Self::EngineFailed,
        Self::Cancelled,
        Self::Internal,
        Self::Timeout,
    ];
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiError {
    pub code: ErrorCode,
    pub message: String,
    pub retryable: bool,
    pub details: Value,
    pub remediation: String,
    pub request_id: String,
}

impl ApiError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            retryable: false,
            details: json!({}),
            remediation: String::new(),
            request_id: String::new(),
        }
    }
    pub fn exit_code(&self) -> u8 {
        match self.code {
            ErrorCode::InvalidArgument => 2,
            ErrorCode::Unsupported | ErrorCode::NeedsInput => 3,
            ErrorCode::NotFound => 4,
            ErrorCode::RevisionConflict => 5,
            ErrorCode::PermissionDenied => 6,
            ErrorCode::ResourceExhausted => 7,
            ErrorCode::SourceExpired | ErrorCode::EngineFailed => 8,
            ErrorCode::Cancelled => 9,
            ErrorCode::Internal => 1,
            ErrorCode::Timeout => 10,
        }
    }
}

//! Common error code every port error maps to; SDK/CLI JSON errors use it unchanged.
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    Unsupported,
    Unavailable,
    InvalidInput,
    Cancelled,
    Timeout,
    Quota,
    Auth,
    Io,
    Internal,
}

impl ErrorCode {
    pub const ALL: [Self; 9] = [
        Self::Unsupported,
        Self::Unavailable,
        Self::InvalidInput,
        Self::Cancelled,
        Self::Timeout,
        Self::Quota,
        Self::Auth,
        Self::Io,
        Self::Internal,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Unsupported => "unsupported",
            Self::Unavailable => "unavailable",
            Self::InvalidInput => "invalid_input",
            Self::Cancelled => "cancelled",
            Self::Timeout => "timeout",
            Self::Quota => "quota",
            Self::Auth => "auth",
            Self::Io => "io",
            Self::Internal => "internal",
        }
    }

    /// Default retry hint when a port error does not decide itself.
    pub const fn default_retryable(self) -> bool {
        matches!(self, Self::Unavailable | Self::Timeout | Self::Io)
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Port-neutral error carried across the SDK/CLI boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortError {
    pub code: ErrorCode,
    pub retryable: bool,
    pub message: String,
}

pub type PortResult<T> = Result<T, PortError>;

impl PortError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            retryable: code.default_retryable(),
            message: message.into(),
        }
    }

    /// Stable JSON shape: `{"code","retryable","message"}`.
    pub fn to_json(&self) -> String {
        format!(
            "{{\"code\":\"{}\",\"retryable\":{},\"message\":{}}}",
            self.code,
            self.retryable,
            json_string(&self.message)
        )
    }
}

impl fmt::Display for PortError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for PortError {}

/// Implemented by every port-specific error enum.
pub trait ToPortError {
    fn error_code(&self) -> ErrorCode;
    fn retryable(&self) -> bool {
        self.error_code().default_retryable()
    }
    fn to_port_error(&self) -> PortError
    where
        Self: fmt::Display,
    {
        PortError {
            code: self.error_code(),
            retryable: self.retryable(),
            message: self.to_string(),
        }
    }
}

fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_shape_and_default_retry() {
        let e = PortError::new(ErrorCode::Timeout, "slow \"x\"");
        assert!(e.retryable);
        assert_eq!(
            e.to_json(),
            r#"{"code":"timeout","retryable":true,"message":"slow \"x\""}"#
        );
        assert!(!PortError::new(ErrorCode::Auth, "no").retryable);
    }
}

//! Request/response types for host-owned browser capture (ADR 20261106032051).

use crate::engine::port::EngineError;

/// Already-admitted opaque navigate target for a capture session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureNavigateTarget {
    pub url: String,
}

/// Optional wait hint after navigate before collecting observations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CaptureWaitHint {
    pub network_idle: bool,
    pub timeout_ms: Option<u64>,
}

/// One observed network request re-exposed to the plugin for Resolve mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedNetworkRequest {
    pub url: String,
    pub method: String,
    pub resource_type: String,
    pub headers: Vec<(String, String)>,
}

/// Errors from [`super::BrowserCaptureSession`].
#[derive(Debug)]
pub enum CaptureError {
    /// Navigate target host is not admitted (deny-by-default).
    NavigateDenied,
    /// Session was already closed.
    Closed,
    /// Underlying engine failure.
    Engine(EngineError),
}

impl std::fmt::Display for CaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NavigateDenied => write!(f, "navigate target not admitted"),
            Self::Closed => write!(f, "capture session closed"),
            Self::Engine(e) => write!(f, "capture engine: {e}"),
        }
    }
}

impl std::error::Error for CaptureError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Engine(e) => Some(e),
            _ => None,
        }
    }
}

impl From<EngineError> for CaptureError {
    fn from(value: EngineError) -> Self {
        Self::Engine(value)
    }
}

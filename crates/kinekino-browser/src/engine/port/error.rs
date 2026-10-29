//! Engine error type shared by every adapter.
use std::fmt;

#[derive(Debug)]
pub enum EngineError {
    /// Engine binary or runtime missing or failed to start.
    Launch(String),
    /// Protocol or engine call failed.
    Call(String),
    /// Capability not offered by this adapter.
    Unsupported(&'static str),
}

pub type EngineResult<T> = Result<T, EngineError>;

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Launch(m) => write!(f, "engine launch failed: {m}"),
            Self::Call(m) => write!(f, "engine call failed: {m}"),
            Self::Unsupported(c) => write!(f, "unsupported capability: {c}"),
        }
    }
}

impl std::error::Error for EngineError {}

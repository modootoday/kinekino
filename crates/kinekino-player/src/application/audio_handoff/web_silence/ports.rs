//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

pub trait WebAudioBackend {
    fn document_generation(&self) -> u64;
    fn is_muted(&self) -> Result<bool, ApiError>;
    fn set_muted(&self, muted: bool) -> Result<(), ApiError>;
}

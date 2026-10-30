//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

pub trait PlaybackStatusPort: Send + Sync {
    fn available(&self) -> bool {
        true
    }
    fn status(&self) -> Result<PlaybackStatus, ApiError>;
}

//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

pub trait ReviewSourceApi: Send + Sync {
    fn writable(&self) -> bool;
    fn media_preparation(&self) -> Option<&dyn super::media_api::MediaPreparationPort> {
        None
    }
    fn timeline_verification_available(&self) -> bool {
        false
    }
    fn storage_limits(&self) -> Option<StoreLimits> {
        None
    }
    fn execute(&self, request: &Request) -> Result<Value, ApiError>;
}

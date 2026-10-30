//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

pub trait SkipHostApi: Send + Sync {
    fn capability(&self) -> SkipHostCapability;
    fn export_capability(&self) -> SkipExportCapability {
        SkipExportCapability {
            available: false,
            issuer_scope: None,
        }
    }
    fn execute(&self, request: &Request) -> Result<Value, ApiError>;
}

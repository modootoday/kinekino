//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

pub trait SnapshotExportPort {
    fn export(&self, request: &ExportSnapshot) -> Result<ExportedSnapshot, ApiError>;
}

#[path = "ports.rs"]
mod ports;
use crate::application::api::{ApiError, ErrorCode};
pub use crate::domain::snapshot_export::{ExportSnapshot, PixelCrop};
pub use ports::SnapshotExportPort;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedSnapshot {
    pub output_path: String,
    pub source_sha256: String,
    pub output_sha256: String,
    pub source_width: u32,
    pub source_height: u32,
    pub crop: PixelCrop,
    pub output_bytes: u64,
    /// Only numeric samples and alpha are preserved; ancillary metadata is not copied.
    pub metadata_preserved: bool,
}
pub fn export(
    port: &(impl SnapshotExportPort + ?Sized),
    request: &ExportSnapshot,
) -> Result<ExportedSnapshot, ApiError> {
    request
        .validate()
        .map_err(|message| ApiError::new(ErrorCode::InvalidArgument, message))?;
    port.export(request)
}

#[cfg(test)]
mod tests {
    use super::*;
    struct NoCall;
    impl SnapshotExportPort for NoCall {
        fn export(&self, _: &ExportSnapshot) -> Result<ExportedSnapshot, ApiError> {
            panic!("Invalid input must not reach the filesystem adapter")
        }
    }
    #[test]
    fn rejects_invalid_crop_before_adapter() {
        let request = ExportSnapshot {
            source_path: "relative.png".into(),
            expected_source_sha256: "a".repeat(64),
            output_path: "other.png".into(),
            crop: PixelCrop {
                x: 0,
                y: 0,
                width: 0,
                height: 1,
            },
        };
        assert_eq!(
            export(&NoCall, &request).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
    }
}

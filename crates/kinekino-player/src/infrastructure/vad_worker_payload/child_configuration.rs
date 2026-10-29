//! Parent-owned selection sidecar uses the existing ordinary file guards.
use super::super::vad_job_asset::{FixedVadFile, GuardedAsset};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_vad::{application::AnalysisControl, domain::ArtifactPin};
use kinekino_vad_executor::application::{
    CHILD_CONFIGURATION_FILE, MAX_CHILD_CONFIGURATION_BYTES, SnapshotLease,
};
use std::{fs::OpenOptions, io::Write, path::Path, sync::Arc, time::Instant};
pub(super) fn stage(
    bytes: Vec<u8>,
    pin: &ArtifactPin,
    root: &Path,
    control: &dyn AnalysisControl,
) -> Result<Arc<dyn SnapshotLease>, ApiError> {
    let path = root.join(CHILD_CONFIGURATION_FILE);
    // Same owned attempt root and immutable guard adapter as existing sidecars.
    let asset = GuardedAsset::new(
        FixedVadFile {
            path: path.clone(),
            pin: pin.clone(),
        },
        MAX_CHILD_CONFIGURATION_BYTES,
    )?;
    if control.checkpoint()? {
        return Err(ApiError::new(
            ErrorCode::Cancelled,
            "Parent configuration staging cancelled",
        ));
    }
    if Instant::now() >= control.deadline() {
        return Err(ApiError::new(
            ErrorCode::Timeout,
            "Original parent configuration cutoff",
        ));
    }
    {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(kinekino_core::file_publication::io_error)?;
        file.write_all(&bytes)
            .and_then(|()| file.sync_all())
            .map_err(kinekino_core::file_publication::io_error)?;
    }
    let (_, lease) = asset.snapshot_with_control(Some(control))?;
    Ok(lease)
}

use super::{
    AnalysisControl, ApiError, ArtifactPin, ErrorCode, FixedVadFile, GuardedAsset, SnapshotLease,
};
use std::{path::Path, sync::Arc, time::Instant};
pub(super) fn check(control: Option<&dyn AnalysisControl>) -> Result<(), ApiError> {
    if let Some(c) = control {
        if c.checkpoint()? {
            return Err(ApiError::new(
                ErrorCode::Cancelled,
                "Original VAD input preparation cancelled; no cleanup ACK",
            ));
        }
        if Instant::now() >= c.deadline() {
            return Err(ApiError::new(
                ErrorCode::Timeout,
                "Original VAD input preparation cutoff",
            ));
        }
    }
    Ok(())
}
pub(super) fn publish(
    root: &Path,
    role: &str,
    pin: &ArtifactPin,
    bytes: &[u8],
    maximum: u64,
    control: Option<&dyn AnalysisControl>,
) -> Result<(FixedVadFile, Arc<dyn SnapshotLease>), ApiError> {
    check(control)?;
    let path = root.join(format!("{role}-{}.bin", pin.sha256()));
    let file = FixedVadFile {
        path: path.clone(),
        pin: pin.clone(),
    };
    let asset = GuardedAsset::new(
        FixedVadFile {
            path: path.clone(),
            pin: pin.clone(),
        },
        maximum,
    )?;
    if !path
        .try_exists()
        .map_err(kinekino_core::file_publication::io_error)?
    {
        match kinekino_core::file_publication::publish_before(
            path.to_str().ok_or_else(|| {
                ApiError::new(ErrorCode::InvalidArgument, "VAD cache path is not Unicode")
            })?,
            bytes,
            control.map(AnalysisControl::deadline),
        ) {
            Ok(()) => {}
            Err(e) if e.code == ErrorCode::RevisionConflict => {}
            Err(e) => return Err(e),
        }
    }
    // Existing immutable guard and exact bytes/SHA decide a CAS collision.
    let (_, lease) = asset.snapshot_with_control(control)?;
    Ok((file, lease))
}

//! Fixed-role parent sidecars. Existing GuardedAsset owns all file/hash policy.
use super::vad_job_asset::{FixedVadFile, GuardedAsset};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_vad::application::AnalysisControl;
use kinekino_vad_executor::{
    ArtifactPin,
    application::{MAX_RELEASE_MANIFEST_BYTES, SnapshotLease},
};
use std::{fs::OpenOptions, io::Write, path::Path, sync::Arc, time::Instant};

pub(super) enum SidecarRole {
    Descriptor,
    Manifest,
    Model,
    License,
    Wav,
}
impl SidecarRole {
    fn name(&self) -> &'static str {
        match self {
            Self::Descriptor => "descriptor.json",
            Self::Manifest => "release.manifest.json",
            Self::Model => "model.onnx",
            Self::License => "model-license.txt",
            Self::Wav => "input.wav",
        }
    }
    fn maximum(&self) -> u64 {
        match self {
            Self::Descriptor => kinekino_vad::protocol::application::MAX_DESCRIPTOR_BYTES,
            Self::Manifest => MAX_RELEASE_MANIFEST_BYTES,
            Self::Model => kinekino_vad::domain::MAX_MODEL_BYTES,
            Self::License => kinekino_vad::domain::MAX_LICENSE_BYTES,
            Self::Wav => kinekino_vad::domain::MAX_AUDIO_BYTES,
        }
    }
}
pub(super) struct PinnedSidecar {
    pub role: SidecarRole,
    pub pin: ArtifactPin,
    pub bytes: Vec<u8>,
}
fn check(control: &dyn AnalysisControl) -> Result<(), ApiError> {
    if control.cancellation_requested() {
        return Err(ApiError::new(
            ErrorCode::Cancelled,
            "Sidecar staging stopped; no cleanup ACK",
        ));
    }
    if Instant::now() >= control.deadline() {
        return Err(ApiError::new(
            ErrorCode::Timeout,
            "Original sidecar cutoff expired",
        ));
    }
    Ok(())
}
pub(super) fn retain_original_request(
    root: &Path,
    pin: &ArtifactPin,
    control: &dyn AnalysisControl,
) -> Result<Arc<dyn SnapshotLease>, ApiError> {
    let asset = GuardedAsset::new(
        FixedVadFile {
            path: root.join("stdin.raw"),
            pin: pin.clone(),
        },
        kinekino_vad::protocol::application::MAX_DESCRIPTOR_BYTES,
    )?;
    let (_, lease) = asset.snapshot_with_control(Some(control))?;
    Ok(lease)
}
pub(super) fn write_sidecars(
    root: &Path,
    files: Vec<PinnedSidecar>,
    control: &dyn AnalysisControl,
) -> Result<Vec<Arc<dyn SnapshotLease>>, ApiError> {
    let mut leases = Vec::new();
    for sidecar in files {
        check(control)?;
        if sidecar.bytes.len() as u64 != sidecar.pin.bytes()
            || sidecar.pin.bytes() > sidecar.role.maximum()
        {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Sidecar size differs from admitted role",
            ));
        }
        let path = root.join(sidecar.role.name());
        let asset = GuardedAsset::new(
            FixedVadFile {
                path: path.clone(),
                pin: sidecar.pin,
            },
            sidecar.role.maximum(),
        )?;
        {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .map_err(kinekino_core::file_publication::io_error)?;
            for chunk in sidecar.bytes.chunks(8192) {
                check(control)?;
                file.write_all(chunk)
                    .map_err(kinekino_core::file_publication::io_error)?;
            }
            file.sync_all()
                .map_err(kinekino_core::file_publication::io_error)?;
        }
        let (_, lease) = asset.snapshot_with_control(Some(control))?;
        leases.push(lease);
    }
    Ok(leases)
}

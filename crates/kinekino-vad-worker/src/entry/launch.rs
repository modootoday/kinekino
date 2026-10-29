use kinekino_media_contracts::{
    api_values::DecimalU64,
    errors::{ApiError, ErrorCode},
};
use kinekino_vad::domain::ArtifactPin;
use std::{
    ffi::OsString,
    path::{Component, PathBuf},
};

/// Shape-checked parent arguments only. A manifest pin is not an admission grant.
pub struct WorkerLaunch {
    pub request: PathBuf,
    pub cancel_signal: PathBuf,
    pub manifest: ArtifactPin,
    pub control_path: PathBuf,
    pub control_pin: ArtifactPin,
}
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::InvalidArgument,
        "Invalid fixed worker launch arguments",
    )
}
impl WorkerLaunch {
    pub fn parse(arguments: impl IntoIterator<Item = OsString>) -> Result<Self, ApiError> {
        let args: Vec<_> = arguments.into_iter().take(17).collect();
        if args.len() != 16
            || args[0] != "--vad-request-v1"
            || args[2] != "--cancel-file"
            || args[4] != "--release-manifest-id"
            || args[6] != "--release-manifest-sha256"
            || args[8] != "--release-manifest-bytes"
            || args[10] != "--worker-control-v1"
            || args[12] != "--worker-control-sha256"
            || args[14] != "--worker-control-bytes"
        {
            return Err(invalid());
        }
        let request = PathBuf::from(&args[1]);
        let cancel_signal = PathBuf::from(&args[3]);
        let control_path = PathBuf::from(&args[11]);
        if !request.is_absolute()
            || !cancel_signal.is_absolute()
            || request.file_name().and_then(|n| n.to_str()) != Some("stdin.raw")
            || cancel_signal.file_name().and_then(|n| n.to_str()) != Some("cancel.signal")
            || request.parent() != cancel_signal.parent()
            || !control_path.is_absolute()
            || control_path.file_name().and_then(|n| n.to_str()) != Some("worker-control.v1.json")
            || request.parent() != control_path.parent()
            || [&request, &cancel_signal, &control_path]
                .into_iter()
                .any(|p| p.components().any(|c| matches!(c, Component::ParentDir)))
        {
            return Err(invalid());
        }
        let bytes = args[9].to_str().ok_or_else(invalid)?;
        if bytes.is_empty()
            || !bytes.bytes().all(|c| c.is_ascii_digit())
            || (bytes.len() > 1 && bytes.starts_with('0'))
        {
            return Err(invalid());
        }
        let manifest = ArtifactPin::new(
            args[5].to_str().ok_or_else(invalid)?.into(),
            args[7].to_str().ok_or_else(invalid)?.into(),
            DecimalU64(bytes.parse().map_err(|_| invalid())?),
        )?;
        if manifest.bytes() > kinekino_vad_executor::application::MAX_RELEASE_MANIFEST_BYTES {
            return Err(invalid());
        }
        let control_bytes = args[15].to_str().ok_or_else(invalid)?;
        if control_bytes.is_empty()
            || !control_bytes.bytes().all(|c| c.is_ascii_digit())
            || (control_bytes.len() > 1 && control_bytes.starts_with('0'))
        {
            return Err(invalid());
        }
        let control_pin = ArtifactPin::new(
            "worker-control".into(),
            args[13].to_str().ok_or_else(invalid)?.into(),
            DecimalU64(control_bytes.parse().map_err(|_| invalid())?),
        )?;
        if control_pin.bytes() > kinekino_vad_executor::application::MAX_WORKER_CONTROL_BYTES as u64
        {
            return Err(invalid());
        }
        Ok(Self {
            request,
            cancel_signal,
            manifest,
            control_path,
            control_pin,
        })
    }
}

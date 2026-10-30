use super::super::super::vad_job_files::VadJobFile;
use super::{
    AnalysisControl, ApiError, ArtifactPin, DecimalU64, ErrorCode, FixedVadFile, GuardedAsset,
    MAX_AUDIO_BYTES, SnapshotLease, cache,
};
use kinekino_vad::application::wav_pcm;
use kinekino_vad_executor::infrastructure::GuardedOutputFile;
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
pub(super) struct PreparedAudio {
    pub path: PathBuf,
    pub source: ArtifactPin,
    pub wav: ArtifactPin,
    pub pcm: ArtifactPin,
    pub samples: DecimalU64,
    pub bytes: Vec<u8>,
    pub lease: Arc<dyn SnapshotLease>,
}
pub(super) fn snapshot(
    root: &Path,
    id: &str,
    expected_sha: &str,
    control: Option<&dyn AnalysisControl>,
) -> Result<PreparedAudio, ApiError> {
    if !crate::domain::identifiers::valid_id(id) {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Invalid opaque audio identity",
        ));
    }
    cache::check(control)?;
    let path = root.join(format!("audio-{id}.wav"));
    if !path
        .try_exists()
        .map_err(kinekino_core::file_publication::io_error)?
    {
        return Err(ApiError::new(
            ErrorCode::NeedsInput,
            "Requested prepared WAV artifact is not available",
        ));
    }
    let original = VadJobFile::from_application_path(&path)?;
    let initial = original.open_actual()?;
    let length = initial
        .metadata()
        .map_err(kinekino_core::file_publication::io_error)?
        .len();
    if length == 0 || length > MAX_AUDIO_BYTES {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Prepared WAV exceeds product input bound",
        ));
    }
    let source = ArtifactPin::new(
        format!("audio-source-{id}"),
        expected_sha.into(),
        DecimalU64(length),
    )?;
    let asset = GuardedAsset::new(
        FixedVadFile {
            path: path.clone(),
            pin: source.clone(),
        },
        MAX_AUDIO_BYTES,
    )?;
    let (bytes, lease) = asset.snapshot_with_control(control)?;
    drop(initial);
    let pcm_bytes = wav_pcm(&bytes)?;
    let pcm = ArtifactPin::new(
        format!("pcm-{id}"),
        format!("{:x}", Sha256::digest(pcm_bytes)),
        DecimalU64(pcm_bytes.len() as u64),
    )?;
    let samples = DecimalU64(pcm_bytes.len() as u64 / 2);
    let wav = ArtifactPin::new(
        format!("wav-{id}"),
        source.sha256().into(),
        DecimalU64(length),
    )?;
    Ok(PreparedAudio {
        path,
        source,
        wav,
        pcm,
        samples,
        bytes,
        lease,
    })
}
#[cfg(test)]
#[path = "audio/tests.rs"]
mod tests;

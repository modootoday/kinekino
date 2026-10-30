use super::raw::RetainedRaw;
use crate::application::api::{ApiError, ErrorCode};
use sha2::{Digest, Sha256};
use std::path::Path;
/// Trusted assembly supplies existing file pins, never a wire authority token.
pub struct PinnedPcmFile<'a> {
    pub path: &'a Path,
    pub sha256: &'a str,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PcmLineage {
    pub full_samples: u64,
    pub clip_start_sample: u64,
    pub clip_samples: u64,
    pub full_data_sha256: String,
    pub clip_data_sha256: String,
    pub replay_data_sha256: String,
}
/// Retains all original files and ancestors. No decode/resample or inferred source binding.
pub struct RetainedPcmLineage {
    facts: PcmLineage,
    _files: [RetainedRaw; 3],
}
impl RetainedPcmLineage {
    pub fn open(
        full: PinnedPcmFile<'_>,
        clip: PinnedPcmFile<'_>,
        replay: PinnedPcmFile<'_>,
        start_sample: u64,
    ) -> Result<Self, ApiError> {
        let mut whole = RetainedRaw::open_bounded(full.path, full.sha256, 64 * 1024 * 1024)?;
        let mut selected = RetainedRaw::open_bounded(clip.path, clip.sha256, 64 * 1024 * 1024)?;
        let mut decoded = RetainedRaw::open_bounded(replay.path, replay.sha256, 64 * 1024 * 1024)?;
        let facts = compare(
            &whole.read_bytes()?,
            &selected.read_bytes()?,
            &decoded.read_bytes()?,
            start_sample,
        )?;
        Ok(Self {
            facts,
            _files: [whole, selected, decoded],
        })
    }
    pub fn facts(&self) -> &PcmLineage {
        &self.facts
    }
}
pub(super) fn compare(
    full_wav: &[u8],
    clip_wav: &[u8],
    replay: &[u8],
    start_sample: u64,
) -> Result<PcmLineage, ApiError> {
    let full = kinekino_vad::application::wav_pcm(full_wav)?;
    let clip = kinekino_vad::application::wav_pcm(clip_wav)?;
    if full != replay {
        return Err(conflict("Replay PCM differs from original WAV data"));
    }
    let start = start_sample
        .checked_mul(2)
        .and_then(|v| usize::try_from(v).ok())
        .ok_or_else(|| invalid("PCM sample offset overflow"))?;
    let end = start
        .checked_add(clip.len())
        .ok_or_else(|| invalid("PCM range overflow"))?;
    let slice = full
        .get(start..end)
        .ok_or_else(|| invalid("Clip exceeds full PCM samples"))?;
    if slice != clip {
        return Err(conflict("Clip differs from exact original sample range"));
    }
    Ok(PcmLineage {
        full_samples: full.len() as u64 / 2,
        clip_start_sample: start_sample,
        clip_samples: clip.len() as u64 / 2,
        full_data_sha256: hash(full),
        clip_data_sha256: hash(clip),
        replay_data_sha256: hash(replay),
    })
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn invalid(message: &str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
fn conflict(message: &str) -> ApiError {
    ApiError::new(ErrorCode::RevisionConflict, message)
}
#[cfg(test)]
mod tests;

use super::*;
use crate::infrastructure::timeline_evidence::{RetainedAudioClock, raw::RetainedRaw};

/// Original raw mono16k/s16 PCM; no WAV header conversion or lineage inference.
pub struct RetainedProducedPcm {
    pub(super) path: PathBuf,
    raw: RetainedRaw,
    samples: u64,
    first_pts: i64,
    end_pts: i128,
}
impl RetainedProducedPcm {
    pub(super) fn open(
        path: &Path,
        sha256: &str,
        clock: &RetainedAudioClock,
    ) -> Result<Self, ApiError> {
        let mut raw = RetainedRaw::open_bounded(path, sha256, 64 * 1024 * 1024)?;
        let bytes = raw.read_bytes()?;
        let (samples, first_pts, end_pts) = validate_count(bytes.len(), clock.frames())?;
        Ok(Self {
            path: path.to_path_buf(),
            raw,
            samples,
            first_pts,
            end_pts,
        })
    }
    pub fn sha256(&self) -> &str {
        self.raw.sha256()
    }
    pub fn samples(&self) -> u64 {
        self.samples
    }
    pub fn first_pts(&self) -> i64 {
        self.first_pts
    }
    pub fn end_pts(&self) -> i128 {
        self.end_pts
    }
    pub(super) fn read_bytes(&self) -> Result<Vec<u8>, ApiError> {
        let mut independent =
            RetainedRaw::open_bounded(&self.path, self.sha256(), 64 * 1024 * 1024)?;
        independent.read_bytes()
    }
}
fn validate_count(
    bytes: usize,
    frames: &[crate::domain::review_source::timeline_evidence::AudioFrame],
) -> Result<(u64, i64, i128), ApiError> {
    let first = frames.first().ok_or_else(|| invalid("PCM clock missing"))?;
    let last = frames.last().ok_or_else(|| invalid("PCM clock missing"))?;
    let samples = frames
        .iter()
        .try_fold(0u64, |sum, frame| sum.checked_add(u64::from(frame.samples)))
        .ok_or_else(|| invalid("PCM clock sample count overflow"))?;
    let end = i128::from(last.pts) + i128::from(last.samples);
    if bytes == 0
        || !bytes.is_multiple_of(2)
        || samples != bytes as u64 / 2
        || end - i128::from(first.pts) != i128::from(samples)
    {
        return Err(ApiError::new(
            ErrorCode::RevisionConflict,
            "Raw PCM byte count differs from retained contiguous audio clock",
        ));
    }
    Ok((samples, first.pts, end))
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::review_source::timeline_evidence::AudioFrame;
    #[test]
    fn exact_pcm_sample_count_preserves_negative_pts_and_rejects_truncation() {
        let frames = [
            AudioFrame {
                sequence: 0,
                pts: -2,
                samples: 4,
            },
            AudioFrame {
                sequence: 1,
                pts: 2,
                samples: 3,
            },
        ];
        assert_eq!(validate_count(14, &frames).unwrap(), (7, -2, 5));
        for bytes in [0, 1, 12, 13, 15, 16] {
            assert!(validate_count(bytes, &frames).is_err());
        }
        assert!(validate_count(14, &[]).is_err());
    }
}

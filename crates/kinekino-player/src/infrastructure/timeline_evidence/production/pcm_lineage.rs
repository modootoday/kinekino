use super::{
    process::{before, failed, receipt_json},
    *,
};
use crate::{
    application::review_source::MutationControl,
    infrastructure::{
        content_digest,
        timeline_evidence::{PcmLineage, RetainedAudioClock, raw::RetainedRaw},
    },
};
use kinekino_process::domain::observation::ProcessReceipt;
use sha2::{Digest, Sha256};
use std::{io::Write, ops::Range};

/// Half-open original audio clock PTS, always in 1/16000 units, not video ticks.
#[derive(Clone, Copy, Debug)]
pub struct PcmClockRange {
    start: i64,
    end: i64,
}
impl PcmClockRange {
    pub fn new(start: i64, end: i64) -> Result<Self, ApiError> {
        let span = i128::from(end) - i128::from(start);
        if !(1..=33_554_432).contains(&span) {
            return Err(invalid("PCM range must be positive and bounded"));
        }
        Ok(Self { start, end })
    }
    fn bytes(self, first_pts: i64, samples: u64) -> Result<Range<usize>, ApiError> {
        let start = i128::from(self.start) - i128::from(first_pts);
        let end = i128::from(self.end) - i128::from(first_pts);
        if start < 0 || end > i128::from(samples) {
            return Err(invalid("PCM clock range exceeds retained source"));
        }
        Ok(
            usize::try_from(start * 2).map_err(|_| invalid("PCM offset overflow"))?
                ..usize::try_from(end * 2).map_err(|_| invalid("PCM endpoint overflow"))?,
        )
    }
}
/// Retains original decode, replay invocation, exact raw clip and clock. Not a Verified lease.
pub struct ProducedPcmLineage<'s, 'a> {
    pub(super) original: &'s ProducedAudioPcm<'s, 'a>,
    _replay: RetainedProducedPcm,
    _clock: RetainedAudioClock,
    _clip: RetainedRaw,
    receipt: ProcessReceipt,
    root: PathBuf,
    facts: PcmLineage,
    _records: Vec<File>,
}
impl ProducedPcmLineage<'_, '_> {
    pub fn facts(&self) -> &PcmLineage {
        &self.facts
    }
    pub fn receipt(&self) -> &ProcessReceipt {
        &self.receipt
    }
    pub fn output_root(&self) -> &Path {
        &self.root
    }
    pub fn check_current(&self) -> Result<(), ApiError> {
        self.original.clock.check_current()
    }
}
fn exact_replay<'a>(
    original: &'a [u8],
    replay: &[u8],
    range: Range<usize>,
) -> Result<&'a [u8], ApiError> {
    if original != replay {
        return Err(ApiError::new(
            ErrorCode::RevisionConflict,
            "Replay PCM differs from original decode bytes",
        ));
    }
    original
        .get(range)
        .ok_or_else(|| invalid("Clip byte range exceeds PCM"))
}
impl<'a> TimelineProductionSession<'a> {
    pub fn pcm_lineage<'s>(
        &'s self,
        original: &'s ProducedAudioPcm<'s, 'a>,
        tool: &'s PinnedTimelineAudioDecoder,
        range: PcmClockRange,
        control: &dyn MutationControl,
    ) -> Result<ProducedPcmLineage<'s, 'a>, Box<TimelineProcessFailure<'s, 'a>>> {
        if !std::ptr::eq(self, original.clock.session)
            || tool.sha256() != original.clock._tool.sha256()
        {
            return Err(before(ApiError::new(
                ErrorCode::SourceExpired,
                "PCM lineage source session or decoder differs",
            )));
        }
        let bytes = range
            .bytes(original.pcm.first_pts(), original.pcm.samples())
            .map_err(before)?;
        original.clock.check_current().map_err(before)?;
        let arguments = crate::infrastructure::timeline_evidence::live_contract::replay_arguments(
            &original.pcm.path,
        );
        let mut owner = self.execute(
            &tool.0,
            "pcm-replay",
            arguments,
            Some(&original.pcm),
            None,
            control,
        )?;
        let Some(receipt) = owner.receipt().cloned() else {
            return Err(failed(invalid("PCM replay receipt missing"), owner));
        };
        let result = (|| {
            owner
                .paths
                .check_limit(self.max_output_bytes)
                .map_err(io_error)?;
            owner._records.push(write_new(
                &owner.root.join("PROCESS.json"),
                &receipt_json(&receipt),
            )?);
            let mut file = owner.paths.stderr.open_read().map_err(io_error)?;
            let (_, clock_sha) = content_digest::sha256(&mut file, self.max_output_bytes)?;
            let clock = RetainedAudioClock::open(owner.paths.stderr.path(), &clock_sha)?;
            let mut file = owner.paths.stdout.open_read().map_err(io_error)?;
            let (_, pcm_sha) = content_digest::sha256(&mut file, self.max_output_bytes)?;
            let replay = RetainedProducedPcm::open(owner.paths.stdout.path(), &pcm_sha, &clock)?;
            let original_bytes = original.pcm.read_bytes()?;
            let replay_bytes = replay.read_bytes()?;
            let clip_bytes = exact_replay(&original_bytes, &replay_bytes, bytes.clone())?;
            control.checkpoint()?;
            self.source.check_current()?;
            if self.deadline.operation_remaining(Instant::now()).is_zero() {
                return Err(ApiError::new(
                    ErrorCode::Timeout,
                    "PCM comparison exceeded original deadline",
                ));
            }
            let clip_path = owner.root.join("clip.s16le");
            let mut clip_file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&clip_path)
                .map_err(io_error)?;
            clip_file.write_all(clip_bytes).map_err(io_error)?;
            clip_file.sync_all().map_err(io_error)?;
            drop(clip_file);
            let clip_sha = format!("{:x}", Sha256::digest(clip_bytes));
            let clip = RetainedRaw::open_bounded(&clip_path, &clip_sha, 64 * 1024 * 1024)?;
            let facts = PcmLineage {
                full_samples: original.pcm.samples(),
                clip_start_sample: bytes.start as u64 / 2,
                clip_samples: clip_bytes.len() as u64 / 2,
                full_data_sha256: original.pcm.sha256().to_owned(),
                clip_data_sha256: clip_sha,
                replay_data_sha256: pcm_sha,
            };
            control.checkpoint()?;
            self.source.check_current()?;
            if self.deadline.operation_remaining(Instant::now()).is_zero() {
                return Err(ApiError::new(
                    ErrorCode::Timeout,
                    "PCM clip write exceeded original deadline",
                ));
            }
            owner._records.push(write_new(&owner.root.join("RETURN.json"), &serde_json::json!({
                "source":self.source.identity(),"original_pcm_root":original.clock.output_root(),"original_pcm_sha256":facts.full_data_sha256,
                "replay_pcm_sha256":facts.replay_data_sha256,"clip_sha256":facts.clip_data_sha256,"full_samples":facts.full_samples,
                "clip_start_sample":facts.clip_start_sample,"clip_samples":facts.clip_samples,"source_clip_start_pts":range.start,"source_clip_end_pts":range.end,
                "clock_time_base":"1/16000","replay_first_pts":replay.first_pts(),"replay_end_pts":replay.end_pts(),"child":receipt_json(&receipt),
                "pcm_lineage_validated":true,"authority_granted":false,"verified_timeline":false,"quality_approved":false
            }))?);
            control.checkpoint()?;
            self.source.check_current()?;
            if self.deadline.operation_remaining(Instant::now()).is_zero() {
                return Err(ApiError::new(
                    ErrorCode::Timeout,
                    "PCM lineage evidence write exceeded original deadline; validated evidence may exist",
                ));
            }
            Ok::<_, ApiError>((replay, clock, clip, facts))
        })();
        match result {
            Ok((replay, clock, clip, facts)) => Ok(ProducedPcmLineage {
                original,
                _replay: replay,
                _clock: clock,
                _clip: clip,
                receipt,
                root: owner.root.clone(),
                facts,
                _records: std::mem::take(&mut owner._records),
            }),
            Err(error) => Err(failed(error, owner)),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pcm_clock_range_uses_signed_origin_and_exact_half_open_sample_offsets() {
        assert_eq!(
            PcmClockRange::new(-1, 3).unwrap().bytes(-2, 7).unwrap(),
            2..10
        );
        assert_eq!(
            PcmClockRange::new(-2, 5).unwrap().bytes(-2, 7).unwrap(),
            0..14
        );
        for (start, end) in [(0, 0), (1, 0), (i64::MIN, i64::MAX), (0, 33_554_433)] {
            assert!(PcmClockRange::new(start, end).is_err());
        }
        assert!(PcmClockRange::new(-3, 0).unwrap().bytes(-2, 7).is_err());
        assert!(PcmClockRange::new(0, 6).unwrap().bytes(-2, 7).is_err());
    }
    #[test]
    fn replay_requires_exact_bytes_and_clip_uses_unmodified_source_slice() {
        let bytes = [1, 2, 3, 4, 5, 6];
        assert_eq!(exact_replay(&bytes, &bytes, 2..6).unwrap(), &[3, 4, 5, 6]);
        assert!(exact_replay(&bytes, &[1, 2, 3, 4, 5, 7], 2..6).is_err());
        assert!(exact_replay(&bytes, &bytes, 2..8).is_err());
    }
}

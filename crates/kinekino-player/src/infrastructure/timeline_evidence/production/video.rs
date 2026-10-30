use super::process::{before, failed, receipt_json};
use super::*;
use crate::{
    application::review_source::MutationControl,
    infrastructure::{content_digest, timeline_evidence::RetainedVideoFrames},
};
use kinekino_process::domain::observation::ProcessReceipt;
pub type PendingVideoProbe<'s, 'a> = super::process::PendingTimelineProcess<'s, 'a>;
pub type VideoProbeFailure<'s, 'a> = super::process::TimelineProcessFailure<'s, 'a>;
#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug)]
pub struct VideoProbeWindow {
    start_us: u64,
    end_us: u64,
}
impl VideoProbeWindow {
    pub fn new(start_us: u64, end_us: u64) -> Result<Self, ApiError> {
        if end_us > 86_400_000_000
            || end_us
                .checked_sub(start_us)
                .is_none_or(|span| span == 0 || span > 60_000_000)
        {
            return Err(invalid(
                "Explicit video window must be positive, at most 60 seconds and within 24 hours",
            ));
        }
        Ok(Self { start_us, end_us })
    }
    fn interval(self) -> String {
        format!(
            "{}.{:06}%{}.{:06}",
            self.start_us / 1_000_000,
            self.start_us % 1_000_000,
            self.end_us / 1_000_000,
            self.end_us % 1_000_000
        )
    }
}
/// Partial video evidence only. Not a TimelineEvidenceLease or Verified grant.
pub struct ProducedVideoFrames<'s, 'a> {
    pub(super) session: &'s TimelineProductionSession<'a>,
    pub(super) _tool: &'s PinnedTimelineProbe,
    frames: RetainedVideoFrames,
    receipt: ProcessReceipt,
    pub(super) interval: String,
    root: PathBuf,
    _records: Vec<File>,
}
impl ProducedVideoFrames<'_, '_> {
    pub fn frames(&self) -> &RetainedVideoFrames {
        &self.frames
    }
    pub fn receipt(&self) -> &ProcessReceipt {
        &self.receipt
    }
    pub fn output_root(&self) -> &Path {
        &self.root
    }
    pub fn check_current(&self) -> Result<(), ApiError> {
        self.session.source.check_current()
    }
}

impl<'a> TimelineProductionSession<'a> {
    pub fn video<'s>(
        &'s self,
        tool: &'s PinnedTimelineProbe,
        window: VideoProbeWindow,
        control: &dyn MutationControl,
    ) -> Result<ProducedVideoFrames<'s, 'a>, Box<VideoProbeFailure<'s, 'a>>> {
        let interval = window.interval();
        let arguments = crate::infrastructure::timeline_evidence::live_contract::video_arguments(
            self.source.path(),
            &interval,
            &self.source.identity().classification,
        )
        .map_err(before)?;
        let mut owner = self.execute(&tool.0, "video", arguments, None, None, control)?;
        let Some(receipt) = owner.receipt().cloned() else {
            return Err(failed(
                invalid("Completed timeline child receipt missing"),
                owner,
            ));
        };
        let final_result = (|| {
            owner._records.push(write_new(
                &owner.root.join("PROCESS.json"),
                &receipt_json(&receipt),
            )?);
            self.source.check_current()?;
            let mut file = owner.paths.stdout.open_read().map_err(io_error)?;
            let (_, sha) = content_digest::sha256(&mut file, self.max_output_bytes)?;
            let frames = RetainedVideoFrames::open_live(
                owner.paths.stdout.path(),
                &sha,
                &self.source.identity().classification,
            )?;
            owner
                ._records
                .push(rooted::file(owner.paths.stderr.path())?);
            let mut stderr = owner.paths.stderr.open_read().map_err(io_error)?;
            let (_, stderr_sha) = content_digest::sha256(&mut stderr, self.max_output_bytes)?;
            control.checkpoint()?;
            self.source.check_current()?;
            if self.deadline.operation_remaining(Instant::now()).is_zero() {
                return Err(ApiError::new(
                    ErrorCode::Timeout,
                    "Timeline video validation exceeded the original deadline",
                ));
            }
            let (numerator, denominator) = frames.time_base().ratio();
            owner._records.push(write_new(&owner.root.join("RETURN.json"),&serde_json::json!({"source":self.source.identity(),"stdout_sha256":sha,"stderr_sha256":stderr_sha,"frames":frames.frames().len(),"time_base":format!("{numerator}/{denominator}"),"stream_index":frames.stream_index(),"child":receipt_json(&receipt),"video_frames_validated":true,"authority_granted":false,"verified_timeline":false,"quality_approved":false}))?);
            control.checkpoint()?;
            self.source.check_current()?;
            if self.deadline.operation_remaining(Instant::now()).is_zero() {
                return Err(ApiError::new(
                    ErrorCode::Timeout,
                    "Timeline video evidence write exceeded the original deadline; validated raw evidence may exist",
                ));
            }
            Ok::<_, ApiError>(frames)
        })();
        match final_result {
            Ok(frames) => Ok(ProducedVideoFrames {
                session: self,
                _tool: tool,
                frames,
                receipt,
                interval,
                root: owner.root.clone(),
                _records: std::mem::take(&mut owner._records),
            }),
            Err(error) => Err(failed(error, owner)),
        }
    }
}

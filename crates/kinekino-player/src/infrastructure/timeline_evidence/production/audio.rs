use super::process::{before, failed, receipt_json};
use super::*;
use crate::{
    application::review_source::MutationControl,
    infrastructure::{content_digest, timeline_evidence::RetainedAudioClock},
};
use kinekino_process::domain::observation::ProcessReceipt;

/// Original source decode clock only: not PCM lineage or Verified authority.
pub struct ProducedAudioClock<'s, 'a> {
    pub(super) session: &'s TimelineProductionSession<'a>,
    pub(super) _tool: &'s PinnedTimelineAudioDecoder,
    clock: RetainedAudioClock,
    receipt: ProcessReceipt,
    root: PathBuf,
    _records: Vec<File>,
}
impl ProducedAudioClock<'_, '_> {
    pub fn clock(&self) -> &RetainedAudioClock {
        &self.clock
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

pub struct ProducedAudioPcm<'s, 'a> {
    pub(super) clock: ProducedAudioClock<'s, 'a>,
    pub(super) pcm: RetainedProducedPcm,
}
impl ProducedAudioPcm<'_, '_> {
    pub fn clock(&self) -> &ProducedAudioClock<'_, '_> {
        &self.clock
    }
    pub fn pcm(&self) -> &RetainedProducedPcm {
        &self.pcm
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum AudioOutput {
    ClockOnly,
    Pcm,
}
impl<'a> TimelineProductionSession<'a> {
    pub fn audio_clock<'s>(
        &'s self,
        tool: &'s PinnedTimelineAudioDecoder,
        control: &dyn MutationControl,
    ) -> Result<ProducedAudioClock<'s, 'a>, Box<super::process::TimelineProcessFailure<'s, 'a>>>
    {
        self.decode_audio(tool, control, AudioOutput::ClockOnly)
            .map(|(clock, _)| clock)
    }
    pub fn audio_pcm<'s>(
        &'s self,
        tool: &'s PinnedTimelineAudioDecoder,
        control: &dyn MutationControl,
    ) -> Result<ProducedAudioPcm<'s, 'a>, Box<super::process::TimelineProcessFailure<'s, 'a>>> {
        let (clock, pcm) = self.decode_audio(tool, control, AudioOutput::Pcm)?;
        let pcm = pcm.ok_or_else(|| before(invalid("Original PCM observation missing")))?;
        Ok(ProducedAudioPcm { clock, pcm })
    }
    fn decode_audio<'s>(
        &'s self,
        tool: &'s PinnedTimelineAudioDecoder,
        control: &dyn MutationControl,
        output: AudioOutput,
    ) -> Result<
        (ProducedAudioClock<'s, 'a>, Option<RetainedProducedPcm>),
        Box<super::process::TimelineProcessFailure<'s, 'a>>,
    > {
        let arguments = crate::infrastructure::timeline_evidence::live_contract::audio_arguments(
            self.source.path(),
            output == AudioOutput::Pcm,
            &self.source.identity().classification,
        );
        let mut owner = self.execute(
            &tool.0,
            if output == AudioOutput::Pcm {
                "audio-pcm"
            } else {
                "audio-clock"
            },
            arguments,
            None,
            None,
            control,
        )?;
        let Some(receipt) = owner.receipt().cloned() else {
            return Err(failed(
                invalid("Completed audio child receipt missing"),
                owner,
            ));
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
            let (_, sha) = content_digest::sha256(&mut file, self.max_output_bytes)?;
            let clock = RetainedAudioClock::open_live(
                owner.paths.stderr.path(),
                &sha,
                self.max_output_bytes,
            )?;
            let pcm = if output == AudioOutput::Pcm {
                let mut file = owner.paths.stdout.open_read().map_err(io_error)?;
                let (_, sha) = content_digest::sha256(&mut file, self.max_output_bytes)?;
                Some(RetainedProducedPcm::open(
                    owner.paths.stdout.path(),
                    &sha,
                    &clock,
                )?)
            } else {
                None
            };
            control.checkpoint()?;
            self.source.check_current()?;
            if self.deadline.operation_remaining(Instant::now()).is_zero() {
                return Err(ApiError::new(
                    ErrorCode::Timeout,
                    "Audio clock validation exceeded original deadline",
                ));
            }
            owner._records.push(write_new(&owner.root.join("RETURN.json"), &serde_json::json!({
                "source":self.source.identity(), "stderr_sha256":sha, "frames":clock.frames().len(),
                "sample_rate":clock.sample_rate(), "child":receipt_json(&receipt),
                "audio_clock_validated":true, "pcm_lineage_validated":false,
                "original_pcm":pcm.as_ref().map(|p| serde_json::json!({"sha256":p.sha256(),"samples":p.samples(),"first_pts":p.first_pts(),"end_pts":p.end_pts()})),
                "authority_granted":false, "verified_timeline":false, "quality_approved":false
            }))?);
            control.checkpoint()?;
            self.source.check_current()?;
            if self.deadline.operation_remaining(Instant::now()).is_zero() {
                return Err(ApiError::new(
                    ErrorCode::Timeout,
                    "Audio evidence write exceeded original deadline; validated raw evidence may exist",
                ));
            }
            Ok::<_, ApiError>((clock, pcm))
        })();
        match result {
            Ok((clock, pcm)) => Ok((
                ProducedAudioClock {
                    session: self,
                    _tool: tool,
                    clock,
                    receipt,
                    root: owner.root.clone(),
                    _records: std::mem::take(&mut owner._records),
                },
                pcm,
            )),
            Err(error) => Err(failed(error, owner)),
        }
    }
}

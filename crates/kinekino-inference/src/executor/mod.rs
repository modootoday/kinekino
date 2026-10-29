//! Transcribe executor (application layer, OS-independent): VAD speech regions → WAV cuts →
//! consent, cost guard and cancel checks → `InferenceBackend` → ledger + offset chunks.
//! Only cut segments leave the machine, never the whole track. Translation, OCR and
//! diarization executors share the same consent, guard and cancel gate (`step`).
mod audio;
mod diarize;
mod ocr;
mod step;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_ai;
mod translate;

pub use audio::{SpeechRegion, speech_regions, wav_segment};
pub use diarize::{Diarization, SpeakerSegment, assign_speakers, diarize};
pub use ocr::{Frame, OcrSpans, ocr_frames};
pub use step::{Spend, StepError};
pub use translate::{Translation, translate, translation_prompt};

/// Job context shared by every AI executor.
pub type AiJob<'a> = TranscribeJob<'a>;

use crate::budget::{CostGuard, GuardDecision, LedgerEntry};
use crate::consent::Consent;
use crate::port::{Chunk, InferenceBackend, InferenceError, InferenceRequest, Output, Provenance};

/// Mono 16-bit PCM track.
#[derive(Debug, Clone, PartialEq)]
pub struct PcmTrack {
    pub sample_rate: u32,
    pub samples: Vec<i16>,
}

impl PcmTrack {
    pub fn duration_secs(&self) -> f64 {
        if self.sample_rate == 0 {
            0.0
        } else {
            self.samples.len() as f64 / f64::from(self.sample_rate)
        }
    }
}

/// Per-job context the composition root fills from settings and the Jobs worker.
pub struct TranscribeJob<'a> {
    pub project: &'a str,
    pub job: &'a str,
    pub month: &'a str,
    pub model: &'a str,
    /// The user already answered the near-limit / unknown-cost dialog with "continue".
    pub confirmed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Transcript {
    pub text: String,
    /// Track-relative seconds.
    pub chunks: Vec<Chunk>,
    pub provenance: Vec<Provenance>,
    pub cost_usd: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TranscribeError {
    /// Guard wants the limit dialog; nothing was sent for region `region`.
    NeedsConfirm { region: usize, reason: String },
    /// Region index that failed and the chunks finished before it.
    Failed {
        region: usize,
        error: InferenceError,
        partial: Box<Transcript>,
    },
}

/// Runs every region in order. Consent is checked once before any upload; the guard before
/// each call; `cancel` between regions and inside the backend.
pub fn transcribe(
    backend: &mut dyn InferenceBackend,
    guard: &mut CostGuard,
    consent: &Consent,
    job: &TranscribeJob<'_>,
    track: &PcmTrack,
    regions: &[SpeechRegion],
    cancel: &dyn Fn() -> bool,
) -> Result<Transcript, TranscribeError> {
    let mut out = Transcript {
        text: String::new(),
        chunks: Vec::new(),
        provenance: Vec::new(),
        cost_usd: 0.0,
    };
    let fail = |region, error, out: &Transcript| TranscribeError::Failed {
        region,
        error,
        partial: Box::new(out.clone()),
    };
    let local = matches!(backend.kind(), crate::port::InferenceKind::LocalPack);
    if !local && let Err(e) = consent.check(job.project, job.job) {
        return Err(fail(0, e, &out));
    }
    for (index, region) in regions.iter().enumerate() {
        if cancel() {
            return Err(fail(index, InferenceError::Cancelled, &out));
        }
        let audio = wav_segment(track, *region).map_err(|e| fail(index, e, &out))?;
        let request = InferenceRequest::Transcribe {
            model: job.model.into(),
            audio,
        };
        match guard.check(job.month, backend.estimate_usd(&request)) {
            Err(e) => return Err(fail(index, e, &out)),
            Ok(GuardDecision::Confirm(reason)) if !job.confirmed => {
                return Err(TranscribeError::NeedsConfirm {
                    region: index,
                    reason,
                });
            }
            Ok(_) => {}
        }
        let response = backend
            .run(&request, cancel)
            .map_err(|e| fail(index, e, &out))?;
        let Output::Transcribe { text, chunks } = response.output else {
            let got = response.output.task().name();
            return Err(fail(
                index,
                InferenceError::Protocol(format!("expected transcribe output, got {got}")),
                &out,
            ));
        };
        let cost = response.provenance.cost_estimate_usd;
        guard.record(LedgerEntry {
            month: job.month.into(),
            backend: response.provenance.backend.clone(),
            model: response.provenance.model.clone(),
            cost_usd: cost,
        });
        out.cost_usd += cost;
        let text = text.trim();
        if !text.is_empty() {
            if !out.text.is_empty() {
                out.text.push(' ');
            }
            out.text.push_str(text);
        }
        if chunks.is_empty() && !text.is_empty() {
            out.chunks.push(Chunk {
                start: region.start,
                end: region.end,
                text: text.into(),
            });
        }
        out.chunks.extend(chunks.into_iter().map(|c| Chunk {
            start: (region.start + c.start).min(region.end),
            end: (region.start + c.end).min(region.end),
            text: c.text,
        }));
        out.provenance.push(response.provenance);
    }
    Ok(out)
}

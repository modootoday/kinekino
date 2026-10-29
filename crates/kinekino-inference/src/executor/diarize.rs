//! Speaker diarization executor: one `Diarize` call per speech region, segments shifted to
//! track time and clamped to their region. Labels are the backend's; a backend that must keep
//! identities across the track should be given one region covering it.
use super::step::{self, Spend, StepError};
use super::{PcmTrack, SpeechRegion, TranscribeJob, wav_segment};
use crate::budget::CostGuard;
use crate::consent::Consent;
use crate::port::{Chunk, InferenceBackend, InferenceRequest, Output};

#[derive(Debug, Clone, PartialEq)]
pub struct SpeakerSegment {
    pub start: f64,
    pub end: f64,
    pub speaker: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Diarization {
    pub segments: Vec<SpeakerSegment>,
    pub spend: Spend,
}

pub fn diarize(
    backend: &mut dyn InferenceBackend,
    guard: &mut CostGuard,
    consent: &Consent,
    job: &TranscribeJob<'_>,
    track: &PcmTrack,
    regions: &[SpeechRegion],
    cancel: &dyn Fn() -> bool,
) -> Result<Diarization, StepError<Diarization>> {
    let mut out = Diarization::default();
    if let Err(e) = step::consent(backend, consent, job) {
        return Err(step::Stop::Error(e).at(0, &out));
    }
    for (index, region) in regions.iter().enumerate() {
        if cancel() {
            let e = crate::port::InferenceError::Cancelled;
            return Err(step::Stop::Error(e).at(index, &out));
        }
        let audio =
            wav_segment(track, *region).map_err(|e| step::Stop::Error(e).at(index, &out))?;
        let request = InferenceRequest::Diarize {
            model: job.model.into(),
            audio,
        };
        let response =
            step::call(backend, guard, job, &request, cancel).map_err(|s| s.at(index, &out))?;
        let Output::Diarize { segments } = response.output else {
            unreachable!("step::call checks the output task")
        };
        out.spend.add(response.provenance);
        out.segments
            .extend(segments.into_iter().filter_map(|(s, e, speaker)| {
                let start = (region.start + s).clamp(region.start, region.end);
                let end = (region.start + e).clamp(region.start, region.end);
                (end > start).then_some(SpeakerSegment {
                    start,
                    end,
                    speaker,
                })
            }));
    }
    Ok(out)
}

/// Speaker with the largest time overlap for each chunk; `None` when nothing overlaps.
pub fn assign_speakers(chunks: &[Chunk], segments: &[SpeakerSegment]) -> Vec<Option<String>> {
    chunks
        .iter()
        .map(|c| {
            let mut best: Option<(&str, f64)> = None;
            for s in segments {
                let overlap = c.end.min(s.end) - c.start.max(s.start);
                if overlap > 0.0 && best.is_none_or(|(_, b)| overlap > b) {
                    best = Some((&s.speaker, overlap));
                }
            }
            best.map(|(s, _)| s.to_string())
        })
        .collect()
}

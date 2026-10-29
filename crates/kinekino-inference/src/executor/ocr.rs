//! On-screen text executor: one `Ocr` call per sampled frame; equal consecutive texts merge
//! into one timed span, empty frames end a span.
use super::TranscribeJob;
use super::step::{self, Spend, StepError};
use crate::budget::CostGuard;
use crate::consent::Consent;
use crate::port::{Chunk, InferenceBackend, InferenceError, InferenceRequest, Output};

/// A sampled frame (track-relative seconds, PNG bytes). Only frames leave the machine.
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub time: f64,
    pub png: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct OcrSpans {
    pub spans: Vec<Chunk>,
    pub spend: Spend,
}

/// `frame_secs` is the sampling interval: a frame's text lasts until the next frame or
/// `time + frame_secs`, whichever is first. Frames must be strictly increasing in time.
pub fn ocr_frames(
    backend: &mut dyn InferenceBackend,
    guard: &mut CostGuard,
    consent: &Consent,
    job: &TranscribeJob<'_>,
    frames: &[Frame],
    frame_secs: f64,
    cancel: &dyn Fn() -> bool,
) -> Result<OcrSpans, StepError<OcrSpans>> {
    let mut out = OcrSpans::default();
    let invalid = |m: &str| step::Stop::Error(InferenceError::InvalidInput(m.into()));
    if frame_secs.is_nan() || frame_secs <= 0.0 {
        return Err(invalid("frame interval must be positive").at(0, &out));
    }
    if frames.windows(2).any(|w| w[1].time <= w[0].time) {
        return Err(invalid("frames must be strictly increasing in time").at(0, &out));
    }
    if let Err(e) = step::consent(backend, consent, job) {
        return Err(step::Stop::Error(e).at(0, &out));
    }
    let mut open = false;
    for (index, frame) in frames.iter().enumerate() {
        let request = InferenceRequest::Ocr {
            model: job.model.into(),
            image: frame.png.clone(),
        };
        let response =
            step::call(backend, guard, job, &request, cancel).map_err(|s| s.at(index, &out))?;
        let Output::Ocr { text } = response.output else {
            unreachable!("step::call checks the output task")
        };
        out.spend.add(response.provenance);
        let end = frames.get(index + 1).map_or(frame.time + frame_secs, |n| {
            n.time.min(frame.time + frame_secs)
        });
        let text = text.trim();
        match out.spans.last_mut() {
            _ if text.is_empty() => open = false,
            Some(last) if open && last.text == text => last.end = end,
            _ => {
                out.spans.push(Chunk {
                    start: frame.time,
                    end,
                    text: text.into(),
                });
                open = true;
            }
        }
    }
    Ok(out)
}

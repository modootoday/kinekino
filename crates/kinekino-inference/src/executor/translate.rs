//! Cue translation executor: one `Generate` call per non-empty cue, timing kept verbatim.
use super::TranscribeJob;
use super::step::{self, Spend, StepError};
use crate::budget::CostGuard;
use crate::consent::Consent;
use crate::port::{Chunk, InferenceBackend, InferenceError, InferenceRequest, Output};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Translation {
    /// Same length and timing as the input cues.
    pub cues: Vec<Chunk>,
    pub spend: Spend,
}

/// Deterministic prompt; the cue text is fenced so it is translated, not followed.
pub fn translation_prompt(target_lang: &str, text: &str) -> String {
    format!(
        "Translate the subtitle line between <cue> tags into {target_lang}. \
         Reply with the translation only, no quotes or notes.\n<cue>{text}</cue>"
    )
}

pub fn translate(
    backend: &mut dyn InferenceBackend,
    guard: &mut CostGuard,
    consent: &Consent,
    job: &TranscribeJob<'_>,
    target_lang: &str,
    cues: &[Chunk],
    cancel: &dyn Fn() -> bool,
) -> Result<Translation, StepError<Translation>> {
    let mut out = Translation::default();
    if target_lang.trim().is_empty() {
        let e = InferenceError::InvalidInput("target language is empty".into());
        return Err(step::Stop::Error(e).at(0, &out));
    }
    if let Err(e) = step::consent(backend, consent, job) {
        return Err(step::Stop::Error(e).at(0, &out));
    }
    for (index, cue) in cues.iter().enumerate() {
        let source = cue.text.trim();
        if source.is_empty() {
            out.cues.push(cue.clone());
            continue;
        }
        let request = InferenceRequest::Generate {
            model: job.model.into(),
            prompt: translation_prompt(target_lang, source),
        };
        let response =
            step::call(backend, guard, job, &request, cancel).map_err(|s| s.at(index, &out))?;
        let Output::Generate { text } = response.output else {
            unreachable!("step::call checks the output task")
        };
        let text = text.trim();
        if text.is_empty() {
            let e = InferenceError::Protocol("empty translation".into());
            return Err(step::Stop::Error(e).at(index, &out));
        }
        out.spend.add(response.provenance);
        out.cues.push(Chunk {
            start: cue.start,
            end: cue.end,
            text: text.into(),
        });
    }
    Ok(out)
}

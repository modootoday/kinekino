//! Borrowed projection of one explicitly accepted, immutable timed-cue correction.
//! No publication, acceptance mutation, time inference or format rounding occurs here.
use super::{
    AcceptanceReceipt, ProposalPayload, ProposalTask, SourceCue, SourceSnapshot, StoredProposal,
};
use crate::domain::api_values::DecimalU64;
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq)]
pub enum ProjectionError {
    InvalidState(&'static str),
    NotAccepted,
    AcceptanceMismatch,
    UnsupportedTask,
    UnsupportedSource,
    MissingCueTiming { cue_index: usize },
}

#[derive(Debug)]
pub struct AcceptedCue<'a> {
    source: &'a SourceCue,
    text: &'a str,
    in_us: DecimalU64,
    out_us: DecimalU64,
}
impl<'a> AcceptedCue<'a> {
    pub fn source(&self) -> &'a SourceCue {
        self.source
    }
    pub fn text(&self) -> &'a str {
        self.text
    }
    pub fn in_us(&self) -> DecimalU64 {
        self.in_us
    }
    pub fn out_us(&self) -> DecimalU64 {
        self.out_us
    }
}

#[derive(Debug)]
pub struct AcceptedSubtitleProjection<'a> {
    acceptance: &'a AcceptanceReceipt,
    cues: Vec<AcceptedCue<'a>>,
}
impl<'a> AcceptedSubtitleProjection<'a> {
    pub fn acceptance(&self) -> &'a AcceptanceReceipt {
        self.acceptance
    }
    pub fn cues(&self) -> &[AcceptedCue<'a>] {
        &self.cues
    }
}

// Compare the caller's exact accepted expectation without cloning or serializing
// potentially large source/generated/payload buffers. This is not a store CAS.
fn same_acceptance(actual: &AcceptanceReceipt, expected: &AcceptanceReceipt) -> bool {
    if actual.proposal_id != expected.proposal_id
        || actual.draft_revision != expected.draft_revision
        || actual.output_revision != expected.output_revision
        || actual.source_id != expected.source_id
        || actual.source_revision != expected.source_revision
        || actual.source_sha256 != expected.source_sha256
        || actual.automatic_application_enabled != expected.automatic_application_enabled
    {
        return false;
    }
    match (&actual.output, &expected.output) {
        (
            ProposalPayload::Correction { edits: actual },
            ProposalPayload::Correction { edits: expected },
        ) => {
            actual.len() == expected.len()
                && actual
                    .iter()
                    .zip(expected)
                    .all(|(a, b)| a.cue_id == b.cue_id && a.text == b.text)
        }
        _ => false,
    }
}

/// Consume a validated stored snapshot and the exact receipt selected by the caller.
/// Source validation is structural: external file/provenance authenticity belongs
/// to the existing SDK/store reader. Returned times are unchanged microseconds.
pub fn project<'a>(
    state: &'a StoredProposal,
    expected: &AcceptanceReceipt,
) -> Result<AcceptedSubtitleProjection<'a>, ProjectionError> {
    state.validate().map_err(ProjectionError::InvalidState)?;
    let acceptance = state
        .acceptance
        .as_ref()
        .ok_or(ProjectionError::NotAccepted)?;
    if state.document.generated.task != ProposalTask::Correction {
        return Err(ProjectionError::UnsupportedTask);
    }
    let SourceSnapshot::Cue(source) = &state.document.source else {
        return Err(ProjectionError::UnsupportedSource);
    };
    let ProposalPayload::Correction { edits } = &acceptance.output else {
        return Err(ProjectionError::UnsupportedTask);
    };
    if !same_acceptance(acceptance, expected) {
        return Err(ProjectionError::AcceptanceMismatch);
    }
    let replacements: BTreeMap<&str, &str> = edits
        .iter()
        .map(|edit| (edit.cue_id.as_str(), edit.text.as_str()))
        .collect();
    let mut cues = Vec::with_capacity(source.cues.len());
    for (cue_index, cue) in source.cues.iter().enumerate() {
        let (Some(in_us), Some(out_us)) = (cue.start_us, cue.end_us) else {
            return Err(ProjectionError::MissingCueTiming { cue_index });
        };
        // Positive pair, unique IDs and bounded edits were checked by the
        // existing StoredProposal/SourceSnapshot/Payload validators above.
        cues.push(AcceptedCue {
            source: cue,
            text: replacements
                .get(cue.cue_id.as_str())
                .copied()
                .unwrap_or(cue.text.as_str()),
            in_us,
            out_us,
        });
    }
    Ok(AcceptedSubtitleProjection { acceptance, cues })
}

#[cfg(test)]
#[path = "accepted_subtitle_projection/tests.rs"]
mod tests;

use super::{VadInput, VadProfile, invalid};
use kinekino_media_contracts::errors::ApiError;

#[derive(Debug)]
pub struct CompleteVadResult {
    profile: VadProfile,
    input: VadInput,
    scores: Vec<f32>,
}
impl CompleteVadResult {
    pub(crate) fn checked(
        profile: VadProfile,
        input: VadInput,
        scores: Vec<f32>,
    ) -> Result<Self, ApiError> {
        if scores.len() as u64 != input.expected_frames() || !valid_scores(&scores) {
            return Err(invalid("Incomplete or invalid VAD scores"));
        }
        Ok(Self {
            profile,
            input,
            scores,
        })
    }
    /// Expected immutable specification; not an actual runtime-module observation.
    pub fn profile(&self) -> &VadProfile {
        &self.profile
    }
    pub fn input(&self) -> &VadInput {
        &self.input
    }
    pub fn scores(&self) -> &[f32] {
        &self.scores
    }
    pub fn speech_frames(&self) -> usize {
        self.scores.iter().filter(|&&s| s >= 0.5).count()
    }
    pub fn last_frame_valid_samples(&self) -> u64 {
        (self.input.selected_samples() - 1) % 512 + 1
    }
}
#[derive(Debug)]
pub struct PartialVadObservation {
    profile: VadProfile,
    input: VadInput,
    processed_samples: u64,
    scores: Vec<f32>,
}
impl PartialVadObservation {
    pub(crate) fn checked(
        profile: &VadProfile,
        input: &VadInput,
        processed_samples: u64,
        scores: Vec<f32>,
    ) -> Result<Self, ApiError> {
        if processed_samples > input.selected_samples()
            || scores.len() as u64 != processed_samples.div_ceil(512)
            || !valid_scores(&scores)
        {
            return Err(invalid("Invalid partial VAD observation"));
        }
        Ok(Self {
            profile: profile.clone(),
            input: input.clone(),
            processed_samples,
            scores,
        })
    }
    /// Expected immutable specification; not an actual runtime-module observation.
    pub fn profile(&self) -> &VadProfile {
        &self.profile
    }
    pub fn input(&self) -> &VadInput {
        &self.input
    }
    pub fn processed_samples(&self) -> u64 {
        self.processed_samples
    }
    pub fn scores(&self) -> &[f32] {
        &self.scores
    }
}
fn valid_scores(scores: &[f32]) -> bool {
    scores
        .iter()
        .all(|s| s.is_finite() && (0.0..=1.0).contains(s))
}
#[derive(Debug)]
pub enum AnalysisOutcome {
    Complete(CompleteVadResult),
    Partial(PartialVadObservation),
}

//! `InferenceBackend` port. Request/response shapes are identical for every adapter;
//! adapters differ only in `provenance`.
mod caps;
mod error;
#[cfg(test)]
mod tests;

pub use caps::{InferenceCaps, InferenceKind, Task};
pub use error::InferenceError;

use kinekino_core::port::Adapter;
use serde::{Deserialize, Serialize};

/// Typed task input. Media is passed as bytes of a cut segment/frame, never a whole video.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "task", rename_all = "snake_case")]
pub enum InferenceRequest {
    Transcribe { model: String, audio: Vec<u8> },
    Diarize { model: String, audio: Vec<u8> },
    Ocr { model: String, image: Vec<u8> },
    SegmentSubject { model: String, image: Vec<u8> },
    Generate { model: String, prompt: String },
    Embed { model: String, text: String },
}

impl InferenceRequest {
    pub fn task(&self) -> Task {
        match self {
            Self::Transcribe { .. } => Task::Transcribe,
            Self::Diarize { .. } => Task::Diarize,
            Self::Ocr { .. } => Task::Ocr,
            Self::SegmentSubject { .. } => Task::SegmentSubject,
            Self::Generate { .. } => Task::Generate,
            Self::Embed { .. } => Task::Embed,
        }
    }
    pub fn model(&self) -> &str {
        match self {
            Self::Transcribe { model, .. }
            | Self::Diarize { model, .. }
            | Self::Ocr { model, .. }
            | Self::SegmentSubject { model, .. }
            | Self::Generate { model, .. }
            | Self::Embed { model, .. } => model,
        }
    }
    /// Bytes that would leave the machine (consent and cost estimates use it).
    pub fn payload_len(&self) -> usize {
        match self {
            Self::Transcribe { audio, .. } | Self::Diarize { audio, .. } => audio.len(),
            Self::Ocr { image, .. } | Self::SegmentSubject { image, .. } => image.len(),
            Self::Generate { prompt, .. } => prompt.len(),
            Self::Embed { text, .. } => text.len(),
        }
    }
}

/// Timestamped text span (seconds).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chunk {
    pub start: f64,
    pub end: f64,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "task", rename_all = "snake_case")]
pub enum Output {
    Transcribe { text: String, chunks: Vec<Chunk> },
    Diarize { segments: Vec<(f64, f64, String)> },
    Ocr { text: String },
    SegmentSubject { mask_png: Vec<u8> },
    Generate { text: String },
    Embed { vector: Vec<f32> },
}

impl Output {
    pub fn task(&self) -> Task {
        match self {
            Self::Transcribe { .. } => Task::Transcribe,
            Self::Diarize { .. } => Task::Diarize,
            Self::Ocr { .. } => Task::Ocr,
            Self::SegmentSubject { .. } => Task::SegmentSubject,
            Self::Generate { .. } => Task::Generate,
            Self::Embed { .. } => Task::Embed,
        }
    }
}

/// Recorded into `ai_provenance`; never carries secrets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    pub backend: String,
    pub model: String,
    pub provider: Option<String>,
    pub cost_estimate_usd: f64,
    /// Set when the registry fell back from the configured mode.
    pub fallback_from: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub output: Output,
    pub provenance: Provenance,
}

/// The ai-context port. Calls run on a Jobs worker; `cancel` is checked between attempts.
pub trait InferenceBackend: Adapter<Kind = InferenceKind, Caps = InferenceCaps> {
    /// Cost estimate before the call; `None` means "unknown" and the guard asks the user.
    fn estimate_usd(&self, request: &InferenceRequest) -> Option<f64>;
    fn run(
        &mut self,
        request: &InferenceRequest,
        cancel: &dyn Fn() -> bool,
    ) -> Result<InferenceResponse, InferenceError>;
}

/// Rejects requests whose task the adapter does not declare.
pub fn require_task(caps: InferenceCaps, request: &InferenceRequest) -> Result<(), InferenceError> {
    let task = request.task();
    if caps.contains(task) {
        Ok(())
    } else {
        Err(InferenceError::Unsupported(task.name().into()))
    }
}

use crate::domain::{ArtifactPin, VadProfile};
use kinekino_media_contracts::errors::ApiError;
use std::{io::Read, time::Instant};

/// Trusted composition port. A caller's JSON is never an implementation of admission.
pub trait ProfileAdmission {
    fn authorize(&self, profile: &VadProfile) -> Result<(), ApiError>;
}
pub trait ArtifactReader {
    fn open(&self, pin: &ArtifactPin) -> Result<Box<dyn Read + '_>, ApiError>;
}
pub trait AnalysisControl {
    fn cancellation_requested(&self) -> bool;
    fn deadline(&self) -> Instant;
    /// Fallible observation; true means actual cancellation, never an IO fault.
    /// Legacy in-process controls retain their original bool behavior.
    fn checkpoint(&self) -> Result<bool, ApiError> {
        Ok(self.cancellation_requested())
    }
}
pub trait ScoreSink {
    fn push(&mut self, score: f32) -> Result<(), ApiError>;
}
#[derive(Debug, Clone, Copy)]
pub enum EngineCompletion {
    Complete,
    Partial { processed_samples: u64 },
}
/// Verified bytes remain owned by the application for this synchronous call.
/// Engine completion grants no process-cleanup or publication authority.
pub trait VadEngine {
    fn run(
        &self,
        profile: &VadProfile,
        model: &[u8],
        runtime: &[u8],
        samples: &[i16],
        control: &dyn AnalysisControl,
        scores: &mut dyn ScoreSink,
    ) -> Result<EngineCompletion, ApiError>;
}

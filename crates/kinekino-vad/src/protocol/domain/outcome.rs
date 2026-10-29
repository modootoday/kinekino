use super::ExpectedWorkerContext;
use crate::domain::{ArtifactPin, VadInput, VadProfile};
use kinekino_media_contracts::errors::ErrorCode;

#[derive(Debug, Clone)]
pub struct DecodedDescriptor {
    pub(crate) raw: ArtifactPin,
    pub(crate) profile: VadProfile,
    pub(crate) input: VadInput,
}
impl DecodedDescriptor {
    pub fn raw(&self) -> &ArtifactPin {
        &self.raw
    }
    pub fn profile(&self) -> &VadProfile {
        &self.profile
    }
    pub fn input(&self) -> &VadInput {
        &self.input
    }
}
#[derive(Debug)]
pub struct ModuleObservation {
    pub(crate) pin: ArtifactPin,
    pub(crate) canonical_path: Option<String>,
}
impl ModuleObservation {
    pub fn pin(&self) -> &ArtifactPin {
        &self.pin
    }
    pub fn canonical_path(&self) -> Option<&str> {
        self.canonical_path.as_deref()
    }
}
/// Worker-reported observations only; a trusted parent must verify them independently.
#[derive(Debug)]
pub struct ActualRuntimeObservation {
    pub(crate) provider: Option<String>,
    pub(crate) precision: Option<String>,
    pub(crate) intra_threads: Option<u32>,
    pub(crate) inter_threads: Option<u32>,
    pub(crate) modules: Vec<ModuleObservation>,
}
impl ActualRuntimeObservation {
    pub fn provider(&self) -> Option<&str> {
        self.provider.as_deref()
    }
    pub fn precision(&self) -> Option<&str> {
        self.precision.as_deref()
    }
    pub fn intra_threads(&self) -> Option<u32> {
        self.intra_threads
    }
    pub fn inter_threads(&self) -> Option<u32> {
        self.inter_threads
    }
    pub fn modules(&self) -> &[ModuleObservation] {
        &self.modules
    }
}
#[derive(Debug)]
pub enum WorkerResult {
    Complete {
        scores: Vec<f32>,
    },
    Partial {
        processed_samples: u64,
        scores: Vec<f32>,
    },
    Failure {
        code: ErrorCode,
        message: String,
    },
}
/// Validated wire data, never a P0 CompleteVadResult or VerifiedJobOutput.
#[derive(Debug)]
pub struct WorkerOutcomeCandidate {
    pub(crate) raw: ArtifactPin,
    pub(crate) context: ExpectedWorkerContext,
    pub(crate) result: WorkerResult,
    pub(crate) actual_runtime: Option<ActualRuntimeObservation>,
}
impl WorkerOutcomeCandidate {
    pub fn raw(&self) -> &ArtifactPin {
        &self.raw
    }
    pub fn context(&self) -> &ExpectedWorkerContext {
        &self.context
    }
    pub fn result(&self) -> &WorkerResult {
        &self.result
    }
    pub fn actual_runtime(&self) -> Option<&ActualRuntimeObservation> {
        self.actual_runtime.as_ref()
    }
}

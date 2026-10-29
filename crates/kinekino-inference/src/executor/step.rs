//! Shared per-call gate for every AI executor: cancel → cost guard (confirm dialog unless the
//! user already confirmed) → backend → ledger. Consent is checked once per job by the caller.
use super::TranscribeJob;
use crate::budget::{CostGuard, GuardDecision, LedgerEntry};
use crate::consent::Consent;
use crate::port::{InferenceBackend, InferenceError, InferenceKind, InferenceRequest};
use crate::port::{InferenceResponse, Provenance};

/// Failure of a multi-call executor. `index` is the input item (cue, frame, region) at which
/// the job stopped; `partial` holds everything finished before it.
#[derive(Debug, Clone, PartialEq)]
pub enum StepError<T> {
    /// Guard wants the limit dialog; nothing was sent for item `index`.
    NeedsConfirm { index: usize, reason: String },
    Failed {
        index: usize,
        error: InferenceError,
        partial: Box<T>,
    },
}

pub(super) enum Stop {
    Confirm(String),
    Error(InferenceError),
}

impl Stop {
    pub(super) fn at<T: Clone>(self, index: usize, partial: &T) -> StepError<T> {
        match self {
            Self::Confirm(reason) => StepError::NeedsConfirm { index, reason },
            Self::Error(error) => StepError::Failed {
                index,
                error,
                partial: Box::new(partial.clone()),
            },
        }
    }
}

/// Local packs never upload, so only remote backends need consent.
pub(super) fn consent(
    backend: &dyn InferenceBackend,
    consent: &Consent,
    job: &TranscribeJob<'_>,
) -> Result<(), InferenceError> {
    if matches!(backend.kind(), InferenceKind::LocalPack) {
        Ok(())
    } else {
        consent.check(job.project, job.job)
    }
}

/// Runs one guarded call and records its cost.
pub(super) fn call(
    backend: &mut dyn InferenceBackend,
    guard: &mut CostGuard,
    job: &TranscribeJob<'_>,
    request: &InferenceRequest,
    cancel: &dyn Fn() -> bool,
) -> Result<InferenceResponse, Stop> {
    if cancel() {
        return Err(Stop::Error(InferenceError::Cancelled));
    }
    match guard
        .check(job.month, backend.estimate_usd(request))
        .map_err(Stop::Error)?
    {
        GuardDecision::Confirm(reason) if !job.confirmed => return Err(Stop::Confirm(reason)),
        _ => {}
    }
    let response = backend.run(request, cancel).map_err(Stop::Error)?;
    if response.output.task() != request.task() {
        return Err(Stop::Error(InferenceError::Protocol(format!(
            "expected {} output, got {}",
            request.task().name(),
            response.output.task().name()
        ))));
    }
    guard.record(LedgerEntry {
        month: job.month.into(),
        backend: response.provenance.backend.clone(),
        model: response.provenance.model.clone(),
        cost_usd: response.provenance.cost_estimate_usd,
    });
    Ok(response)
}

/// Accumulated provenance and cost shared by the executor results.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Spend {
    pub provenance: Vec<Provenance>,
    pub cost_usd: f64,
}

impl Spend {
    pub(super) fn add(&mut self, provenance: Provenance) {
        self.cost_usd += provenance.cost_estimate_usd;
        self.provenance.push(provenance);
    }
}

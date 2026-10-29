use super::{CompleteVadResult, PartialVadObservation};

/// Data observed inside a worker. Distinct from ordinary AnalysisOutcome and
/// from a jobs output candidate, cancellation marker or cleanup acknowledgement.
#[derive(Debug)]
pub enum WorkerAnalysisObservation {
    Inferred(CompleteVadResult),
    Partial(PartialVadObservation),
}

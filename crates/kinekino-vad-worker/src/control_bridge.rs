//! Borrowed fallible control plus one conservative view of the original cutoff.
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_vad::application::AnalysisControl;
use kinekino_vad_executor::{application::WorkerControl, domain::WorkerControlState};
use std::{cell::Cell, time::Instant};

pub struct WorkerAnalysisControl<'a> {
    original: &'a dyn WorkerControl,
    deadline: Instant,
    cancelled: Cell<bool>,
}
impl<'a> WorkerAnalysisControl<'a> {
    /// Caller supplies the once-projected original absolute cutoff, not a timeout.
    pub fn new(
        original: &'a dyn WorkerControl,
        projected_deadline: Instant,
    ) -> Result<Self, ApiError> {
        let view = Self {
            original,
            deadline: projected_deadline,
            cancelled: Cell::new(false),
        };
        view.checkpoint()?;
        Ok(view)
    }
}
impl AnalysisControl for WorkerAnalysisControl<'_> {
    /// Cached genuine observation only; fallible consumers must use checkpoint.
    fn cancellation_requested(&self) -> bool {
        self.cancelled.get()
    }
    fn deadline(&self) -> Instant {
        self.deadline
    }
    fn checkpoint(&self) -> Result<bool, ApiError> {
        let state = self.original.checkpoint()?;
        if Instant::now() >= self.deadline {
            return Err(ApiError::new(
                ErrorCode::Timeout,
                "Original projected worker operation cutoff expired",
            ));
        }
        let cancelled = state == WorkerControlState::CancellationRequested;
        self.cancelled.set(cancelled);
        Ok(cancelled)
    }
}

#[cfg(test)]
#[path = "control_bridge_tests.rs"]
mod tests;

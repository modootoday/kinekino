use super::RunControl;
use crate::domain::{CounterReading, WorkerControlState, WorkerOperationCutoff};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_process::domain::deadline::Deadline;
use std::time::{Duration, Instant};

pub const MAX_WORKER_CONTROL_BYTES: usize = 2048;
/// Decodes already bounded/EOF-observed bytes. File identity, actual byte pin
/// and original launch authority remain the concrete reader's responsibility.
pub fn decode_worker_control(bytes: &[u8]) -> Result<WorkerOperationCutoff, ApiError> {
    if bytes.len() > MAX_WORKER_CONTROL_BYTES {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Worker control exceeds bound",
        ));
    }
    let cutoff: WorkerOperationCutoff = serde_json::from_slice(bytes)
        .map_err(|error| ApiError::new(ErrorCode::InvalidArgument, error.to_string()))?;
    cutoff.validate()?;
    Ok(cutoff)
}

/// The implementation verifies the original still-live parent identity on each
/// sample. Frequency/domain copied from an untrusted DTO is not an implementation.
pub trait CounterClock {
    fn sample(&self) -> Result<CounterReading, ApiError>;
}
pub trait WorkerCancellation {
    /// Only the exact original guarded signal can request cancellation. Read,
    /// replacement, shape and permission errors remain errors, never booleans.
    fn requested(&self) -> Result<bool, ApiError>;
}
pub trait WorkerControl {
    /// Implementations propagate clock/signal faults unchanged. Cancellation is
    /// a cooperative marker; neither it nor an expired counter is cleanup ACK.
    fn checkpoint(&self) -> Result<WorkerControlState, ApiError>;
}
pub struct WorkerControlView<'a> {
    cutoff: &'a WorkerOperationCutoff,
    request_sha256: &'a str,
    clock: &'a dyn CounterClock,
    cancellation: &'a dyn WorkerCancellation,
}
impl<'a> WorkerControlView<'a> {
    pub fn new(
        cutoff: &'a WorkerOperationCutoff,
        request_sha256: &'a str,
        clock: &'a dyn CounterClock,
        cancellation: &'a dyn WorkerCancellation,
    ) -> Result<Self, ApiError> {
        cutoff.validate()?;
        if cutoff.request_sha256 != request_sha256 {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Worker control belongs to another original request",
            ));
        }
        let view = Self {
            cutoff,
            request_sha256,
            clock,
            cancellation,
        };
        view.checkpoint()?;
        Ok(view)
    }
}
impl WorkerControl for WorkerControlView<'_> {
    fn checkpoint(&self) -> Result<WorkerControlState, ApiError> {
        let before = self.clock.sample()?;
        self.cutoff
            .checkpoint(self.request_sha256, &before, false)?;
        let cancelled = self.cancellation.requested()?;
        let now = self.clock.sample()?;
        if before.domain != now.domain || now.ticks < before.ticks {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Worker clock changed during signal observation",
            ));
        }
        self.cutoff.checkpoint(self.request_sha256, &now, cancelled)
    }
}
pub fn prepare_worker_cutoff(
    clock: &dyn CounterClock,
    original: Deadline,
    caller: &dyn RunControl,
    request_sha256: String,
) -> Result<WorkerOperationCutoff, ApiError> {
    let before = clock.sample()?;
    let sampled = Instant::now();
    let remaining = original
        .operation_remaining(sampled)
        .min(caller.deadline().saturating_duration_since(sampled));
    let after = clock.sample()?;
    if caller.cancellation_requested() {
        return Err(ApiError::new(
            ErrorCode::Cancelled,
            "Original worker preparation cancelled; no cleanup ACK",
        ));
    }
    WorkerOperationCutoff::correlate(before, after, remaining, request_sha256)
}
/// Fixed conservative compatibility view for legacy Instant consumers. It is
/// not a new timeout, scope or authority; fallible QPC checkpoints remain required.
pub fn project_worker_deadline(
    clock: &dyn CounterClock,
    cutoff: &WorkerOperationCutoff,
) -> Result<Instant, ApiError> {
    let before = clock.sample()?;
    let sampled = Instant::now();
    let after = clock.sample()?;
    if before.domain != after.domain || after.ticks < before.ticks {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Worker clock correlation changed",
        ));
    }
    cutoff.checkpoint(&cutoff.request_sha256, &after, false)?;
    let ticks = cutoff
        .operation_cutoff
        .0
        .checked_sub(after.ticks)
        .and_then(|ticks| ticks.checked_sub(cutoff.uncertainty_ticks.0))
        .ok_or_else(|| {
            ApiError::new(
                ErrorCode::Timeout,
                "Worker cutoff has no conservative remaining interval",
            )
        })?;
    let nanos = u128::from(ticks) * 1_000_000_000 / u128::from(cutoff.clock.frequency_hz.0);
    let seconds = u64::try_from(nanos / 1_000_000_000).map_err(|_| {
        ApiError::new(
            ErrorCode::InvalidArgument,
            "Worker deadline projection overflow",
        )
    })?;
    let duration = Duration::new(seconds, (nanos % 1_000_000_000) as u32);
    if duration.is_zero() {
        return Err(ApiError::new(
            ErrorCode::Timeout,
            "Worker cutoff interval rounded to zero",
        ));
    }
    sampled
        .checked_add(duration)
        .ok_or_else(|| ApiError::new(ErrorCode::InvalidArgument, "Worker deadline view overflow"))
}

#[cfg(test)]
#[path = "worker_control_tests.rs"]
mod tests;

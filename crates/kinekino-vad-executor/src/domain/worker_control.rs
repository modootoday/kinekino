//! Same-live-parent clock correlation metadata, never an admission credential.
use kinekino_media_contracts::{
    api_values::DecimalU64,
    errors::{ApiError, ErrorCode},
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Counter domain kinds an OS adapter may produce (ADR 0008).
pub const COUNTER_KINDS: [&str; 2] = ["windows-qpc/1", "posix-monotonic/1"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CounterDomain {
    pub kind: String,
    pub frequency_hz: DecimalU64,
    pub parent_pid: u32,
    pub parent_creation_filetime: DecimalU64,
}
impl CounterDomain {
    pub fn validate(&self) -> Result<(), ApiError> {
        if !COUNTER_KINDS.contains(&self.kind.as_str())
            || self.frequency_hz.0 == 0
            || self.frequency_hz.0 > i64::MAX as u64
            || self.parent_pid == 0
            || self.parent_creation_filetime.0 == 0
        {
            return Err(invalid("Invalid live-parent counter domain"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CounterReading {
    /// Must come from an OS-verified still-live original parent, not copied JSON.
    pub domain: CounterDomain,
    pub ticks: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerOperationCutoff {
    pub schema_version: String,
    pub clock: CounterDomain,
    pub request_sha256: String,
    pub sampled_before: DecimalU64,
    pub sampled_after: DecimalU64,
    pub uncertainty_ticks: DecimalU64,
    pub operation_cutoff: DecimalU64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerControlState {
    Continue,
    CancellationRequested,
}
fn invalid(message: &str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
fn changed() -> ApiError {
    ApiError::new(
        ErrorCode::SourceExpired,
        "Original worker clock or request binding changed",
    )
}
fn expired() -> ApiError {
    ApiError::new(
        ErrorCode::Timeout,
        "Original worker operation cutoff expired",
    )
}

impl WorkerOperationCutoff {
    /// The trusted parent samples before/Instant/after. An early counter basis,
    /// downward duration rounding and explicit uncertainty can only shorten it.
    pub fn correlate(
        before: CounterReading,
        after: CounterReading,
        remaining_at_instant: Duration,
        request_sha256: String,
    ) -> Result<Self, ApiError> {
        before.domain.validate()?;
        if before.domain != after.domain || after.ticks < before.ticks {
            return Err(changed());
        }
        if before.ticks > i64::MAX as u64 || after.ticks > i64::MAX as u64 {
            return Err(changed());
        }
        let frequency = before.domain.frequency_hz.0;
        let delta = remaining_at_instant
            .as_nanos()
            .checked_mul(u128::from(frequency))
            .and_then(|value| u64::try_from(value / 1_000_000_000).ok())
            .ok_or_else(|| invalid("Worker counter projection overflow"))?;
        // One cross-thread tick, one comparison tick, plus at most one ns of
        // opaque Instant/Duration conversion rounding, rounded upward.
        let uncertainty = 2u64
            .checked_add(frequency.div_ceil(1_000_000_000))
            .ok_or_else(|| invalid("Worker counter uncertainty overflow"))?;
        let cutoff = before
            .ticks
            .checked_add(delta)
            .and_then(|value| value.checked_sub(uncertainty))
            .filter(|value| *value > after.ticks && *value <= i64::MAX as u64)
            .ok_or_else(expired)?;
        let value = Self {
            schema_version: "worker-control/1".into(),
            clock: before.domain,
            request_sha256,
            sampled_before: DecimalU64(before.ticks),
            sampled_after: DecimalU64(after.ticks),
            uncertainty_ticks: DecimalU64(uncertainty),
            operation_cutoff: DecimalU64(cutoff),
        };
        value.validate()?;
        Ok(value)
    }
    pub fn validate(&self) -> Result<(), ApiError> {
        self.clock.validate()?;
        let minimum = 2 + self.clock.frequency_hz.0.div_ceil(1_000_000_000);
        if self.schema_version != "worker-control/1"
            || !kinekino_jobs::domain::valid_sha256(&self.request_sha256)
            || self.sampled_after.0 < self.sampled_before.0
            || self.operation_cutoff.0 <= self.sampled_after.0
            || self.operation_cutoff.0 > i64::MAX as u64
            || self.uncertainty_ticks.0 < minimum
        {
            return Err(invalid("Invalid worker operation cutoff"));
        }
        Ok(())
    }
    /// Clock equality is checked against verified observations; it is not a
    /// proof of parent authority. The guarded launch/request provides that bind.
    pub fn checkpoint(
        &self,
        request_sha256: &str,
        now: &CounterReading,
        cancellation_requested: bool,
    ) -> Result<WorkerControlState, ApiError> {
        self.validate()?;
        if self.request_sha256 != request_sha256
            || self.clock != now.domain
            || now.ticks < self.sampled_before.0
        {
            return Err(changed());
        }
        // Expiry takes precedence. Cancellation is cooperative observation, not ACK.
        if now.ticks >= self.operation_cutoff.0 {
            return Err(expired());
        }
        Ok(if cancellation_requested {
            WorkerControlState::CancellationRequested
        } else {
            WorkerControlState::Continue
        })
    }
}

#[cfg(test)]
#[path = "worker_control_tests.rs"]
mod tests;

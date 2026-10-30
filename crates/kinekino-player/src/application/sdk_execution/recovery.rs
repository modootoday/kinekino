//! Bounded recovery authority belonging to one original running operation.
use super::*;
use state::Phase;
const MAX_ATTEMPTS: u8 = 16;

/// Adapter must retain its original owner. Reporting is allowed only after
/// physical closure and a failed receipt write; it is never a cleanup ACK.
pub trait ReceiptRecoveryControl: Sync {
    fn receipt_failed(&self) -> Result<u8, ApiError>;
    fn take_retry(&self, generation: u8) -> Result<bool, ApiError>;
}
#[derive(Clone, Debug)]
pub struct CleanupRetryToken {
    authority: Arc<()>,
    generation: u8,
}
#[derive(Clone, Debug)]
pub struct ReceiptRecoverySnapshot {
    pub token: CleanupRetryToken,
    pub retry_requested: bool,
    pub retry_consumed: bool,
    pub attempt_budget_exhausted: bool,
}
impl ReceiptRecoverySnapshot {
    pub fn generation(&self) -> u8 {
        self.token.generation
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CleanupRetryDisposition {
    Requested,
    AlreadyRequested,
}
#[derive(Default)]
pub(super) struct RecoveryState {
    generation: u8,
    pending: Option<ReceiptRecoverySnapshot>,
}
impl ReceiptRecoveryControl for worker::RunningControl {
    fn receipt_failed(&self) -> Result<u8, ApiError> {
        let mut state = self.shared.lock()?;
        let entry = state.entries.get_mut(&self.sequence).ok_or_else(stale)?;
        if !matches!(entry.phase, Phase::Running) {
            return Err(pending());
        }
        if entry
            .recovery
            .pending
            .as_ref()
            .is_some_and(|p| !p.retry_consumed)
        {
            return Err(pending());
        }
        let generation = entry
            .recovery
            .generation
            .checked_add(1)
            .filter(|g| *g <= MAX_ATTEMPTS);
        let Some(generation) = generation else {
            let observation = entry.recovery.pending.as_mut().ok_or_else(pending)?;
            let changed = !observation.attempt_budget_exhausted;
            observation.attempt_budget_exhausted = true;
            drop(state);
            if changed {
                self.shared.notify();
            }
            return Err(capacity());
        };
        entry.recovery.generation = generation;
        entry.recovery.pending = Some(ReceiptRecoverySnapshot {
            token: CleanupRetryToken {
                authority: Arc::new(()),
                generation,
            },
            retry_requested: false,
            retry_consumed: false,
            attempt_budget_exhausted: false,
        });
        drop(state);
        self.shared.notify();
        Ok(generation)
    }
    fn take_retry(&self, generation: u8) -> Result<bool, ApiError> {
        let mut state = self.shared.lock()?;
        let entry = state.entries.get_mut(&self.sequence).ok_or_else(stale)?;
        if !matches!(entry.phase, Phase::Running) {
            return Err(pending());
        }
        let observation = entry.recovery.pending.as_mut().ok_or_else(pending)?;
        if observation.token.generation != generation {
            return Err(stale());
        }
        if !observation.retry_requested || observation.retry_consumed {
            return Ok(false);
        }
        observation.retry_consumed = true;
        Ok(true)
    }
}
impl ExecutionHandle {
    pub fn receipt_recovery(
        &self,
        id: &OperationId,
    ) -> Result<Option<ReceiptRecoverySnapshot>, ApiError> {
        self.receipt_recovery_for(id, None)
    }
    pub(super) fn receipt_recovery_for(
        &self,
        id: &OperationId,
        authority: Option<&Arc<()>>,
    ) -> Result<Option<ReceiptRecoverySnapshot>, ApiError> {
        self.shared.check(id)?;
        let state = self.shared.lock()?;
        state.authorize(authority)?;
        let entry = state.entries.get(&id.sequence).ok_or_else(stale)?;
        Ok(if matches!(entry.phase, Phase::Running) {
            entry.recovery.pending.clone()
        } else {
            None
        })
    }
    pub fn retry_receipt(
        &self,
        id: &OperationId,
        token: &CleanupRetryToken,
    ) -> Result<CleanupRetryDisposition, ApiError> {
        self.retry_receipt_for(id, token, None)
    }
    pub(super) fn retry_receipt_for(
        &self,
        id: &OperationId,
        token: &CleanupRetryToken,
        authority: Option<&Arc<()>>,
    ) -> Result<CleanupRetryDisposition, ApiError> {
        self.shared.check(id)?;
        let mut state = self.shared.lock()?;
        state.authorize(authority)?;
        let entry = state.entries.get_mut(&id.sequence).ok_or_else(stale)?;
        if !matches!(entry.phase, Phase::Running) {
            return Err(pending());
        }
        let observation = entry.recovery.pending.as_mut().ok_or_else(pending)?;
        if !Arc::ptr_eq(&observation.token.authority, &token.authority)
            || observation.token.generation != token.generation
        {
            return Err(stale());
        }
        if observation.attempt_budget_exhausted {
            return Err(capacity());
        }
        let disposition = if observation.retry_requested {
            CleanupRetryDisposition::AlreadyRequested
        } else {
            observation.retry_requested = true;
            CleanupRetryDisposition::Requested
        };
        drop(state);
        self.shared.changed.notify_all();
        Ok(disposition)
    }
}

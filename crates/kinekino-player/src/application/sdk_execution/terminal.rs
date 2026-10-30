use super::*;
use state::Phase;
impl ExecutionHandle {
    /// Trusted owner diagnostics; IDs are not serialized renderer authority.
    pub fn retained_operations(&self) -> Result<Vec<OperationId>, ApiError> {
        Ok(self
            .shared
            .lock()?
            .entries
            .keys()
            .map(|sequence| OperationId {
                owner: self.shared.owner.clone(),
                sequence: *sequence,
            })
            .collect())
    }
    pub fn peek(&self, id: &OperationId) -> Result<ExecutionSnapshot, ApiError> {
        self.peek_for(id, None, true)
    }
    pub(super) fn peek_for(
        &self,
        id: &OperationId,
        authority: Option<&Arc<()>>,
        trusted_diagnostic: bool,
    ) -> Result<ExecutionSnapshot, ApiError> {
        self.shared.check(id)?;
        let state = self.shared.lock()?;
        if !trusted_diagnostic {
            state.authorize(authority)?;
        }
        if state.acknowledged.contains(&id.sequence) {
            return Ok(ExecutionSnapshot::Acknowledged);
        }
        let entry = state.entries.get(&id.sequence).ok_or_else(stale)?;
        Ok(match &entry.phase {
            Phase::Queued => ExecutionSnapshot::Queued,
            Phase::Running => ExecutionSnapshot::Running {
                cancellation_requested: entry.cancelling,
            },
            Phase::Terminal(terminal) => ExecutionSnapshot::Terminal(terminal.clone()),
        })
    }
    pub fn acknowledge(&self, id: &OperationId) -> Result<AckReceipt, ApiError> {
        self.acknowledge_for(id, None)
    }
    pub(super) fn acknowledge_for(
        &self,
        id: &OperationId,
        authority: Option<&Arc<()>>,
    ) -> Result<AckReceipt, ApiError> {
        self.shared.check(id)?;
        let mut state = self.shared.lock()?;
        state.authorize(authority)?;
        if state.acknowledged.contains(&id.sequence) {
            return Ok(AckReceipt {
                already_acknowledged: true,
            });
        }
        if !matches!(
            state.entries.get(&id.sequence).map(|e| &e.phase),
            Some(Phase::Terminal(_))
        ) {
            return Err(pending());
        }
        let entry = state.entries.remove(&id.sequence).ok_or_else(stale)?;
        state.request_bytes -= entry.request_bytes;
        state.terminal_reserved -= self.shared.budget.max_terminal_bytes;
        state.acknowledged.push_back(id.sequence);
        while state.acknowledged.len() > self.shared.budget.max_operations * 2 {
            state.acknowledged.pop_front();
        }
        Ok(AckReceipt {
            already_acknowledged: false,
        })
    }
    pub fn notification_error(&self) -> Result<Option<Arc<ApiError>>, ApiError> {
        Ok(self.shared.lock()?.notification_error.clone())
    }
}

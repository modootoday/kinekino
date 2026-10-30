use super::*;
use state::{Phase, Shared, State};
pub(super) fn cancel_queued(shared: &Shared, state: &mut State) -> bool {
    let queued = state.queue.drain(..).collect::<Vec<_>>();
    let changed = !queued.is_empty();
    for sequence in queued {
        if let Some(entry) = state.entries.get_mut(&sequence)
            && cancel_entry(shared, entry)
        {
            state.faulted = true;
        }
    }
    changed
}
fn cancel_entry(shared: &Shared, entry: &mut state::Entry) -> bool {
    if !matches!(entry.phase, Phase::Queued) {
        return false;
    }
    let mut error = ApiError::new(
        ErrorCode::Cancelled,
        "SDK request cancelled before execution",
    );
    error.details = serde_json::json!({"execution_started":false});
    let response = Response::failure(entry.request.context.request_id.clone(), error);
    let measurement = admission::encoded_size(&response, shared.budget.max_terminal_bytes);
    entry.cancelling = true;
    entry.phase = Phase::Terminal(Arc::new(ExecutionTerminal {
        request: entry.request.clone(),
        response: Arc::new(response),
        cancellation_requested: true,
        queued_cancelled: true,
        measurement,
    }));
    measurement.is_faulted()
}
impl ExecutionHandle {
    pub fn cancel(&self, id: &OperationId) -> Result<CancelDisposition, ApiError> {
        self.cancel_for(id, None)
    }
    pub(super) fn cancel_for(
        &self,
        id: &OperationId,
        authority: Option<&Arc<()>>,
    ) -> Result<CancelDisposition, ApiError> {
        self.shared.check(id)?;
        let mut state = self.shared.lock()?;
        state.authorize(authority)?;
        if state.acknowledged.contains(&id.sequence) {
            return Ok(CancelDisposition::AlreadyAcknowledged);
        }
        let entry = state.entries.get_mut(&id.sequence).ok_or_else(stale)?;
        let outcome = if matches!(entry.phase, Phase::Queued) {
            let fault = cancel_entry(&self.shared, entry);
            state.queue.retain(|sequence| *sequence != id.sequence);
            if fault {
                state.faulted = true;
                state.closing = true;
                cancel_queued(&self.shared, &mut state);
            }
            CancelDisposition::QueuedCancelled
        } else {
            entry.cancelling = true;
            if let Phase::Terminal(old) = &entry.phase {
                entry.phase = Phase::Terminal(Arc::new(ExecutionTerminal {
                    request: old.request.clone(),
                    response: old.response.clone(),
                    cancellation_requested: true,
                    queued_cancelled: old.queued_cancelled,
                    measurement: old.measurement,
                }));
            }
            CancelDisposition::Requested
        };
        drop(state);
        self.shared.changed.notify_all();
        self.shared.notify();
        Ok(outcome)
    }
}

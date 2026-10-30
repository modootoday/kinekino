use super::*;
use state::Phase;
pub(super) struct RunningControl {
    pub(super) shared: Arc<state::Shared>,
    pub(super) sequence: u64,
}
impl crate::application::review_source::MutationControl for RunningControl {
    fn checkpoint(&self) -> Result<(), ApiError> {
        let state = self.shared.lock()?;
        let entry = state
            .entries
            .get(&self.sequence)
            .ok_or_else(|| internal("Original execution is unavailable"))?;
        if entry.cancelling {
            Err(ApiError::new(
                ErrorCode::Cancelled,
                "SDK operation cooperatively cancelled",
            ))
        } else {
            Ok(())
        }
    }
}
pub(super) fn run(shared: Arc<state::Shared>) {
    loop {
        let work = {
            let Ok(mut state) = shared.lock() else {
                return;
            };
            loop {
                if let Some(sequence) = state.queue.pop_front() {
                    if let Some(entry) = state.entries.get_mut(&sequence)
                        && matches!(entry.phase, Phase::Queued)
                    {
                        entry.phase = Phase::Running;
                        break Some((sequence, entry.request.clone()));
                    }
                } else if state.closing {
                    state.worker_finished = true;
                    break None;
                } else {
                    match shared.changed.wait(state) {
                        Ok(next) => state = next,
                        Err(_) => return,
                    }
                }
            }
        };
        let Some((sequence, request)) = work else {
            shared.notify();
            return;
        };
        let response = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let control = RunningControl {
                shared: shared.clone(),
                sequence,
            };
            shared
                .client
                .execute_recoverable((*request).clone(), &control, &control)
        }))
        .unwrap_or_else(|_| {
            let mut error = internal("SDK worker panicked; execution may have occurred");
            error.details = serde_json::json!({"execution_may_have_occurred":true});
            Response::failure(request.context.request_id.clone(), error)
        });
        let measurement = admission::encoded_size(&response, shared.budget.max_terminal_bytes);
        if let Ok(mut state) = shared.lock() {
            if let Some(entry) = state.entries.get_mut(&sequence) {
                entry.phase = Phase::Terminal(Arc::new(ExecutionTerminal {
                    request,
                    response: Arc::new(response),
                    cancellation_requested: entry.cancelling,
                    queued_cancelled: false,
                    measurement,
                }));
            }
            let invalid_response=state.entries.get(&sequence).is_some_and(|entry|matches!(&entry.phase,Phase::Terminal(terminal) if terminal.validate_response().is_err()));
            if measurement.is_faulted() || invalid_response {
                state.faulted = true;
                state.closing = true;
                control::cancel_queued(&shared, &mut state);
            }
        }
        shared.notify();
    }
}

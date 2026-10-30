use super::*;
use std::io::Write;
pub(super) fn validate_budget(b: &ExecutionBudget) -> Result<(), ApiError> {
    if b.max_operations == 0
        || b.max_request_bytes == 0
        || b.max_total_request_bytes == 0
        || b.max_terminal_bytes == 0
        || b.max_total_terminal_bytes < b.max_terminal_bytes
        || b.max_operations.checked_mul(2).is_none()
    {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Invalid SDK execution budget",
        ));
    }
    Ok(())
}
pub(crate) fn encoded_size(value: &impl serde::Serialize, limit: usize) -> EncodingMeasurement {
    struct Counter {
        bytes: usize,
        limit: usize,
        overflow: bool,
    }
    impl Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            match self.bytes.checked_add(bytes.len()) {
                Some(next) if next <= self.limit => {
                    self.bytes = next;
                    Ok(bytes.len())
                }
                _ => {
                    self.overflow = true;
                    Err(std::io::ErrorKind::FileTooLarge.into())
                }
            }
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut counter = Counter {
        bytes: 0,
        limit,
        overflow: false,
    };
    if serde_json::to_writer(&mut counter, value).is_err() {
        if counter.overflow {
            EncodingMeasurement::LowerBound(limit.saturating_add(1))
        } else {
            EncodingMeasurement::SerializationFailed
        }
    } else {
        EncodingMeasurement::Exact(counter.bytes)
    }
}
impl ExecutionHandle {
    pub fn submit(&self, request: Request) -> Result<OperationId, ApiError> {
        self.submit_for(request, None)
    }
    pub(super) fn submit_for(
        &self,
        request: Request,
        authority: Option<&Arc<()>>,
    ) -> Result<OperationId, ApiError> {
        request.validate()?;
        let budget = self.shared.budget;
        let bytes = match encoded_size(&request, budget.max_request_bytes) {
            EncodingMeasurement::Exact(bytes) => bytes,
            EncodingMeasurement::LowerBound(_) => return Err(capacity()),
            EncodingMeasurement::SerializationFailed => {
                return Err(internal("SDK request serialization failed"));
            }
        };
        let charged = bytes.checked_mul(2).ok_or_else(capacity)?;
        let mut state = self.shared.lock()?;
        if state.closing || state.faulted {
            return Err(closing());
        }
        state.authorize(authority)?;
        let request_total = state
            .request_bytes
            .checked_add(charged)
            .ok_or_else(capacity)?;
        let terminal_total = state
            .terminal_reserved
            .checked_add(budget.max_terminal_bytes)
            .ok_or_else(capacity)?;
        if state.entries.len() >= budget.max_operations
            || request_total > budget.max_total_request_bytes
            || terminal_total > budget.max_total_terminal_bytes
        {
            return Err(capacity());
        }
        let sequence = state.next;
        state.next = sequence.checked_add(1).ok_or_else(capacity)?;
        state.entries.insert(
            sequence,
            state::Entry {
                recovery: recovery::RecoveryState::default(),
                request: Arc::new(request),
                request_bytes: charged,
                cancelling: false,
                phase: state::Phase::Queued,
            },
        );
        state.queue.push_back(sequence);
        state.request_bytes = request_total;
        state.terminal_reserved = terminal_total;
        drop(state);
        self.shared.changed.notify_one();
        Ok(OperationId {
            owner: self.shared.owner.clone(),
            sequence,
        })
    }
}

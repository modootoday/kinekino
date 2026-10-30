use crate::application::api::{ApiError, Request, Response};
use std::{fmt, sync::Arc};

#[derive(Debug, Clone, Copy)]
pub struct ExecutionBudget {
    pub max_operations: usize,
    pub max_request_bytes: usize,
    pub max_total_request_bytes: usize,
    pub max_terminal_bytes: usize,
    pub max_total_terminal_bytes: usize,
}
#[derive(Clone)]
pub struct OperationId {
    pub(super) owner: Arc<()>,
    pub(super) sequence: u64,
}
impl PartialEq for OperationId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.owner, &other.owner) && self.sequence == other.sequence
    }
}
impl Eq for OperationId {}
impl fmt::Debug for OperationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OperationId")
            .field("sequence", &self.sequence)
            .finish_non_exhaustive()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CancelDisposition {
    QueuedCancelled,
    Requested,
    AlreadyAcknowledged,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodingMeasurement {
    Exact(usize),
    LowerBound(usize),
    SerializationFailed,
}
impl EncodingMeasurement {
    pub fn is_faulted(self) -> bool {
        !matches!(self, Self::Exact(_))
    }
}
#[derive(Debug)]
pub struct ExecutionTerminal {
    pub request: Arc<Request>,
    pub response: Arc<Response>,
    pub cancellation_requested: bool,
    pub queued_cancelled: bool,
    pub measurement: EncodingMeasurement,
}
#[derive(Debug, Clone)]
pub enum ExecutionSnapshot {
    Queued,
    Running { cancellation_requested: bool },
    Terminal(Arc<ExecutionTerminal>),
    Acknowledged,
}
impl ExecutionTerminal {
    pub fn validate_response(&self) -> Result<(), ApiError> {
        if self.response.api_version != crate::application::api::API_VERSION
            || self.response.request_id != self.request.context.request_id
            || !matches!(
                (self.response.ok, &self.response.data, &self.response.error),
                (true, Some(_), None) | (false, None, Some(_))
            )
        {
            let mut error = ApiError::new(
                crate::application::api::ErrorCode::EngineFailed,
                "SDK terminal envelope differs; original response retained",
            );
            error.details = serde_json::json!({"execution_may_have_occurred":true});
            return Err(error);
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AckReceipt {
    pub already_acknowledged: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionCloseReceipt {
    pub worker_stopped: bool,
    pub unacknowledged: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerCloseState {
    /// The loop may have finished, but actual thread return/join is not confirmed.
    ExitPending {
        loop_finished: bool,
    },
    Joined,
}
#[derive(Debug, Clone)]
pub enum ExecutionCloseProgress {
    TerminalAckPending {
        unacknowledged: usize,
        worker: WorkerCloseState,
    },
    WorkerExitPending {
        loop_finished: bool,
    },
    /// Original operation responses remain accessible through retained_operations/peek.
    Faulted {
        kind: ExecutionCloseFaultKind,
        error: Arc<ApiError>,
        unacknowledged: usize,
        worker: WorkerCloseState,
    },
    Closed(ExecutionCloseReceipt),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionCloseFaultKind {
    Terminal,
    WorkerJoin,
}
pub trait CompletionNotifier: Send + Sync {
    fn notify(&self) -> Result<(), ApiError>;
}

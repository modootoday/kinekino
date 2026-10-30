//! Owned SDK scheduling with opt-in checkpoints, never forced result replacement.
mod admission;
mod attachment;
mod close;
mod control;
pub mod host;
mod initial_proposal;
mod recovery;
mod state;
mod terminal;
#[cfg(test)]
mod tests;
mod types;
mod worker;
use crate::application::api::{ApiError, ApplicationClientHandle, ErrorCode, Request, Response};
pub(crate) use admission::encoded_size;
pub use attachment::ExecutionAttachment;
pub use initial_proposal::InitialProposalInspection;
pub use recovery::{
    CleanupRetryDisposition, CleanupRetryToken, ReceiptRecoveryControl, ReceiptRecoverySnapshot,
};
use std::sync::{Arc, Condvar, Mutex};
pub use types::*;
#[must_use = "Keep the owner and explicitly close after terminal ACK"]
pub struct ExecutionOwner {
    handle: ExecutionHandle,
}
#[derive(Clone)]
pub struct ExecutionHandle {
    shared: Arc<state::Shared>,
}
impl ExecutionOwner {
    pub fn start(
        client: ApplicationClientHandle,
        budget: ExecutionBudget,
        notifier: Arc<dyn CompletionNotifier>,
    ) -> Result<Self, ApiError> {
        admission::validate_budget(&budget)?;
        let shared = Arc::new(state::Shared {
            #[cfg(test)]
            join_attempts: std::sync::atomic::AtomicUsize::new(0),
            owner: Arc::new(()),
            budget,
            client,
            notifier,
            state: Mutex::new(state::State::new()),
            changed: Condvar::new(),
            worker: Mutex::new(None),
        });
        let service = shared.clone();
        let worker = std::thread::Builder::new()
            .name("sdk-execution".into())
            .spawn(move || worker::run(service))
            .map_err(|_| internal("SDK worker could not start"))?;
        *shared
            .worker
            .lock()
            .map_err(|_| internal("SDK worker ownership is unavailable"))? = Some(worker);
        Ok(Self {
            handle: ExecutionHandle { shared },
        })
    }
    pub fn handle(&self) -> ExecutionHandle {
        self.handle.clone()
    }
    pub fn close(&self) -> Result<ExecutionCloseReceipt, ApiError> {
        self.handle.close()
    }
    pub fn poll_close(&self) -> Result<ExecutionCloseProgress, ApiError> {
        self.handle.poll_close()
    }
}
impl Drop for ExecutionOwner {
    fn drop(&mut self) {
        let _ = self.handle.close_for(None, true);
    }
}
fn internal(message: &str) -> ApiError {
    ApiError::new(ErrorCode::EngineFailed, message)
}
fn pending() -> ApiError {
    ApiError::new(
        ErrorCode::NeedsInput,
        "SDK execution cleanup requires terminal ACK and worker completion",
    )
}
fn stale() -> ApiError {
    ApiError::new(
        ErrorCode::SourceExpired,
        "SDK operation belongs to another owner or is no longer retained",
    )
}
fn closing() -> ApiError {
    ApiError::new(
        ErrorCode::NeedsInput,
        "SDK execution owner is closing or faulted",
    )
}
fn capacity() -> ApiError {
    ApiError::new(
        ErrorCode::ResourceExhausted,
        "SDK execution count or byte budget is full",
    )
}

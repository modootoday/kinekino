use super::*;
use std::sync::atomic::{AtomicBool, Ordering};
#[must_use = "Keep the exclusive consumer attachment until its operations and worker are closed"]
pub struct ExecutionAttachment {
    handle: ExecutionHandle,
    authority: Arc<()>,
    released: AtomicBool,
}
impl ExecutionHandle {
    pub fn attach(&self) -> Result<ExecutionAttachment, ApiError> {
        let mut state = self.shared.lock()?;
        if state.closing || state.attachment.is_some() || !state.entries.is_empty() {
            return Err(ApiError::new(
                ErrorCode::NeedsInput,
                "SDK execution requires one quiescent exclusive consumer",
            ));
        }
        let authority = Arc::new(());
        state.attachment = Some(authority.clone());
        Ok(ExecutionAttachment {
            handle: self.clone(),
            authority,
            released: AtomicBool::new(false),
        })
    }
}
impl ExecutionAttachment {
    pub fn poll_close(&self) -> Result<ExecutionCloseProgress, ApiError> {
        self.handle.poll_close_for(Some(&self.authority))
    }
    pub fn receipt_recovery(
        &self,
        id: &OperationId,
    ) -> Result<Option<ReceiptRecoverySnapshot>, ApiError> {
        self.handle.receipt_recovery_for(id, Some(&self.authority))
    }
    pub fn retry_receipt(
        &self,
        id: &OperationId,
        token: &CleanupRetryToken,
    ) -> Result<CleanupRetryDisposition, ApiError> {
        self.handle
            .retry_receipt_for(id, token, Some(&self.authority))
    }
    pub fn submit(&self, request: Request) -> Result<OperationId, ApiError> {
        self.handle.submit_for(request, Some(&self.authority))
    }
    pub fn peek(&self, id: &OperationId) -> Result<ExecutionSnapshot, ApiError> {
        self.handle.peek_for(id, Some(&self.authority), false)
    }
    pub fn acknowledge(&self, id: &OperationId) -> Result<AckReceipt, ApiError> {
        self.handle.acknowledge_for(id, Some(&self.authority))
    }
    pub fn cancel(&self, id: &OperationId) -> Result<CancelDisposition, ApiError> {
        self.handle.cancel_for(id, Some(&self.authority))
    }
    /// Release a quiescent attachment; final worker close belongs to ExecutionOwner.
    pub fn release(&self) -> Result<(), ApiError> {
        if self.released.load(Ordering::Acquire) {
            return Ok(());
        }
        let mut state = self.handle.shared.lock()?;
        if self.released.load(Ordering::Acquire) {
            return Ok(());
        }
        if state
            .attachment
            .as_ref()
            .is_none_or(|active| !Arc::ptr_eq(active, &self.authority))
        {
            return Err(stale());
        }
        if !state.entries.is_empty() {
            return Err(pending());
        }
        state.attachment = None;
        self.released.store(true, Ordering::Release);
        Ok(())
    }
}
impl Drop for ExecutionAttachment {
    fn drop(&mut self) {
        if self.released.load(Ordering::Acquire) {
            return;
        }
        if let Ok(mut state) = self.handle.shared.lock() {
            if state
                .attachment
                .as_ref()
                .is_none_or(|active| !Arc::ptr_eq(active, &self.authority))
            {
                return;
            }
            state.attachment = None;
            if !state.entries.is_empty() {
                state.closing = true;
                control::cancel_queued(&self.handle.shared, &mut state);
            }
        }
        self.handle.shared.changed.notify_all();
    }
}

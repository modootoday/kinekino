//! One attachment on the original worker. No worker, controller or parent join owner.
mod terminal;
#[cfg(test)]
mod tests;
use super::*;
use kinekino_player::application::{
    api::{ErrorCode, Request as ApiRequest},
    sdk_execution::{
        CancelDisposition, ExecutionAttachment, ExecutionHandle, ExecutionSnapshot,
        ExecutionTerminal, OperationId,
    },
};

pub(super) struct AttachedExecution {
    attachment: ExecutionAttachment,
    operation: Option<OwnedOperation>,
}
struct OwnedOperation {
    intent: CaptureIntent,
    request: ApiRequest,
    operation: Option<OperationId>,
    terminal: Option<Arc<ExecutionTerminal>>,
    disposition: Option<CaptureExecutionDisposition>,
    first_fault: Option<Arc<ApiError>>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureExecutionDisposition {
    Consumed,
    Discarded { reason: String },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureExecutionProgress {
    Idle,
    Unsubmitted,
    Queued,
    Running {
        cancellation_requested: bool,
    },
    TerminalReady {
        cancellation_requested: bool,
        queued_cancelled: bool,
    },
    ConsumedAckPending,
    DiscardedAckPending,
}
pub enum CaptureDispatchProgress {
    Submitted(OperationId),
    Retained(Arc<ApiError>),
}
#[must_use = "The rejected original intent still belongs to its pending view"]
pub struct RejectedCaptureIntent {
    pub error: ApiError,
    pub intent: CaptureIntent,
}
#[must_use = "Retain original request/terminal evidence through the parent close decision"]
pub struct CaptureExecutionAcknowledgement {
    pub request: ApiRequest,
    pub operation: Option<OperationId>,
    pub terminal: Option<Arc<ExecutionTerminal>>,
    pub disposition: CaptureExecutionDisposition,
    pub first_fault: Option<Arc<ApiError>>,
    // Keep the original typed payload and presentation owner, even for unsubmitted discard.
    pub(super) _intent: CaptureIntent,
}
fn needs_input(message: &str) -> Arc<ApiError> {
    Arc::new(ApiError::new(ErrorCode::NeedsInput, message))
}
fn retain_fault(operation: &mut OwnedOperation, error: ApiError) -> Arc<ApiError> {
    let error = Arc::new(error);
    operation.first_fault.get_or_insert_with(|| error.clone());
    error
}
impl CaptureIntent {
    fn wire_request(&self, context: RequestContext) -> Result<ApiRequest, ApiError> {
        let (command, payload) = match &self.request {
            Request::Edit(input) => ("proposals.edit_capture", serde_json::to_value(input)),
            Request::Accept(input) => ("proposals.accept_capture", serde_json::to_value(input)),
        };
        Ok(ApiRequest {
            context,
            command: command.into(),
            payload: payload
                .map_err(|error| ApiError::new(ErrorCode::Internal, error.to_string()))?,
        })
    }
}
impl CaptureMutationAdapter {
    pub(super) fn execution_operation_pending(&self) -> bool {
        self.execution
            .as_ref()
            .is_some_and(|state| state.operation.is_some())
    }
    /// This handle must come from the existing parent ExecutionOwner. Never start another worker.
    pub fn attach_execution(&mut self, original: &ExecutionHandle) -> Result<(), ApiError> {
        if self.execution.is_some() || self.pending.is_some() || self.closing {
            return Err(ApiError::new(
                ErrorCode::NeedsInput,
                "Capture view is attached, pending or closing",
            ));
        }
        let attachment = original.attach()?;
        self.execution = Some(AttachedExecution {
            attachment,
            operation: None,
        });
        Ok(())
    }
    pub fn execution_terminal(&self) -> Option<&Arc<ExecutionTerminal>> {
        self.execution
            .as_ref()?
            .operation
            .as_ref()?
            .terminal
            .as_ref()
    }
    pub fn execution_fault(&self) -> Option<&Arc<ApiError>> {
        self.execution
            .as_ref()?
            .operation
            .as_ref()?
            .first_fault
            .as_ref()
    }
    pub fn execution_operation(&self) -> Option<&OperationId> {
        self.execution
            .as_ref()?
            .operation
            .as_ref()?
            .operation
            .as_ref()
    }
    pub fn execution_closing(&self) -> bool {
        self.closing
    }
    pub fn execution_attached(&self) -> bool {
        self.execution.is_some()
    }

    /// A rejected intent is returned intact. Admission failure keeps it in this adapter.
    /// Closing retains an already prepared intent for explicit discard without submitting it.
    pub fn dispatch_to_execution(
        &mut self,
        intent: CaptureIntent,
        context: RequestContext,
    ) -> Result<CaptureDispatchProgress, Box<RejectedCaptureIntent>> {
        let rejection = if !Arc::ptr_eq(&intent.owner, &self.owner)
            || intent.serial != self.serial
            || self.pending.is_none()
        {
            Some("Capture intent belongs to another pending selection")
        } else if self.execution.is_none() || self.execution_operation_pending() {
            Some("Capture execution is absent or still owns an operation")
        } else {
            None
        };
        if let Some(message) = rejection {
            return Err(Box::new(RejectedCaptureIntent {
                error: ApiError::new(ErrorCode::NeedsInput, message),
                intent,
            }));
        }
        let request = match intent.wire_request(context) {
            Ok(request) => request,
            Err(error) => return Err(Box::new(RejectedCaptureIntent { error, intent })),
        };
        self.execution
            .as_mut()
            .expect("checked attachment")
            .operation = Some(OwnedOperation {
            intent,
            request,
            operation: None,
            terminal: None,
            disposition: None,
            first_fault: None,
        });
        match self.retry_execution_submission() {
            Ok(operation) => Ok(CaptureDispatchProgress::Submitted(operation)),
            Err(error) => Ok(CaptureDispatchProgress::Retained(error)),
        }
    }
    /// Explicit retry of the same immutable wire payload, only if never admitted.
    pub fn retry_execution_submission(&mut self) -> Result<OperationId, Arc<ApiError>> {
        let state = self
            .execution
            .as_mut()
            .ok_or_else(|| needs_input("No original attachment"))?;
        let pending = state
            .operation
            .as_mut()
            .ok_or_else(|| needs_input("No retained capture intent"))?;
        if self.closing || pending.operation.is_some() || pending.disposition.is_some() {
            return Err(retain_fault(
                pending,
                ApiError::new(
                    ErrorCode::NeedsInput,
                    "Closing or admitted capture intent cannot be resubmitted",
                ),
            ));
        }
        match state.attachment.submit(pending.request.clone()) {
            Ok(operation) => {
                pending.operation = Some(operation.clone());
                Ok(operation)
            }
            Err(error) => Err(retain_fault(pending, error)),
        }
    }
    pub fn poll_execution(&mut self) -> Result<CaptureExecutionProgress, Arc<ApiError>> {
        let state = self
            .execution
            .as_mut()
            .ok_or_else(|| needs_input("No original attachment"))?;
        let Some(pending) = state.operation.as_mut() else {
            return Ok(CaptureExecutionProgress::Idle);
        };
        match &pending.disposition {
            Some(CaptureExecutionDisposition::Consumed) => {
                return Ok(CaptureExecutionProgress::ConsumedAckPending);
            }
            Some(CaptureExecutionDisposition::Discarded { .. }) => {
                return Ok(CaptureExecutionProgress::DiscardedAckPending);
            }
            None => {}
        }
        let Some(operation) = pending.operation.as_ref() else {
            return Ok(CaptureExecutionProgress::Unsubmitted);
        };
        match state
            .attachment
            .peek(operation)
            .map_err(|error| retain_fault(pending, error))?
        {
            ExecutionSnapshot::Queued => Ok(CaptureExecutionProgress::Queued),
            ExecutionSnapshot::Running {
                cancellation_requested,
            } => Ok(CaptureExecutionProgress::Running {
                cancellation_requested,
            }),
            ExecutionSnapshot::Terminal(terminal) => {
                let progress = CaptureExecutionProgress::TerminalReady {
                    cancellation_requested: terminal.cancellation_requested,
                    queued_cancelled: terminal.queued_cancelled,
                };
                pending.terminal = Some(terminal);
                Ok(progress)
            }
            ExecutionSnapshot::Acknowledged => Err(retain_fault(
                pending,
                ApiError::new(
                    ErrorCode::EngineFailed,
                    "Original terminal was acknowledged outside its capture consumer",
                ),
            )),
        }
    }
    /// Cancellation cannot prove no mutation. Poll and consume/discard the actual terminal.
    pub fn request_execution_close(&mut self) -> Result<CaptureExecutionProgress, Arc<ApiError>> {
        self.closing = true;
        if let Some(state) = &mut self.execution
            && let Some(pending) = &mut state.operation
            && pending.disposition.is_none()
            && let Some(operation) = &pending.operation
        {
            match state.attachment.cancel(operation) {
                Ok(CancelDisposition::QueuedCancelled | CancelDisposition::Requested) => {}
                Ok(CancelDisposition::AlreadyAcknowledged) => {
                    return Err(retain_fault(
                        pending,
                        ApiError::new(
                            ErrorCode::EngineFailed,
                            "Original capture was acknowledged outside close",
                        ),
                    ));
                }
                Err(error) => return Err(retain_fault(pending, error)),
            }
        }
        self.poll_execution()
    }
    /// Parent may poll its original ExecutionOwner only after this succeeds.
    pub fn release_execution(&mut self) -> Result<(), Arc<ApiError>> {
        if self.pending.is_some() || self.execution_operation_pending() {
            return Err(needs_input(
                "Original intent/terminal still needs explicit consumption or discard and ACK",
            ));
        }
        let state = self
            .execution
            .as_ref()
            .ok_or_else(|| needs_input("No original attachment"))?;
        state.attachment.release().map_err(Arc::new)?;
        self.execution = None;
        Ok(())
    }
}

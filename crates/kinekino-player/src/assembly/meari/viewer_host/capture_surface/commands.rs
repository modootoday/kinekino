use super::*;
use crate::application::api::{ApiError, ErrorCode, RequestContext};
use crate::presentation::web_ui_adapter::proposals::capture_mutation::{
    CaptureIntent, CaptureMutationAdapter,
};
use input::Command;

fn invalid(message: impl Into<String>) -> ViewerHostError {
    ViewerHostError::Sdk(Arc::new(ApiError::new(ErrorCode::NeedsInput, message)))
}
impl CapturedSurface {
    fn submit(
        &mut self,
        adapter: &mut CaptureMutationAdapter,
        intent: CaptureIntent,
        context: RequestContext,
    ) -> Result<(), ViewerHostError> {
        match adapter.dispatch_to_execution(intent, context) {
            Ok(_) => Ok(()),
            Err(rejected) => {
                let original = &rejected.error;
                let error = Arc::new(ApiError {
                    code: original.code,
                    message: original.message.clone(),
                    retryable: original.retryable,
                    details: original.details.clone(),
                    remediation: original.remediation.clone(),
                    request_id: original.request_id.clone(),
                });
                self.rejected_intent = Some(rejected);
                Err(ViewerHostError::Sdk(error))
            }
        }
    }
    pub(super) fn drain(
        &mut self,
        adapter: &mut CaptureMutationAdapter,
        context: RequestContext,
    ) -> Result<(), ViewerHostError> {
        while let Some(command) = self.inputs.queue.pop_front() {
            self.dirty = true;
            let result = self.apply_command(command, adapter, context.clone());
            if let Err(error) = result {
                self.error.get_or_insert_with(|| error.clone());
                self.model.message =
                    "요청을 완료하지 못했습니다. 입력과 처리 상태를 확인한 뒤 다시 시도하세요."
                        .into();
                // Remaining admitted text stays owned; never replay a mutation implicitly.
                self.inputs.drained();
                return Err(error);
            }
        }
        self.inputs.drained();
        Ok(())
    }
    fn apply_command(
        &mut self,
        command: Command,
        adapter: &mut CaptureMutationAdapter,
        context: RequestContext,
    ) -> Result<(), ViewerHostError> {
        if self.evidence_blocked {
            if matches!(command, Command::Retry) {
                self.evidence_retry = true;
                return Ok(());
            }
            return Err(invalid("Original Viewer evidence publication is pending"));
        }
        let progress = adapter.poll_execution().map_err(ViewerHostError::Sdk)?;
        let idle = progress == CaptureExecutionProgress::Idle && !adapter.execution_closing();
        match command {
            Command::ReturnViewing => {
                self.return_requested = true;
                Ok(())
            }
            Command::Revert => {
                if !idle {
                    return Err(invalid("Original capture operation is still pending"));
                }
                self.model.revert_local(adapter).map_err(invalid)
            }
            Command::Front(text) | Command::Back(text) if !idle => {
                let _ = text;
                Err(invalid("Original capture operation is still pending"))
            }
            Command::Front(text) => self.model.text(true, text).map_err(invalid),
            Command::Back(text) => self.model.text(false, text).map_err(invalid),
            Command::Save => {
                if !idle
                    || !self.model.dirty()
                    || self.acknowledged.len() >= 8
                    || self.rejected_intent.is_some()
                {
                    return Err(invalid(
                        "Capture save is pending, unchanged, or evidence capacity is full",
                    ));
                }
                let intent = adapter
                    .edit(self.model.payload())
                    .map_err(|e| invalid(format!("{e:?}")))?;
                self.model.retry_accept = false;
                self.submit(adapter, intent, context)
            }
            Command::Accept => {
                if !idle
                    || self.model.dirty()
                    || self.acknowledged.len() >= 8
                    || self.rejected_intent.is_some()
                {
                    return Err(invalid("Save local Card text before explicit acceptance"));
                }
                let revision = adapter.display().state.document.source.source_revision();
                let intent = adapter
                    .accept(revision)
                    .map_err(|e| invalid(format!("{e:?}")))?;
                self.model.retry_accept = true;
                self.submit(adapter, intent, context)
            }
            Command::Retry => {
                if matches!(
                    progress,
                    CaptureExecutionProgress::ConsumedAckPending
                        | CaptureExecutionProgress::DiscardedAckPending
                ) {
                    self.acknowledge(adapter)
                } else if progress == CaptureExecutionProgress::Unsubmitted {
                    adapter
                        .retry_execution_submission()
                        .map(|_| ())
                        .map_err(ViewerHostError::Sdk)
                } else {
                    if !idle || self.model.dirty() || self.acknowledged.len() >= 8 {
                        return Err(invalid("Original acceptance retry is not ready"));
                    }
                    let intent = adapter
                        .retry_accept()
                        .map_err(|e| invalid(format!("{e:?}")))?;
                    self.submit(adapter, intent, context)
                }
            }
            Command::Consume => {
                adapter
                    .consume_execution_terminal()
                    .map_err(ViewerHostError::Sdk)?;
                self.model.adopted(adapter).map_err(invalid)?;
                self.acknowledge(adapter)
            }
            Command::Discard => {
                adapter
                    .discard_execution_terminal("Explicit Card response discard".into())
                    .map_err(ViewerHostError::Sdk)?;
                self.acknowledge(adapter)
            }
        }
    }
    fn acknowledge(&mut self, adapter: &mut CaptureMutationAdapter) -> Result<(), ViewerHostError> {
        if self.acknowledged.len() >= 8 {
            return Err(invalid(
                "Original acknowledgement evidence capacity is full",
            ));
        }
        self.acknowledged.push(
            adapter
                .acknowledge_execution_terminal()
                .map_err(ViewerHostError::Sdk)?,
        );
        Ok(())
    }
}

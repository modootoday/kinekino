use super::*;

fn decode(
    terminal: &ExecutionTerminal,
) -> Result<Result<CaptureMutationResult, ApiError>, ApiError> {
    if terminal.measurement.is_faulted() {
        return Err(ApiError::new(
            ErrorCode::EngineFailed,
            "Capture terminal measurement requires explicit recovery; original retained",
        ));
    }
    terminal.validate_response()?;
    let response = &terminal.response;
    if response.ok {
        serde_json::from_value(response.data.as_ref().expect("validated envelope").clone())
            .map(Ok)
            .map_err(|error| ApiError::new(ErrorCode::EngineFailed, error.to_string()))
    } else {
        let value = serde_json::to_value(response.error.as_ref().expect("validated envelope"))
            .map_err(|error| ApiError::new(ErrorCode::EngineFailed, error.to_string()))?;
        serde_json::from_value(value)
            .map(Err)
            .map_err(|error| ApiError::new(ErrorCode::EngineFailed, error.to_string()))
    }
}
impl CaptureMutationAdapter {
    /// Explicit consumer decision. The result is applied once, before a separate terminal ACK.
    pub fn consume_execution_terminal(
        &mut self,
    ) -> Result<CaptureExecutionProgress, Arc<ApiError>> {
        match self.poll_execution()? {
            CaptureExecutionProgress::ConsumedAckPending => {
                return Ok(CaptureExecutionProgress::ConsumedAckPending);
            }
            CaptureExecutionProgress::TerminalReady { .. } => {}
            _ => {
                return Err(needs_input(
                    "Original capture terminal is not ready for consumption",
                ));
            }
        }
        let state = self.execution.as_mut().expect("polled attachment");
        let pending = state.operation.as_mut().expect("polled operation");
        let terminal = pending.terminal.as_ref().expect("polled terminal");
        let result = decode(terminal).map_err(|error| retain_fault(pending, error))?;
        let completion = CaptureCompletion {
            owner: pending.intent.owner.clone(),
            serial: pending.intent.serial,
            result,
        };
        if let Err(rejected) = self.apply_completion(completion) {
            let pending = self
                .execution
                .as_mut()
                .expect("retained attachment")
                .operation
                .as_mut()
                .expect("retained operation");
            return Err(retain_fault(
                pending,
                ApiError::new(
                    ErrorCode::EngineFailed,
                    format!(
                        "Capture completion rejected {:?}; original terminal retained",
                        rejected.reason
                    ),
                ),
            ));
        }
        self.execution
            .as_mut()
            .expect("retained attachment")
            .operation
            .as_mut()
            .expect("retained operation")
            .disposition = Some(CaptureExecutionDisposition::Consumed);
        Ok(CaptureExecutionProgress::ConsumedAckPending)
    }
    /// Explicit discard is not rollback or a claim that the SDK did not execute.
    /// Raw success/error/codec evidence remains owned until ACK returns it to the caller.
    pub fn discard_execution_terminal(
        &mut self,
        reason: String,
    ) -> Result<CaptureExecutionProgress, Arc<ApiError>> {
        self.poll_execution()?;
        let state = self.execution.as_mut().expect("polled attachment");
        let pending = state
            .operation
            .as_mut()
            .ok_or_else(|| needs_input("No retained capture operation"))?;
        if matches!(
            pending.disposition,
            Some(CaptureExecutionDisposition::Consumed)
        ) {
            return Err(needs_input(
                "An applied capture result cannot be relabeled discarded",
            ));
        }
        if pending.disposition.is_none() {
            if pending.operation.is_some() && pending.terminal.is_none() {
                return Err(needs_input(
                    "Cancel is not terminal; original operation still owns its result",
                ));
            }
            pending.disposition = Some(CaptureExecutionDisposition::Discarded { reason });
            self.pending = None;
        }
        Ok(CaptureExecutionProgress::DiscardedAckPending)
    }
    /// ACK failure leaves operation/terminal/disposition in place. Never reapplies or resubmits.
    pub fn acknowledge_execution_terminal(
        &mut self,
    ) -> Result<CaptureExecutionAcknowledgement, Arc<ApiError>> {
        let state = self
            .execution
            .as_mut()
            .ok_or_else(|| needs_input("No original attachment"))?;
        let pending = state
            .operation
            .as_mut()
            .ok_or_else(|| needs_input("No retained capture operation"))?;
        if pending.disposition.is_none() {
            return Err(needs_input(
                "Explicit terminal consumption or discard is required before ACK",
            ));
        }
        if let Some(operation) = &pending.operation {
            if pending.terminal.is_none() {
                return Err(needs_input("Original terminal evidence is missing"));
            }
            state
                .attachment
                .acknowledge(operation)
                .map_err(|error| retain_fault(pending, error))?;
        }
        let pending = state
            .operation
            .take()
            .expect("retained through ACK success");
        Ok(CaptureExecutionAcknowledgement {
            request: pending.request,
            operation: pending.operation,
            terminal: pending.terminal,
            disposition: pending.disposition.expect("explicit disposition"),
            first_fault: pending.first_fault,
            _intent: pending.intent,
        })
    }
}

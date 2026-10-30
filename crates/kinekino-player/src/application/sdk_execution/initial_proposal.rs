use super::*;
use crate::{
    application::{api::RequestContext, proposals::InspectProposal},
    domain::proposals::StoredProposal,
};
/// Trusted startup inspection, before any Meari document/scope exists. Creates no worker or store.
pub struct InitialProposalInspection {
    execution: ExecutionHandle,
    operation: OperationId,
    selected: InspectProposal,
    limit: usize,
    decoded: Option<Arc<Result<StoredProposal, ApiError>>>,
    raw: Option<Arc<ExecutionTerminal>>,
    acknowledged: bool,
}
impl InitialProposalInspection {
    pub fn start(
        execution: ExecutionHandle,
        context: RequestContext,
        selected: InspectProposal,
        max_typed_wire_bytes: usize,
    ) -> Result<Self, ApiError> {
        if max_typed_wire_bytes == 0 {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Initial typed byte budget is empty",
            ));
        }
        let request = Request {
            context,
            command: "proposals.inspect".into(),
            payload: serde_json::to_value(&selected)
                .map_err(|_| internal("Initial inspection encoding failed"))?,
        };
        let operation = execution.submit(request)?;
        Ok(Self {
            execution,
            operation,
            selected,
            limit: max_typed_wire_bytes,
            decoded: None,
            raw: None,
            acknowledged: false,
        })
    }
    pub fn retained_terminal(&self) -> Result<Option<Arc<ExecutionTerminal>>, ApiError> {
        if let Some(raw) = &self.raw {
            return Ok(Some(raw.clone()));
        }
        match self.execution.peek(&self.operation)? {
            ExecutionSnapshot::Terminal(t) => Ok(Some(t)),
            ExecutionSnapshot::Queued | ExecutionSnapshot::Running { .. } => Ok(None),
            ExecutionSnapshot::Acknowledged => Ok(None),
        }
    }
    pub fn peek_terminal(
        &mut self,
    ) -> Result<Option<Arc<Result<StoredProposal, ApiError>>>, ApiError> {
        if let Some(decoded) = &self.decoded {
            return Ok(Some(decoded.clone()));
        }
        let Some(terminal) = self.retained_terminal()? else {
            return Ok(None);
        };
        self.raw = Some(terminal.clone());
        let response = &terminal.response;
        terminal.validate_response()?;
        if terminal.measurement.is_faulted() {
            return Err(internal(
                "Initial inspection terminal requires explicit recovery",
            ));
        }
        let decoded = if response.ok && response.error.is_none() {
            let value = response
                .data
                .as_ref()
                .ok_or_else(|| internal("Initial inspection reply is incomplete"))?;
            if encoded_size(value, self.limit).is_faulted() {
                return Err(capacity());
            }
            let stored: StoredProposal = serde_json::from_value(value.clone())
                .map_err(|_| internal("Initial inspection reply type differs"))?;
            stored
                .validate()
                .map_err(|_| internal("Initial inspection state is invalid"))?;
            if stored.document.proposal_id != self.selected.proposal_id {
                return Err(internal("Initial inspection selection differs"));
            }
            Ok(stored)
        } else if !response.ok && response.data.is_none() {
            let error = response
                .error
                .as_ref()
                .ok_or_else(|| internal("Initial inspection error is absent"))?;
            if encoded_size(error, self.limit).is_faulted() {
                return Err(capacity());
            }
            Err(serde_json::from_value(
                serde_json::to_value(error)
                    .map_err(|_| internal("Initial error encoding failed"))?,
            )
            .map_err(|_| internal("Initial error type differs"))?)
        } else {
            return Err(internal("Initial inspection outcome is inconsistent"));
        };
        let decoded = Arc::new(decoded);
        self.decoded = Some(decoded.clone());
        Ok(Some(decoded))
    }
    /// Called after the trusted startup owner accepts and retains the immutable prepared data.
    /// This does not acknowledge window creation, mounting, rendering, or user quality.
    pub fn acknowledge(&mut self) -> Result<AckReceipt, ApiError> {
        if self.acknowledged {
            return Ok(AckReceipt {
                already_acknowledged: true,
            });
        }
        if self.decoded.is_none() {
            return Err(pending());
        }
        let receipt = self.execution.acknowledge(&self.operation)?;
        self.acknowledged = true;
        Ok(receipt)
    }
    pub fn cancel(&self) -> Result<CancelDisposition, ApiError> {
        if self.acknowledged {
            Ok(CancelDisposition::AlreadyAcknowledged)
        } else {
            self.execution.cancel(&self.operation)
        }
    }
}

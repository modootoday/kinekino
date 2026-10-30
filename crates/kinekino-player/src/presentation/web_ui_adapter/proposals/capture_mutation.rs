//! Presentation ownership only; the shared SDK checks capture authority and CAS.
use crate::application::api::ProposalClient;
use kinekino_media_contracts::errors::ApiError;
use kinekino_player::{
    application::{
        api::{ApplicationClient, RequestContext},
        proposals::guarded::{
            AcceptCapturedProposal, CaptureAnchor, CaptureExpectation, CaptureMutationResult,
            EditCapturedProposal,
        },
    },
    domain::{
        api_values::DecimalU64,
        proposals::{AcceptanceReceipt, ProposalPayload, StoredProposal},
    },
};
use serde::Serialize;
use std::sync::Arc;
mod execution;
pub use execution::{
    CaptureDispatchProgress, CaptureExecutionAcknowledgement, CaptureExecutionDisposition,
    CaptureExecutionProgress, RejectedCaptureIntent,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureAction {
    Edit,
    Accept,
    RetryAccept,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureViewError {
    Busy,
    Exhausted,
    NoAcceptRetry,
    StaleCompletion,
}

enum Request {
    Edit(EditCapturedProposal),
    Accept(AcceptCapturedProposal),
}

pub struct CaptureIntent {
    owner: Arc<()>,
    serial: u64,
    request: Request,
}
pub struct CaptureCompletion {
    owner: Arc<()>,
    serial: u64,
    result: Result<CaptureMutationResult, ApiError>,
}
pub struct RejectedCaptureCompletion {
    pub reason: CaptureViewError,
    pub completion: CaptureCompletion,
}
impl CaptureIntent {
    /// The existing execution owner may move this intent to its SDK worker.
    /// This method never creates a worker, context policy or automatic retry.
    pub fn execute(
        self,
        client: &dyn ApplicationClient,
        context: RequestContext,
    ) -> CaptureCompletion {
        let result = match self.request {
            Request::Edit(request) => client.edit_captured_proposal(context, request),
            Request::Accept(request) => client.accept_captured_proposal(context, request),
        };
        CaptureCompletion {
            owner: self.owner,
            serial: self.serial,
            result,
        }
    }
}

#[derive(Serialize)]
pub struct CaptureDisplay<'a> {
    pub anchor: &'a CaptureAnchor,
    pub state: &'a StoredProposal,
    pub continuation: Option<&'a CaptureExpectation>,
    pub pending: Option<CaptureAction>,
    pub sdk_error: Option<&'a ApiError>,
    pub reused_acceptance: bool,
    pub acceptance: Option<&'a AcceptanceReceipt>,
}

pub struct CaptureMutationAdapter {
    owner: Arc<()>,
    serial: u64,
    anchor: CaptureAnchor,
    state: StoredProposal,
    continuation: Option<CaptureExpectation>,
    pending: Option<CaptureAction>,
    accept_retry: Option<AcceptCapturedProposal>,
    error: Option<ApiError>,
    reused_acceptance: bool,
    acceptance: Option<AcceptanceReceipt>,
    execution: Option<execution::AttachedExecution>,
    closing: bool,
}
impl CaptureMutationAdapter {
    /// Inputs come from the original selected project capture, never a callback payload.
    /// Neither these inputs nor this adapter grants mutation permission.
    pub fn new(anchor: CaptureAnchor, captured: StoredProposal) -> Self {
        let acceptance = captured.acceptance.clone();
        Self {
            owner: Arc::new(()),
            serial: 0,
            anchor,
            state: captured,
            continuation: None,
            pending: None,
            accept_retry: None,
            error: None,
            reused_acceptance: false,
            acceptance,
            execution: None,
            closing: false,
        }
    }
    pub fn display(&self) -> CaptureDisplay<'_> {
        CaptureDisplay {
            anchor: &self.anchor,
            state: &self.state,
            continuation: self.continuation.as_ref(),
            pending: self.pending,
            sdk_error: self.error.as_ref(),
            reused_acceptance: self.reused_acceptance,
            acceptance: self.acceptance.as_ref(),
        }
    }
    pub fn select(
        &mut self,
        anchor: CaptureAnchor,
        captured: StoredProposal,
    ) -> Result<(), CaptureViewError> {
        if self.pending.is_some() || self.execution.is_some() || self.closing {
            return Err(CaptureViewError::Busy);
        }
        // A fresh owner invalidates any completion from the previous selection.
        *self = Self::new(anchor, captured);
        Ok(())
    }
    fn begin(
        &mut self,
        action: CaptureAction,
        request: Request,
    ) -> Result<CaptureIntent, CaptureViewError> {
        if self.pending.is_some() || self.execution_operation_pending() || self.closing {
            return Err(CaptureViewError::Busy);
        }
        let serial = self
            .serial
            .checked_add(1)
            .ok_or(CaptureViewError::Exhausted)?;
        self.serial = serial;
        self.pending = Some(action);
        Ok(CaptureIntent {
            owner: self.owner.clone(),
            serial,
            request,
        })
    }
    pub fn edit(&mut self, draft: ProposalPayload) -> Result<CaptureIntent, CaptureViewError> {
        let request = EditCapturedProposal {
            anchor: self.anchor.clone(),
            expected: self.continuation.clone(),
            draft,
        };
        let intent = self.begin(CaptureAction::Edit, Request::Edit(request))?;
        self.accept_retry = None;
        Ok(intent)
    }
    pub fn accept(
        &mut self,
        target_source_revision: DecimalU64,
    ) -> Result<CaptureIntent, CaptureViewError> {
        let request = AcceptCapturedProposal {
            anchor: self.anchor.clone(),
            expected: self.continuation.clone(),
            target_source_revision,
        };
        let intent = self.begin(CaptureAction::Accept, Request::Accept(request.clone()))?;
        self.accept_retry = Some(request);
        Ok(intent)
    }
    pub fn retry_accept(&mut self) -> Result<CaptureIntent, CaptureViewError> {
        if self.pending.is_some() {
            return Err(CaptureViewError::Busy);
        }
        let request = self
            .accept_retry
            .clone()
            .ok_or(CaptureViewError::NoAcceptRetry)?;
        self.begin(CaptureAction::RetryAccept, Request::Accept(request))
    }
    pub fn complete(
        &mut self,
        completion: CaptureCompletion,
    ) -> Result<(), Box<RejectedCaptureCompletion>> {
        if self.execution.is_some() {
            return Err(Box::new(RejectedCaptureCompletion {
                reason: CaptureViewError::Busy,
                completion,
            }));
        }
        self.apply_completion(completion)
    }
    fn apply_completion(
        &mut self,
        completion: CaptureCompletion,
    ) -> Result<(), Box<RejectedCaptureCompletion>> {
        if !Arc::ptr_eq(&self.owner, &completion.owner)
            || self.serial != completion.serial
            || self.pending.is_none()
        {
            return Err(Box::new(RejectedCaptureCompletion {
                reason: CaptureViewError::StaleCompletion,
                completion,
            }));
        }
        self.pending = None;
        match completion.result {
            Ok(result) => {
                self.state = result.state;
                self.continuation = Some(result.continuation);
                self.reused_acceptance = result.reused_acceptance;
                self.acceptance = result.acceptance;
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;

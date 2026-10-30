//! One current inspection on the existing SDK pending consumer, before Card scope binding.
use super::async_proposals::AsyncProposalRegistration;
use crate::{
    application::{
        api::{ApiError, ErrorCode, RequestContext},
        proposals::InspectProposal,
        sdk_execution::ExecutionHandle,
    },
    presentation::web_ui_adapter::proposals::{
        bootstrap::BootstrapSelection,
        pending_host::{
            PendingHostInput, PendingOutput, PendingRegistryProposalHost, PendingRequest,
            ProposalAdmission, ProposalTerminal,
        },
    },
};
use kinekino_ui_plugins::infrastructure::typed_async_sdk_plugin::AsyncCodecBudget;
use meari::application::plugins::*;
use std::{cell::RefCell, rc::Rc};

pub struct CardInspectionInput {
    pub execution: ExecutionHandle,
    pub scope: PluginScope,
    pub registry: Rc<RefCell<PluginRegistry>>,
    pub context: Rc<RefCell<RequestContext>>,
    pub namespace: String,
    pub limit: usize,
    pub codec: AsyncCodecBudget,
}
#[must_use]
pub struct CardInspectionStartFailure {
    pub error: PluginError,
    pub registration: Option<AsyncProposalRegistration>,
}
impl CardInspectionStartFailure {
    pub fn retry_unbind(&mut self) -> Result<(), PluginError> {
        if let Some(registration) = &mut self.registration {
            registration.unbind()?;
        }
        Ok(())
    }
}

/// Retains the original ticket/result while PendingRegistryProposalHost owns decode/cancel/ACK.
#[must_use = "Retain the inspection through original terminal ACK and explicit scope release"]
pub struct CardCurrentInspection {
    host: PendingRegistryProposalHost,
    registration: AsyncProposalRegistration,
    selection: Option<BootstrapSelection>,
    ticket: Option<PluginTicket>,
    terminal: Option<ProposalTerminal>,
    unexpected_immediate: Option<Box<Result<PendingOutput, ApiError>>>,
    released: bool,
}
impl CardCurrentInspection {
    pub fn scope(&self) -> &PluginScope {
        self.host.scope()
    }
    pub fn new(input: CardInspectionInput) -> Result<Self, Box<CardInspectionStartFailure>> {
        let registration = AsyncProposalRegistration::register_output_reviews(
            input.registry.clone(),
            &input.scope,
            input.context.clone(),
            input.execution,
            input.codec,
        )
        .map_err(|error| {
            Box::new(CardInspectionStartFailure {
                error,
                registration: None,
            })
        })?;
        let handle = match registration.handle() {
            Ok(handle) => handle.clone(),
            Err(error) => {
                return Err(Box::new(CardInspectionStartFailure {
                    error,
                    registration: Some(registration),
                }));
            }
        };
        let host = PendingRegistryProposalHost::new(PendingHostInput {
            registry: input.registry,
            registration: handle,
            scope: input.scope,
            context: input.context,
            namespace: input.namespace,
            pump: registration.pump().clone(),
            limit: input.limit,
        });
        let host = match host {
            Ok(host) => host,
            Err(error) => {
                return Err(Box::new(CardInspectionStartFailure {
                    error,
                    registration: Some(registration),
                }));
            }
        };
        Ok(Self {
            host,
            registration,
            selection: None,
            ticket: None,
            terminal: None,
            unexpected_immediate: None,
            released: false,
        })
    }
    pub fn begin(&mut self, selection: BootstrapSelection) -> Result<(), ApiError> {
        if self.selection.is_some() || self.released {
            return Err(ApiError::new(
                ErrorCode::NeedsInput,
                "Current Card inspection already selected or released",
            ));
        }
        let admission = self.host.submit(PendingRequest::Inspect(InspectProposal {
            proposal_id: selection.proposal_id.clone(),
        }))?;
        self.selection = Some(selection);
        match admission {
            ProposalAdmission::Pending(ticket) => {
                self.ticket = Some(ticket);
                Ok(())
            }
            ProposalAdmission::Immediate(result) => {
                self.unexpected_immediate = Some(result);
                Err(ApiError::new(
                    ErrorCode::EngineFailed,
                    "Native async inspection returned an immediate outcome; original result retained",
                ))
            }
        }
    }
    pub fn selection(&self) -> Option<&BootstrapSelection> {
        self.selection.as_ref()
    }
    pub fn terminal(&self) -> Option<&ProposalTerminal> {
        self.terminal.as_ref()
    }
    pub fn unexpected_immediate(&self) -> Option<&Result<PendingOutput, ApiError>> {
        self.unexpected_immediate.as_deref()
    }
    pub fn pending_ticket(&self) -> Option<&PluginTicket> {
        self.ticket.as_ref()
    }
    pub fn poll(&mut self) -> Result<bool, PluginError> {
        let Some(ticket) = &self.ticket else {
            return Ok(self.terminal.is_some());
        };
        let Some(terminal) = self.host.poll(ticket)? else {
            return Ok(false);
        };
        self.terminal = Some(terminal);
        self.ticket = None;
        Ok(true)
    }
    /// Release only after the original result/ACK has been accepted; no execution shutdown.
    pub fn release(&mut self) -> Result<(), PluginError> {
        if self.released {
            return Ok(());
        }
        if self.ticket.is_some() {
            return Err(PluginError::new(
                PluginErrorCode::Busy,
                "Original Card inspection ticket still pending",
            ));
        }
        // This host tracks only the one accepted ticket; the registration is the sole
        // unregister owner. Calling host.close as well would unregister the same lease twice.
        self.registration.unbind()?;
        self.released = true;
        Ok(())
    }
    pub fn is_released(&self) -> bool {
        self.released
    }
    /// Close retries use the exact original ticket and existing host cancellation/ACK path.
    pub fn poll_close(&mut self) -> Result<bool, PluginError> {
        if let Some(ticket) = &self.ticket {
            self.host.cancel(ticket)?;
        }
        if self.ticket.is_some() && !self.poll()? {
            return Ok(false);
        }
        self.release()?;
        Ok(true)
    }
}

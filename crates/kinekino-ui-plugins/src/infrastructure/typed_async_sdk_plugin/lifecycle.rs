use super::AsyncSdkBinding;
use meari::application::plugins::*;
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

struct Entry {
    ticket: PluginTicket,
    accepted: Option<PluginEvent>,
}
struct State {
    bindings: BTreeMap<String, Box<dyn AsyncSdkBinding>>,
    entries: Vec<Entry>,
    limit: usize,
    max_terminal_wire_bytes: usize,
    closing: bool,
}
/// Encoded-copy reservation, not a bound on arbitrary DTO Deserialize heap allocation.
pub struct AsyncCodecBudget {
    pub max_pending: usize,
    pub max_terminal_wire_bytes: usize,
    pub max_total_codec_wire_bytes: usize,
}
pub struct TypedAsyncSdkPlugin {
    manifest: PluginManifest,
    state: Rc<RefCell<State>>,
}
#[derive(Clone)]
pub struct AsyncSdkPump {
    state: Rc<RefCell<State>>,
}
/// A result already accepted by Core remains observable even when owner ACK fails.
pub struct TerminalAcceptance {
    pub event: PluginEvent,
    pub owner_ack: Result<(), PluginError>,
}
fn failure(code: PluginErrorCode, message: &str) -> PluginError {
    PluginError::new(code, message)
}
fn borrowed() -> PluginError {
    failure(PluginErrorCode::Busy, "Async SDK owner is already borrowed")
}
fn stale() -> PluginError {
    failure(PluginErrorCode::Stale, "Unknown actual async SDK ticket")
}
impl State {
    fn binding(&self, capability: &str) -> Result<&dyn AsyncSdkBinding, PluginError> {
        self.bindings
            .get(capability)
            .map(|binding| binding.as_ref())
            .ok_or_else(|| {
                failure(
                    PluginErrorCode::MissingCapability,
                    "Unknown async SDK capability",
                )
            })
    }
}
impl TypedAsyncSdkPlugin {
    pub fn new(
        manifest: PluginManifest,
        bindings: BTreeMap<String, Box<dyn AsyncSdkBinding>>,
        budget: AsyncCodecBudget,
    ) -> Result<(Self, AsyncSdkPump), PluginError> {
        manifest.validate()?;
        let reserved = budget
            .max_pending
            .checked_mul(budget.max_terminal_wire_bytes)
            .and_then(|bytes| bytes.checked_mul(4));
        if budget.max_pending == 0
            || budget.max_terminal_wire_bytes == 0
            || reserved.is_none_or(|bytes| bytes > budget.max_total_codec_wire_bytes)
            || manifest.capabilities.len() != bindings.len()
            || bindings.iter().any(|(name, binding)| {
                manifest.capabilities.get(name) != Some(binding.descriptor())
                    || binding.descriptor().error_schema.is_none()
                    || binding.descriptor().event_schema.is_some()
            })
        {
            return Err(failure(
                PluginErrorCode::InvalidManifest,
                "Async SDK bindings, schemas or tracking budget differ",
            ));
        }
        let state = Rc::new(RefCell::new(State {
            bindings,
            entries: Vec::new(),
            limit: budget.max_pending,
            max_terminal_wire_bytes: budget.max_terminal_wire_bytes,
            closing: false,
        }));
        Ok((
            Self {
                manifest,
                state: state.clone(),
            },
            AsyncSdkPump { state },
        ))
    }
}
impl PluginAdapter for TypedAsyncSdkPlugin {
    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }
    fn validate_payload(
        &self,
        capability: &str,
        direction: PayloadDirection,
        payload: &PluginPayload,
    ) -> Result<(), PluginError> {
        self.state
            .try_borrow()
            .map_err(|_| borrowed())?
            .binding(capability)?
            .validate(direction, payload)
    }
    fn dispatch(&mut self, invocation: &PluginInvocation) -> Result<PluginDispatch, PluginError> {
        let mut state = self.state.try_borrow_mut().map_err(|_| borrowed())?;
        if state.closing {
            return Err(failure(
                PluginErrorCode::Closing,
                "Async SDK owner is closing",
            ));
        }
        if state.entries.len() >= state.limit {
            return Err(failure(
                PluginErrorCode::Busy,
                "Async SDK tracking capacity reached",
            ));
        }
        if state
            .entries
            .iter()
            .any(|entry| entry.ticket == invocation.ticket)
        {
            return Err(stale());
        }
        state
            .bindings
            .get_mut(invocation.ticket.capability())
            .ok_or_else(stale)?
            .start(invocation)?;
        state.entries.push(Entry {
            ticket: invocation.ticket.clone(),
            accepted: None,
        });
        Ok(PluginDispatch::Pending)
    }
    fn cancel(&mut self, ticket: &PluginTicket) -> Result<CancelReceipt, PluginError> {
        let mut state = self.state.try_borrow_mut().map_err(|_| borrowed())?;
        if !state.entries.iter().any(|entry| &entry.ticket == ticket) {
            return Err(stale());
        }
        let receipt = state
            .bindings
            .get_mut(ticket.capability())
            .ok_or_else(stale)?
            .cancel(ticket)?;
        // Even queued removal uses a canonical terminal and ACK; never erase a ready result here.
        match receipt {
            CancelReceipt::Requested | CancelReceipt::Completed => Ok(CancelReceipt::Requested),
            CancelReceipt::AlreadyTerminal => Err(failure(
                PluginErrorCode::Adapter,
                "Owner acknowledged before registry terminal acceptance",
            )),
        }
    }
    fn shutdown(&mut self) -> Result<(), PluginError> {
        let mut state = self.state.try_borrow_mut().map_err(|_| borrowed())?;
        state.closing = true;
        if !state.entries.is_empty() {
            return Err(failure(
                PluginErrorCode::Busy,
                "Async SDK terminal ownership remains",
            ));
        }
        for binding in state.bindings.values_mut() {
            binding.shutdown()?;
        }
        Ok(())
    }
}
impl AsyncSdkPump {
    /// Stops all adapter admissions while the host continues terminal drain/ACK retries.
    pub fn begin_close(&self) -> Result<(), PluginError> {
        self.state.try_borrow_mut().map_err(|_| borrowed())?.closing = true;
        Ok(())
    }
    pub fn tickets(&self) -> Result<Vec<PluginTicket>, PluginError> {
        Ok(self
            .state
            .try_borrow()
            .map_err(|_| borrowed())?
            .entries
            .iter()
            .map(|entry| entry.ticket.clone())
            .collect())
    }
    /// No registry borrow may be held on entry. Early completion is read only after dispatch.
    pub fn accept(
        &self,
        registry: &mut PluginRegistry,
        ticket: &PluginTicket,
    ) -> Result<Option<TerminalAcceptance>, PluginError> {
        let accepted = {
            let state = self.state.try_borrow().map_err(|_| borrowed())?;
            let index = state
                .entries
                .iter()
                .position(|entry| &entry.ticket == ticket)
                .ok_or_else(stale)?;
            state.entries[index].accepted.clone()
        };
        let event = if let Some(event) = accepted {
            event
        } else {
            let terminal = {
                let mut state = self.state.try_borrow_mut().map_err(|_| borrowed())?;
                let limit = state.max_terminal_wire_bytes;
                state
                    .bindings
                    .get_mut(ticket.capability())
                    .ok_or_else(stale)?
                    .peek_terminal(ticket, limit)?
            };
            let Some(terminal) = terminal else {
                return Ok(None);
            };
            let wire_bytes = match &terminal {
                PluginEventKind::Completed(payload) => super::codec::wire_bytes(payload)?,
                PluginEventKind::Failed(error) => super::codec::wire_bytes(error)?,
                _ => {
                    return Err(failure(
                        PluginErrorCode::InvalidPayload,
                        "Expected typed terminal-only envelope",
                    ));
                }
            };
            if wire_bytes
                > self
                    .state
                    .try_borrow()
                    .map_err(|_| borrowed())?
                    .max_terminal_wire_bytes
            {
                return Err(failure(
                    PluginErrorCode::InvalidPayload,
                    "Async SDK wire envelope exceeds caller codec budget",
                ));
            }
            // Failed validation leaves both registry pending and the provider result intact.
            let event = registry.accept_event(ticket, terminal)?;
            let mut state = self.state.try_borrow_mut().map_err(|_| borrowed())?;
            let entry = state
                .entries
                .iter_mut()
                .find(|entry| &entry.ticket == ticket)
                .ok_or_else(stale)?;
            entry.accepted = Some(event.clone());
            event
        };
        let owner_ack = {
            let mut state = self.state.try_borrow_mut().map_err(|_| borrowed())?;
            let result = state
                .bindings
                .get_mut(ticket.capability())
                .ok_or_else(stale)?
                .acknowledge(ticket);
            if result.is_ok() {
                state.entries.retain(|entry| &entry.ticket != ticket);
            }
            result
        };
        Ok(Some(TerminalAcceptance { event, owner_ack }))
    }
}

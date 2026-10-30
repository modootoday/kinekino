//! Explicit async consumer; the legacy synchronous RegistryProposalHost is unchanged.
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_player::application::proposals::output_review::{
    InspectOutputReviewTarget, ListOutputDecisions, OutputDecisionHistory, OutputReviewInspection,
    RecordOutputDecision, StoredOutputDecision,
};
use kinekino_player::{
    application::{
        ai_provenance::ImportHistoricalProposal,
        api::{Request, RequestContext},
        proposals::{AcceptProposal, CreateProposal, EditProposal, InspectProposal},
    },
    domain::{api_values::DecimalU64, proposals::*},
};
use kinekino_ui_plugins::infrastructure::typed_async_sdk_plugin::AsyncSdkPump;
use meari::application::plugins::*;
use std::{cell::RefCell, rc::Rc};

pub enum PendingRequest {
    InspectRawReview(kinekino_player::application::model_review::InspectReviewPackage),
    ReadRawReviewAsset(kinekino_player::application::model_review::ReadReviewTextAsset),
    RecordRawReview(kinekino_player::domain::model_review::RecordReviewDecision),
    RawReviewDecisions(kinekino_player::application::model_review::ListReviewDecisions),
    Create(CreateProposal),
    Inspect(InspectProposal),
    Edit {
        revision: DecimalU64,
        input: EditProposal,
    },
    Accept {
        revision: DecimalU64,
        input: AcceptProposal,
    },
    Import(ImportHistoricalProposal),
    InspectOutputReview(InspectOutputReviewTarget),
    RecordOutputDecision {
        revision: DecimalU64,
        input: Box<RecordOutputDecision>,
    },
    OutputDecisions(ListOutputDecisions),
}
pub enum PendingOutput {
    RawReviewInspected(Box<kinekino_player::application::model_review::ReviewPackageView>),
    RawReviewAsset(kinekino_player::application::model_review::ReviewTextAsset),
    RawReviewRecorded(kinekino_player::application::model_review::StoredDecision),
    RawReviewHistory(Vec<kinekino_player::application::model_review::StoredDecision>),
    Created(ProposalDocument),
    Inspected(StoredProposal),
    Edited(ProposalDocument),
    Accepted(AcceptanceReceipt),
    Imported(StoredProposal),
    OutputReviewInspected(Box<OutputReviewInspection>),
    OutputDecisionRecorded(Box<StoredOutputDecision>),
    OutputDecisionsListed(OutputDecisionHistory),
}
pub enum ProposalAdmission {
    Immediate(Box<Result<PendingOutput, ApiError>>),
    Pending(PluginTicket),
}
pub struct ProposalTerminal {
    pub ticket: PluginTicket,
    pub outcome: Result<PendingOutput, ApiError>,
    pub cancellation_requested: bool,
}
struct Entry {
    ticket: PluginTicket,
    descriptor: CapabilityDescriptor,
    accepted: Option<PluginEvent>,
}
pub struct PendingRegistryProposalHost {
    registry: Rc<RefCell<PluginRegistry>>,
    registration: RegistrationHandle,
    scope: PluginScope,
    context: Rc<RefCell<RequestContext>>,
    namespace: String,
    pump: AsyncSdkPump,
    entries: Vec<Entry>,
    limit: usize,
    closing: bool,
    closed: bool,
    completion_guard_blocked: bool,
}
pub struct PendingHostInput {
    pub registry: Rc<RefCell<PluginRegistry>>,
    pub registration: RegistrationHandle,
    pub scope: PluginScope,
    pub context: Rc<RefCell<RequestContext>>,
    pub namespace: String,
    pub pump: AsyncSdkPump,
    pub limit: usize,
}
fn plugin_error(error: PluginError) -> ApiError {
    let mut result = ApiError::new(ErrorCode::Internal, error.message.clone());
    result.details = serde_json::json!({"plugin_error":error});
    result
}
fn busy() -> PluginError {
    PluginError::new(
        PluginErrorCode::Busy,
        "Pending proposal owner is borrowed or requires drain",
    )
}
fn stale() -> PluginError {
    PluginError::new(
        PluginErrorCode::Stale,
        "Unknown pending proposal ticket or scope",
    )
}
fn codec(error: serde_json::Error) -> ApiError {
    ApiError::new(ErrorCode::Internal, error.to_string())
}
fn decode(
    command: &str,
    payload: &PluginPayload,
    descriptor: &CapabilityDescriptor,
) -> Result<PendingOutput, PluginError> {
    let schema = &descriptor.reply_schema;
    match command {
        "reviews.inspect" => payload
            .decode(schema)
            .map(PendingOutput::RawReviewInspected),
        "reviews.read_text_asset" => payload.decode(schema).map(PendingOutput::RawReviewAsset),
        "reviews.record_decision" => payload.decode(schema).map(PendingOutput::RawReviewRecorded),
        "reviews.decisions" => payload.decode(schema).map(PendingOutput::RawReviewHistory),
        "proposals.create" => payload.decode(schema).map(PendingOutput::Created),
        "proposals.inspect" => payload.decode(schema).map(PendingOutput::Inspected),
        "proposals.edit" => payload.decode(schema).map(PendingOutput::Edited),
        "proposals.accept" => payload.decode(schema).map(PendingOutput::Accepted),
        "proposals.import_historical" => payload.decode(schema).map(PendingOutput::Imported),
        "proposals.inspect_output_review_target" => payload
            .decode(schema)
            .map(PendingOutput::OutputReviewInspected),
        "proposals.record_output_decision" => payload
            .decode(schema)
            .map(PendingOutput::OutputDecisionRecorded),
        "proposals.output_decisions" => payload
            .decode(schema)
            .map(PendingOutput::OutputDecisionsListed),
        _ => Err(stale()),
    }
}
fn decode_error(
    error: &PluginError,
    descriptor: &CapabilityDescriptor,
) -> Result<ApiError, PluginError> {
    let payload = error.details.as_ref().ok_or_else(|| {
        PluginError::new(
            PluginErrorCode::InvalidPayload,
            "Canonical SDK error details required",
        )
    })?;
    payload.decode(descriptor.error_schema.as_ref().ok_or_else(stale)?)
}
impl PendingRegistryProposalHost {
    pub fn registry_handle(&self) -> Rc<RefCell<PluginRegistry>> {
        self.registry.clone()
    }
    pub(super) fn uses_registry(&self, registry: &Rc<RefCell<PluginRegistry>>) -> bool {
        Rc::ptr_eq(&self.registry, registry)
    }
    pub(super) fn begin_close(&mut self) -> Result<(), PluginError> {
        self.closing = true;
        self.completion_guard_blocked = false;
        self.pump.begin_close()?;
        let mut registry = self.registry.try_borrow_mut().map_err(|_| {
            self.completion_guard_blocked = true;
            busy()
        })?;
        for entry in &self.entries {
            registry.cancel(&entry.ticket)?;
        }
        Ok(())
    }
    pub(super) fn quiescent(&self) -> Result<(), PluginError> {
        if self.entries.is_empty() {
            Ok(())
        } else {
            Err(busy())
        }
    }
    pub fn scope(&self) -> &PluginScope {
        &self.scope
    }
    pub fn new(input: PendingHostInput) -> Result<Self, PluginError> {
        let PendingHostInput {
            registry,
            registration,
            scope,
            context,
            namespace,
            pump,
            limit,
        } = input;
        if namespace.is_empty() || limit == 0 {
            return Err(PluginError::new(
                PluginErrorCode::InvalidManifest,
                "Explicit namespace and positive tracking budget required",
            ));
        }
        {
            let current = registry.try_borrow().map_err(|_| busy())?;
            if current.scope() != &scope {
                return Err(stale());
            }
            let manifest = current.manifest(&registration)?;
            for command in [
                "proposals.create",
                "proposals.inspect",
                "proposals.edit",
                "proposals.accept",
                "proposals.import_historical",
            ] {
                if !manifest.capabilities.contains_key(command) {
                    return Err(PluginError::new(
                        PluginErrorCode::MissingCapability,
                        "Async proposal consumer requires all five capabilities",
                    ));
                }
            }
        }
        Ok(Self {
            registry,
            registration,
            scope,
            context,
            namespace,
            pump,
            entries: Vec::new(),
            limit,
            closing: false,
            closed: false,
            completion_guard_blocked: false,
        })
    }
    pub fn submit(&mut self, input: PendingRequest) -> Result<ProposalAdmission, ApiError> {
        if self.closing || self.closed {
            return Err(plugin_error(PluginError::new(
                PluginErrorCode::Closing,
                "Pending proposal host is closing",
            )));
        }
        if self.entries.len() >= self.limit {
            return Err(plugin_error(busy()));
        }
        let (command, revision, payload) = match input {
            PendingRequest::InspectRawReview(input) => {
                ("reviews.inspect", None, serde_json::to_value(input))
            }
            PendingRequest::ReadRawReviewAsset(input) => {
                ("reviews.read_text_asset", None, serde_json::to_value(input))
            }
            PendingRequest::RecordRawReview(input) => {
                ("reviews.record_decision", None, serde_json::to_value(input))
            }
            PendingRequest::RawReviewDecisions(input) => {
                ("reviews.decisions", None, serde_json::to_value(input))
            }
            PendingRequest::Create(input) => {
                ("proposals.create", None, serde_json::to_value(input))
            }
            PendingRequest::Inspect(input) => {
                ("proposals.inspect", None, serde_json::to_value(input))
            }
            PendingRequest::Edit { revision, input } => (
                "proposals.edit",
                Some(revision),
                serde_json::to_value(input),
            ),
            PendingRequest::Accept { revision, input } => (
                "proposals.accept",
                Some(revision),
                serde_json::to_value(input),
            ),
            PendingRequest::Import(input) => (
                "proposals.import_historical",
                None,
                serde_json::to_value(input),
            ),
            PendingRequest::InspectOutputReview(input) => (
                "proposals.inspect_output_review_target",
                None,
                serde_json::to_value(input),
            ),
            PendingRequest::RecordOutputDecision { revision, input } => (
                "proposals.record_output_decision",
                Some(revision),
                serde_json::to_value(input),
            ),
            PendingRequest::OutputDecisions(input) => (
                "proposals.output_decisions",
                None,
                serde_json::to_value(input),
            ),
        };
        let request = Request {
            command: command.into(),
            payload: payload.map_err(codec)?,
            context: RequestContext {
                request_id: self.namespace.clone(),
                expected_revision: revision.map(|revision| revision.0.to_string()),
                ..Default::default()
            },
        };
        request.validate()?;
        let descriptor = {
            let current = self
                .registry
                .try_borrow()
                .map_err(|_| plugin_error(busy()))?;
            if current.scope() != &self.scope {
                return Err(plugin_error(stale()));
            }
            current
                .manifest(&self.registration)
                .map_err(plugin_error)?
                .capabilities
                .get(command)
                .cloned()
                .ok_or_else(|| {
                    if kinekino_player::application::proposals::output_review::is_command(command)
                        || kinekino_player::application::model_review::is_command(command)
                    {
                        ApiError::new(
                            ErrorCode::Unsupported,
                            "Output review capability is not registered",
                        )
                    } else {
                        plugin_error(stale())
                    }
                })?
        };
        *self
            .context
            .try_borrow_mut()
            .map_err(|_| plugin_error(busy()))? = request.context;
        let receipt = self
            .registry
            .try_borrow_mut()
            .map_err(|_| plugin_error(busy()))?
            .dispatch(
                &self.registration,
                command,
                PluginPayload {
                    schema: descriptor.request_schema.clone(),
                    value: request.payload,
                },
            )
            .map_err(plugin_error)?;
        match receipt.outcome {
            PluginDispatch::Pending => {
                self.entries.push(Entry {
                    ticket: receipt.ticket.clone(),
                    descriptor,
                    accepted: None,
                });
                Ok(ProposalAdmission::Pending(receipt.ticket))
            }
            PluginDispatch::Immediate(result) => {
                Ok(ProposalAdmission::Immediate(Box::new(match result {
                    Ok(payload) => {
                        Ok(decode(command, &payload, &descriptor).map_err(plugin_error)?)
                    }
                    Err(error) => Err(decode_error(&error, &descriptor).map_err(plugin_error)?),
                })))
            }
        }
    }
    /// Call outside document/registry guards; ACK failure preserves the actual ticket.
    pub fn poll(&mut self, ticket: &PluginTicket) -> Result<Option<ProposalTerminal>, PluginError> {
        self.completion_guard_blocked = false;
        let index = self
            .entries
            .iter()
            .position(|entry| &entry.ticket == ticket)
            .ok_or_else(stale)?;
        if self.entries[index].accepted.is_none() {
            let result = {
                let mut registry = self.registry.try_borrow_mut().map_err(|_| {
                    self.completion_guard_blocked = true;
                    busy()
                })?;
                self.pump.accept(&mut registry, ticket)?
            };
            let Some(result) = result else {
                return Ok(None);
            };
            result.owner_ack?;
            self.entries[index].accepted = Some(result.event);
        }
        let entry = &self.entries[index];
        let event = entry.accepted.as_ref().ok_or_else(stale)?;
        let outcome = match &event.event {
            PluginEventKind::Completed(payload) => {
                Ok(decode(ticket.capability(), payload, &entry.descriptor)?)
            }
            PluginEventKind::Failed(error) => Err(decode_error(error, &entry.descriptor)?),
            _ => {
                return Err(PluginError::new(
                    PluginErrorCode::InvalidPayload,
                    "Expected terminal-only canonical SDK result",
                ));
            }
        };
        let cancellation_requested = event.cancellation_requested;
        self.entries.remove(index);
        Ok(Some(ProposalTerminal {
            ticket: ticket.clone(),
            outcome,
            cancellation_requested,
        }))
    }
    pub fn cancel(&mut self, ticket: &PluginTicket) -> Result<CancelReceipt, PluginError> {
        if !self.entries.iter().any(|entry| &entry.ticket == ticket) {
            return Err(stale());
        }
        self.registry
            .try_borrow_mut()
            .map_err(|_| busy())?
            .cancel(ticket)
    }
    pub fn pending_tickets(&self) -> Vec<PluginTicket> {
        self.entries
            .iter()
            .map(|entry| entry.ticket.clone())
            .collect()
    }
    pub fn completion_guard_blocked(&self) -> bool {
        self.completion_guard_blocked
    }
    /// Busy keeps this host, its ledger and SDK owner available to poll and retry close.
    pub fn close(&mut self) -> Result<(), PluginError> {
        if self.closed {
            return Ok(());
        }
        self.begin_close()?;
        self.quiescent()?;
        let mut registry = self.registry.try_borrow_mut().map_err(|_| busy())?;
        registry.unregister(&self.registration)?;
        self.closed = true;
        Ok(())
    }
}

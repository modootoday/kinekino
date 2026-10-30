use meari::{
    application::{host_callbacks::HostCallbacks, plugins::*},
    domain::{
        build::{ContentHash, TargetKind},
        host_callbacks::{CallbackArguments, CallbackReference, CallbackSignature},
    },
};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

use super::intent_queue::output_review_intents::{OutputReviewAction, OutputReviewField};
use super::{IntentError, NodeLease, ReviewAction, ReviewIntents, TextInput};
pub(super) const REVIEW_INTENT_PLUGIN_ID: &str = "com.kinekino.player.review-intents";

pub struct IntentPins {
    pub contract: ContentHash,
    pub provider: ContentHash,
    pub request: ContentHash,
    pub reply: ContentHash,
}

pub enum QueueBinding {
    RawReviewAction {
        node: NodeLease,
        action: super::raw_review_state::RawReviewButton,
    },
    RawReviewText {
        node: NodeLease,
        field: super::raw_review_state::RawReviewField,
    },
    OutputReviewAction {
        node: NodeLease,
        action: OutputReviewAction,
    },
    OutputReviewText {
        node: NodeLease,
        field: OutputReviewField,
    },
    InspectorOpen {
        node: NodeLease,
        mailbox: Rc<RefCell<super::inspector_intents::InspectorIntents>>,
    },
    DocumentChoice {
        node: NodeLease,
    },
    UnboundText {
        node: NodeLease,
    },
    Text {
        node: NodeLease,
        field: TextInput,
    },
    Action {
        node: NodeLease,
        action: ReviewAction,
    },
}
impl QueueBinding {
    pub fn signature(&self) -> CallbackSignature {
        match self {
            Self::InspectorOpen { .. } => CallbackSignature::Boolean,
            Self::Text { .. }
            | Self::RawReviewText { .. }
            | Self::DocumentChoice { .. }
            | Self::UnboundText { .. }
            | Self::OutputReviewText { .. } => CallbackSignature::Text,
            Self::Action { .. }
            | Self::OutputReviewAction { .. }
            | Self::RawReviewAction { .. } => CallbackSignature::Action,
        }
    }
}

/// Shared by product lifetime assembly and the actual SDK consumer fixture.
pub fn bind_review_callbacks(
    scope: PluginScope,
    registration: &RegistrationHandle,
    signatures: &BTreeMap<String, CallbackSignature>,
    schema: &ContentHash,
) -> Result<HostCallbacks, PluginError> {
    let mut callbacks = HostCallbacks::new(scope);
    for (prop, signature) in signatures {
        CallbackReference {
            binding: prop.clone(),
            signature: *signature,
        }
        .validate()
        .map_err(|message| PluginError::new(PluginErrorCode::InvalidPayload, message))?;
        let request_schema = schema.clone();
        callbacks.bind(
            prop.clone(),
            *signature,
            registration.clone(),
            prop.clone(),
            move |arguments| PluginPayload::encode(request_schema.clone(), arguments),
        )?;
    }
    Ok(callbacks)
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueuedAck {
    pub queued: bool,
}

/// This plugin enqueues presentation input only; it never calls the SDK or controller.
pub struct QueueCallbackAdapter {
    scope: PluginScope,
    manifest: PluginManifest,
    queue: Rc<RefCell<ReviewIntents>>,
    bindings: BTreeMap<String, QueueBinding>,
}
impl QueueCallbackAdapter {
    pub fn new(
        scope: PluginScope,
        queue: Rc<RefCell<ReviewIntents>>,
        bindings: BTreeMap<String, QueueBinding>,
        pins: IntentPins,
        targets: BTreeSet<TargetKind>,
    ) -> Result<Self, PluginError> {
        Self::with_plugin_id(
            scope,
            queue,
            bindings,
            pins,
            targets,
            REVIEW_INTENT_PLUGIN_ID.into(),
        )
    }
    pub fn with_plugin_id(
        scope: PluginScope,
        queue: Rc<RefCell<ReviewIntents>>,
        bindings: BTreeMap<String, QueueBinding>,
        pins: IntentPins,
        targets: BTreeSet<TargetKind>,
        plugin_id: String,
    ) -> Result<Self, PluginError> {
        if queue
            .try_borrow()
            .map_err(|_| {
                PluginError::new(
                    PluginErrorCode::Adapter,
                    "Presentation intent queue is borrowed",
                )
            })?
            .scope()
            .view
            != scope
        {
            return Err(PluginError::new(
                PluginErrorCode::Stale,
                "Review queue document differs",
            ));
        }
        let manifest = PluginManifest {
            api_version: PLUGIN_API_VERSION,
            id: plugin_id,
            version: "1".into(),
            contract: pins.contract,
            provider: pins.provider,
            capabilities: bindings
                .keys()
                .map(|name| {
                    (
                        name.clone(),
                        CapabilityDescriptor {
                            targets: targets.clone(),
                            request_schema: pins.request.clone(),
                            reply_schema: pins.reply.clone(),
                            error_schema: None,
                            event_schema: None,
                        },
                    )
                })
                .collect(),
            assets: BTreeMap::new(),
        };
        manifest.validate()?;
        Ok(Self {
            scope,
            manifest,
            queue,
            bindings,
        })
    }
    fn descriptor(&self, capability: &str) -> Result<&CapabilityDescriptor, PluginError> {
        self.manifest.capabilities.get(capability).ok_or_else(|| {
            PluginError::new(
                PluginErrorCode::MissingCapability,
                "Unknown review callback",
            )
        })
    }
}
fn queue_error(error: IntentError) -> PluginError {
    PluginError::new(
        match error {
            IntentError::Stale => PluginErrorCode::Stale,
            IntentError::Capacity | IntentError::TextCapacity => PluginErrorCode::Busy,
            _ => PluginErrorCode::InvalidPayload,
        },
        format!("Presentation intent rejected: {error:?}"),
    )
}

impl PluginAdapter for QueueCallbackAdapter {
    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }
    fn validate_payload(
        &self,
        capability: &str,
        direction: PayloadDirection,
        payload: &PluginPayload,
    ) -> Result<(), PluginError> {
        let descriptor = self.descriptor(capability)?;
        match direction {
            PayloadDirection::Request => {
                let arguments = payload.decode::<CallbackArguments>(&descriptor.request_schema)?;
                arguments.validate().map_err(|message| {
                    PluginError::new(PluginErrorCode::InvalidPayload, message)
                })?;
                if arguments.signature() != self.bindings[capability].signature() {
                    return Err(PluginError::new(
                        PluginErrorCode::InvalidPayload,
                        "Review callback signature differs",
                    ));
                }
                if let CallbackArguments::Text(text) = &arguments {
                    self.queue
                        .try_borrow()
                        .map_err(|_| {
                            PluginError::new(
                                PluginErrorCode::Adapter,
                                "Presentation input validation is borrowed",
                            )
                        })?
                        .validate_single_text(text)
                        .map_err(queue_error)?;
                }
            }
            PayloadDirection::Reply => {
                if !payload
                    .decode::<QueuedAck>(&descriptor.reply_schema)?
                    .queued
                {
                    return Err(PluginError::new(
                        PluginErrorCode::InvalidPayload,
                        "Invalid enqueue acknowledgement",
                    ));
                }
            }
            _ => {
                return Err(PluginError::new(
                    PluginErrorCode::InvalidPayload,
                    "Review intent emits no events or error payloads",
                ));
            }
        }
        Ok(())
    }
    fn dispatch(&mut self, invocation: &PluginInvocation) -> Result<PluginDispatch, PluginError> {
        if invocation.ticket.scope() != &self.scope {
            return Err(PluginError::new(
                PluginErrorCode::Stale,
                "Review callback document differs",
            ));
        }
        let capability = invocation.ticket.capability();
        self.validate_payload(capability, PayloadDirection::Request, &invocation.payload)?;
        let descriptor = self.descriptor(capability)?;
        let arguments = invocation
            .payload
            .decode::<CallbackArguments>(&descriptor.request_schema)?;
        let mut queue = self.queue.try_borrow_mut().map_err(|_| {
            PluginError::new(
                PluginErrorCode::Adapter,
                "Presentation intent queue is borrowed",
            )
        })?;
        let result = match (&self.bindings[capability], arguments) {
            (QueueBinding::OutputReviewAction { node, action }, CallbackArguments::Action) => {
                queue.enqueue_output_review_action(node, *action)
            }
            (QueueBinding::OutputReviewText { node, field }, CallbackArguments::Text(text)) => {
                queue.enqueue_output_review_text(node, *field, text)
            }
            (QueueBinding::InspectorOpen { node, mailbox }, CallbackArguments::Boolean(open)) => {
                queue.check(node).map_err(queue_error)?;
                mailbox
                    .try_borrow_mut()
                    .map_err(|_| {
                        PluginError::new(PluginErrorCode::Adapter, "Inspector queue is borrowed")
                    })?
                    .enqueue_open(open)
                    .map_err(|_| IntentError::Capacity)
            }
            (QueueBinding::RawReviewAction { node, action }, CallbackArguments::Action) => {
                queue.enqueue_raw_action(node, *action)
            }
            (QueueBinding::RawReviewText { node, field }, CallbackArguments::Text(text)) => {
                queue.enqueue_raw_text(node, *field, text)
            }
            (QueueBinding::DocumentChoice { node }, CallbackArguments::Text(text)) => {
                queue.enqueue_document_choice(node, text)
            }
            (QueueBinding::UnboundText { node }, CallbackArguments::Text(_)) => {
                queue.reject_unbound_text(node)
            }
            (QueueBinding::Text { node, field }, CallbackArguments::Text(text)) => {
                queue.enqueue_text(node, field.clone(), text)
            }
            (QueueBinding::Action { node, action }, CallbackArguments::Action) => {
                queue.enqueue_action(node, *action)
            }
            _ => {
                return Err(PluginError::new(
                    PluginErrorCode::InvalidPayload,
                    "Review callback arguments differ",
                ));
            }
        };
        Ok(PluginDispatch::Immediate(match result {
            Ok(()) => Ok(PluginPayload::encode(
                descriptor.reply_schema.clone(),
                &QueuedAck { queued: true },
            )?),
            Err(error) => Err(queue_error(error)),
        }))
    }
    fn cancel(&mut self, _: &PluginTicket) -> Result<CancelReceipt, PluginError> {
        Ok(CancelReceipt::AlreadyTerminal)
    }
    fn shutdown(&mut self) -> Result<(), PluginError> {
        self.queue
            .try_borrow_mut()
            .map_err(|_| {
                PluginError::new(
                    PluginErrorCode::Adapter,
                    "Presentation intent queue is borrowed",
                )
            })?
            .close();
        Ok(())
    }
}

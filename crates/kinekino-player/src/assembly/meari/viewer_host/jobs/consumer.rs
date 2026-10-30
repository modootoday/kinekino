//! Extracted single-query lifecycle shared by list and the existing opt-in detail consumer.
use crate::application::api::ApiError;
use kinekino_ui_plugins::infrastructure::typed_async_sdk_plugin::{
    AsyncSdkPump, TerminalAcceptance, TypedAsyncSdkPlugin,
};
use meari::{
    application::{
        host_callbacks::{
            CallbackBindings, CallbackHandle, HostCallbacks, LocalActionInvocation, LocalActionKey,
            LocalDispatchError, PendingCallback,
        },
        plugins::*,
    },
    domain::{
        build::{ContentHash, TargetKind},
        host_callbacks::CallbackSignature,
    },
};

pub(crate) struct ReadBinding<I> {
    pub manifest: PluginManifest,
    pub command: &'static str,
    pub contract: ContentHash,
    pub target: I,
    pub refresh: &'static str,
    pub retry: &'static str,
}
pub(crate) enum DecodedRead<O> {
    Completed(O),
    Failed(String),
    Cancelled,
}
#[must_use = "Retain the original query consumer until its attachment is released"]
pub(crate) struct ReadConsumer {
    registry: PluginRegistry,
    scope: PluginScope,
    registration: RegistrationHandle,
    callbacks: HostCallbacks,
    refresh_handle: CallbackHandle,
    retry_key: LocalActionKey,
    retry_prop: &'static str,
    descriptor: CapabilityDescriptor,
    pump: AsyncSdkPump,
    pending: Option<PluginTicket>,
    retained: Option<TerminalAcceptance>,
    ack_blocked: bool,
    consumer_blocked: bool,
    cancel_requested: bool,
    first_fault: Option<PluginError>,
    retry_requested: bool,
    invalidated: bool,
    closed: bool,
}
impl ReadConsumer {
    pub(crate) fn new<I: serde::Serialize + 'static>(
        scope: PluginScope,
        binding: ReadBinding<I>,
        adapter: (TypedAsyncSdkPlugin, AsyncSdkPump),
    ) -> Result<Self, PluginError> {
        let descriptor = binding
            .manifest
            .capabilities
            .get(binding.command)
            .cloned()
            .ok_or_else(|| {
                PluginError::new(
                    PluginErrorCode::MissingCapability,
                    "Read capability is absent",
                )
            })?;
        let mut registry =
            PluginRegistry::new(binding.contract, TargetKind::Native, scope.clone(), 1)?;
        let (plugin, pump) = adapter;
        let registration = registry.register(
            Box::new(plugin),
            binding.manifest.capabilities.keys().cloned().collect(),
        )?;
        let mut callbacks = HostCallbacks::new(scope.clone());
        let schema = descriptor.request_schema.clone();
        let target = binding.target;
        let refresh_handle = callbacks.bind(
            binding.refresh.into(),
            CallbackSignature::Action,
            registration.clone(),
            binding.command.into(),
            move |_| PluginPayload::encode(schema.clone(), &target),
        )?;
        let retry_key = callbacks.bind_local(binding.retry.into(), CallbackSignature::Action)?;
        Ok(Self {
            registry,
            scope,
            registration,
            callbacks,
            refresh_handle,
            retry_key,
            retry_prop: binding.retry,
            descriptor,
            pump,
            pending: None,
            retained: None,
            ack_blocked: false,
            consumer_blocked: false,
            cancel_requested: false,
            first_fault: None,
            retry_requested: false,
            invalidated: false,
            closed: false,
        })
    }
    pub(crate) fn bindings(&self) -> Result<CallbackBindings, PluginError> {
        self.callbacks.export_bindings()
    }
    pub(crate) fn bind_local(
        &mut self,
        name: &str,
        signature: CallbackSignature,
    ) -> Result<LocalActionKey, PluginError> {
        self.callbacks.bind_local(name.into(), signature)
    }
    pub(crate) fn local(
        &self,
        pending: &PendingCallback,
        scope: &PluginScope,
    ) -> Result<LocalActionInvocation, LocalDispatchError> {
        self.callbacks
            .dispatch_local(&pending.handle, pending.invocation.arguments.clone(), scope)
    }
    pub(crate) fn retained(&self) -> Option<&TerminalAcceptance> {
        self.retained.as_ref()
    }
    pub(crate) fn pending(&self) -> Option<&PluginTicket> {
        self.pending.as_ref()
    }
    pub(crate) fn first_fault(&self) -> Option<&PluginError> {
        self.first_fault.as_ref()
    }
    pub(crate) fn blocked(&self) -> bool {
        self.ack_blocked || self.consumer_blocked
    }
    pub(crate) fn invalidated(&self) -> bool {
        self.invalidated
    }

    /// Returns true only for a newly submitted read; retained-result retry never resubmits.
    pub(crate) fn dispatch(
        &mut self,
        event: &PendingCallback,
        scope: &PluginScope,
    ) -> Result<bool, PluginError> {
        if self.invalidated || self.closed {
            return Err(PluginError::new(
                PluginErrorCode::Closing,
                "Read consumer is closing",
            ));
        }
        if &self.scope != scope {
            return Err(PluginError::new(
                PluginErrorCode::Stale,
                "Foreign read scope",
            ));
        }
        if event.invocation.prop == self.retry_prop {
            let invocation = self
                .callbacks
                .dispatch_local(&event.handle, event.invocation.arguments.clone(), scope)
                .map_err(|_| PluginError::new(PluginErrorCode::Stale, "Retired read callback"))?;
            if !invocation.matches_key(&self.retry_key) {
                return Err(PluginError::new(
                    PluginErrorCode::Stale,
                    "Foreign result retry",
                ));
            }
            self.retry_retained()?;
            return Ok(false);
        }
        self.submit_handle(&event.handle, event.invocation.arguments.clone())?;
        Ok(true)
    }
    pub(crate) fn submit_from_host(&mut self) -> Result<(), PluginError> {
        let handle = self.refresh_handle.clone();
        self.submit_handle(
            &handle,
            meari::domain::host_callbacks::CallbackArguments::Action,
        )
    }
    fn submit_handle(
        &mut self,
        handle: &CallbackHandle,
        arguments: meari::domain::host_callbacks::CallbackArguments,
    ) -> Result<(), PluginError> {
        if self.invalidated || self.closed {
            return Err(PluginError::new(
                PluginErrorCode::Closing,
                "Read consumer is closing",
            ));
        }
        if self.pending.is_some() {
            return Err(PluginError::new(
                PluginErrorCode::Busy,
                "Original read or ACK remains",
            ));
        }
        let receipt = self
            .callbacks
            .dispatch(handle, arguments, &mut self.registry)?;
        self.pending = Some(receipt.ticket);
        Ok(())
    }
    pub(crate) fn retry_retained(&mut self) -> Result<(), PluginError> {
        if !self.blocked() || self.pending.is_none() {
            return Err(PluginError::new(
                PluginErrorCode::Stale,
                "No retained read result",
            ));
        }
        self.retry_requested = true;
        Ok(())
    }
    pub(crate) fn poll(&mut self) -> Result<bool, PluginError> {
        let Some(ticket) = self.pending.clone() else {
            return Ok(false);
        };
        if self.blocked() && !std::mem::take(&mut self.retry_requested) {
            return Ok(false);
        }
        let acceptance = match self.pump.accept(&mut self.registry, &ticket) {
            Ok(Some(value)) => value,
            Ok(None) => return Ok(false),
            Err(failure) => {
                self.first_fault.get_or_insert_with(|| failure.clone());
                self.consumer_blocked = true;
                return Err(failure);
            }
        };
        self.consumer_blocked = false;
        self.retained = Some(acceptance);
        let accepted = self.retained.as_ref().expect("stored acceptance");
        self.ack_blocked = accepted.owner_ack.is_err();
        if let PluginEventKind::Failed(failure) = &accepted.event.event {
            self.first_fault.get_or_insert_with(|| failure.clone());
        }
        if let Err(failure) = &accepted.owner_ack {
            self.first_fault.get_or_insert_with(|| failure.clone());
        } else {
            self.pending = None;
        }
        Ok(true)
    }
    pub(crate) fn decoded<O: serde::de::DeserializeOwned>(
        &self,
    ) -> Result<DecodedRead<O>, PluginError> {
        let accepted = self
            .retained
            .as_ref()
            .ok_or_else(|| PluginError::new(PluginErrorCode::Stale, "No accepted read"))?;
        Ok(match &accepted.event.event {
            PluginEventKind::Completed(payload) => {
                match payload.decode::<O>(&self.descriptor.reply_schema) {
                    Ok(record) => DecodedRead::Completed(record),
                    Err(error) => DecodedRead::Failed(error.message),
                }
            }
            PluginEventKind::Failed(failure) => {
                let sdk = failure
                    .details
                    .as_ref()
                    .zip(self.descriptor.error_schema.as_ref())
                    .and_then(|(payload, schema)| payload.decode::<ApiError>(schema).ok());
                DecodedRead::Failed(
                    sdk.map(|error| format!("{:?}: {}", error.code, error.message))
                        .unwrap_or_else(|| failure.message.clone()),
                )
            }
            PluginEventKind::Cancelled => DecodedRead::Cancelled,
            PluginEventKind::Progress(_) => {
                DecodedRead::Failed("Unexpected progress for a single read".into())
            }
        })
    }
    /// Query cancellation only; no jobs.cancel/retry or parent ExecutionOwner shutdown.
    pub(crate) fn close(&mut self) -> Result<bool, PluginError> {
        if self.closed {
            return Ok(true);
        }
        if !self.invalidated {
            self.callbacks.invalidate()?;
            self.invalidated = true;
        }
        self.pump.begin_close()?;
        if let Some(ticket) = self.pending.clone() {
            if !self.ack_blocked && !self.cancel_requested {
                self.registry.cancel(&ticket)?;
                self.cancel_requested = true;
            }
            self.poll()?;
            if self.pending.is_some() {
                return Ok(false);
            }
        }
        self.registry.unregister(&self.registration)?;
        self.closed = true;
        Ok(true)
    }
}

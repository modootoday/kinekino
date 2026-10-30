//! Typed SDK bindings; registration, grants and scope checks remain in Meari.
use kinekino_media_contracts::errors::ApiError;
use meari::application::plugins::*;
use serde::{Serialize, de::DeserializeOwned};
use std::{collections::BTreeMap, marker::PhantomData};

pub trait SdkBinding {
    fn descriptor(&self) -> &CapabilityDescriptor;
    fn validate(
        &self,
        direction: PayloadDirection,
        payload: &PluginPayload,
    ) -> Result<(), PluginError>;
    fn invoke(&self, invocation: &PluginInvocation) -> Result<PluginDispatch, PluginError>;
}

struct TypedBinding<I, O, F> {
    descriptor: CapabilityDescriptor,
    invoke: F,
    types: PhantomData<fn(I) -> O>,
}

pub fn binding<I, O, F>(descriptor: CapabilityDescriptor, invoke: F) -> Box<dyn SdkBinding>
where
    I: DeserializeOwned + 'static,
    O: Serialize + DeserializeOwned + 'static,
    F: Fn(&PluginTicket, I) -> Result<O, ApiError> + 'static,
{
    Box::new(TypedBinding {
        descriptor,
        invoke,
        types: PhantomData,
    })
}

impl<I, O, F> SdkBinding for TypedBinding<I, O, F>
where
    I: DeserializeOwned,
    O: Serialize + DeserializeOwned,
    F: Fn(&PluginTicket, I) -> Result<O, ApiError>,
{
    fn descriptor(&self) -> &CapabilityDescriptor {
        &self.descriptor
    }

    fn validate(
        &self,
        direction: PayloadDirection,
        payload: &PluginPayload,
    ) -> Result<(), PluginError> {
        match direction {
            PayloadDirection::Request => {
                payload.decode::<I>(&self.descriptor.request_schema)?;
            }
            PayloadDirection::Reply => {
                payload.decode::<O>(&self.descriptor.reply_schema)?;
            }
            PayloadDirection::Error => {
                let schema = self
                    .descriptor
                    .error_schema
                    .as_ref()
                    .ok_or_else(|| invalid("SDK error schema required"))?;
                payload.decode::<ApiError>(schema)?;
            }
            PayloadDirection::Event => {
                return Err(invalid("Synchronous SDK binding emits no events"));
            }
        }
        Ok(())
    }

    fn invoke(&self, invocation: &PluginInvocation) -> Result<PluginDispatch, PluginError> {
        let input = invocation
            .payload
            .decode::<I>(&self.descriptor.request_schema)?;
        let outcome = match (self.invoke)(&invocation.ticket, input) {
            Ok(output) => Ok(PluginPayload::encode(
                self.descriptor.reply_schema.clone(),
                &output,
            )?),
            Err(error) => {
                let schema = self
                    .descriptor
                    .error_schema
                    .clone()
                    .ok_or_else(|| invalid("SDK error schema required"))?;
                let mut failure = PluginError::new(PluginErrorCode::Adapter, error.message.clone());
                failure.details = Some(PluginPayload::encode(schema, &error)?);
                Err(failure)
            }
        };
        Ok(PluginDispatch::Immediate(outcome))
    }
}

fn invalid(message: &str) -> PluginError {
    PluginError::new(PluginErrorCode::InvalidPayload, message)
}

pub struct TypedSdkPlugin {
    manifest: PluginManifest,
    bindings: BTreeMap<String, Box<dyn SdkBinding>>,
}

impl TypedSdkPlugin {
    pub fn new(
        manifest: PluginManifest,
        bindings: BTreeMap<String, Box<dyn SdkBinding>>,
    ) -> Result<Self, PluginError> {
        manifest.validate()?;
        if manifest.capabilities.len() != bindings.len()
            || bindings.iter().any(|(id, binding)| {
                manifest.capabilities.get(id) != Some(binding.descriptor())
                    || binding.descriptor().error_schema.is_none()
                    || binding.descriptor().event_schema.is_some()
            })
        {
            return Err(PluginError::new(
                PluginErrorCode::InvalidManifest,
                "SDK binding inventory/schema differs from manifest",
            ));
        }
        Ok(Self { manifest, bindings })
    }

    fn binding(&self, capability: &str) -> Result<&dyn SdkBinding, PluginError> {
        self.bindings
            .get(capability)
            .map(|binding| binding.as_ref())
            .ok_or_else(|| {
                PluginError::new(
                    PluginErrorCode::MissingCapability,
                    "Unknown product SDK binding",
                )
            })
    }
}

impl PluginAdapter for TypedSdkPlugin {
    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }
    fn validate_payload(
        &self,
        capability: &str,
        direction: PayloadDirection,
        payload: &PluginPayload,
    ) -> Result<(), PluginError> {
        self.binding(capability)?.validate(direction, payload)
    }
    fn dispatch(&mut self, invocation: &PluginInvocation) -> Result<PluginDispatch, PluginError> {
        self.binding(invocation.ticket.capability())?
            .invoke(invocation)
    }
    fn cancel(&mut self, _: &PluginTicket) -> Result<CancelReceipt, PluginError> {
        Ok(CancelReceipt::AlreadyTerminal)
    }
    fn shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

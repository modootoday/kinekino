use super::AsyncSdkPort;
use kinekino_media_contracts::errors::ApiError;
use meari::application::plugins::*;
use serde::{Serialize, de::DeserializeOwned};
use std::marker::PhantomData;

// Same wire ceiling as Core. Count before allocating a JSON Value, including escaped UTF-8.
const MAX_TERMINAL_BYTES: usize = 1024 * 1024;
struct WireCounter {
    bytes: usize,
    limit: usize,
}
impl std::io::Write for WireCounter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.bytes = self
            .bytes
            .checked_add(bytes.len())
            .filter(|size| *size <= self.limit)
            .ok_or_else(|| std::io::Error::other("Async SDK terminal exceeds wire budget"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn bounded(value: &impl Serialize) -> Result<(), PluginError> {
    bounded_to(value, MAX_TERMINAL_BYTES)
}
fn bounded_to(value: &impl Serialize, limit: usize) -> Result<(), PluginError> {
    serde_json::to_writer(WireCounter { bytes: 0, limit }, value)
        .map_err(|error| invalid(&error.to_string()))
}
#[derive(Serialize)]
struct PayloadEnvelope<'a, T> {
    schema: &'a meari::domain::build::ContentHash,
    value: &'a T,
}
#[derive(Serialize)]
struct FailureEnvelope<'a> {
    code: PluginErrorCode,
    message: &'a str,
    details: PayloadEnvelope<'a, ApiError>,
}

pub(super) fn wire_bytes(value: &impl Serialize) -> Result<usize, PluginError> {
    struct Counter(usize);
    impl std::io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0 = self
                .0
                .checked_add(bytes.len())
                .ok_or_else(|| std::io::Error::other("Wire length overflow"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut counter = Counter(0);
    serde_json::to_writer(&mut counter, value).map_err(|error| invalid(&error.to_string()))?;
    Ok(counter.0)
}

pub trait AsyncSdkBinding {
    fn descriptor(&self) -> &CapabilityDescriptor;
    fn validate(
        &self,
        direction: PayloadDirection,
        payload: &PluginPayload,
    ) -> Result<(), PluginError>;
    fn start(&mut self, invocation: &PluginInvocation) -> Result<(), PluginError>;
    fn peek_terminal(
        &mut self,
        ticket: &PluginTicket,
        max_wire_bytes: usize,
    ) -> Result<Option<PluginEventKind>, PluginError>;
    fn acknowledge(&mut self, ticket: &PluginTicket) -> Result<(), PluginError>;
    fn cancel(&mut self, ticket: &PluginTicket) -> Result<CancelReceipt, PluginError>;
    fn shutdown(&mut self) -> Result<(), PluginError>;
}
struct TypedAsyncBinding<I, O, P> {
    descriptor: CapabilityDescriptor,
    port: P,
    types: PhantomData<fn(I) -> O>,
}
pub fn async_binding<I, O, P>(descriptor: CapabilityDescriptor, port: P) -> Box<dyn AsyncSdkBinding>
where
    I: DeserializeOwned + 'static,
    O: Serialize + DeserializeOwned + 'static,
    P: AsyncSdkPort<I, O> + 'static,
{
    Box::new(TypedAsyncBinding {
        descriptor,
        port,
        types: PhantomData,
    })
}
pub(super) fn invalid(message: &str) -> PluginError {
    PluginError::new(PluginErrorCode::InvalidPayload, message)
}
impl<I, O, P> AsyncSdkBinding for TypedAsyncBinding<I, O, P>
where
    I: DeserializeOwned,
    O: Serialize + DeserializeOwned,
    P: AsyncSdkPort<I, O>,
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
                payload.decode::<ApiError>(
                    self.descriptor
                        .error_schema
                        .as_ref()
                        .ok_or_else(|| invalid("SDK error schema required"))?,
                )?;
            }
            PayloadDirection::Event => {
                return Err(invalid("Terminal-only SDK binding emits no progress"));
            }
        }
        Ok(())
    }
    fn start(&mut self, invocation: &PluginInvocation) -> Result<(), PluginError> {
        let input = invocation
            .payload
            .decode::<I>(&self.descriptor.request_schema)?;
        self.port.start(&invocation.ticket, input)
    }
    fn peek_terminal(
        &mut self,
        ticket: &PluginTicket,
        max_wire_bytes: usize,
    ) -> Result<Option<PluginEventKind>, PluginError> {
        let Some(result) = self.port.peek_terminal(ticket)? else {
            return Ok(None);
        };
        Ok(Some(match result.as_ref() {
            Ok(output) => {
                bounded(output)?;
                bounded_to(
                    &PayloadEnvelope {
                        schema: &self.descriptor.reply_schema,
                        value: output,
                    },
                    max_wire_bytes,
                )?;
                PluginEventKind::Completed(PluginPayload::encode(
                    self.descriptor.reply_schema.clone(),
                    output,
                )?)
            }
            Err(error) => {
                bounded(error)?;
                let schema = self
                    .descriptor
                    .error_schema
                    .clone()
                    .ok_or_else(|| invalid("SDK error schema required"))?;
                bounded_to(
                    &FailureEnvelope {
                        code: PluginErrorCode::Adapter,
                        message: &error.message,
                        details: PayloadEnvelope {
                            schema: &schema,
                            value: error,
                        },
                    },
                    max_wire_bytes,
                )?;
                let mut failure = PluginError::new(PluginErrorCode::Adapter, error.message.clone());
                failure.details = Some(PluginPayload::encode(schema, error)?);
                PluginEventKind::Failed(failure)
            }
        }))
    }
    fn acknowledge(&mut self, ticket: &PluginTicket) -> Result<(), PluginError> {
        self.port.acknowledge(ticket)
    }
    fn cancel(&mut self, ticket: &PluginTicket) -> Result<CancelReceipt, PluginError> {
        self.port.cancel(ticket)
    }
    fn shutdown(&mut self) -> Result<(), PluginError> {
        self.port.shutdown()
    }
}

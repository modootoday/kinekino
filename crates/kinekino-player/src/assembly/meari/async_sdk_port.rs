use crate::application::{
    api::{ApiError, ErrorCode, Request, RequestContext},
    sdk_execution::{
        CancelDisposition, ExecutionAttachment, ExecutionSnapshot, OperationId, encoded_size,
    },
};
use kinekino_ui_plugins::infrastructure::typed_async_sdk_plugin::AsyncSdkPort;
use meari::application::plugins::*;
use serde::{Serialize, de::DeserializeOwned};
use std::{marker::PhantomData, rc::Rc, sync::Arc};
struct Entry<O> {
    ticket: PluginTicket,
    operation: OperationId,
    typed: Option<Arc<Result<O, ApiError>>>,
}
pub(crate) struct SdkPort<I, O, C> {
    execution: Arc<ExecutionAttachment>,
    command: &'static str,
    descriptor: CapabilityDescriptor,
    context: Rc<C>,
    wire_limit: usize,
    entries: Vec<Entry<O>>,
    input: PhantomData<fn(I)>,
}
fn invalid() -> PluginError {
    PluginError::new(
        PluginErrorCode::InvalidPayload,
        "SDK terminal requires explicit codec or budget recovery; original retained",
    )
}
impl<I, O, C> SdkPort<I, O, C> {
    pub fn new(
        execution: Arc<ExecutionAttachment>,
        command: &'static str,
        descriptor: CapabilityDescriptor,
        context: Rc<C>,
        wire_limit: usize,
    ) -> Self {
        Self {
            execution,
            command,
            descriptor,
            context,
            wire_limit,
            entries: Vec::new(),
            input: PhantomData,
        }
    }
    fn failure(&self, error: ApiError, admission: bool) -> PluginError {
        let code = if admission && error.code == ErrorCode::ResourceExhausted {
            PluginErrorCode::Busy
        } else {
            PluginErrorCode::Adapter
        };
        let mut result = PluginError::new(code, error.message.clone());
        if let Some(schema) = &self.descriptor.error_schema {
            result.details = PluginPayload::encode(schema.clone(), &error).ok();
        }
        result
    }
}
impl<I, O, C> AsyncSdkPort<I, O> for SdkPort<I, O, C>
where
    I: Serialize,
    O: DeserializeOwned,
    C: Fn(&PluginTicket) -> Result<RequestContext, ApiError>,
{
    fn start(&mut self, ticket: &PluginTicket, input: I) -> Result<(), PluginError> {
        if self.entries.iter().any(|e| e.ticket == *ticket) {
            return Err(PluginError::new(
                PluginErrorCode::Duplicate,
                "SDK operation already admitted",
            ));
        }
        let context = (self.context)(ticket).map_err(|error| self.failure(error, true))?;
        let payload = serde_json::to_value(input).map_err(|_| invalid())?;
        let owned = ticket.clone();
        self.entries.try_reserve(1).map_err(|_| {
            PluginError::new(
                PluginErrorCode::Busy,
                "SDK ticket tracking capacity unavailable",
            )
        })?;
        let operation = self
            .execution
            .submit(Request {
                context,
                command: self.command.into(),
                payload,
            })
            .map_err(|error| self.failure(error, true))?;
        self.entries.push(Entry {
            ticket: owned,
            operation,
            typed: None,
        });
        Ok(())
    }
    fn peek_terminal(
        &mut self,
        ticket: &PluginTicket,
    ) -> Result<Option<Arc<Result<O, ApiError>>>, PluginError> {
        let entry = self
            .entries
            .iter_mut()
            .find(|entry| entry.ticket == *ticket)
            .ok_or_else(|| PluginError::new(PluginErrorCode::Stale, "Unknown async SDK ticket"))?;
        if let Some(typed) = &entry.typed {
            return Ok(Some(typed.clone()));
        }
        let terminal = match self
            .execution
            .peek(&entry.operation)
            .map_err(|_| invalid())?
        {
            ExecutionSnapshot::Queued | ExecutionSnapshot::Running { .. } => return Ok(None),
            ExecutionSnapshot::Terminal(terminal) => terminal,
            ExecutionSnapshot::Acknowledged => return Err(invalid()),
        };
        if terminal.measurement.is_faulted() {
            return Err(invalid());
        }
        let response = &terminal.response;
        terminal.validate_response().map_err(|_| invalid())?;
        let typed = if response.ok && response.error.is_none() {
            let value = response.data.as_ref().ok_or_else(invalid)?;
            if encoded_size(value, self.wire_limit).is_faulted() {
                return Err(invalid());
            }
            Ok(serde_json::from_value(value.clone()).map_err(|_| invalid())?)
        } else if !response.ok && response.data.is_none() {
            let error = response.error.as_ref().ok_or_else(invalid)?;
            if encoded_size(error, self.wire_limit).is_faulted() {
                return Err(invalid());
            }
            Err(
                serde_json::from_value(serde_json::to_value(error).map_err(|_| invalid())?)
                    .map_err(|_| invalid())?,
            )
        } else {
            return Err(invalid());
        };
        let typed = Arc::new(typed);
        entry.typed = Some(typed.clone());
        Ok(Some(typed))
    }
    fn acknowledge(&mut self, ticket: &PluginTicket) -> Result<(), PluginError> {
        let index = self
            .entries
            .iter()
            .position(|entry| entry.ticket == *ticket)
            .ok_or_else(|| PluginError::new(PluginErrorCode::Stale, "Unknown async SDK ticket"))?;
        self.execution
            .acknowledge(&self.entries[index].operation)
            .map_err(|_| invalid())?;
        self.entries.remove(index);
        Ok(())
    }
    fn cancel(&mut self, ticket: &PluginTicket) -> Result<CancelReceipt, PluginError> {
        let entry = self
            .entries
            .iter()
            .find(|entry| entry.ticket == *ticket)
            .ok_or_else(|| PluginError::new(PluginErrorCode::Stale, "Unknown async SDK ticket"))?;
        match self
            .execution
            .cancel(&entry.operation)
            .map_err(|_| invalid())?
        {
            CancelDisposition::QueuedCancelled | CancelDisposition::Requested => {
                Ok(CancelReceipt::Requested)
            }
            CancelDisposition::AlreadyAcknowledged => Ok(CancelReceipt::AlreadyTerminal),
        }
    }
    fn shutdown(&mut self) -> Result<(), PluginError> {
        if !self.entries.is_empty() {
            return Err(PluginError::new(
                PluginErrorCode::Busy,
                "SDK terminal ACK remains",
            ));
        }
        self.execution
            .release()
            .map_err(|_| PluginError::new(PluginErrorCode::Busy, "SDK attachment cleanup remains"))
    }
}

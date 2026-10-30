//! Exact existing transition wire adaptation on the original SDK execution attachment.
use crate::{
    application::{
        api::{ApiError, RequestContext},
        playback_session::transition::{
            BeginAdjacentResolution, EpisodeDirection, SubmittedTransition,
        },
        sdk_execution::ExecutionAttachment,
    },
    assembly::meari::async_sdk_port::SdkPort,
    domain::api_values::DecimalU64,
};
use kinekino_ui_plugins::infrastructure::typed_async_sdk_plugin::AsyncSdkPort;
use meari::application::plugins::*;
use serde::Serialize;
use std::{rc::Rc, sync::Arc};

/// Matches the payload assembled by the existing EpisodeTransitionClient, not a second policy.
#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct StartWire {
    expected_engine_instance_id: DecimalU64,
    catalog_result_sha256: String,
    resolution_submission_key: String,
    paused: bool,
    volume_percent: f64,
    execution_timeout_ms: DecimalU64,
}
pub(super) struct StartPort<C> {
    inner: SdkPort<StartWire, SubmittedTransition, C>,
    context: Rc<C>,
    direction: EpisodeDirection,
    error_schema: Option<meari::domain::build::ContentHash>,
}
impl<C> StartPort<C> {
    pub(super) fn new(
        execution: Arc<ExecutionAttachment>,
        command: &'static str,
        direction: EpisodeDirection,
        descriptor: CapabilityDescriptor,
        context: Rc<C>,
    ) -> Self {
        Self {
            inner: SdkPort::new(
                execution,
                command,
                descriptor.clone(),
                context.clone(),
                256 * 1024,
            ),
            context,
            direction,
            error_schema: descriptor.error_schema,
        }
    }
}
impl<C> AsyncSdkPort<BeginAdjacentResolution, SubmittedTransition> for StartPort<C>
where
    C: Fn(&PluginTicket) -> Result<RequestContext, ApiError>,
{
    fn start(
        &mut self,
        ticket: &PluginTicket,
        input: BeginAdjacentResolution,
    ) -> Result<(), PluginError> {
        if !matches!(
            (self.direction, input.direction),
            (EpisodeDirection::Previous, EpisodeDirection::Previous)
                | (EpisodeDirection::Next, EpisodeDirection::Next)
        ) {
            return Err(PluginError::new(
                PluginErrorCode::InvalidPayload,
                "Transition direction differs from the registered command",
            ));
        }
        let context = (self.context)(ticket).map_err(|error| {
            let mut result = PluginError::new(PluginErrorCode::Adapter, error.message.clone());
            if let Some(schema) = &self.error_schema {
                result.details = PluginPayload::encode(schema.clone(), &error).ok();
            }
            result
        })?;
        // scoped_context is the existing deterministic function of original context + ticket.
        // This is the resolve-job key only; no whole-transition idempotency is asserted.
        self.inner.start(
            ticket,
            StartWire {
                expected_engine_instance_id: input.expected_instance_id,
                catalog_result_sha256: input.catalog_result_sha256,
                resolution_submission_key: context.request_id,
                paused: input.paused,
                volume_percent: input.volume_percent,
                execution_timeout_ms: input.execution_timeout_ms,
            },
        )
    }
    fn peek_terminal(
        &mut self,
        ticket: &PluginTicket,
    ) -> Result<Option<Arc<Result<SubmittedTransition, ApiError>>>, PluginError> {
        self.inner.peek_terminal(ticket)
    }
    fn acknowledge(&mut self, ticket: &PluginTicket) -> Result<(), PluginError> {
        self.inner.acknowledge(ticket)
    }
    fn cancel(&mut self, ticket: &PluginTicket) -> Result<CancelReceipt, PluginError> {
        self.inner.cancel(ticket)
    }
    fn shutdown(&mut self) -> Result<(), PluginError> {
        self.inner.shutdown()
    }
}
